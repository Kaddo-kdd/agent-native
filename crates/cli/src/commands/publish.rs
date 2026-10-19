//! `publish` subcommand implementation.
//!
//! Orchestrates atomic build, versioning, manifest generation and publication
//! to a local filesystem store with traceability metadata.

use serde::Serialize;
use std::path::Path;

use crate::agent::{FixtureLlmProvider, HttpLlmProvider, SynthesizerLlmProvider};
use crate::output::{CliExitCode, OutputFormat, emit_error, emit_result};
use crate::publish::{PublishOptions, build_and_publish_components_with_options};
use crate::spec::{MarkdownRequirementsInterpreter, RequirementsInterpreter};

/// Publish command report model.
#[derive(Debug, Serialize)]
pub struct PublishReport {
    pub success: bool,
    pub spec_version: String,
    pub version_id: String,
    pub active_pointer_file: String,
    pub published_components: Vec<String>,
    pub artifacts_count: usize,
    pub summary: String,
}

/// Executes the `publish` command.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn execute_publish(
    spec_path: &Path,
    dist_dir: &Path,
    _source_wi: &[String],
    git_commit: Option<&str>,
    mode: &str,
    format: OutputFormat,
) -> CliExitCode {
    if !spec_path.exists() {
        return emit_error(
            format,
            &format!("Specification file does not exist: {}", spec_path.display()),
            CliExitCode::UsageOrIoError,
        );
    }

    let content = match std::fs::read_to_string(spec_path) {
        Ok(c) => c,
        Err(err) => {
            return emit_error(
                format,
                &format!(
                    "Failed to read specification file {}: {err}",
                    spec_path.display()
                ),
                CliExitCode::UsageOrIoError,
            );
        }
    };

    let interpreter = MarkdownRequirementsInterpreter;
    let file_name = spec_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("spec.md");
    let spec = match interpreter.interpret_narrative(&content, file_name) {
        Ok(s) => s,
        Err(err) => {
            return emit_error(
                format,
                &format!("Failed to parse specification: {err}"),
                CliExitCode::DomainFailure,
            );
        }
    };

    let options = PublishOptions {
        wit_root: None,
        git_commit: git_commit.map(String::from),
        fail_point: None,
    };

    let publish_res = match mode.to_ascii_lowercase().as_str() {
        "synthesizer" => {
            let provider = SynthesizerLlmProvider::new("synthesizer");
            build_and_publish_components_with_options(&provider, &spec, dist_dir, &options)
        }
        "fixtures" => {
            let provider = FixtureLlmProvider::new("fixtures");
            build_and_publish_components_with_options(&provider, &spec, dist_dir, &options)
        }
        "real" => {
            let provider = HttpLlmProvider::from_env();
            build_and_publish_components_with_options(&provider, &spec, dist_dir, &options)
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

    match publish_res {
        Ok(pub_outcome) => {
            let active_ptr_path = dist_dir.join("active.json");
            let comp_ids: Vec<String> = pub_outcome
                .artifacts
                .iter()
                .map(|a| a.component_id.clone())
                .collect();
            let count = pub_outcome.artifacts.len();
            let report = PublishReport {
                success: true,
                spec_version: spec.spec_version,
                version_id: pub_outcome.version_id.clone(),
                active_pointer_file: active_ptr_path.display().to_string(),
                published_components: comp_ids,
                artifacts_count: count,
                summary: format!(
                    "Published version {} with {count} component artifact(s)",
                    pub_outcome.version_id
                ),
            };

            emit_result(
                format,
                &report,
                |r| {
                    format!(
                        "Publication SUCCESSFUL:\nVersion:        {}\nComponents:     {:?}\nActive pointer: {}",
                        r.version_id, r.published_components, r.active_pointer_file
                    )
                },
                CliExitCode::Success,
            )
        }
        Err(err) => emit_error(
            format,
            &format!("Publication failed: {err}"),
            CliExitCode::DomainFailure,
        ),
    }
}
