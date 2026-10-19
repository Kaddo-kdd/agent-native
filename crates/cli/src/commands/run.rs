//! `run` subcommand implementation.
//!
//! Executes an exported function on a loaded WASM component using the standalone runtime.

use serde::Serialize;
use std::path::Path;

use agent_native_runtime::{Engine, RuntimeError, Val};

use crate::output::{CliExitCode, OutputFormat, emit_error, emit_result};

/// Execution output data model.
#[derive(Debug, Serialize)]
pub struct RunCommandOutput {
    pub success: bool,
    pub component_path: String,
    pub function_name: String,
    pub results: Vec<serde_json::Value>,
    pub results_formatted: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Helper to convert a CLI argument string into a Component Model `Val`.
#[must_use]
pub fn parse_arg_to_val(s: &str) -> Val {
    if let Ok(b) = s.parse::<bool>() {
        return Val::Bool(b);
    }
    if let Ok(n) = s.parse::<i32>() {
        return Val::S32(n);
    }
    if let Ok(n) = s.parse::<i64>() {
        return Val::S64(n);
    }
    Val::String(s.to_string())
}

/// Helper to serialize a Component Model `Val` to JSON.
#[must_use]
pub fn val_to_json(val: &Val) -> serde_json::Value {
    match val {
        Val::Bool(b) => serde_json::Value::Bool(*b),
        Val::S8(n) => serde_json::json!(*n),
        Val::U8(n) => serde_json::json!(*n),
        Val::S16(n) => serde_json::json!(*n),
        Val::U16(n) => serde_json::json!(*n),
        Val::S32(n) => serde_json::json!(*n),
        Val::U32(n) => serde_json::json!(*n),
        Val::S64(n) => serde_json::json!(*n),
        Val::U64(n) => serde_json::json!(*n),
        Val::Float32(f) => serde_json::json!(*f),
        Val::Float64(f) => serde_json::json!(*f),
        Val::Char(c) => serde_json::Value::String(c.to_string()),
        Val::String(s) => serde_json::Value::String(s.clone()),
        Val::List(list) => {
            let items: Vec<serde_json::Value> = list.iter().map(val_to_json).collect();
            serde_json::Value::Array(items)
        }
        Val::Record(fields) => {
            let mut map = serde_json::Map::new();
            for (name, v) in fields {
                map.insert(name.clone(), val_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        Val::Option(opt) => match opt {
            Some(v) => val_to_json(v),
            None => serde_json::Value::Null,
        },
        Val::Result(Ok(opt)) => {
            let inner = opt.as_deref().map_or(serde_json::Value::Null, val_to_json);
            serde_json::json!({ "ok": inner })
        }
        Val::Result(Err(opt)) => {
            let inner = opt.as_deref().map_or(serde_json::Value::Null, val_to_json);
            serde_json::json!({ "err": inner })
        }
        _ => serde_json::Value::String(format!("{val:?}")),
    }
}

/// Executes the `run` command.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn execute_run(
    component_path: &Path,
    function_name: &str,
    args: &[String],
    format: OutputFormat,
) -> CliExitCode {
    if !component_path.exists() {
        return emit_error(
            format,
            &format!(
                "Component file does not exist: {}",
                component_path.display()
            ),
            CliExitCode::UsageOrIoError,
        );
    }

    let component_bytes = match std::fs::read(component_path) {
        Ok(b) => b,
        Err(err) => {
            return emit_error(
                format,
                &format!(
                    "Failed to read component file {}: {err}",
                    component_path.display()
                ),
                CliExitCode::UsageOrIoError,
            );
        }
    };

    let engine = match Engine::new() {
        Ok(e) => e,
        Err(err) => {
            return emit_error(
                format,
                &format!("Failed to initialize runtime engine: {err}"),
                CliExitCode::DomainFailure,
            );
        }
    };

    let component = match engine.load_component(&component_bytes) {
        Ok(c) => c,
        Err(err) => {
            return emit_error(
                format,
                &format!("Failed to load component: {err}"),
                CliExitCode::DomainFailure,
            );
        }
    };

    let parsed_args: Vec<Val> = args.iter().map(|s| parse_arg_to_val(s)).collect();

    // Try direct invocation first, then common Component Model qualifier candidates if bare function name passed
    let mut invoke_result = component.invoke(function_name, &parsed_args);
    if let Err(RuntimeError::FunctionNotFound(_)) = &invoke_result {
        let candidates = vec![
            format!("ops#{function_name}"),
            format!("an:synthesized/ops@0.1.0#{function_name}"),
            format!("ops#{}", function_name.replace('_', "-")),
            format!(
                "an:synthesized/ops@0.1.0#{}",
                function_name.replace('_', "-")
            ),
        ];
        for candidate in candidates {
            let candidate_res = component.invoke(&candidate, &parsed_args);
            if !matches!(candidate_res, Err(RuntimeError::FunctionNotFound(_))) {
                invoke_result = candidate_res;
                break;
            }
        }
    }

    match invoke_result {
        Ok(vals) => {
            let results_json: Vec<serde_json::Value> = vals.iter().map(val_to_json).collect();
            let results_formatted: Vec<String> = vals.iter().map(|v| format!("{v:?}")).collect();

            let out = RunCommandOutput {
                success: true,
                component_path: component_path.display().to_string(),
                function_name: function_name.to_string(),
                results: results_json,
                results_formatted,
                error: None,
            };

            emit_result(
                format,
                &out,
                |o| {
                    if o.results_formatted.is_empty() {
                        format!(
                            "Function '{}' executed successfully (no return value)",
                            o.function_name
                        )
                    } else {
                        let mut s = format!(
                            "Function '{}' returned {} value(s):\n",
                            o.function_name,
                            o.results_formatted.len()
                        );
                        for (i, res) in o.results_formatted.iter().enumerate() {
                            s.push_str(&format!("  [{i}]: {res}\n"));
                        }
                        s
                    }
                },
                CliExitCode::Success,
            )
        }
        Err(err) => {
            let out = RunCommandOutput {
                success: false,
                component_path: component_path.display().to_string(),
                function_name: function_name.to_string(),
                results: Vec::new(),
                results_formatted: Vec::new(),
                error: Some(err.to_string()),
            };

            emit_result(
                format,
                &out,
                |o| {
                    format!(
                        "Execution FAILED: Function '{}' error: {}",
                        o.function_name,
                        o.error.as_deref().unwrap_or("")
                    )
                },
                CliExitCode::DomainFailure,
            )
        }
    }
}
