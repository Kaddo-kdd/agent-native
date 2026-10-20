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
use agent_native_ir::{VerifyOptions, verify_ir_with_options};
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
    let verify_opts = VerifyOptions {
        allow_experimental: wit_ctx.allow_experimental(),
    };
    let diags = verify_ir_with_options(root, &verify_opts);
    if !diags.is_empty() {
        return Err(diags);
    }

    let ir_version = root
        .get("ir_version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.1");

    let mut record_defs: HashMap<String, RecordDef> = HashMap::new();
    let mut known_records: HashSet<String> = HashSet::new();
    if let Some(types_arr) = root.get("types").and_then(|v| v.as_array()) {
        for type_val in types_arr {
            let type_obj = type_val.as_object().unwrap();
            let name = type_obj["name"].as_str().unwrap().to_string();
            known_records.insert(name.clone());
            let mut fields = Vec::new();
            for f_val in type_obj["fields"].as_array().unwrap() {
                let f_obj = f_val.as_object().unwrap();
                let f_name = f_obj["name"].as_str().unwrap().to_string();
                let f_ty = match f_obj["type"].as_str().unwrap() {
                    "i32" => IrType::I32,
                    "bool" => IrType::Bool,
                    "string" => IrType::String,
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
                        let p_ty_str = p_obj["type"].as_str().unwrap();
                        let p_ty =
                            IrType::from_str_for_version(p_ty_str, &known_records, ir_version)
                                .unwrap_or_else(|| IrType::Record(p_ty_str.to_string()));
                        parsed_params.push((p_name, p_ty));
                    }
                }
                let ret_str = imp_obj
                    .get("returns")
                    .and_then(|v| v.as_str())
                    .unwrap_or("i32");
                let ret_ty = IrType::from_str_for_version(ret_str, &known_records, ir_version)
                    .unwrap_or_else(|| IrType::Record(ret_str.to_string()));
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
            let p_ty_str = p_val["type"].as_str().unwrap();
            let p_ty = IrType::from_str_for_version(p_ty_str, &known_records, ir_version)
                .unwrap_or_else(|| IrType::Record(p_ty_str.to_string()));
            params.push((p_name, p_ty));
        }
        let ret_str = func_obj["returns"].as_str().unwrap();
        let returns = IrType::from_str_for_version(ret_str, &known_records, ir_version)
            .unwrap_or_else(|| IrType::Record(ret_str.to_string()));
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

    let is_cm_version = ir_version == "0.4" || ir_version == "0.5" || ir_version == "0.6";

    let wit_contract_def = if is_cm_version {
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

    let needs_memory = ir_version == "0.2"
        || ir_version == "0.3"
        || ir_version == "0.6"
        || !ctx.records.is_empty();

    let mut module = Module::new();
    let mut types = TypeSection::new();
    let mut import_section = ImportSection::new();
    let mut func_section = FunctionSection::new();
    let mut exports = ExportSection::new();
    let mut code = CodeSection::new();

    for (imp_idx, imp_name) in import_order.iter().enumerate() {
        let imp_sig = &ctx.imports[imp_name];
        let mut wasm_params: Vec<ValType> = Vec::new();
        for (_, p_ty) in &imp_sig.params {
            if matches!(p_ty, IrType::String) {
                wasm_params.push(ValType::I32);
                wasm_params.push(ValType::I32);
            } else {
                wasm_params.push(ValType::I32);
            }
        }
        let returns_via_retptr = ir_version == "0.6"
            && matches!(
                imp_sig.returns,
                IrType::String | IrType::Result { .. } | IrType::Record(_)
            );
        let results: &[ValType] = if returns_via_retptr {
            wasm_params.push(ValType::I32);
            &[]
        } else {
            &[ValType::I32]
        };
        let type_idx = imp_idx as u32;
        types.ty().function(wasm_params, results.iter().copied());

        let (imp_mod, imp_field) = if is_cm_version {
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
        let body = &func_obj["body"];
        let sig = &ctx.functions[name];

        let mut param_map = HashMap::new();
        let mut param_type_map = HashMap::new();
        let mut wasm_params = Vec::new();
        let mut string_param_pairs: Vec<(String, u32, u32)> = Vec::new();

        for (p_name, p_ty) in &sig.params {
            param_type_map.insert(p_name.clone(), p_ty.clone());
            if ir_version == "0.6" && matches!(p_ty, IrType::String) {
                let ptr_idx = wasm_params.len() as u32;
                wasm_params.push(ValType::I32);
                let len_idx = wasm_params.len() as u32;
                wasm_params.push(ValType::I32);
                string_param_pairs.push((p_name.clone(), ptr_idx, len_idx));
            } else {
                let idx = wasm_params.len() as u32;
                param_map.insert(p_name.clone(), idx);
                wasm_params.push(ValType::I32);
            }
        }

        let results = [ValType::I32];
        let func_type_idx = num_imports + func_idx as u32;
        types.ty().function(wasm_params.clone(), results);
        func_section.function(func_type_idx);

        if is_export {
            exports.export(name, ExportKind::Func, num_imports + func_idx as u32);
            if ir_version == "0.6" && name.contains('_') {
                exports.export(
                    &name.replace('_', "-"),
                    ExportKind::Func,
                    num_imports + func_idx as u32,
                );
            }
            if is_cm_version {
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

        let base_local = wasm_params.len() as u32;
        if ir_version == "0.6" {
            let num_str_descs = string_param_pairs.len() as u32;
            let rec_ptr_local = base_local + num_str_descs;
            let total_extra_locals = num_str_descs + 4;
            let mut wasm_func = Function::new([(total_extra_locals, ValType::I32)]);

            for (offset, (p_name, ptr_idx, len_idx)) in string_param_pairs.iter().enumerate() {
                let desc_local = base_local + offset as u32;
                param_map.insert(p_name.clone(), desc_local);
                wasm_func.instruction(&Instruction::GlobalGet(0));
                wasm_func.instruction(&Instruction::LocalTee(desc_local));
                wasm_func.instruction(&Instruction::I32Const(8));
                wasm_func.instruction(&Instruction::I32Add);
                wasm_func.instruction(&Instruction::GlobalSet(0));

                wasm_func.instruction(&Instruction::LocalGet(desc_local));
                wasm_func.instruction(&Instruction::LocalGet(*ptr_idx));
                wasm_func.instruction(&Instruction::I32Store(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));

                wasm_func.instruction(&Instruction::LocalGet(desc_local));
                wasm_func.instruction(&Instruction::LocalGet(*len_idx));
                wasm_func.instruction(&Instruction::I32Store(MemArg {
                    offset: 4,
                    align: 2,
                    memory_index: 0,
                }));
            }

            emit_expr(
                body,
                &param_map,
                &param_type_map,
                rec_ptr_local,
                &ctx,
                &mut wasm_func,
            );
            wasm_func.instruction(&Instruction::End);
            code.function(&wasm_func);
        } else {
            // Preserve 100% byte-identical v0.1..v0.5 local layout
            let rec_ptr_local = base_local;
            let mut wasm_func = Function::new([(1, ValType::I32)]);
            emit_expr(
                body,
                &param_map,
                &param_type_map,
                rec_ptr_local,
                &ctx,
                &mut wasm_func,
            );
            wasm_func.instruction(&Instruction::End);
            code.function(&wasm_func);
        }
    }

    if ir_version == "0.6" {
        let realloc_func_idx = num_imports + functions.len() as u32;
        types.ty().function(
            [ValType::I32, ValType::I32, ValType::I32, ValType::I32],
            [ValType::I32],
        );
        func_section.function(realloc_func_idx);
        exports.export("cabi_realloc", ExportKind::Func, realloc_func_idx);

        // (param $old_ptr 0) (param $old_size 1) (param $align 2) (param $new_size 3) (local $ptr 4)
        let mut realloc_fn = Function::new([(1, ValType::I32)]);
        realloc_fn.instruction(&Instruction::GlobalGet(0));
        realloc_fn.instruction(&Instruction::LocalGet(2));
        realloc_fn.instruction(&Instruction::I32Const(1));
        realloc_fn.instruction(&Instruction::I32Sub);
        realloc_fn.instruction(&Instruction::I32Add);
        realloc_fn.instruction(&Instruction::LocalGet(2));
        realloc_fn.instruction(&Instruction::I32Const(1));
        realloc_fn.instruction(&Instruction::I32Sub);
        realloc_fn.instruction(&Instruction::I32Const(-1));
        realloc_fn.instruction(&Instruction::I32Xor);
        realloc_fn.instruction(&Instruction::I32And);
        realloc_fn.instruction(&Instruction::LocalTee(4));
        realloc_fn.instruction(&Instruction::LocalGet(3));
        realloc_fn.instruction(&Instruction::I32Add);
        realloc_fn.instruction(&Instruction::I32Const(3));
        realloc_fn.instruction(&Instruction::I32Add);
        realloc_fn.instruction(&Instruction::I32Const(-4));
        realloc_fn.instruction(&Instruction::I32And);
        realloc_fn.instruction(&Instruction::GlobalSet(0));
        realloc_fn.instruction(&Instruction::LocalGet(4));
        realloc_fn.instruction(&Instruction::End);
        code.function(&realloc_fn);
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

    if is_cm_version {
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
                // Embedded fallback for IR v0.4 compatibility only. v0.5+
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
        } else if ir_version == "0.6" {
            let wit_content = synthesize_v06_wit(&ctx, functions);
            let mut resolve = wit_parser::Resolve::new();
            let pkg = resolve
                .push_str("synthesized-v06.wit", &wit_content)
                .map_err(|e| {
                    vec![Diagnostic {
                        code: "E_WIT_PARSE_ERROR".to_string(),
                        function: None,
                        path: "$".to_string(),
                        message: format!("Failed to parse synthesized v0.6 WIT contract: {e}"),
                        expected: Some("valid WIT syntax".to_string()),
                        found: None,
                    }]
                })?;
            let world_name = "component-world";
            let world = resolve.select_world(pkg, Some(world_name)).map_err(|e| {
                vec![Diagnostic {
                    code: "E_WIT_WORLD_NOT_FOUND".to_string(),
                    function: None,
                    path: "$".to_string(),
                    message: format!("World '{world_name}' not found in synthesized WIT: {e}"),
                    expected: Some(format!("world '{world_name}'")),
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
                    path: "$".to_string(),
                    message: format!("Failed to encode synthesized WIT metadata: {e}"),
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

    if is_cm_version {
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

fn to_wit_kebab(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (idx, ch) in s.chars().enumerate() {
        if ch == '_' {
            out.push('-');
        } else if ch.is_ascii_uppercase() {
            if idx > 0 && !out.ends_with('-') {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn wit_type_name(ty: &IrType) -> String {
    match ty {
        IrType::I32 => "s32".to_string(),
        IrType::Bool => "bool".to_string(),
        IrType::String => "string".to_string(),
        IrType::Record(name) => to_wit_kebab(name),
        IrType::Result { ok, err, .. } => {
            format!("result<{}, {}>", wit_type_name(ok), wit_type_name(err))
        }
    }
}

fn synthesize_v06_wit(ctx: &Context, functions: &[Value]) -> String {
    let mut out = String::from("package kaddo:agent-native@0.6.0;\n\nworld component-world {\n");
    let mut rec_names: Vec<&String> = ctx.records.keys().collect();
    rec_names.sort();
    for r_name in rec_names {
        let rec = &ctx.records[r_name];
        out.push_str(&format!("  record {} {{\n", to_wit_kebab(r_name)));
        for (f_name, f_ty) in &rec.fields {
            out.push_str(&format!(
                "    {}: {},\n",
                to_wit_kebab(f_name),
                wit_type_name(f_ty)
            ));
        }
        out.push_str("  }\n");
    }
    for func_val in functions {
        let func_obj = func_val.as_object().unwrap();
        let is_export = func_obj
            .get("export")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if !is_export {
            continue;
        }
        let name = func_obj["name"].as_str().unwrap();
        let sig = &ctx.functions[name];
        let params_str = sig
            .params
            .iter()
            .map(|(p_name, p_ty)| format!("{}: {}", to_wit_kebab(p_name), wit_type_name(p_ty)))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "  export {}: func({}) -> {};\n",
            to_wit_kebab(name),
            params_str,
            wit_type_name(&sig.returns)
        ));
    }
    out.push_str("}\n");
    out
}

fn record_field_offsets(rec_def: &RecordDef) -> (Vec<u64>, i32) {
    let mut offsets = Vec::with_capacity(rec_def.fields.len());
    let mut cur: u64 = 0;
    for (_, f_ty) in &rec_def.fields {
        offsets.push(cur);
        cur += if matches!(f_ty, IrType::String) { 8 } else { 4 };
    }
    (offsets, cur as i32)
}

fn result_payload_offset(ok: &IrType, err: &IrType) -> u64 {
    if matches!(ok, IrType::Bool) && matches!(err, IrType::Bool) {
        1
    } else {
        4
    }
}

fn emit_expr(
    expr: &Value,
    params: &HashMap<String, u32>,
    param_types: &HashMap<String, IrType>,
    rec_ptr_local: u32,
    ctx: &Context,
    func: &mut Function,
) {
    let s0 = rec_ptr_local;
    let s1 = rec_ptr_local + 1;
    let s2 = rec_ptr_local + 2;
    let s3 = rec_ptr_local + 3;

    let obj = expr.as_object().unwrap();
    let op = obj["op"].as_str().unwrap();
    match op {
        "const" => {
            let ty = obj["type"].as_str().unwrap();
            match ty {
                "i32" => {
                    let val_i32 = obj["value"].as_i64().unwrap() as i32;
                    func.instruction(&Instruction::I32Const(val_i32));
                }
                "bool" => {
                    let val_i32 = if obj["value"].as_bool().unwrap() {
                        1
                    } else {
                        0
                    };
                    func.instruction(&Instruction::I32Const(val_i32));
                }
                "string" => {
                    let s = obj["value"].as_str().unwrap();
                    let bytes = s.as_bytes();
                    let len = bytes.len() as i32;
                    let aligned_total = (8 + len + 3) & !3;

                    // Allocate `aligned_total` bytes at `s0` (`desc_ptr`)
                    func.instruction(&Instruction::GlobalGet(0));
                    func.instruction(&Instruction::LocalTee(s0));
                    func.instruction(&Instruction::I32Const(aligned_total));
                    func.instruction(&Instruction::I32Add);
                    func.instruction(&Instruction::GlobalSet(0));

                    // [s0 + 0] = s0 + 8 (data_ptr)
                    func.instruction(&Instruction::LocalGet(s0));
                    func.instruction(&Instruction::LocalGet(s0));
                    func.instruction(&Instruction::I32Const(8));
                    func.instruction(&Instruction::I32Add);
                    func.instruction(&Instruction::I32Store(MemArg {
                        offset: 0,
                        align: 2,
                        memory_index: 0,
                    }));

                    // [s0 + 4] = len
                    func.instruction(&Instruction::LocalGet(s0));
                    func.instruction(&Instruction::I32Const(len));
                    func.instruction(&Instruction::I32Store(MemArg {
                        offset: 4,
                        align: 2,
                        memory_index: 0,
                    }));

                    for (idx, &b) in bytes.iter().enumerate() {
                        func.instruction(&Instruction::LocalGet(s0));
                        func.instruction(&Instruction::I32Const(i32::from(b)));
                        func.instruction(&Instruction::I32Store8(MemArg {
                            offset: 8 + idx as u64,
                            align: 0,
                            memory_index: 0,
                        }));
                    }

                    func.instruction(&Instruction::LocalGet(s0));
                }
                _ => unreachable!(),
            }
        }
        "param" => {
            let name = obj["name"].as_str().unwrap();
            let local_idx = params[name];
            func.instruction(&Instruction::LocalGet(local_idx));
        }
        "not" => {
            emit_expr(&obj["value"], params, param_types, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::I32Eqz);
        }
        "add" | "sub" | "mul" | "div_s" | "eq" | "lt_s" | "gte_s" | "and" | "or" => {
            if op == "eq"
                && ctx.ir_version == "0.6"
                && matches!(
                    infer_expr_type(&obj["left"], param_types, ctx),
                    Some(IrType::String)
                )
            {
                // String equality: evaluate left & right onto stack, then compare length & bytes.
                emit_expr(&obj["left"], params, param_types, rec_ptr_local, ctx, func);
                emit_expr(&obj["right"], params, param_types, rec_ptr_local, ctx, func);
                func.instruction(&Instruction::LocalSet(s1));
                func.instruction(&Instruction::LocalSet(s0));

                func.instruction(&Instruction::LocalGet(s0));
                func.instruction(&Instruction::I32Load(MemArg {
                    offset: 4,
                    align: 2,
                    memory_index: 0,
                }));
                func.instruction(&Instruction::LocalTee(s2));
                func.instruction(&Instruction::LocalGet(s1));
                func.instruction(&Instruction::I32Load(MemArg {
                    offset: 4,
                    align: 2,
                    memory_index: 0,
                }));
                func.instruction(&Instruction::I32Ne);
                func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
                func.instruction(&Instruction::I32Const(0));
                func.instruction(&Instruction::Else);
                // Lengths match (`s2`). Compare bytes `0..s2` using index `s3`.
                func.instruction(&Instruction::I32Const(0));
                func.instruction(&Instruction::LocalSet(s3));
                func.instruction(&Instruction::Block(BlockType::Result(ValType::I32)));
                func.instruction(&Instruction::Loop(BlockType::Empty));
                func.instruction(&Instruction::LocalGet(s3));
                func.instruction(&Instruction::LocalGet(s2));
                func.instruction(&Instruction::I32GeU);
                func.instruction(&Instruction::If(BlockType::Empty));
                func.instruction(&Instruction::I32Const(1));
                func.instruction(&Instruction::Br(2));
                func.instruction(&Instruction::End);

                // Load left[s3]
                func.instruction(&Instruction::LocalGet(s0));
                func.instruction(&Instruction::I32Load(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
                func.instruction(&Instruction::LocalGet(s3));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::I32Load8U(MemArg {
                    offset: 0,
                    align: 0,
                    memory_index: 0,
                }));

                // Load right[s3]
                func.instruction(&Instruction::LocalGet(s1));
                func.instruction(&Instruction::I32Load(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
                func.instruction(&Instruction::LocalGet(s3));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::I32Load8U(MemArg {
                    offset: 0,
                    align: 0,
                    memory_index: 0,
                }));

                func.instruction(&Instruction::I32Ne);
                func.instruction(&Instruction::If(BlockType::Empty));
                func.instruction(&Instruction::I32Const(0));
                func.instruction(&Instruction::Br(2));
                func.instruction(&Instruction::End);

                func.instruction(&Instruction::LocalGet(s3));
                func.instruction(&Instruction::I32Const(1));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::LocalSet(s3));
                func.instruction(&Instruction::Br(0));
                func.instruction(&Instruction::End); // end loop
                func.instruction(&Instruction::I32Const(1));
                func.instruction(&Instruction::End); // end block
                func.instruction(&Instruction::End); // end if
                return;
            }

            emit_expr(&obj["left"], params, param_types, rec_ptr_local, ctx, func);
            emit_expr(&obj["right"], params, param_types, rec_ptr_local, ctx, func);
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
            emit_expr(
                &obj["condition"],
                params,
                param_types,
                rec_ptr_local,
                ctx,
                func,
            );
            func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
            emit_expr(&obj["then"], params, param_types, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::Else);
            emit_expr(&obj["else"], params, param_types, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::End);
        }
        "call" => {
            let callee_name = obj["function"].as_str().unwrap();
            let callee_sig = &ctx.functions[callee_name];
            let args_arr = obj["args"].as_array().unwrap();
            for (arg_idx, arg_val) in args_arr.iter().enumerate() {
                emit_expr(arg_val, params, param_types, rec_ptr_local, ctx, func);
                if ctx.ir_version == "0.6" && matches!(callee_sig.params[arg_idx].1, IrType::String)
                {
                    func.instruction(&Instruction::LocalSet(s0));
                    func.instruction(&Instruction::LocalGet(s0));
                    func.instruction(&Instruction::I32Load(MemArg {
                        offset: 0,
                        align: 2,
                        memory_index: 0,
                    }));
                    func.instruction(&Instruction::LocalGet(s0));
                    func.instruction(&Instruction::I32Load(MemArg {
                        offset: 4,
                        align: 2,
                        memory_index: 0,
                    }));
                }
            }
            func.instruction(&Instruction::Call(callee_sig.index));
        }
        "call_import" => {
            let import_name = obj["import"].as_str().unwrap();
            let imp_sig = &ctx.imports[import_name];
            let args_arr = obj["args"].as_array().unwrap();
            for (arg_idx, arg_val) in args_arr.iter().enumerate() {
                emit_expr(arg_val, params, param_types, rec_ptr_local, ctx, func);
                if ctx.ir_version == "0.6" && matches!(imp_sig.params[arg_idx].1, IrType::String) {
                    func.instruction(&Instruction::LocalSet(s0));
                    func.instruction(&Instruction::LocalGet(s0));
                    func.instruction(&Instruction::I32Load(MemArg {
                        offset: 0,
                        align: 2,
                        memory_index: 0,
                    }));
                    func.instruction(&Instruction::LocalGet(s0));
                    func.instruction(&Instruction::I32Load(MemArg {
                        offset: 4,
                        align: 2,
                        memory_index: 0,
                    }));
                }
            }
            let returns_via_retptr = ctx.ir_version == "0.6"
                && matches!(
                    imp_sig.returns,
                    IrType::String | IrType::Result { .. } | IrType::Record(_)
                );
            if returns_via_retptr {
                func.instruction(&Instruction::GlobalGet(0));
                func.instruction(&Instruction::LocalTee(s0));
                func.instruction(&Instruction::GlobalGet(0));
                func.instruction(&Instruction::I32Const(16));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::GlobalSet(0));
                func.instruction(&Instruction::Call(imp_sig.index));
                func.instruction(&Instruction::LocalGet(s0));
            } else {
                func.instruction(&Instruction::Call(imp_sig.index));
            }
        }
        "record.construct" => {
            let rec_type_name = obj["type"].as_str().unwrap();
            let rec_def = &ctx.records[rec_type_name];
            let fields_map = obj["fields"].as_object().unwrap();
            let (offsets, total_size_bytes) = record_field_offsets(rec_def);

            if ctx.ir_version == "0.6"
                && rec_def
                    .fields
                    .iter()
                    .any(|(_, t)| matches!(t, IrType::String))
            {
                // Evaluate all fields onto the stack first, then allocate record & store in reverse
                for (f_name, _) in &rec_def.fields {
                    let f_expr = &fields_map[f_name];
                    emit_expr(f_expr, params, param_types, rec_ptr_local, ctx, func);
                }
                func.instruction(&Instruction::GlobalGet(0));
                func.instruction(&Instruction::LocalSet(s0));
                func.instruction(&Instruction::GlobalGet(0));
                func.instruction(&Instruction::I32Const(total_size_bytes));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::GlobalSet(0));

                for (f_idx, (_, f_ty)) in rec_def.fields.iter().enumerate().rev() {
                    let byte_offset = offsets[f_idx];
                    func.instruction(&Instruction::LocalSet(s1));
                    if matches!(f_ty, IrType::String) {
                        func.instruction(&Instruction::LocalGet(s0));
                        func.instruction(&Instruction::LocalGet(s1));
                        func.instruction(&Instruction::I32Load(MemArg {
                            offset: 0,
                            align: 2,
                            memory_index: 0,
                        }));
                        func.instruction(&Instruction::I32Store(MemArg {
                            offset: byte_offset,
                            align: 2,
                            memory_index: 0,
                        }));

                        func.instruction(&Instruction::LocalGet(s0));
                        func.instruction(&Instruction::LocalGet(s1));
                        func.instruction(&Instruction::I32Load(MemArg {
                            offset: 4,
                            align: 2,
                            memory_index: 0,
                        }));
                        func.instruction(&Instruction::I32Store(MemArg {
                            offset: byte_offset + 4,
                            align: 2,
                            memory_index: 0,
                        }));
                    } else {
                        func.instruction(&Instruction::LocalGet(s0));
                        func.instruction(&Instruction::LocalGet(s1));
                        func.instruction(&Instruction::I32Store(MemArg {
                            offset: byte_offset,
                            align: 2,
                            memory_index: 0,
                        }));
                    }
                }
                func.instruction(&Instruction::LocalGet(s0));
            } else {
                // Preserve 100% byte-identical v0.2..v0.5 instruction sequence
                func.instruction(&Instruction::GlobalGet(0));
                func.instruction(&Instruction::LocalSet(rec_ptr_local));

                func.instruction(&Instruction::GlobalGet(0));
                func.instruction(&Instruction::I32Const(total_size_bytes));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::GlobalSet(0));

                for (f_idx, (f_name, _)) in rec_def.fields.iter().enumerate() {
                    let f_expr = &fields_map[f_name];
                    let byte_offset = (f_idx * 4) as u64;

                    func.instruction(&Instruction::LocalGet(rec_ptr_local));
                    emit_expr(f_expr, params, param_types, rec_ptr_local, ctx, func);
                    func.instruction(&Instruction::I32Store(MemArg {
                        offset: byte_offset,
                        align: 2,
                        memory_index: 0,
                    }));
                }

                func.instruction(&Instruction::LocalGet(rec_ptr_local));
            }
        }
        "record.get" => {
            let target_expr = &obj["target"];
            let field_name = obj["field"].as_str().unwrap();

            let rec_type_name = match infer_expr_type(target_expr, param_types, ctx) {
                Some(IrType::Record(n)) => n,
                _ => panic!("Expected record target in record.get"),
            };
            let rec_def = &ctx.records[&rec_type_name];
            let (offsets, _) = record_field_offsets(rec_def);
            let (f_idx, (_, f_ty)) = rec_def
                .fields
                .iter()
                .enumerate()
                .find(|(_, (n, _))| n == field_name)
                .expect("field in record definition");
            let byte_offset = offsets[f_idx];

            emit_expr(target_expr, params, param_types, rec_ptr_local, ctx, func);
            if matches!(f_ty, IrType::String) {
                if byte_offset > 0 {
                    func.instruction(&Instruction::I32Const(byte_offset as i32));
                    func.instruction(&Instruction::I32Add);
                }
            } else {
                func.instruction(&Instruction::I32Load(MemArg {
                    offset: byte_offset,
                    align: 2,
                    memory_index: 0,
                }));
            }
        }
        "string.len" => {
            emit_expr(&obj["value"], params, param_types, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::I32Load(MemArg {
                offset: 4,
                align: 2,
                memory_index: 0,
            }));
        }
        "string.concat" => {
            emit_expr(&obj["left"], params, param_types, rec_ptr_local, ctx, func);
            emit_expr(&obj["right"], params, param_types, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::LocalSet(s1)); // right_desc
            func.instruction(&Instruction::LocalSet(s0)); // left_desc

            // s2 = left_len + right_len
            func.instruction(&Instruction::LocalGet(s0));
            func.instruction(&Instruction::I32Load(MemArg {
                offset: 4,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::LocalGet(s1));
            func.instruction(&Instruction::I32Load(MemArg {
                offset: 4,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::LocalSet(s2));

            // Allocate (8 + ((s2 + 3) & !3)) at s3 (`new_desc`)
            func.instruction(&Instruction::GlobalGet(0));
            func.instruction(&Instruction::LocalTee(s3));
            func.instruction(&Instruction::LocalGet(s2));
            func.instruction(&Instruction::I32Const(11));
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::I32Const(-4));
            func.instruction(&Instruction::I32And);
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::GlobalSet(0));

            // [s3 + 0] = s3 + 8
            func.instruction(&Instruction::LocalGet(s3));
            func.instruction(&Instruction::LocalGet(s3));
            func.instruction(&Instruction::I32Const(8));
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::I32Store(MemArg {
                offset: 0,
                align: 2,
                memory_index: 0,
            }));

            // [s3 + 4] = s2 (total_len)
            func.instruction(&Instruction::LocalGet(s3));
            func.instruction(&Instruction::LocalGet(s2));
            func.instruction(&Instruction::I32Store(MemArg {
                offset: 4,
                align: 2,
                memory_index: 0,
            }));

            // memory.copy(dst = s3 + 8, src = [s0 + 0], len = [s0 + 4])
            func.instruction(&Instruction::LocalGet(s3));
            func.instruction(&Instruction::I32Const(8));
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::LocalGet(s0));
            func.instruction(&Instruction::I32Load(MemArg {
                offset: 0,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::LocalGet(s0));
            func.instruction(&Instruction::I32Load(MemArg {
                offset: 4,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::MemoryCopy {
                src_mem: 0,
                dst_mem: 0,
            });

            // memory.copy(dst = s3 + 8 + [s0 + 4], src = [s1 + 0], len = [s1 + 4])
            func.instruction(&Instruction::LocalGet(s3));
            func.instruction(&Instruction::I32Const(8));
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::LocalGet(s0));
            func.instruction(&Instruction::I32Load(MemArg {
                offset: 4,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::LocalGet(s1));
            func.instruction(&Instruction::I32Load(MemArg {
                offset: 0,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::LocalGet(s1));
            func.instruction(&Instruction::I32Load(MemArg {
                offset: 4,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::MemoryCopy {
                src_mem: 0,
                dst_mem: 0,
            });

            func.instruction(&Instruction::LocalGet(s3));
        }
        "result.ok" | "result.err" => {
            let known_recs: HashSet<String> = ctx.records.keys().cloned().collect();
            let res_ty =
                IrType::from_str_for_version(obj["type"].as_str().unwrap(), &known_recs, "0.6")
                    .expect("verified result type");
            let IrType::Result { ok, err, .. } = res_ty else {
                unreachable!();
            };
            let pay_offset = result_payload_offset(&ok, &err);
            let branch_ty = if op == "result.ok" { *ok } else { *err };
            let discr = if op == "result.ok" { 0 } else { 1 };

            emit_expr(&obj["value"], params, param_types, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::LocalSet(s0));

            // Allocate 12 bytes at s1 (`res_ptr`)
            func.instruction(&Instruction::GlobalGet(0));
            func.instruction(&Instruction::LocalTee(s1));
            func.instruction(&Instruction::I32Const(12));
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::GlobalSet(0));

            // Store discriminant at [s1 + 0]
            func.instruction(&Instruction::LocalGet(s1));
            func.instruction(&Instruction::I32Const(discr));
            if pay_offset == 1 {
                func.instruction(&Instruction::I32Store8(MemArg {
                    offset: 0,
                    align: 0,
                    memory_index: 0,
                }));
            } else {
                func.instruction(&Instruction::I32Store(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
            }

            if matches!(branch_ty, IrType::String) {
                func.instruction(&Instruction::LocalGet(s1));
                func.instruction(&Instruction::LocalGet(s0));
                func.instruction(&Instruction::I32Load(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
                func.instruction(&Instruction::I32Store(MemArg {
                    offset: pay_offset,
                    align: 2,
                    memory_index: 0,
                }));

                func.instruction(&Instruction::LocalGet(s1));
                func.instruction(&Instruction::LocalGet(s0));
                func.instruction(&Instruction::I32Load(MemArg {
                    offset: 4,
                    align: 2,
                    memory_index: 0,
                }));
                func.instruction(&Instruction::I32Store(MemArg {
                    offset: pay_offset + 4,
                    align: 2,
                    memory_index: 0,
                }));
            } else if pay_offset == 1 {
                func.instruction(&Instruction::LocalGet(s1));
                func.instruction(&Instruction::LocalGet(s0));
                func.instruction(&Instruction::I32Store8(MemArg {
                    offset: 1,
                    align: 0,
                    memory_index: 0,
                }));
            } else {
                func.instruction(&Instruction::LocalGet(s1));
                func.instruction(&Instruction::LocalGet(s0));
                func.instruction(&Instruction::I32Store(MemArg {
                    offset: pay_offset,
                    align: 2,
                    memory_index: 0,
                }));
            }

            func.instruction(&Instruction::LocalGet(s1));
        }
        "result.is_ok" => {
            emit_expr(&obj["value"], params, param_types, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::I32Load8U(MemArg {
                offset: 0,
                align: 0,
                memory_index: 0,
            }));
            func.instruction(&Instruction::I32Eqz);
        }
        "result.unwrap_ok" | "result.unwrap_err" => {
            let Some(IrType::Result { ok, err, .. }) =
                infer_expr_type(&obj["value"], param_types, ctx)
            else {
                panic!("Expected result target in {op}");
            };
            let pay_offset = result_payload_offset(&ok, &err);
            let branch_ty = if op == "result.unwrap_ok" { *ok } else { *err };

            emit_expr(&obj["value"], params, param_types, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::LocalTee(s0));
            func.instruction(&Instruction::I32Load8U(MemArg {
                offset: 0,
                align: 0,
                memory_index: 0,
            }));
            if op == "result.unwrap_ok" {
                func.instruction(&Instruction::I32Eqz);
            }
            func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
            if matches!(branch_ty, IrType::String) {
                func.instruction(&Instruction::LocalGet(s0));
                func.instruction(&Instruction::I32Const(pay_offset as i32));
                func.instruction(&Instruction::I32Add);
            } else if pay_offset == 1 {
                func.instruction(&Instruction::LocalGet(s0));
                func.instruction(&Instruction::I32Load8U(MemArg {
                    offset: 1,
                    align: 0,
                    memory_index: 0,
                }));
            } else {
                func.instruction(&Instruction::LocalGet(s0));
                func.instruction(&Instruction::I32Load(MemArg {
                    offset: pay_offset,
                    align: 2,
                    memory_index: 0,
                }));
            }
            func.instruction(&Instruction::Else);
            emit_expr(
                &obj["default"],
                params,
                param_types,
                rec_ptr_local,
                ctx,
                func,
            );
            func.instruction(&Instruction::End);
        }
        _ => unreachable!(),
    }
}

fn infer_expr_type(
    expr: &Value,
    param_types: &HashMap<String, IrType>,
    ctx: &Context,
) -> Option<IrType> {
    let obj = expr.as_object()?;
    let op = obj.get("op")?.as_str()?;
    match op {
        "const" => match obj.get("type")?.as_str()? {
            "i32" => Some(IrType::I32),
            "bool" => Some(IrType::Bool),
            "string" => Some(IrType::String),
            _ => None,
        },
        "record.construct" => {
            let rec_name = obj.get("type")?.as_str()?;
            Some(IrType::Record(rec_name.to_string()))
        }
        "record.get" => {
            let target_ty = infer_expr_type(obj.get("target")?, param_types, ctx)?;
            let IrType::Record(rec_name) = target_ty else {
                return None;
            };
            let field_name = obj.get("field")?.as_str()?;
            let rec_def = ctx.records.get(&rec_name)?;
            rec_def
                .fields
                .iter()
                .find(|(n, _)| n == field_name)
                .map(|(_, ty)| ty.clone())
        }
        "param" => {
            let name = obj.get("name")?.as_str()?;
            param_types.get(name).cloned()
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
        "if" => infer_expr_type(obj.get("then")?, param_types, ctx),
        "string.len" => Some(IrType::I32),
        "string.concat" => Some(IrType::String),
        "result.ok" | "result.err" => {
            let known_recs: HashSet<String> = ctx.records.keys().cloned().collect();
            IrType::from_str_for_version(obj.get("type")?.as_str()?, &known_recs, "0.6")
        }
        "result.is_ok" => Some(IrType::Bool),
        "result.unwrap_ok" => match infer_expr_type(obj.get("value")?, param_types, ctx)? {
            IrType::Result { ok, .. } => Some(*ok),
            _ => None,
        },
        "result.unwrap_err" => match infer_expr_type(obj.get("value")?, param_types, ctx)? {
            IrType::Result { err, .. } => Some(*err),
            _ => None,
        },
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
