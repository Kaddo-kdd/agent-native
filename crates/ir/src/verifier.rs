//! Semantic IR v0.5 verifier.
//!
//! Migrated verbatim from `_reference/poc-agent-native/crates/irtool/src/lib.rs`
//! (lines 97-1690), per matriz de extracción C-01 and ADR-003. Preserves the
//! POC semantics 1:1 including diagnostic codes, path localisation and the
//! `check_expr` recursive type-checking strategy.
//!
//! Public entry point: [`verify_ir`]. The remaining helpers are private.
//!
//! The following `clippy::pedantic` and `clippy::style` lints are intentionally
//! allowed here: refactoring the POC code is explicitly out of scope for AN-006
//! (see WI "Out of scope"). Any cleanup lives in a follow-up Work Item that
//! brings tests first.
#![allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    clippy::must_use_candidate,
    clippy::match_single_binding,
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
    clippy::checked_conversions
)]

use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::context::Context;
use crate::diagnostic::Diagnostic;
use crate::model::{FuncSignature, ImportSignature, IrType, RecordDef, WitContractDef};

fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Options controlling Semantic IR verification (INIT-07-02 / ADR-003).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VerifyOptions {
    /// Permits experimental `ir_version: "0.6"` features (`string`, `result<T, E>`).
    /// Defaults to `false` so `v0.5` remains the strict default baseline.
    pub allow_experimental: bool,
}

impl VerifyOptions {
    /// Enables experimental IR v0.6 verification (`--experimental`).
    #[must_use]
    pub const fn experimental() -> Self {
        Self {
            allow_experimental: true,
        }
    }
}

/// Validates a Semantic IR v0.1..v0.5 JSON document and returns all diagnostics.
/// An empty vector means the IR is both structurally and semantically valid.
///
/// Experimental `ir_version: "0.6"` documents are rejected with
/// `E_EXPERIMENTAL_IR_VERSION` unless [`verify_ir_with_options`] is called with
/// [`VerifyOptions::experimental`].
pub fn verify_ir(root: &Value) -> Vec<Diagnostic> {
    verify_ir_with_options(root, &VerifyOptions::default())
}

/// Validates a Semantic IR JSON document under explicit [`VerifyOptions`].
pub fn verify_ir_with_options(root: &Value, options: &VerifyOptions) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let Some(obj) = root.as_object() else {
        diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$".to_string(),
            message: "Root of IR document must be a JSON object.".to_string(),
            expected: Some("object".to_string()),
            found: Some(value_kind(root).to_string()),
        });
        return diags;
    };

    let ir_version = match obj.get("ir_version").and_then(|v| v.as_str()) {
        Some("0.1") => "0.1",
        Some("0.2") => "0.2",
        Some("0.3") => "0.3",
        Some("0.4") => "0.4",
        Some("0.5") => "0.5",
        Some("0.6") => {
            if !options.allow_experimental {
                diags.push(Diagnostic {
                    code: "E_EXPERIMENTAL_IR_VERSION".to_string(),
                    function: None,
                    path: "$.ir_version".to_string(),
                    message:
                        "Semantic IR v0.6 is experimental and requires explicit opt-in (--experimental)."
                            .to_string(),
                    expected: Some("\"0.5\" (stable) or \"0.6\" with --experimental".to_string()),
                    found: Some("0.6".to_string()),
                });
                return diags;
            }
            "0.6"
        }
        Some(other) => {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: None,
                path: "$.ir_version".to_string(),
                message: "ir_version must be \"0.1\", \"0.2\", \"0.3\", \"0.4\", or \"0.5\"."
                    .to_string(),
                expected: Some("\"0.1\" | \"0.2\" | \"0.3\" | \"0.4\" | \"0.5\"".to_string()),
                found: Some(other.to_string()),
            });
            "0.1"
        }
        None => {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: None,
                path: "$.ir_version".to_string(),
                message: "Missing required property 'ir_version'.".to_string(),
                expected: Some("\"0.1\" | \"0.2\" | \"0.3\" | \"0.4\"".to_string()),
                found: None,
            });
            "0.1"
        }
    };

    for key in obj.keys() {
        let is_allowed = match ir_version {
            "0.1" => matches!(key.as_str(), "ir_version" | "spec_version" | "functions"),
            "0.2" => matches!(
                key.as_str(),
                "ir_version" | "spec_version" | "types" | "functions"
            ),
            "0.3" => matches!(
                key.as_str(),
                "ir_version" | "spec_version" | "capabilities" | "imports" | "types" | "functions"
            ),
            "0.4" | "0.5" | "0.6" => matches!(
                key.as_str(),
                "ir_version"
                    | "spec_version"
                    | "wit_contract"
                    | "capabilities"
                    | "imports"
                    | "types"
                    | "functions"
            ),
            _ => false,
        };
        if !is_allowed {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: None,
                path: format!("$.{key}"),
                message: format!("Unknown root property '{key}'."),
                expected: Some(if ir_version == "0.1" {
                    "ir_version | spec_version | functions".to_string()
                } else if ir_version == "0.2" {
                    "ir_version | spec_version | types | functions".to_string()
                } else if ir_version == "0.3" {
                    "ir_version | spec_version | capabilities | imports | types | functions".to_string()
                } else {
                    "ir_version | spec_version | wit_contract | capabilities | imports | types | functions".to_string()
                }),
                found: Some(key.clone()),
            });
        }
    }

    match obj.get("spec_version") {
        Some(Value::Number(n)) if n.as_i64().is_some_and(|v| v >= 1) => {}
        Some(other) => diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$.spec_version".to_string(),
            message: "spec_version must be an integer >= 1.".to_string(),
            expected: Some("integer >= 1".to_string()),
            found: Some(other.to_string()),
        }),
        None => diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$.spec_version".to_string(),
            message: "Missing required property 'spec_version'.".to_string(),
            expected: Some("integer >= 1".to_string()),
            found: None,
        }),
    }

    let mut wit_contract_def = None;
    if ir_version == "0.4" || ir_version == "0.5" || ir_version == "0.6" {
        if let Some(wit_val) = obj.get("wit_contract") {
            if let Some(wit_obj) = wit_val.as_object() {
                let pkg = wit_obj
                    .get("package")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let world = wit_obj.get("world").and_then(|v| v.as_str()).unwrap_or("");
                let exp_iface = wit_obj
                    .get("export_interface")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let mut imp_ifaces = Vec::new();
                if let Some(arr) = wit_obj.get("import_interfaces").and_then(|v| v.as_array()) {
                    for iface_val in arr {
                        if let Some(s) = iface_val.as_str() {
                            imp_ifaces.push(s.to_string());
                        }
                    }
                }
                let wit_file = wit_obj
                    .get("wit_file")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                if pkg.is_empty() {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: None,
                        path: "$.wit_contract.package".to_string(),
                        message: "Missing 'package' in wit_contract.".to_string(),
                        expected: Some("package string (e.g. kaddo:commercial@0.1.0)".to_string()),
                        found: None,
                    });
                }
                if exp_iface.is_empty() {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: None,
                        path: "$.wit_contract.export_interface".to_string(),
                        message: "Missing 'export_interface' in wit_contract.".to_string(),
                        expected: Some("export_interface name".to_string()),
                        found: None,
                    });
                }
                wit_contract_def = Some(WitContractDef {
                    package: pkg.to_string(),
                    world: world.to_string(),
                    export_interface: exp_iface.to_string(),
                    import_interfaces: imp_ifaces,
                    wit_file,
                });
            } else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: None,
                    path: "$.wit_contract".to_string(),
                    message: "wit_contract must be an object.".to_string(),
                    expected: Some("object".to_string()),
                    found: Some(value_kind(wit_val).to_string()),
                });
            }
        } else if ir_version == "0.4" || ir_version == "0.5" {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: None,
                path: "$.wit_contract".to_string(),
                message: "Missing required property 'wit_contract' in IR v0.4.".to_string(),
                expected: Some("wit_contract object".to_string()),
                found: None,
            });
        }
    }

    let mut record_defs: HashMap<String, RecordDef> = HashMap::new();
    let mut known_record_names: HashSet<String> = HashSet::new();

    if let Some(types_val) = obj.get("types") {
        if let Some(types_arr) = types_val.as_array() {
            for (t_idx, type_val) in types_arr.iter().enumerate() {
                let path = format!("$.types[{t_idx}]");
                let Some(type_obj) = type_val.as_object() else {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: None,
                        path,
                        message: "Each type declaration must be a JSON object.".to_string(),
                        expected: Some("object".to_string()),
                        found: Some(value_kind(type_val).to_string()),
                    });
                    continue;
                };

                let name = match type_obj.get("name").and_then(|v| v.as_str()) {
                    Some(n) if is_valid_identifier(n) => n.to_string(),
                    Some(n) => {
                        diags.push(Diagnostic {
                            code: "E_SCHEMA_INVALID".to_string(),
                            function: None,
                            path: format!("{path}.name"),
                            message: format!("Type name '{n}' is not a valid identifier."),
                            expected: Some("^[A-Za-z_][A-Za-z0-9_]*$".to_string()),
                            found: Some(n.to_string()),
                        });
                        continue;
                    }
                    None => {
                        diags.push(Diagnostic {
                            code: "E_SCHEMA_INVALID".to_string(),
                            function: None,
                            path: format!("{path}.name"),
                            message: "Missing string property 'name' in type declaration."
                                .to_string(),
                            expected: Some("identifier string".to_string()),
                            found: None,
                        });
                        continue;
                    }
                };

                if known_record_names.contains(&name) {
                    diags.push(Diagnostic {
                        code: "E_DUPLICATE_TYPE".to_string(),
                        function: None,
                        path: format!("{path}.name"),
                        message: format!("Duplicate type name '{name}'."),
                        expected: Some("unique type name".to_string()),
                        found: Some(name.clone()),
                    });
                    continue;
                }
                known_record_names.insert(name.clone());

                let fields_arr = match type_obj.get("fields").and_then(|v| v.as_array()) {
                    Some(arr) if !arr.is_empty() => arr,
                    _ => {
                        diags.push(Diagnostic {
                            code: "E_SCHEMA_INVALID".to_string(),
                            function: None,
                            path: format!("{path}.fields"),
                            message: format!("Type '{name}' must have a non-empty 'fields' array."),
                            expected: Some("non-empty array".to_string()),
                            found: type_obj.get("fields").map(|v| value_kind(v).to_string()),
                        });
                        continue;
                    }
                };

                let mut field_names = HashSet::new();
                let mut parsed_fields = Vec::new();

                for (f_idx, f_val) in fields_arr.iter().enumerate() {
                    let f_path = format!("{path}.fields[{f_idx}]");
                    let Some(f_obj) = f_val.as_object() else {
                        diags.push(Diagnostic {
                            code: "E_SCHEMA_INVALID".to_string(),
                            function: None,
                            path: f_path,
                            message: "Field must be a JSON object.".to_string(),
                            expected: Some("object".to_string()),
                            found: Some(value_kind(f_val).to_string()),
                        });
                        continue;
                    };

                    let f_name = match f_obj.get("name").and_then(|v| v.as_str()) {
                        Some(fnm) if is_valid_identifier(fnm) => fnm.to_string(),
                        Some(fnm) => {
                            diags.push(Diagnostic {
                                code: "E_SCHEMA_INVALID".to_string(),
                                function: None,
                                path: format!("{f_path}.name"),
                                message: format!("Field name '{fnm}' is not a valid identifier."),
                                expected: Some("^[A-Za-z_][A-Za-z0-9_]*$".to_string()),
                                found: Some(fnm.to_string()),
                            });
                            continue;
                        }
                        None => {
                            diags.push(Diagnostic {
                                code: "E_SCHEMA_INVALID".to_string(),
                                function: None,
                                path: format!("{f_path}.name"),
                                message: "Missing string property 'name' in field.".to_string(),
                                expected: Some("identifier string".to_string()),
                                found: None,
                            });
                            continue;
                        }
                    };

                    if field_names.contains(&f_name) {
                        diags.push(Diagnostic {
                            code: "E_DUPLICATE_PARAM".to_string(),
                            function: None,
                            path: format!("{f_path}.name"),
                            message: format!(
                                "Duplicate field name '{f_name}' in record type '{name}'."
                            ),
                            expected: Some("unique field name".to_string()),
                            found: Some(f_name.clone()),
                        });
                    }
                    field_names.insert(f_name.clone());

                    let f_type_str = f_obj.get("type").and_then(|v| v.as_str());
                    let f_type = match f_type_str {
                        Some("i32") => IrType::I32,
                        Some("bool") => IrType::Bool,
                        Some("string") if ir_version == "0.6" => IrType::String,
                        Some(other) => {
                            diags.push(Diagnostic {
                                code: "E_UNKNOWN_TYPE".to_string(),
                                function: None,
                                path: format!("{f_path}.type"),
                                message: format!("Unsupported or unknown field type '{other}'."),
                                expected: Some(if ir_version == "0.6" {
                                    "i32 | bool | string".to_string()
                                } else {
                                    "i32 | bool".to_string()
                                }),
                                found: Some(other.to_string()),
                            });
                            IrType::I32
                        }
                        None => {
                            diags.push(Diagnostic {
                                code: "E_SCHEMA_INVALID".to_string(),
                                function: None,
                                path: format!("{f_path}.type"),
                                message: "Missing 'type' in field declaration.".to_string(),
                                expected: Some("i32 | bool".to_string()),
                                found: None,
                            });
                            IrType::I32
                        }
                    };

                    parsed_fields.push((f_name, f_type));
                }

                record_defs.insert(
                    name.clone(),
                    RecordDef {
                        name,
                        fields: parsed_fields,
                    },
                );
            }
        }
    }

    let Some(functions_val) = obj.get("functions") else {
        diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$.functions".to_string(),
            message: "Missing required property 'functions'.".to_string(),
            expected: Some("non-empty array".to_string()),
            found: None,
        });
        return diags;
    };

    let Some(functions) = functions_val.as_array() else {
        diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$.functions".to_string(),
            message: "'functions' must be an array.".to_string(),
            expected: Some("array".to_string()),
            found: Some(value_kind(functions_val).to_string()),
        });
        return diags;
    };

    if functions.is_empty() {
        diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$.functions".to_string(),
            message: "'functions' array must contain at least one function.".to_string(),
            expected: Some("non-empty array".to_string()),
            found: Some("[]".to_string()),
        });
        return diags;
    }

    let mut function_signatures: HashMap<String, FuncSignature> = HashMap::new();
    let mut func_names = HashSet::new();

    for (f_idx, func_val) in functions.iter().enumerate() {
        let f_path = format!("$.functions[{f_idx}]");
        let Some(func_obj) = func_val.as_object() else {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: None,
                path: f_path,
                message: "Function entry must be a JSON object.".to_string(),
                expected: Some("object".to_string()),
                found: Some(value_kind(func_val).to_string()),
            });
            continue;
        };

        let name = match func_obj.get("name").and_then(|v| v.as_str()) {
            Some(n) if is_valid_identifier(n) => n.to_string(),
            Some(n) => {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: None,
                    path: format!("{f_path}.name"),
                    message: format!("Function name '{n}' is not a valid identifier."),
                    expected: Some("^[A-Za-z_][A-Za-z0-9_]*$".to_string()),
                    found: Some(n.to_string()),
                });
                format!("$anon_{f_idx}")
            }
            None => {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: None,
                    path: format!("{f_path}.name"),
                    message: "Missing string property 'name' in function.".to_string(),
                    expected: Some("identifier string".to_string()),
                    found: None,
                });
                format!("$anon_{f_idx}")
            }
        };

        if func_names.contains(&name) {
            diags.push(Diagnostic {
                code: "E_DUPLICATE_FUNCTION".to_string(),
                function: Some(name.clone()),
                path: format!("{f_path}.name"),
                message: format!("Duplicate function name '{name}'."),
                expected: Some("unique function name".to_string()),
                found: Some(name.clone()),
            });
        }
        func_names.insert(name.clone());

        let is_export = func_obj
            .get("export")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let returns_ty = match func_obj.get("returns").and_then(|v| v.as_str()) {
            Some(s) => match IrType::from_str_for_version(s, &known_record_names, ir_version) {
                Some(ty) => ty,
                None => {
                    diags.push(Diagnostic {
                        code: "E_UNKNOWN_TYPE".to_string(),
                        function: Some(name.clone()),
                        path: format!("{f_path}.returns"),
                        message: format!("Unknown return type '{s}' in function '{name}'."),
                        expected: Some(if ir_version == "0.6" {
                            "i32 | bool | string | result<T, E> | declared record type".to_string()
                        } else {
                            "i32 | bool | declared record type".to_string()
                        }),
                        found: Some(s.to_string()),
                    });
                    IrType::I32
                }
            },
            None => {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: Some(name.clone()),
                    path: format!("{f_path}.returns"),
                    message: "Missing 'returns' property in function.".to_string(),
                    expected: Some("type string".to_string()),
                    found: None,
                });
                IrType::I32
            }
        };

        let mut parsed_params = Vec::new();
        let mut param_names = HashSet::new();

        if let Some(params_arr) = func_obj.get("params").and_then(|v| v.as_array()) {
            for (p_idx, p_val) in params_arr.iter().enumerate() {
                let p_path = format!("{f_path}.params[{p_idx}]");
                let Some(p_obj) = p_val.as_object() else {
                    continue;
                };
                let p_name = match p_obj.get("name").and_then(|v| v.as_str()) {
                    Some(pn) if is_valid_identifier(pn) => pn.to_string(),
                    _ => continue,
                };
                if param_names.contains(&p_name) {
                    diags.push(Diagnostic {
                        code: "E_DUPLICATE_PARAM".to_string(),
                        function: Some(name.clone()),
                        path: format!("{p_path}.name"),
                        message: format!("Duplicate parameter '{p_name}' in function '{name}'."),
                        expected: Some("unique parameter name".to_string()),
                        found: Some(p_name.clone()),
                    });
                }
                param_names.insert(p_name.clone());

                let p_type = match p_obj.get("type").and_then(|v| v.as_str()) {
                    Some(s) => {
                        match IrType::from_str_for_version(s, &known_record_names, ir_version) {
                            Some(ty) => ty,
                            None => {
                                diags.push(Diagnostic {
                                code: "E_UNKNOWN_TYPE".to_string(),
                                function: Some(name.clone()),
                                path: format!("{p_path}.type"),
                                message: format!(
                                    "Unknown parameter type '{s}' for parameter '{p_name}' in function '{name}'."
                                ),
                                expected: Some(if ir_version == "0.6" {
                                    "i32 | bool | string | result<T, E> | declared record type"
                                        .to_string()
                                } else {
                                    "i32 | bool | declared record type".to_string()
                                }),
                                found: Some(s.to_string()),
                            });
                                IrType::I32
                            }
                        }
                    }
                    None => IrType::I32,
                };
                parsed_params.push((p_name, p_type));
            }
        }

        let wit_interface = func_obj
            .get("wit_interface")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if (ir_version == "0.4" || ir_version == "0.5" || ir_version == "0.6") && is_export {
            if let Some(ref wit_c) = wit_contract_def {
                match &wit_interface {
                    Some(iface) if iface == &wit_c.export_interface => {}
                    Some(iface) => {
                        diags.push(Diagnostic {
                            code: "E_UNKNOWN_EXPORT_INTERFACE".to_string(),
                            function: Some(name.clone()),
                            path: format!("{f_path}.wit_interface"),
                            message: format!(
                                "Exported function '{name}' specifies wit_interface '{iface}', but wit_contract exports '{}'.",
                                wit_c.export_interface
                            ),
                            expected: Some(wit_c.export_interface.clone()),
                            found: Some(iface.clone()),
                        });
                    }
                    None => {
                        diags.push(Diagnostic {
                            code: "E_UNKNOWN_EXPORT_INTERFACE".to_string(),
                            function: Some(name.clone()),
                            path: format!("{f_path}.wit_interface"),
                            message: format!(
                                "Exported function '{name}' in IR v{ir_version} must declare wit_interface matching '{}'.",
                                wit_c.export_interface
                            ),
                            expected: Some(wit_c.export_interface.clone()),
                            found: None,
                        });
                    }
                }
            }
        }

        function_signatures.insert(
            name.clone(),
            FuncSignature {
                name,
                export: is_export,
                wit_interface,
                params: parsed_params,
                returns: returns_ty,
                index: f_idx as u32,
            },
        );
    }

    let mut capabilities = HashSet::new();
    if let Some(caps_val) = obj.get("capabilities") {
        if let Some(caps_arr) = caps_val.as_array() {
            for (c_idx, c_val) in caps_arr.iter().enumerate() {
                if let Some(s) = c_val.as_str() {
                    capabilities.insert(s.to_string());
                } else {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: None,
                        path: format!("$.capabilities[{c_idx}]"),
                        message: "Capability name must be a string.".to_string(),
                        expected: Some("string".to_string()),
                        found: Some(value_kind(c_val).to_string()),
                    });
                }
            }
        }
    }

    let mut imports: HashMap<String, ImportSignature> = HashMap::new();
    let mut import_order: Vec<String> = Vec::new();
    if let Some(imp_val) = obj.get("imports") {
        if let Some(imp_arr) = imp_val.as_array() {
            for (i_idx, item_val) in imp_arr.iter().enumerate() {
                let path = format!("$.imports[{i_idx}]");
                let Some(imp_obj) = item_val.as_object() else {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: None,
                        path,
                        message: "Import must be a JSON object.".to_string(),
                        expected: Some("object".to_string()),
                        found: Some(value_kind(item_val).to_string()),
                    });
                    continue;
                };

                let module = imp_obj.get("module").and_then(|v| v.as_str()).unwrap_or("");
                let name = imp_obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let capability = imp_obj
                    .get("capability")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if !capabilities.contains(capability) {
                    diags.push(Diagnostic {
                        code: "E_UNDECLARED_CAPABILITY".to_string(),
                        function: None,
                        path: format!("{path}.capability"),
                        message: format!(
                            "Import '{name}' requires capability '{capability}', which is not declared in root 'capabilities'."
                        ),
                        expected: Some("declared capability in root 'capabilities'".to_string()),
                        found: Some(capability.to_string()),
                    });
                }

                let mut parsed_params = Vec::new();
                if let Some(p_arr) = imp_obj.get("params").and_then(|v| v.as_array()) {
                    for (p_idx, p_val) in p_arr.iter().enumerate() {
                        let p_path = format!("{path}.params[{p_idx}]");
                        if let Some(p_obj) = p_val.as_object() {
                            let p_name = p_obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let p_ty_str = p_obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            let p_type = match IrType::from_str_for_version(
                                p_ty_str,
                                &known_record_names,
                                ir_version,
                            ) {
                                Some(t) => t,
                                None => {
                                    diags.push(Diagnostic {
                                        code: "E_UNKNOWN_TYPE".to_string(),
                                        function: None,
                                        path: format!("{p_path}.type"),
                                        message: format!(
                                            "Unknown type '{p_ty_str}' for import param '{p_name}'."
                                        ),
                                        expected: Some("valid type".to_string()),
                                        found: Some(p_ty_str.to_string()),
                                    });
                                    IrType::I32
                                }
                            };
                            parsed_params.push((p_name.to_string(), p_type));
                        }
                    }
                }

                let ret_str = imp_obj
                    .get("returns")
                    .and_then(|v| v.as_str())
                    .unwrap_or("i32");
                let ret_type =
                    match IrType::from_str_for_version(ret_str, &known_record_names, ir_version) {
                        Some(t) => t,
                        None => {
                            diags.push(Diagnostic {
                                code: "E_UNKNOWN_TYPE".to_string(),
                                function: None,
                                path: format!("{path}.returns"),
                                message: format!(
                                    "Unknown return type '{ret_str}' for import '{name}'."
                                ),
                                expected: Some("valid type".to_string()),
                                found: Some(ret_str.to_string()),
                            });
                            IrType::I32
                        }
                    };

                let wit_interface = imp_obj
                    .get("wit_interface")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                imports.insert(
                    name.to_string(),
                    ImportSignature {
                        module: module.to_string(),
                        name: name.to_string(),
                        capability: capability.to_string(),
                        wit_interface,
                        params: parsed_params,
                        returns: ret_type,
                        index: i_idx as u32,
                    },
                );
                import_order.push(name.to_string());
            }
        }
    }

    let num_imports = import_order.len() as u32;

    let ctx = Context {
        ir_version: ir_version.to_string(),
        wit_contract: wit_contract_def,
        capabilities,
        imports,
        import_order,
        records: record_defs,
        functions: function_signatures,
        num_imports,
    };

    // Second pass: typecheck function bodies
    for (f_idx, func_val) in functions.iter().enumerate() {
        let f_path = format!("$.functions[{f_idx}]");
        let Some(func_obj) = func_val.as_object() else {
            continue;
        };
        let Some(name) = func_obj.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let sig = &ctx.functions[name];

        let reqs = func_obj.get("requirements").and_then(|v| v.as_array());
        if sig.export && (reqs.is_none() || reqs.unwrap().is_empty()) {
            diags.push(Diagnostic {
                code: "E_MISSING_REQUIREMENT".to_string(),
                function: Some(name.to_string()),
                path: format!("{f_path}.requirements"),
                message: format!(
                    "Exported function '{name}' must declare at least one non-empty requirement ID."
                ),
                expected: Some("non-empty array of requirement strings".to_string()),
                found: reqs.map(|_| "[]".to_string()),
            });
        }

        let mut param_map: HashMap<String, IrType> = HashMap::new();
        for (p_name, p_type) in &sig.params {
            param_map.insert(p_name.clone(), p_type.clone());
        }

        let Some(body_val) = func_obj.get("body") else {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: Some(name.to_string()),
                path: format!("{f_path}.body"),
                message: format!("Function '{name}' is missing 'body'."),
                expected: Some("expression object".to_string()),
                found: None,
            });
            continue;
        };

        if let Some(body_ty) = check_expr(
            body_val,
            &format!("{f_path}.body"),
            Some(name),
            &param_map,
            &ctx,
            &mut diags,
        ) {
            if body_ty != sig.returns {
                diags.push(Diagnostic {
                    code: "E_RETURN_TYPE".to_string(),
                    function: Some(name.to_string()),
                    path: format!("{f_path}.body"),
                    message: format!(
                        "Function '{name}' body produces type '{}', but declares returns '{}'.",
                        body_ty.as_str(),
                        sig.returns.as_str()
                    ),
                    expected: Some(sig.returns.as_str().to_string()),
                    found: Some(body_ty.as_str().to_string()),
                });
            }
        }
    }

    diags
}

fn check_expr(
    expr: &Value,
    path: &str,
    func_name: Option<&str>,
    params: &HashMap<String, IrType>,
    ctx: &Context,
    diags: &mut Vec<Diagnostic>,
) -> Option<IrType> {
    let Some(obj) = expr.as_object() else {
        diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: func_name.map(ToOwned::to_owned),
            path: path.to_string(),
            message: "Expression must be a JSON object.".to_string(),
            expected: Some("object".to_string()),
            found: Some(value_kind(expr).to_string()),
        });
        return None;
    };

    let Some(op) = obj.get("op").and_then(|v| v.as_str()) else {
        diags.push(Diagnostic {
            code: "E_UNSUPPORTED_OP".to_string(),
            function: func_name.map(ToOwned::to_owned),
            path: format!("{path}.op"),
            message: "Expression is missing a string 'op' field.".to_string(),
            expected: Some("valid operator string".to_string()),
            found: obj.get("op").map(ToString::to_string),
        });
        return None;
    };

    match op {
        "const" => {
            let type_str = obj.get("type").and_then(|v| v.as_str());
            let ir_ty = match type_str {
                Some("i32") => IrType::I32,
                Some("bool") => IrType::Bool,
                Some("string") if ctx.ir_version == "0.6" => IrType::String,
                _ => {
                    diags.push(Diagnostic {
                        code: "E_CONST_TYPE".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.type"),
                        message: if ctx.ir_version == "0.6" {
                            "Const expression 'type' must be 'i32', 'bool', or 'string'."
                                .to_string()
                        } else {
                            "Const expression 'type' must be 'i32' or 'bool'.".to_string()
                        },
                        expected: Some(if ctx.ir_version == "0.6" {
                            "i32 | bool | string".to_string()
                        } else {
                            "i32 | bool".to_string()
                        }),
                        found: obj.get("type").map(ToString::to_string),
                    });
                    return None;
                }
            };

            let Some(val) = obj.get("value") else {
                diags.push(Diagnostic {
                    code: "E_CONST_TYPE".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.value"),
                    message: "Const expression is missing 'value'.".to_string(),
                    expected: Some(ir_ty.as_str().to_string()),
                    found: None,
                });
                return None;
            };

            match (&ir_ty, val) {
                (IrType::I32, Value::Number(n)) => {
                    if let Some(i) = n.as_i64() {
                        if i >= i64::from(i32::MIN) && i <= i64::from(i32::MAX) {
                            Some(IrType::I32)
                        } else {
                            diags.push(Diagnostic {
                                code: "E_CONST_TYPE".to_string(),
                                function: func_name.map(ToOwned::to_owned),
                                path: format!("{path}.value"),
                                message: format!("Integer literal {i} is out of signed i32 range."),
                                expected: Some("i32 (-2147483648..=2147483647)".to_string()),
                                found: Some(i.to_string()),
                            });
                            None
                        }
                    } else {
                        diags.push(Diagnostic {
                            code: "E_CONST_TYPE".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: format!("{path}.value"),
                            message: "Const of type 'i32' requires a signed integer value."
                                .to_string(),
                            expected: Some("integer".to_string()),
                            found: Some(val.to_string()),
                        });
                        None
                    }
                }
                (IrType::Bool, Value::Bool(_)) => Some(IrType::Bool),
                (IrType::String, Value::String(_)) => Some(IrType::String),
                (expected_ty, other_val) => {
                    diags.push(Diagnostic {
                        code: "E_CONST_TYPE".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.value"),
                        message: format!(
                            "Const value {} does not match declared const type '{}'.",
                            other_val,
                            expected_ty.as_str()
                        ),
                        expected: Some(expected_ty.as_str().to_string()),
                        found: Some(value_kind(other_val).to_string()),
                    });
                    None
                }
            }
        }
        "param" => {
            let Some(name) = obj.get("name").and_then(|v| v.as_str()) else {
                diags.push(Diagnostic {
                    code: "E_UNKNOWN_PARAM".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.name"),
                    message: "Param expression is missing string 'name'.".to_string(),
                    expected: Some("declared parameter name".to_string()),
                    found: None,
                });
                return None;
            };
            if let Some(ty) = params.get(name) {
                Some(ty.clone())
            } else {
                let mut available: Vec<&str> = params.keys().map(String::as_str).collect();
                available.sort_unstable();
                diags.push(Diagnostic {
                    code: "E_UNKNOWN_PARAM".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: path.to_string(),
                    message: format!(
                        "Unknown parameter '{name}'; available parameters: {available:?}"
                    ),
                    expected: Some(available.join(" | ")),
                    found: Some(name.to_string()),
                });
                None
            }
        }
        "not" => {
            let Some(inner) = obj.get("value") else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.value"),
                    message: "Unary 'not' expression is missing 'value'.".to_string(),
                    expected: Some("bool expression".to_string()),
                    found: None,
                });
                return None;
            };
            let inner_ty = check_expr(
                inner,
                &format!("{path}.value"),
                func_name,
                params,
                ctx,
                diags,
            )?;
            if inner_ty != IrType::Bool {
                diags.push(Diagnostic {
                    code: "E_TYPE_MISMATCH".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: path.to_string(),
                    message: format!(
                        "Operator 'not' requires operand of type 'bool', found '{}'.",
                        inner_ty.as_str()
                    ),
                    expected: Some("bool".to_string()),
                    found: Some(inner_ty.as_str().to_string()),
                });
                return None;
            }
            Some(IrType::Bool)
        }
        "add" | "sub" | "mul" | "div_s" | "lt_s" | "gte_s" | "eq" | "and" | "or" => {
            let left_val = obj.get("left");
            let right_val = obj.get("right");
            if left_val.is_none() || right_val.is_none() {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: path.to_string(),
                    message: format!("Binary operator '{op}' requires both 'left' and 'right'."),
                    expected: Some("left and right expressions".to_string()),
                    found: None,
                });
                return None;
            }
            let left_val = left_val.unwrap();
            let right_val = right_val.unwrap();

            let left_ty = check_expr(
                left_val,
                &format!("{path}.left"),
                func_name,
                params,
                ctx,
                diags,
            );
            let right_ty = check_expr(
                right_val,
                &format!("{path}.right"),
                func_name,
                params,
                ctx,
                diags,
            );

            if op == "div_s" && is_const_zero(right_val) {
                diags.push(Diagnostic {
                    code: "E_CONST_DIV_ZERO".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.right"),
                    message: "Static division by zero is not allowed in 'div_s'.".to_string(),
                    expected: Some("non-zero i32 divisor".to_string()),
                    found: Some("0".to_string()),
                });
            }

            let (Some(lt), Some(rt)) = (left_ty, right_ty) else {
                return None;
            };

            match op {
                "add" | "sub" | "mul" | "div_s" => {
                    if lt == IrType::I32 && rt == IrType::I32 {
                        Some(IrType::I32)
                    } else {
                        diags.push(Diagnostic {
                            code: "E_TYPE_MISMATCH".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: path.to_string(),
                            message: format!(
                                "Arithmetic operator '{op}' requires (i32, i32), found ({}, {}).",
                                lt.as_str(),
                                rt.as_str()
                            ),
                            expected: Some("(i32, i32)".to_string()),
                            found: Some(format!("({}, {})", lt.as_str(), rt.as_str())),
                        });
                        None
                    }
                }
                "lt_s" | "gte_s" => {
                    if lt == IrType::I32 && rt == IrType::I32 {
                        Some(IrType::Bool)
                    } else {
                        diags.push(Diagnostic {
                            code: "E_TYPE_MISMATCH".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: path.to_string(),
                            message: format!(
                                "Comparison operator '{op}' requires (i32, i32), found ({}, {}).",
                                lt.as_str(),
                                rt.as_str()
                            ),
                            expected: Some("(i32, i32)".to_string()),
                            found: Some(format!("({}, {})", lt.as_str(), rt.as_str())),
                        });
                        None
                    }
                }
                "eq" => {
                    if (lt == IrType::I32 && rt == IrType::I32)
                        || (lt == IrType::Bool && rt == IrType::Bool)
                        || (ctx.ir_version == "0.6" && lt == IrType::String && rt == IrType::String)
                    {
                        Some(IrType::Bool)
                    } else {
                        diags.push(Diagnostic {
                            code: "E_TYPE_MISMATCH".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: path.to_string(),
                            message: format!(
                                "Equality operator 'eq' requires matching operand types (i32, i32) or (bool, bool), found ({}, {}).",
                                lt.as_str(),
                                rt.as_str()
                            ),
                            expected: Some(format!("({}, {})", lt.as_str(), lt.as_str())),
                            found: Some(format!("({}, {})", lt.as_str(), rt.as_str())),
                        });
                        None
                    }
                }
                "and" | "or" => {
                    if lt == IrType::Bool && rt == IrType::Bool {
                        Some(IrType::Bool)
                    } else {
                        diags.push(Diagnostic {
                            code: "E_TYPE_MISMATCH".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: path.to_string(),
                            message: format!(
                                "Logical operator '{op}' requires (bool, bool), found ({}, {}).",
                                lt.as_str(),
                                rt.as_str()
                            ),
                            expected: Some("(bool, bool)".to_string()),
                            found: Some(format!("({}, {})", lt.as_str(), rt.as_str())),
                        });
                        None
                    }
                }
                _ => unreachable!(),
            }
        }
        "if" => {
            let cond_val = obj.get("condition");
            let then_val = obj.get("then");
            let else_val = obj.get("else");
            if cond_val.is_none() || then_val.is_none() || else_val.is_none() {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: path.to_string(),
                    message: "'if' expression requires 'condition', 'then', and 'else'."
                        .to_string(),
                    expected: Some("condition, then, else".to_string()),
                    found: None,
                });
                return None;
            }
            let cond_ty = check_expr(
                cond_val.unwrap(),
                &format!("{path}.condition"),
                func_name,
                params,
                ctx,
                diags,
            );
            let then_ty = check_expr(
                then_val.unwrap(),
                &format!("{path}.then"),
                func_name,
                params,
                ctx,
                diags,
            );
            let else_ty = check_expr(
                else_val.unwrap(),
                &format!("{path}.else"),
                func_name,
                params,
                ctx,
                diags,
            );

            if let Some(ct) = cond_ty {
                if ct != IrType::Bool {
                    diags.push(Diagnostic {
                        code: "E_IF_CONDITION".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.condition"),
                        message: format!(
                            "'if.condition' must evaluate to 'bool', found '{}'.",
                            ct.as_str()
                        ),
                        expected: Some("bool".to_string()),
                        found: Some(ct.as_str().to_string()),
                    });
                }
            }

            if let (Some(tt), Some(et)) = (then_ty, else_ty) {
                if tt != et {
                    diags.push(Diagnostic {
                        code: "E_IF_BRANCH_TYPE".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: path.to_string(),
                        message: format!(
                            "'if' branches must return the same type, found then='{}' and else='{}'.",
                            tt.as_str(),
                            et.as_str()
                        ),
                        expected: Some(tt.as_str().to_string()),
                        found: Some(et.as_str().to_string()),
                    });
                    return None;
                }
                return Some(tt);
            }
            None
        }
        "call" => {
            let Some(callee_name) = obj.get("function").and_then(|v| v.as_str()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.function"),
                    message: "'call' expression requires a string 'function' property.".to_string(),
                    expected: Some("function name string".to_string()),
                    found: None,
                });
                return None;
            };

            let Some(sig) = ctx.functions.get(callee_name) else {
                diags.push(Diagnostic {
                    code: "E_UNKNOWN_FUNCTION".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.function"),
                    message: format!("Unknown function '{callee_name}' in 'call' expression."),
                    expected: Some("declared function name".to_string()),
                    found: Some(callee_name.to_string()),
                });
                return None;
            };

            let Some(args_arr) = obj.get("args").and_then(|v| v.as_array()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.args"),
                    message: "'call' expression requires an 'args' array.".to_string(),
                    expected: Some("array".to_string()),
                    found: None,
                });
                return None;
            };

            if args_arr.len() != sig.params.len() {
                diags.push(Diagnostic {
                    code: "E_CALL_ARITY".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.args"),
                    message: format!(
                        "Function '{callee_name}' expects {} argument(s), but {} were provided.",
                        sig.params.len(),
                        args_arr.len()
                    ),
                    expected: Some(format!("{} arguments", sig.params.len())),
                    found: Some(format!("{} arguments", args_arr.len())),
                });
                return None;
            }

            let mut match_ok = true;
            for (arg_idx, arg_val) in args_arr.iter().enumerate() {
                let arg_path = format!("{path}.args[{arg_idx}]");
                let expected_ty = &sig.params[arg_idx].1;
                let actual_ty = check_expr(arg_val, &arg_path, func_name, params, ctx, diags);
                if let Some(act) = actual_ty {
                    if act != *expected_ty {
                        diags.push(Diagnostic {
                            code: "E_TYPE_MISMATCH".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: arg_path,
                            message: format!(
                                "Argument {} for function '{callee_name}' expects type '{}', found '{}'.",
                                arg_idx,
                                expected_ty.as_str(),
                                act.as_str()
                            ),
                            expected: Some(expected_ty.as_str().to_string()),
                            found: Some(act.as_str().to_string()),
                        });
                        match_ok = false;
                    }
                } else {
                    match_ok = false;
                }
            }

            if match_ok {
                Some(sig.returns.clone())
            } else {
                None
            }
        }
        "call_import" => {
            let Some(import_name) = obj.get("import").and_then(|v| v.as_str()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.import"),
                    message: "'call_import' expression requires a string 'import' property."
                        .to_string(),
                    expected: Some("import name string".to_string()),
                    found: None,
                });
                return None;
            };

            let Some(sig) = ctx.imports.get(import_name) else {
                diags.push(Diagnostic {
                    code: "E_UNKNOWN_IMPORT".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.import"),
                    message: format!("Unknown import '{import_name}' in 'call_import' expression."),
                    expected: Some("declared import name".to_string()),
                    found: Some(import_name.to_string()),
                });
                return None;
            };

            if !ctx.capabilities.contains(&sig.capability) {
                diags.push(Diagnostic {
                    code: "E_UNDECLARED_CAPABILITY".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.import"),
                    message: format!(
                        "Invocation of import '{import_name}' requires capability '{}', which is not declared in root 'capabilities'.",
                        sig.capability
                    ),
                    expected: Some(format!("capability '{}' in root 'capabilities'", sig.capability)),
                    found: None,
                });
            }

            let Some(args_arr) = obj.get("args").and_then(|v| v.as_array()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.args"),
                    message: "'call_import' expression requires an 'args' array.".to_string(),
                    expected: Some("array".to_string()),
                    found: None,
                });
                return None;
            };

            if args_arr.len() != sig.params.len() {
                diags.push(Diagnostic {
                    code: "E_CALL_ARITY".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.args"),
                    message: format!(
                        "Import '{import_name}' expects {} argument(s), but {} were provided.",
                        sig.params.len(),
                        args_arr.len()
                    ),
                    expected: Some(format!("{} arguments", sig.params.len())),
                    found: Some(format!("{} arguments", args_arr.len())),
                });
                return None;
            }

            let mut match_ok = true;
            for (arg_idx, arg_val) in args_arr.iter().enumerate() {
                let arg_path = format!("{path}.args[{arg_idx}]");
                let expected_ty = &sig.params[arg_idx].1;
                let actual_ty = check_expr(arg_val, &arg_path, func_name, params, ctx, diags);
                if let Some(act) = actual_ty {
                    if act != *expected_ty {
                        diags.push(Diagnostic {
                            code: "E_TYPE_MISMATCH".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: arg_path,
                            message: format!(
                                "Argument {} for import '{import_name}' expects type '{}', found '{}'.",
                                arg_idx,
                                expected_ty.as_str(),
                                act.as_str()
                            ),
                            expected: Some(expected_ty.as_str().to_string()),
                            found: Some(act.as_str().to_string()),
                        });
                        match_ok = false;
                    }
                } else {
                    match_ok = false;
                }
            }

            if match_ok {
                Some(sig.returns.clone())
            } else {
                None
            }
        }
        "record.construct" => {
            let Some(rec_type_name) = obj.get("type").and_then(|v| v.as_str()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.type"),
                    message: "'record.construct' requires a string 'type' property.".to_string(),
                    expected: Some("record type name".to_string()),
                    found: None,
                });
                return None;
            };

            let Some(rec_def) = ctx.records.get(rec_type_name) else {
                diags.push(Diagnostic {
                    code: "E_UNKNOWN_TYPE".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.type"),
                    message: format!(
                        "Unknown record type '{rec_type_name}' in 'record.construct'."
                    ),
                    expected: Some("declared record type".to_string()),
                    found: Some(rec_type_name.to_string()),
                });
                return None;
            };

            let Some(fields_map) = obj.get("fields").and_then(|v| v.as_object()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.fields"),
                    message: "'record.construct' requires a 'fields' object.".to_string(),
                    expected: Some("object".to_string()),
                    found: None,
                });
                return None;
            };

            let mut construct_ok = true;
            let declared_field_names: HashSet<&str> =
                rec_def.fields.iter().map(|(n, _)| n.as_str()).collect();

            // Check for unknown fields
            for key in fields_map.keys() {
                if !declared_field_names.contains(key.as_str()) {
                    diags.push(Diagnostic {
                        code: "E_UNKNOWN_FIELD".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.fields.{key}"),
                        message: format!(
                            "Unknown field '{key}' for record type '{rec_type_name}'."
                        ),
                        expected: Some(
                            rec_def
                                .fields
                                .iter()
                                .map(|(n, _)| n.as_str())
                                .collect::<Vec<_>>()
                                .join(" | "),
                        ),
                        found: Some(key.clone()),
                    });
                    construct_ok = false;
                }
            }

            // Check missing fields and type check provided fields
            for (f_name, f_expected_ty) in &rec_def.fields {
                match fields_map.get(f_name) {
                    Some(f_expr) => {
                        let f_path = format!("{path}.fields.{f_name}");
                        let actual_ty = check_expr(f_expr, &f_path, func_name, params, ctx, diags);
                        if let Some(act) = actual_ty {
                            if act != *f_expected_ty {
                                diags.push(Diagnostic {
                                    code: "E_TYPE_MISMATCH".to_string(),
                                    function: func_name.map(ToOwned::to_owned),
                                    path: f_path,
                                    message: format!(
                                        "Field '{f_name}' in record '{rec_type_name}' expects type '{}', found '{}'.",
                                        f_expected_ty.as_str(),
                                        act.as_str()
                                    ),
                                    expected: Some(f_expected_ty.as_str().to_string()),
                                    found: Some(act.as_str().to_string()),
                                });
                                construct_ok = false;
                            }
                        } else {
                            construct_ok = false;
                        }
                    }
                    None => {
                        diags.push(Diagnostic {
                            code: "E_MISSING_FIELD".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: format!("{path}.fields.{f_name}"),
                            message: format!(
                                "Missing required field '{f_name}' in constructor for record '{rec_type_name}'."
                            ),
                            expected: Some(format!("{f_name}: {}", f_expected_ty.as_str())),
                            found: None,
                        });
                        construct_ok = false;
                    }
                }
            }

            if construct_ok {
                Some(IrType::Record(rec_type_name.to_string()))
            } else {
                None
            }
        }
        "record.get" => {
            let Some(target_val) = obj.get("target") else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.target"),
                    message: "'record.get' requires a 'target' expression.".to_string(),
                    expected: Some("record expression".to_string()),
                    found: None,
                });
                return None;
            };

            let Some(field_name) = obj.get("field").and_then(|v| v.as_str()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.field"),
                    message: "'record.get' requires a string 'field' property.".to_string(),
                    expected: Some("field name string".to_string()),
                    found: None,
                });
                return None;
            };

            let target_ty = check_expr(
                target_val,
                &format!("{path}.target"),
                func_name,
                params,
                ctx,
                diags,
            )?;

            match target_ty {
                IrType::Record(rec_name) => {
                    let Some(rec_def) = ctx.records.get(&rec_name) else {
                        diags.push(Diagnostic {
                            code: "E_UNKNOWN_TYPE".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: format!("{path}.target"),
                            message: format!("Unknown record type '{rec_name}'."),
                            expected: Some("valid record type".to_string()),
                            found: Some(rec_name),
                        });
                        return None;
                    };

                    if let Some((_, f_ty)) =
                        rec_def.fields.iter().find(|(name, _)| name == field_name)
                    {
                        Some(f_ty.clone())
                    } else {
                        let available: Vec<&str> =
                            rec_def.fields.iter().map(|(n, _)| n.as_str()).collect();
                        diags.push(Diagnostic {
                            code: "E_UNKNOWN_FIELD".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: format!("{path}.field"),
                            message: format!(
                                "Unknown field '{field_name}' on record type '{rec_name}'; available: {available:?}"
                            ),
                            expected: Some(available.join(" | ")),
                            found: Some(field_name.to_string()),
                        });
                        None
                    }
                }
                other => {
                    diags.push(Diagnostic {
                        code: "E_TYPE_MISMATCH".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.target"),
                        message: format!(
                            "'record.get' target must be a record, found '{}'.",
                            other.as_str()
                        ),
                        expected: Some("record".to_string()),
                        found: Some(other.as_str().to_string()),
                    });
                    None
                }
            }
        }
        "string.len" if ctx.ir_version == "0.6" => {
            let Some(inner) = obj.get("value") else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.value"),
                    message: "'string.len' expression requires 'value'.".to_string(),
                    expected: Some("string expression".to_string()),
                    found: None,
                });
                return None;
            };
            let inner_ty = check_expr(
                inner,
                &format!("{path}.value"),
                func_name,
                params,
                ctx,
                diags,
            )?;
            if inner_ty != IrType::String {
                diags.push(Diagnostic {
                    code: "E_TYPE_MISMATCH".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.value"),
                    message: format!(
                        "'string.len' requires operand of type 'string', found '{}'.",
                        inner_ty.as_str()
                    ),
                    expected: Some("string".to_string()),
                    found: Some(inner_ty.as_str().to_string()),
                });
                return None;
            }
            Some(IrType::I32)
        }
        "string.concat" if ctx.ir_version == "0.6" => {
            let left_val = obj.get("left");
            let right_val = obj.get("right");
            if left_val.is_none() || right_val.is_none() {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: path.to_string(),
                    message: "'string.concat' requires both 'left' and 'right'.".to_string(),
                    expected: Some("left and right string expressions".to_string()),
                    found: None,
                });
                return None;
            }
            let lt = check_expr(
                left_val.unwrap(),
                &format!("{path}.left"),
                func_name,
                params,
                ctx,
                diags,
            );
            let rt = check_expr(
                right_val.unwrap(),
                &format!("{path}.right"),
                func_name,
                params,
                ctx,
                diags,
            );
            let (Some(lt), Some(rt)) = (lt, rt) else {
                return None;
            };
            if lt != IrType::String || rt != IrType::String {
                diags.push(Diagnostic {
                    code: "E_TYPE_MISMATCH".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: path.to_string(),
                    message: format!(
                        "'string.concat' requires (string, string), found ({}, {}).",
                        lt.as_str(),
                        rt.as_str()
                    ),
                    expected: Some("(string, string)".to_string()),
                    found: Some(format!("({}, {})", lt.as_str(), rt.as_str())),
                });
                return None;
            }
            Some(IrType::String)
        }
        "result.ok" | "result.err" if ctx.ir_version == "0.6" => {
            let Some(ty_str) = obj.get("type").and_then(|v| v.as_str()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.type"),
                    message: format!(
                        "'{op}' requires a 'type' property (e.g. 'result<i32, string>')."
                    ),
                    expected: Some("result<T, E>".to_string()),
                    found: None,
                });
                return None;
            };
            let known_recs: HashSet<String> = ctx.records.keys().cloned().collect();
            let Some(res_ty) = IrType::from_str_for_version(ty_str, &known_recs, "0.6") else {
                diags.push(Diagnostic {
                    code: "E_UNKNOWN_TYPE".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.type"),
                    message: format!("Unknown or invalid result type '{ty_str}' in '{op}'."),
                    expected: Some("result<T, E>".to_string()),
                    found: Some(ty_str.to_string()),
                });
                return None;
            };
            let IrType::Result {
                ref ok, ref err, ..
            } = res_ty
            else {
                diags.push(Diagnostic {
                    code: "E_TYPE_MISMATCH".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.type"),
                    message: format!("'{op}' requires a 'result<T, E>' type, found '{ty_str}'."),
                    expected: Some("result<T, E>".to_string()),
                    found: Some(ty_str.to_string()),
                });
                return None;
            };
            let Some(val_expr) = obj.get("value") else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.value"),
                    message: format!("'{op}' requires a 'value' expression."),
                    expected: Some("expression".to_string()),
                    found: None,
                });
                return None;
            };
            let val_ty = check_expr(
                val_expr,
                &format!("{path}.value"),
                func_name,
                params,
                ctx,
                diags,
            )?;
            let expected_branch_ty = if op == "result.ok" {
                ok.as_ref()
            } else {
                err.as_ref()
            };
            if &val_ty != expected_branch_ty {
                diags.push(Diagnostic {
                    code: "E_TYPE_MISMATCH".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.value"),
                    message: format!(
                        "'{op}' for '{}' expects payload of type '{}', found '{}'.",
                        res_ty.as_str(),
                        expected_branch_ty.as_str(),
                        val_ty.as_str()
                    ),
                    expected: Some(expected_branch_ty.as_str().to_string()),
                    found: Some(val_ty.as_str().to_string()),
                });
                return None;
            }
            Some(res_ty)
        }
        "result.is_ok" if ctx.ir_version == "0.6" => {
            let Some(val_expr) = obj.get("value") else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.value"),
                    message: "'result.is_ok' requires a 'value' expression.".to_string(),
                    expected: Some("result<T, E> expression".to_string()),
                    found: None,
                });
                return None;
            };
            let val_ty = check_expr(
                val_expr,
                &format!("{path}.value"),
                func_name,
                params,
                ctx,
                diags,
            )?;
            if !matches!(val_ty, IrType::Result { .. }) {
                diags.push(Diagnostic {
                    code: "E_TYPE_MISMATCH".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.value"),
                    message: format!(
                        "'result.is_ok' requires a 'result<T, E>' operand, found '{}'.",
                        val_ty.as_str()
                    ),
                    expected: Some("result<T, E>".to_string()),
                    found: Some(val_ty.as_str().to_string()),
                });
                return None;
            }
            Some(IrType::Bool)
        }
        "result.unwrap_ok" | "result.unwrap_err" if ctx.ir_version == "0.6" => {
            let val_expr = obj.get("value");
            let def_expr = obj.get("default");
            if val_expr.is_none() || def_expr.is_none() {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: path.to_string(),
                    message: format!("'{op}' requires both 'value' and 'default' expressions."),
                    expected: Some("value and default expressions".to_string()),
                    found: None,
                });
                return None;
            }
            let val_ty = check_expr(
                val_expr.unwrap(),
                &format!("{path}.value"),
                func_name,
                params,
                ctx,
                diags,
            );
            let def_ty = check_expr(
                def_expr.unwrap(),
                &format!("{path}.default"),
                func_name,
                params,
                ctx,
                diags,
            );
            let (Some(vt), Some(dt)) = (val_ty, def_ty) else {
                return None;
            };
            let IrType::Result { ok, err, .. } = vt else {
                diags.push(Diagnostic {
                    code: "E_TYPE_MISMATCH".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.value"),
                    message: format!(
                        "'{op}' requires 'value' of type 'result<T, E>', found '{}'.",
                        vt.as_str()
                    ),
                    expected: Some("result<T, E>".to_string()),
                    found: Some(vt.as_str().to_string()),
                });
                return None;
            };
            let target_ty = if op == "result.unwrap_ok" { *ok } else { *err };
            if dt != target_ty {
                diags.push(Diagnostic {
                    code: "E_TYPE_MISMATCH".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.default"),
                    message: format!(
                        "'{op}' default expression must have type '{}', found '{}'.",
                        target_ty.as_str(),
                        dt.as_str()
                    ),
                    expected: Some(target_ty.as_str().to_string()),
                    found: Some(dt.as_str().to_string()),
                });
                return None;
            }
            Some(target_ty)
        }
        other => {
            diags.push(Diagnostic {
                code: "E_UNSUPPORTED_OP".to_string(),
                function: func_name.map(ToOwned::to_owned),
                path: format!("{path}.op"),
                message: format!("Unsupported operator '{other}' in Semantic IR."),
                expected: Some(
                    "const | param | not | add | sub | mul | div_s | eq | lt_s | gte_s | and | or | if | call | call_import | record.construct | record.get"
                        .to_string(),
                ),
                found: Some(other.to_string()),
            });
            None
        }
    }
}

fn is_const_zero(expr: &Value) -> bool {
    let Some(obj) = expr.as_object() else {
        return false;
    };
    obj.get("op").and_then(|v| v.as_str()) == Some("const")
        && obj.get("value").and_then(|v| v.as_i64()) == Some(0)
}

fn value_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

// Note: the POC also defines `wit_interface_id` here, but it is only called
// by `compile_ir_to_wasm` and `emit_expr`, both of which belong to
// `agent-native-compiler` per the matriz de extracción C-01. The helper is
// migrated alongside those functions, not here.
