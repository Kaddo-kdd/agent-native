//! `verify` subcommand implementation.
//!
//! Verifies static correctness and schema compliance of Semantic IR (.json)
//! or narrative requirement specifications (.md).

use serde::Serialize;
use std::path::Path;

use agent_native_ir::{Diagnostic, VerifyOptions, verify_ir_with_options};

use crate::output::{CliExitCode, OutputFormat, emit_error, emit_result};
use crate::spec::{MarkdownRequirementsInterpreter, RequirementsInterpreter};

/// Verification report data model.
#[derive(Debug, Serialize)]
pub struct VerifyReport {
    pub valid: bool,
    pub input_file: String,
    pub format_type: String,
    pub error_count: usize,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// Executes the `verify` command (default stable IR v0.5 rules).
#[must_use]
pub fn execute_verify(input: &Path, format: OutputFormat) -> CliExitCode {
    execute_verify_with_options(input, false, format)
}

/// Executes the `verify` command with explicit `--experimental` support for IR v0.6.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn execute_verify_with_options(
    input: &Path,
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

    if is_json {
        let root: serde_json::Value = match serde_json::from_str(&content) {
            Ok(val) => val,
            Err(err) => {
                let diag = Diagnostic {
                    code: "PARSE_SYNTAX_ERROR".to_string(),
                    function: None,
                    path: "/".to_string(),
                    message: format!("Malformed JSON: {err}"),
                    expected: None,
                    found: None,
                };
                let report = VerifyReport {
                    valid: false,
                    input_file: input.display().to_string(),
                    format_type: "semantic_ir_json".to_string(),
                    error_count: 1,
                    diagnostics: vec![diag],
                    summary: Some("JSON syntax error".to_string()),
                };
                return emit_result(
                    format,
                    &report,
                    |r| {
                        format!(
                            "Verification FAILED: {}\n{}",
                            r.input_file,
                            r.summary.as_deref().unwrap_or("")
                        )
                    },
                    CliExitCode::DomainFailure,
                );
            }
        };

        let verify_opts = VerifyOptions {
            allow_experimental: experimental,
        };
        let diagnostics = verify_ir_with_options(&root, &verify_opts);
        let error_count = diagnostics.len();
        let valid = error_count == 0;

        let report = VerifyReport {
            valid,
            input_file: input.display().to_string(),
            format_type: "semantic_ir_json".to_string(),
            error_count,
            diagnostics,
            summary: Some(if valid {
                "Semantic IR is valid".to_string()
            } else {
                format!("{error_count} error(s) found during IR verification")
            }),
        };

        let exit_code = if valid {
            CliExitCode::Success
        } else {
            CliExitCode::DomainFailure
        };

        emit_result(
            format,
            &report,
            |r| {
                if r.valid {
                    format!(
                        "Verification SUCCESSFUL: {}\nType: Semantic IR\nStatus: Valid (0 errors)",
                        r.input_file
                    )
                } else {
                    let mut s = format!(
                        "Verification FAILED: {}\nFound {} error(s):\n",
                        r.input_file, r.error_count
                    );
                    for d in &r.diagnostics {
                        s.push_str(&format!("  [{}] {} (at {})\n", d.code, d.message, d.path));
                    }
                    s
                }
            },
            exit_code,
        )
    } else {
        // Assume Markdown spec
        let interpreter = MarkdownRequirementsInterpreter;
        let file_name = input
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("spec.md");
        match interpreter.interpret_narrative(&content, file_name) {
            Ok(spec) => {
                let comp_count = spec.components.len();
                let report = VerifyReport {
                    valid: true,
                    input_file: input.display().to_string(),
                    format_type: "narrative_markdown_spec".to_string(),
                    error_count: 0,
                    diagnostics: Vec::new(),
                    summary: Some(format!(
                        "Specification v{} is valid ({} component(s), {} requirement(s))",
                        spec.spec_version,
                        comp_count,
                        spec.requirements.len()
                    )),
                };
                emit_result(
                    format,
                    &report,
                    |r| {
                        format!(
                            "Verification SUCCESSFUL: {}\n{}",
                            r.input_file,
                            r.summary.as_deref().unwrap_or("")
                        )
                    },
                    CliExitCode::Success,
                )
            }
            Err(err) => {
                let diag = Diagnostic {
                    code: "SPEC_PARSE_ERROR".to_string(),
                    function: None,
                    path: "/spec".to_string(),
                    message: format!("Specification parse error: {err}"),
                    expected: None,
                    found: None,
                };
                let report = VerifyReport {
                    valid: false,
                    input_file: input.display().to_string(),
                    format_type: "narrative_markdown_spec".to_string(),
                    error_count: 1,
                    diagnostics: vec![diag],
                    summary: Some(format!("Specification parse failed: {err}")),
                };
                emit_result(
                    format,
                    &report,
                    |r| {
                        format!(
                            "Verification FAILED: {}\n{}",
                            r.input_file,
                            r.summary.as_deref().unwrap_or("")
                        )
                    },
                    CliExitCode::DomainFailure,
                )
            }
        }
    }
}
