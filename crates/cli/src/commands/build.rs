//! `build` subcommand implementation.
//!
//! Compiles a `SystemSpec` (narrative markdown) or Semantic IR (.json)
//! into a WebAssembly Component Model binary (.wasm) with deterministic SHA-256.

use serde::Serialize;
use std::path::Path;

use agent_native_ir::{Diagnostic, VerifyOptions, verify_ir_with_options};

use crate::agent::{FixtureLlmProvider, HttpLlmProvider, LlmProvider, SynthesizerLlmProvider};
use crate::output::{CliExitCode, OutputFormat, emit_error, emit_result};
use crate::spec::{MarkdownRequirementsInterpreter, RequirementsInterpreter};

/// Build result report data model.
#[derive(Debug, Serialize)]
pub struct BuildReport {
    pub success: bool,
    pub component: String,
    pub mode: String,
    pub wasm_bytes_len: usize,
    pub sha256: String,
    pub output_path: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
    pub summary: String,
}

/// Executes the `build` command (default stable IR v0.5 rules).
#[must_use]
pub fn execute_build(
    input: &Path,
    output: Option<&Path>,
    component: Option<&str>,
    mode: &str,
    format: OutputFormat,
) -> CliExitCode {
    execute_build_with_options(input, output, component, mode, false, format)
}

/// Executes the `build` command with explicit `--experimental` support for IR v0.6.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn execute_build_with_options(
    input: &Path,
    output: Option<&Path>,
    component: Option<&str>,
    mode: &str,
    experimental: bool,
    format: OutputFormat,
) -> CliExitCode {
    if !input.exists() {
        return emit_error(
            format,
            &format!("Input file does not exist: {}", input.display()),
            CliExitCode::UsageOrIoError,
        );
    }

    let content = match std::fs::read_to_string(input) {
        Ok(c) => c,
        Err(err) => {
            return emit_error(
                format,
                &format!("Failed to read input file {}: {err}", input.display()),
                CliExitCode::UsageOrIoError,
            );
        }
    };

    let is_json = input
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        || content.trim_start().starts_with('{');

    let (ir_value, comp_name, spec_opt) = if is_json {
        let val: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(err) => {
                return emit_error(
                    format,
                    &format!("Failed to parse Semantic IR JSON: {err}"),
                    CliExitCode::DomainFailure,
                );
            }
        };
        let c_name = component
            .map(String::from)
            .or_else(|| {
                val.get("component")
                    .and_then(|c| c.get("name"))
                    .and_then(|n| n.as_str())
                    .map(String::from)
            })
            .unwrap_or_else(|| "unnamed_component".to_string());
        (val, c_name, None)
    } else {
        // Parse narrative spec
        let interpreter = MarkdownRequirementsInterpreter;
        let file_name = input
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("spec.md");
        let spec = match interpreter.interpret_narrative(&content, file_name) {
            Ok(s) => s,
            Err(err) => {
                return emit_error(
                    format,
                    &format!("Failed to parse narrative specification: {err}"),
                    CliExitCode::DomainFailure,
                );
            }
        };

        if spec.components.is_empty() {
            return emit_error(
                format,
                "Specification defines zero components",
                CliExitCode::DomainFailure,
            );
        }

        let target_comp_id = if let Some(c) = component {
            c.to_string()
        } else {
            spec.components[0].id.clone()
        };

        let proposed_ir = match mode.to_ascii_lowercase().as_str() {
            "synthesizer" => {
                let provider = SynthesizerLlmProvider::new("synthesizer");
                provider.propose_ir(&target_comp_id, &spec, &[])
            }
            "fixtures" => {
                let provider = FixtureLlmProvider::new("fixtures");
                provider.propose_ir(&target_comp_id, &spec, &[])
            }
            "real" => {
                let provider = HttpLlmProvider::from_env();
                provider.propose_ir(&target_comp_id, &spec, &[])
            }
            other => {
                return emit_error(
                    format,
                    &format!(
                        "Unsupported generation mode: '{other}'. Expected 'synthesizer', 'fixtures', or 'real'"
                    ),
                    CliExitCode::UsageOrIoError,
                );
            }
        };

        let ir = match proposed_ir {
            Ok(ir) => ir,
            Err(err) => {
                return emit_error(
                    format,
                    &format!("IR generation failed in mode '{mode}': {err}"),
                    CliExitCode::DomainFailure,
                );
            }
        };

        (ir, target_comp_id, Some(spec))
    };

    // Verify IR
    let verify_opts = VerifyOptions {
        allow_experimental: experimental,
    };
    let diagnostics = verify_ir_with_options(&ir_value, &verify_opts);
    if !diagnostics.is_empty() {
        let err_len = diagnostics.len();
        let report = BuildReport {
            success: false,
            component: comp_name,
            mode: mode.to_string(),
            wasm_bytes_len: 0,
            sha256: String::new(),
            output_path: None,
            diagnostics,
            summary: format!("{err_len} verification error(s) prevented compilation"),
        };
        return emit_result(
            format,
            &report,
            |r| {
                format!(
                    "Build FAILED: Component '{}'\nDiagnostics:\n{}",
                    r.component, r.summary
                )
            },
            CliExitCode::DomainFailure,
        );
    }

    // Compile IR to WASM with targeted WIT fallback
    let base_ctx = agent_native_compiler::CompileContext::default().with_experimental(experimental);
    let compile_res = agent_native_compiler::compile_ir_to_wasm_with_context(&ir_value, &base_ctx)
        .or_else(|initial_diags| {
            // Activate fallback ONLY when compilation failure is specifically due to a missing WIT file
            let is_wit_missing = initial_diags.iter().any(|d| d.code == "E_WIT_NOT_FOUND");
            if is_wit_missing {
                if let Some(spec) = spec_opt.as_ref() {
                    let temp_dir = std::env::temp_dir().join(format!(
                        "agent_native_cli_wit_{}_{}",
                        comp_name,
                        std::process::id()
                    ));
                    let staging_wit = temp_dir.join("wit");
                    if let Err(err) = std::fs::create_dir_all(&staging_wit) {
                        let mut combined = initial_diags;
                        combined.push(Diagnostic {
                            code: "E_WIT_STAGING_IO".to_string(),
                            function: None,
                            path: "$.wit_contract".to_string(),
                            message: format!(
                                "Failed to create temporary WIT staging directory: {err}"
                            ),
                            expected: None,
                            found: None,
                        });
                        return Err(combined);
                    }
                    let fallback_file = staging_wit.join(format!("{comp_name}.wit"));
                    let content = crate::spec::generate_wit_for_component(spec, &comp_name);
                    if let Err(err) = std::fs::write(&fallback_file, content) {
                        let _ = std::fs::remove_dir_all(&temp_dir);
                        let mut combined = initial_diags;
                        combined.push(Diagnostic {
                            code: "E_WIT_STAGING_IO".to_string(),
                            function: None,
                            path: "$.wit_contract".to_string(),
                            message: format!(
                                "Failed to write temporary synthesized WIT file: {err}"
                            ),
                            expected: None,
                            found: None,
                        });
                        return Err(combined);
                    }
                    let fallback_ctx = agent_native_compiler::CompileContext::new(&temp_dir)
                        .with_experimental(experimental);
                    let fallback_res = agent_native_compiler::compile_ir_to_wasm_with_context(
                        &ir_value,
                        &fallback_ctx,
                    );
                    let _ = std::fs::remove_dir_all(&temp_dir);
                    match fallback_res {
                        Ok(bytes) => Ok(bytes),
                        Err(second_diags) => {
                            let mut combined = initial_diags;
                            combined.extend(second_diags);
                            Err(combined)
                        }
                    }
                } else {
                    Err(initial_diags)
                }
            } else {
                Err(initial_diags)
            }
        });

    let wasm_bytes = match compile_res {
        Ok(bytes) => bytes,
        Err(diags) => {
            let report = BuildReport {
                success: false,
                component: comp_name,
                mode: mode.to_string(),
                wasm_bytes_len: 0,
                sha256: String::new(),
                output_path: None,
                diagnostics: diags,
                summary: "Compiler backend reported diagnostics".to_string(),
            };
            return emit_result(
                format,
                &report,
                |r| {
                    let mut s = format!(
                        "Build FAILED: Compilation rejected component '{}'\n",
                        r.component
                    );
                    for d in &r.diagnostics {
                        s.push_str(&format!("  [{}] {} (at {})\n", d.code, d.message, d.path));
                    }
                    s
                },
                CliExitCode::DomainFailure,
            );
        }
    };

    let sha256 = agent_native_compiler::sha256_hex(&wasm_bytes);
    let output_path_str = if let Some(out) = output {
        if let Some(parent) = out.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        if let Err(err) = std::fs::write(out, &wasm_bytes) {
            return emit_error(
                format,
                &format!("Failed to write WASM artifact to {}: {err}", out.display()),
                CliExitCode::UsageOrIoError,
            );
        }
        Some(out.display().to_string())
    } else {
        None
    };

    let report = BuildReport {
        success: true,
        component: comp_name,
        mode: mode.to_string(),
        wasm_bytes_len: wasm_bytes.len(),
        sha256,
        output_path: output_path_str,
        diagnostics: Vec::new(),
        summary: "Component compiled successfully".to_string(),
    };

    emit_result(
        format,
        &report,
        |r| {
            let mut s = format!(
                "Build SUCCESSFUL: Component '{}'\nMode:       {}\nWASM Size:  {} bytes\nSHA-256:    {}",
                r.component, r.mode, r.wasm_bytes_len, r.sha256
            );
            if let Some(ref p) = r.output_path {
                s.push_str(&format!("\nOutput:     {p}"));
            }
            s
        },
        CliExitCode::Success,
    )
}
