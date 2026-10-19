//! `test` subcommand implementation.
//!
//! Executes YAML acceptance test suites against compiled WASM components
//! using the standalone runtime.

use serde::Serialize;
use std::path::Path;

use agent_native_runtime::Engine;
use agent_native_runtime::testing::{SuiteReport, parse_suite, run_suite};

use crate::output::{CliExitCode, OutputFormat, emit_error, emit_result};

/// Test command output wrapper.
#[derive(Debug, Serialize)]
pub struct TestCommandOutput {
    pub success: bool,
    pub suite_path: String,
    pub component_path: String,
    pub report: SuiteReport,
}

/// Executes the `test` command.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn execute_test(suite_path: &Path, component_path: &Path, format: OutputFormat) -> CliExitCode {
    if !suite_path.exists() {
        return emit_error(
            format,
            &format!("Test suite file does not exist: {}", suite_path.display()),
            CliExitCode::UsageOrIoError,
        );
    }

    if !component_path.exists() {
        return emit_error(
            format,
            &format!(
                "WASM component file does not exist: {}",
                component_path.display()
            ),
            CliExitCode::UsageOrIoError,
        );
    }

    let suite_str = match std::fs::read_to_string(suite_path) {
        Ok(s) => s,
        Err(err) => {
            return emit_error(
                format,
                &format!(
                    "Failed to read test suite file {}: {err}",
                    suite_path.display()
                ),
                CliExitCode::UsageOrIoError,
            );
        }
    };

    let suite = match parse_suite(&suite_str) {
        Ok(s) => s,
        Err(err) => {
            return emit_error(
                format,
                &format!(
                    "Failed to parse YAML test suite {}: {err}",
                    suite_path.display()
                ),
                CliExitCode::UsageOrIoError,
            );
        }
    };

    let component_bytes = match std::fs::read(component_path) {
        Ok(b) => b,
        Err(err) => {
            return emit_error(
                format,
                &format!(
                    "Failed to read WASM component {}: {err}",
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
                &format!(
                    "Failed to load WASM component {}: {err}",
                    component_path.display()
                ),
                CliExitCode::DomainFailure,
            );
        }
    };

    let report = match run_suite(&component, &suite) {
        Ok(r) => r,
        Err(err) => {
            return emit_error(
                format,
                &format!("Test suite execution aborted with executor error: {err}"),
                CliExitCode::DomainFailure,
            );
        }
    };

    let success = report.failed == 0 && report.errored == 0;
    let output = TestCommandOutput {
        success,
        suite_path: suite_path.display().to_string(),
        component_path: component_path.display().to_string(),
        report,
    };

    let exit_code = if success {
        CliExitCode::Success
    } else {
        CliExitCode::DomainFailure
    };

    emit_result(
        format,
        &output,
        |out| {
            let r = &out.report;
            let mut s = format!(
                "Test Suite:  {}\nComponent:   {}\nResults:     {} total, {} passed, {} failed, {} errored\n\n",
                r.suite, out.component_path, r.total, r.passed, r.failed, r.errored
            );
            for c in &r.cases {
                s.push_str(&format!("  [{:?}] {}\n", c.status, c.id));
                for ev in &c.evidence {
                    s.push_str(&format!("        evidence: {ev}\n"));
                }
            }
            if out.success {
                s.push_str("\nSuite Result: PASSED");
            } else {
                s.push_str("\nSuite Result: FAILED");
            }
            s
        },
        exit_code,
    )
}
