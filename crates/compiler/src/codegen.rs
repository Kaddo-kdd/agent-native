//! IR → WASM Component Model code generation.
//!
//! Migrated verbatim from `_reference/poc-agent-native/crates/irtool/src/lib.rs`
//! (lines 1699-2348), per matriz de extracción C-01 and ADR-001/003/004.
//! The compiler verifies the IR via `agent_native_ir::verify_ir` first and
//! emits deterministic WASM Component Model bytes.
//!
//! Clippy pedantic is suppressed for this ported module (see verifier for
//! rationale). Refactor is tracked as follow-up with tests already in place.
#![allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::manual_let_else,
    clippy::single_match_else,
    clippy::match_same_arms,
    clippy::collapsible_else_if,
    clippy::needless_continue,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::uninlined_format_args,
    clippy::checked_conversions,
    clippy::match_single_binding,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::bool_to_int_with_if
)]

use std::collections::{HashMap, HashSet};

use agent_native_ir::context::Context;
use agent_native_ir::diagnostic::Diagnostic;
use agent_native_ir::model::{FuncSignature, ImportSignature, IrType, RecordDef, WitContractDef};
use agent_native_ir::verify_ir;
use serde_json::Value;
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, CustomSection, EntityType, ExportKind, ExportSection,
    Function, FunctionSection, GlobalSection, GlobalType, ImportSection, Instruction, MemArg,
    MemorySection, MemoryType, Module, TypeSection, ValType,
};

use crate::context::{CompileContext, ResolveOutcome};

/// WIT interface identifier string used by the compiler to lower imports.
fn wit_interface_id(package: &str, iface: &str) -> String {
    if let Some((ns_pkg, ver)) = package.split_once('@') {
        format!("{ns_pkg}/{iface}@{ver}")
    } else {
        format!("{package}/{iface}")
    }
}

/// Backwards-compatible entry point: compiles the IR using a context
/// anchored at the current working directory.
///
/// # When to use
///
/// This shim is safe when the IR does **not** reference an external WIT
/// file (IR v0.1–v0.3, or v0.4+ with embedded contracts). For those
/// inputs the cwd is irrelevant and the output is byte-identical to
/// [`compile_ir_to_wasm_with_context`].
///
/// # When to prefer `compile_ir_to_wasm_with_context`
///
/// If the IR carries `wit_contract.wit_file` referencing a file on disk
/// (typical of IR v0.5 fixtures), **prefer**
/// [`compile_ir_to_wasm_with_context`] with an explicit
/// [`CompileContext`]. The explicit API is immune to the process working
/// directory and resolves `wit_file` deterministically against
/// `ctx.wit_root()`; this shim depends on the cwd and can silently pick
/// up a wrong file (or `E_WIT_NOT_FOUND`) when invoked from a subcrate.
pub fn compile_ir_to_wasm(root: &Value) -> Result<Vec<u8>, Vec<Diagnostic>> {
    compile_ir_to_wasm_with_context(root, &CompileContext::default())
}

/// AN-016 explicit-context entry point. The compiler resolves
/// `wit_contract.wit_file` against `ctx.wit_root()` instead of relying on
/// the process working directory, so the same IR + WIT produce the same
/// bytes regardless of where the build runs. No `../../` fallback chains.
pub fn compile_ir_to_wasm_with_context(
    root: &Value,
    wit_ctx: &CompileContext,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let diags = verify_ir(root);
    if !diags.is_empty() {
        return Err(diags);
    }

    let ir_version = root
        .get("ir_version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.1");

    let mut record_defs: HashMap<String, RecordDef> = HashMap::new();
    if let Some(types_arr) = root.get("types").and_then(|v| v.as_array()) {
        for type_val in types_arr {
            let type_obj = type_val.as_object().unwrap();
            let name = type_obj["name"].as_str().unwrap().to_string();
            let mut fields = Vec::new();
            for f_val in type_obj["fields"].as_array().unwrap() {
                let f_obj = f_val.as_object().unwrap();
                let f_name = f_obj["name"].as_str().unwrap().to_string();
                let f_ty = match f_obj["type"].as_str().unwrap() {
                    "i32" => IrType::I32,
                    "bool" => IrType::Bool,
                    other => IrType::Record(other.to_string()),
                };
                fields.push((f_name, f_ty));
            }
            record_defs.insert(name.clone(), RecordDef { name, fields });
        }
    }

    let mut capabilities = HashSet::new();
    if let Some(caps_val) = root.get("capabilities") {
        if let Some(caps_arr) = caps_val.as_array() {
            for c_val in caps_arr {
                if let Some(s) = c_val.as_str() {
                    capabilities.insert(s.to_string());
                }
            }
        }
    }

    let mut import_signatures: HashMap<String, ImportSignature> = HashMap::new();
    let mut import_order: Vec<String> = Vec::new();
    if let Some(imp_val) = root.get("imports") {
        if let Some(imp_arr) = imp_val.as_array() {
            for (i_idx, item_val) in imp_arr.iter().enumerate() {
                let imp_obj = item_val.as_object().unwrap();
                let module = imp_obj["module"].as_str().unwrap().to_string();
                let name = imp_obj["name"].as_str().unwrap().to_string();
                let capability = imp_obj
                    .get("capability")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mut parsed_params = Vec::new();
                if let Some(p_arr) = imp_obj.get("params").and_then(|v| v.as_array()) {
                    for p_val in p_arr {
                        let p_obj = p_val.as_object().unwrap();
                        let p_name = p_obj["name"].as_str().unwrap().to_string();
                        let p_ty = match p_obj["type"].as_str().unwrap() {
                            "i32" => IrType::I32,
                            "bool" => IrType::Bool,
                            other => IrType::Record(other.to_string()),
                        };
                        parsed_params.push((p_name, p_ty));
                    }
                }
                let ret_ty = match imp_obj
                    .get("returns")
                    .and_then(|v| v.as_str())
                    .unwrap_or("i32")
                {
                    "i32" => IrType::I32,
                    "bool" => IrType::Bool,
                    other => IrType::Record(other.to_string()),
                };
                let wit_interface = imp_obj
                    .get("wit_interface")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                import_signatures.insert(
                    name.clone(),
                    ImportSignature {
                        module,
                        name: name.clone(),
                        capability,
                        wit_interface,
                        params: parsed_params,
                        returns: ret_ty,
                        index: i_idx as u32,
                    },
                );
                import_order.push(name);
            }
        }
    }
    let num_imports = import_order.len() as u32;

    let functions = root["functions"]
        .as_array()
        .expect("verified functions array");

    let mut function_signatures: HashMap<String, FuncSignature> = HashMap::new();
    for (f_idx, func_val) in functions.iter().enumerate() {
        let func_obj = func_val.as_object().unwrap();
        let name = func_obj["name"].as_str().unwrap().to_string();
        let is_export = func_obj
            .get("export")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let wit_interface = func_obj
            .get("wit_interface")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let params_arr = func_obj["params"].as_array().unwrap();
        let mut params = Vec::new();
        for p_val in params_arr {
            let p_name = p_val["name"].as_str().unwrap().to_string();
            let p_ty = match p_val["type"].as_str().unwrap() {
                "i32" => IrType::I32,
                "bool" => IrType::Bool,
                other => IrType::Record(other.to_string()),
            };
            params.push((p_name, p_ty));
        }
        let returns = match func_obj["returns"].as_str().unwrap() {
            "i32" => IrType::I32,
            "bool" => IrType::Bool,
            other => IrType::Record(other.to_string()),
        };
        function_signatures.insert(
            name.clone(),
            FuncSignature {
                name,
                export: is_export,
                wit_interface,
                params,
                returns,
                index: num_imports + f_idx as u32,
            },
        );
    }

    let wit_contract_def = if ir_version == "0.4" || ir_version == "0.5" {
        root.get("wit_contract")
            .and_then(|v| v.as_object())
            .map(|obj| WitContractDef {
                package: obj
                    .get("package")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                world: obj
                    .get("world")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                export_interface: obj
                    .get("export_interface")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                import_interfaces: obj
                    .get("import_interfaces")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                wit_file: obj
                    .get("wit_file")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
    } else {
        None
    };

    let ctx = Context {
        ir_version: ir_version.to_string(),
        wit_contract: wit_contract_def,
        capabilities,
        imports: import_signatures,
        import_order: import_order.clone(),
        records: record_defs,
        functions: function_signatures,
        num_imports,
    };

    let needs_memory = ir_version == "0.2" || ir_version == "0.3" || !ctx.records.is_empty();

    let mut module = Module::new();
    let mut types = TypeSection::new();
    let mut import_section = ImportSection::new();
    let mut func_section = FunctionSection::new();
    let mut exports = ExportSection::new();
    let mut code = CodeSection::new();

    for (imp_idx, imp_name) in import_order.iter().enumerate() {
        let imp_sig = &ctx.imports[imp_name];
        let wasm_params: Vec<ValType> = imp_sig.params.iter().map(|_| ValType::I32).collect();
        let results = [ValType::I32];
        let type_idx = imp_idx as u32;
        types.ty().function(wasm_params, results);

        let (imp_mod, imp_field) = if ir_version == "0.4" || ir_version == "0.5" {
            if let (Some(contract), Some(wit_iface)) = (&ctx.wit_contract, &imp_sig.wit_interface) {
                let pkg = if imp_sig.module.contains(':') && imp_sig.module.contains('@') {
                    &imp_sig.module
                } else {
                    &contract.package
                };
                (
                    wit_interface_id(pkg, wit_iface),
                    imp_sig.name.replace('_', "-"),
                )
            } else {
                (imp_sig.module.clone(), imp_sig.name.clone())
            }
        } else {
            (imp_sig.module.clone(), imp_sig.name.clone())
        };

        import_section.import(&imp_mod, &imp_field, EntityType::Function(type_idx));
    }

    for (func_idx, func_val) in functions.iter().enumerate() {
        let func_obj = func_val.as_object().unwrap();
        let name = func_obj["name"].as_str().unwrap();
        let is_export = func_obj
            .get("export")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let params_arr = func_obj["params"].as_array().unwrap();
        let body = &func_obj["body"];

        let mut param_map = HashMap::new();
        let mut wasm_params = Vec::with_capacity(params_arr.len());
        for (p_idx, p_val) in params_arr.iter().enumerate() {
            let p_name = p_val["name"].as_str().unwrap().to_string();
            param_map.insert(p_name, p_idx as u32);
            wasm_params.push(ValType::I32);
        }

        let results = [ValType::I32];
        let func_type_idx = num_imports + func_idx as u32;
        types.ty().function(wasm_params, results);
        func_section.function(func_type_idx);

        if is_export {
            exports.export(name, ExportKind::Func, num_imports + func_idx as u32);
            if ir_version == "0.4" || ir_version == "0.5" {
                if let (Some(contract), Some(wit_iface)) = (
                    &ctx.wit_contract,
                    func_obj.get("wit_interface").and_then(|v| v.as_str()),
                ) {
                    let wit_export_name = format!(
                        "{}#{}",
                        wit_interface_id(&contract.package, wit_iface),
                        name.replace('_', "-")
                    );
                    exports.export(
                        &wit_export_name,
                        ExportKind::Func,
                        num_imports + func_idx as u32,
                    );
                }
            }
        }

        // Local 0..num_params-1: params
        // Local num_params: temporary rec_ptr (i32)
        let rec_ptr_local = params_arr.len() as u32;
        let mut wasm_func = Function::new([(1, ValType::I32)]);
        emit_expr(body, &param_map, rec_ptr_local, &ctx, &mut wasm_func);
        wasm_func.instruction(&Instruction::End);
        code.function(&wasm_func);
    }

    module.section(&types);
    if num_imports > 0 {
        module.section(&import_section);
    }
    module.section(&func_section);

    if needs_memory {
        let mut memory_section = MemorySection::new();
        memory_section.memory(MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        module.section(&memory_section);

        let mut global_section = GlobalSection::new();
        global_section.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i32_const(1024),
        );
        module.section(&global_section);

        exports.export("memory", ExportKind::Memory, 0);
    }

    module.section(&exports);
    module.section(&code);

    if ir_version == "0.4" || ir_version == "0.5" {
        if let Some(ref wit_contract) = ctx.wit_contract {
            let wit_content = if let Some(ref p) = wit_contract.wit_file {
                match wit_ctx.resolve(p) {
                    ResolveOutcome::Found(path) => std::fs::read_to_string(&path).map_err(|e| {
                        vec![Diagnostic {
                            code: "E_WIT_NOT_FOUND".to_string(),
                            function: None,
                            path: "$.wit_contract.wit_file".to_string(),
                            message: format!(
                                "Failed to read WIT file '{}' (resolved from '{p}'): {e}",
                                path.display()
                            ),
                            expected: Some("readable .wit file".to_string()),
                            found: None,
                        }]
                    })?,
                    ResolveOutcome::NotFound { tried } => {
                        return Err(vec![Diagnostic {
                            code: "E_WIT_NOT_FOUND".to_string(),
                            function: None,
                            path: "$.wit_contract.wit_file".to_string(),
                            message: format!(
                                "WIT file '{p}' not found. Tried: [{}]. Compile with an explicit CompileContext whose wit_root points to the directory containing the file.",
                                tried
                                    .iter()
                                    .map(|p| p.display().to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            expected: Some(
                                "accessible .wit file resolvable from the CompileContext"
                                    .to_string(),
                            ),
                            found: None,
                        }]);
                    }
                }
            } else if ir_version == "0.4" {
                // Embedded fallback for IR v0.4 compatibility only. v0.5
                // (ADR-003) always expects an explicit `wit_file` field.
                include_str!("../wit/order.wit").to_string()
            } else {
                let p1 = format!("wit/{}.wit", wit_contract.world);
                let p2 = format!("wit/{}.wit", wit_contract.export_interface);
                let candidates = [p1.clone(), p2.clone()];
                let mut found: Option<String> = None;
                for cand in &candidates {
                    if let ResolveOutcome::Found(path) = wit_ctx.resolve(cand) {
                        if let Ok(txt) = std::fs::read_to_string(&path) {
                            found = Some(txt);
                            break;
                        }
                    }
                }
                let Some(txt) = found else {
                    return Err(vec![Diagnostic {
                        code: "E_WIT_NOT_FOUND".to_string(),
                        function: None,
                        path: "$.wit_contract".to_string(),
                        message: format!(
                            "No WIT file found for world '{}' or export interface '{}'. Fallback defaults are disabled in IR v0.5.",
                            wit_contract.world, wit_contract.export_interface
                        ),
                        expected: Some(format!(
                            "wit/{}.wit or wit/{}.wit, resolvable from the CompileContext",
                            wit_contract.world, wit_contract.export_interface
                        )),
                        found: None,
                    }]);
                };
                txt
            };
            let mut resolve = wit_parser::Resolve::new();
            let pkg = resolve
                .push_str("contract.wit", &wit_content)
                .map_err(|e| {
                    vec![Diagnostic {
                        code: "E_WIT_PARSE_ERROR".to_string(),
                        function: None,
                        path: "$.wit_contract".to_string(),
                        message: format!("Failed to parse WIT contract: {e}"),
                        expected: Some("valid WIT syntax".to_string()),
                        found: None,
                    }]
                })?;

            let world_name = if wit_contract.world.is_empty() {
                if ir_version == "0.4" {
                    "order-processor"
                } else {
                    return Err(vec![Diagnostic {
                        code: "E_WIT_CONTRACT_MISMATCH".to_string(),
                        function: None,
                        path: "$.wit_contract.world".to_string(),
                        message: "World name cannot be empty in Semantic IR v0.5.".to_string(),
                        expected: Some("non-empty world identifier".to_string()),
                        found: None,
                    }]);
                }
            } else {
                &wit_contract.world
            };

            let world = resolve.select_world(pkg, Some(world_name)).map_err(|e| {
                vec![Diagnostic {
                    code: "E_WIT_WORLD_NOT_FOUND".to_string(),
                    function: None,
                    path: "$.wit_contract.world".to_string(),
                    message: format!("World '{world_name}' not found in WIT package: {e}"),
                    expected: Some(format!("world '{world_name}' in package")),
                    found: None,
                }]
            })?;

            let encoded_metadata = wit_component::metadata::encode(
                &resolve,
                world,
                wit_component::StringEncoding::UTF8,
                None,
            )
            .map_err(|e| {
                vec![Diagnostic {
                    code: "E_WIT_METADATA_ENCODE".to_string(),
                    function: None,
                    path: "$.wit_contract".to_string(),
                    message: format!("Failed to encode WIT metadata: {e}"),
                    expected: None,
                    found: None,
                }]
            })?;

            let custom = CustomSection {
                name: format!("component-type:{world_name}").into(),
                data: encoded_metadata.into(),
            };
            module.section(&custom);
        } else if ir_version == "0.5" {
            return Err(vec![Diagnostic {
                code: "E_WIT_CONTRACT_MISSING".to_string(),
                function: None,
                path: "$".to_string(),
                message: "Semantic IR v0.5 requires a valid 'wit_contract' declaration. Defaults are forbidden.".to_string(),
                expected: Some("wit_contract object".to_string()),
                found: None,
            }]);
        }
    }

    let core_bytes = module.finish();

    if ir_version == "0.4" || ir_version == "0.5" {
        let component_bytes = wit_component::ComponentEncoder::default()
            .module(&core_bytes)
            .map_err(|e| {
                vec![Diagnostic {
                    code: "E_COMPONENT_ENCODER".to_string(),
                    function: None,
                    path: "$".to_string(),
                    message: format!("Failed to create ComponentEncoder: {e}"),
                    expected: None,
                    found: None,
                }]
            })?
            .encode()
            .map_err(|e| {
                let msg = e
                    .chain()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                vec![Diagnostic {
                    code: "E_COMPONENT_ENCODE".to_string(),
                    function: None,
                    path: "$".to_string(),
                    message: format!("Failed to encode WebAssembly Component: {msg}"),
                    expected: None,
                    found: None,
                }]
            })?;
        return Ok(component_bytes);
    }

    Ok(core_bytes)
}

fn emit_expr(
    expr: &Value,
    params: &HashMap<String, u32>,
    rec_ptr_local: u32,
    ctx: &Context,
    func: &mut Function,
) {
    let obj = expr.as_object().unwrap();
    let op = obj["op"].as_str().unwrap();
    match op {
        "const" => {
            let ty = obj["type"].as_str().unwrap();
            let val_i32 = match ty {
                "i32" => obj["value"].as_i64().unwrap() as i32,
                "bool" => {
                    if obj["value"].as_bool().unwrap() {
                        1
                    } else {
                        0
                    }
                }
                _ => unreachable!(),
            };
            func.instruction(&Instruction::I32Const(val_i32));
        }
        "param" => {
            let name = obj["name"].as_str().unwrap();
            let local_idx = params[name];
            func.instruction(&Instruction::LocalGet(local_idx));
        }
        "not" => {
            emit_expr(&obj["value"], params, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::I32Eqz);
        }
        "add" | "sub" | "mul" | "div_s" | "eq" | "lt_s" | "gte_s" | "and" | "or" => {
            emit_expr(&obj["left"], params, rec_ptr_local, ctx, func);
            emit_expr(&obj["right"], params, rec_ptr_local, ctx, func);
            match op {
                "add" => func.instruction(&Instruction::I32Add),
                "sub" => func.instruction(&Instruction::I32Sub),
                "mul" => func.instruction(&Instruction::I32Mul),
                "div_s" => func.instruction(&Instruction::I32DivS),
                "eq" => func.instruction(&Instruction::I32Eq),
                "lt_s" => func.instruction(&Instruction::I32LtS),
                "gte_s" => func.instruction(&Instruction::I32GeS),
                "and" => func.instruction(&Instruction::I32And),
                "or" => func.instruction(&Instruction::I32Or),
                _ => unreachable!(),
            };
        }
        "if" => {
            emit_expr(&obj["condition"], params, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
            emit_expr(&obj["then"], params, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::Else);
            emit_expr(&obj["else"], params, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::End);
        }
        "call" => {
            let callee_name = obj["function"].as_str().unwrap();
            let callee_sig = &ctx.functions[callee_name];
            let args_arr = obj["args"].as_array().unwrap();
            for arg_val in args_arr {
                emit_expr(arg_val, params, rec_ptr_local, ctx, func);
            }
            func.instruction(&Instruction::Call(callee_sig.index));
        }
        "call_import" => {
            let import_name = obj["import"].as_str().unwrap();
            let imp_sig = &ctx.imports[import_name];
            let args_arr = obj["args"].as_array().unwrap();
            for arg_val in args_arr {
                emit_expr(arg_val, params, rec_ptr_local, ctx, func);
            }
            func.instruction(&Instruction::Call(imp_sig.index));
        }
        "record.construct" => {
            let rec_type_name = obj["type"].as_str().unwrap();
            let rec_def = &ctx.records[rec_type_name];
            let fields_map = obj["fields"].as_object().unwrap();
            let total_size_bytes = (rec_def.fields.len() * 4) as i32;

            // 1. Get current heap pointer and store in rec_ptr_local
            func.instruction(&Instruction::GlobalGet(0));
            func.instruction(&Instruction::LocalSet(rec_ptr_local));

            // 2. Bump global heap pointer
            func.instruction(&Instruction::GlobalGet(0));
            func.instruction(&Instruction::I32Const(total_size_bytes));
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::GlobalSet(0));

            // 3. For each field, store at (rec_ptr_local + offset)
            for (f_idx, (f_name, _)) in rec_def.fields.iter().enumerate() {
                let f_expr = &fields_map[f_name];
                let byte_offset = (f_idx * 4) as u64;

                func.instruction(&Instruction::LocalGet(rec_ptr_local));
                emit_expr(f_expr, params, rec_ptr_local, ctx, func);
                func.instruction(&Instruction::I32Store(MemArg {
                    offset: byte_offset,
                    align: 2,
                    memory_index: 0,
                }));
            }

            // 4. Leave rec_ptr_local on stack
            func.instruction(&Instruction::LocalGet(rec_ptr_local));
        }
        "record.get" => {
            let target_expr = &obj["target"];
            let field_name = obj["field"].as_str().unwrap();

            // Find target record type
            let rec_type_name = match infer_expr_type(target_expr, params, ctx) {
                Some(IrType::Record(n)) => n,
                _ => panic!("Expected record target in record.get"),
            };
            let rec_def = &ctx.records[&rec_type_name];
            let (f_idx, _) = rec_def
                .fields
                .iter()
                .enumerate()
                .find(|(_, (n, _))| n == field_name)
                .expect("field in record definition");

            emit_expr(target_expr, params, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::I32Load(MemArg {
                offset: (f_idx * 4) as u64,
                align: 2,
                memory_index: 0,
            }));
        }
        _ => unreachable!(),
    }
}

fn infer_expr_type(expr: &Value, params: &HashMap<String, u32>, ctx: &Context) -> Option<IrType> {
    let obj = expr.as_object()?;
    let op = obj.get("op")?.as_str()?;
    match op {
        "record.construct" => {
            let rec_name = obj.get("type")?.as_str()?;
            Some(IrType::Record(rec_name.to_string()))
        }
        "param" => {
            let name = obj.get("name")?.as_str()?;
            let _local_idx = params.get(name)?;
            for sig in ctx.functions.values() {
                if let Some((_, ty)) = sig.params.iter().find(|(n, _)| n == name) {
                    return Some(ty.clone());
                }
            }
            None
        }
        "call" => {
            let func_name = obj.get("function")?.as_str()?;
            let sig = ctx.functions.get(func_name)?;
            Some(sig.returns.clone())
        }
        "call_import" => {
            let import_name = obj.get("import")?.as_str()?;
            let sig = ctx.imports.get(import_name)?;
            Some(sig.returns.clone())
        }
        "if" => infer_expr_type(obj.get("then")?, params, ctx),
        _ => None,
    }
}

/// Validates a WebAssembly binary (Core WASM or Component) using `wasmparser` (the engine behind `wasm-tools validate`).
pub fn validate_wasm_bytes(bytes: &[u8]) -> Result<(), String> {
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(bytes)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Converts a WebAssembly binary into WAT text format using `wasmprinter` (the engine behind `wasm-tools print`).
pub fn print_wasm_to_wat(bytes: &[u8]) -> Result<String, String> {
    wasmprinter::print_bytes(bytes).map_err(|e| e.to_string())
}
