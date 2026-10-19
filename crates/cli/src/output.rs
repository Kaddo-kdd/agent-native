//! Structured output handling and exit codes for Agent-Native CLI.
//!
//! Follows ADR-001 (CLI as boundary adapter) and AN-028:
//! - Exit code 0: Success.
//! - Exit code 1: Domain failure (verification diagnostic errors, compilation failure,
//!   acceptance test failures, runtime execution errors, publishing validation failures).
//! - Exit code 2: Usage / argument errors, missing input files, or I/O errors.

use clap::ValueEnum;
use serde::Serialize;
use std::process::ExitCode;

/// Standardized CLI exit codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliExitCode {
    /// Total success (all criteria satisfied).
    Success = 0,
    /// Domain error (validation, compilation, test, runtime or publish failure).
    DomainFailure = 1,
    /// CLI argument syntax error, missing input file or I/O failure.
    UsageOrIoError = 2,
}

impl From<CliExitCode> for ExitCode {
    fn from(code: CliExitCode) -> Self {
        ExitCode::from(code as u8)
    }
}

/// Output format for command results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Human-readable plain text.
    #[default]
    Human,
    /// Structured JSON machine-parseable output.
    Json,
}

/// Helper to render output according to selected format and return exit code.
#[must_use]
pub fn emit_result<T: Serialize>(
    format: OutputFormat,
    data: &T,
    human_renderer: impl FnOnce(&T) -> String,
    exit_code: CliExitCode,
) -> CliExitCode {
    match format {
        OutputFormat::Json => {
            if let Ok(json) = serde_json::to_string_pretty(data) {
                println!("{json}");
            } else {
                eprintln!(r#"{{"error": "Failed to serialize JSON output"}}"#);
            }
        }
        OutputFormat::Human => {
            println!("{}", human_renderer(data));
        }
    }
    exit_code
}

/// Helper to emit an error according to selected format.
#[must_use]
pub fn emit_error(
    format: OutputFormat,
    error_message: &str,
    exit_code: CliExitCode,
) -> CliExitCode {
    match format {
        OutputFormat::Json => {
            let val = serde_json::json!({
                "success": false,
                "error": error_message,
                "exit_code": exit_code as u8
            });
            if let Ok(json) = serde_json::to_string_pretty(&val) {
                println!("{json}");
            }
        }
        OutputFormat::Human => {
            eprintln!("Error: {error_message}");
        }
    }
    exit_code
}
