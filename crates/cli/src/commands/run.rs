//! `run` subcommand implementation.
//!
//! Executes an exported function on a loaded WASM component using the standalone runtime,
//! with support for offline SHA-256 integrity and Minisign (Ed25519) cryptographic signature
//! verification before component instantiation (INIT-06-04, AN-041, AN-042),
//! as well as configurable computational fuel and memory execution limits (AN-047).

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use wasma_runtime::{Engine, ExecutionLimits, RuntimeError, Val};

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
    pub fuel_consumed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<VerificationSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Verification details included in the run command output.
#[derive(Debug, Serialize, Clone)]
pub struct VerificationSummary {
    pub integrity_verified: bool,
    pub computed_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_sha256: Option<String>,
    pub signature_verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_path: Option<String>,
}

/// Options configuring load-time verification for `run`.
#[derive(Debug, Default, Clone)]
pub struct RunVerificationOptions<'a> {
    pub verify: bool,
    pub expected_sha256: Option<&'a str>,
    pub manifest_path: Option<&'a Path>,
    pub signature_path: Option<&'a Path>,
    pub public_key: Option<&'a str>,
}

/// Options configuring execution resource quotas and limits for `run`.
#[derive(Debug, Default, Clone, Copy)]
pub struct RunExecutionLimits {
    pub fuel: Option<u64>,
    pub max_memory_bytes: Option<usize>,
    pub max_table_elements: Option<u32>,
}

/// Computes the canonical lowercase SHA-256 hex digest for arbitrary bytes.
#[must_use]
pub fn compute_sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Loads a Minisign Ed25519 public key from a file path or direct base64 string.
pub fn load_minisign_public_key(
    key_str_or_path: &str,
) -> Result<minisign_verify::PublicKey, String> {
    let trimmed = key_str_or_path.trim();
    let path = Path::new(trimmed);
    let key_data = if path.exists() {
        std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read public key file {}: {e}", path.display()))?
    } else {
        trimmed.to_string()
    };

    // If key format contains comments (standard Minisign format), parse base64 line
    let base64_str = key_data
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("untrusted comment:"))
        .last()
        .unwrap_or(trimmed);

    minisign_verify::PublicKey::from_base64(base64_str)
        .map_err(|e| format!("Failed to decode Minisign public key: {e}"))
}

/// Loads a Minisign signature from a `.minisig` file.
pub fn load_minisign_signature(sig_path: &Path) -> Result<minisign_verify::Signature, String> {
    if !sig_path.exists() {
        return Err(format!(
            "Signature file does not exist: {}",
            sig_path.display()
        ));
    }
    minisign_verify::Signature::from_file(sig_path).map_err(|e| {
        format!(
            "Failed to parse Minisign signature from {}: {e}",
            sig_path.display()
        )
    })
}

/// Resolves the expected SHA-256 digest from explicit args, manifest, or adjacent files.
pub fn resolve_expected_sha256(
    component_path: &Path,
    explicit_expected: Option<&str>,
    manifest_path: Option<&Path>,
    auto_discover: bool,
) -> Result<Option<String>, String> {
    if let Some(explicit) = explicit_expected {
        return Ok(Some(explicit.trim().to_lowercase()));
    }

    let manifest_file_to_check = if let Some(m_path) = manifest_path {
        if !m_path.exists() {
            return Err(format!(
                "Manifest file does not exist: {}",
                m_path.display()
            ));
        }
        Some(m_path.to_path_buf())
    } else if auto_discover {
        let parent = component_path.parent().unwrap_or_else(|| Path::new("."));
        let candidate_manifest = parent.join("manifest.json");
        let candidate_sha = parent.join(format!(
            "{}.sha256",
            component_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));
        if candidate_manifest.exists() {
            Some(candidate_manifest)
        } else if candidate_sha.exists() {
            let content = std::fs::read_to_string(&candidate_sha)
                .map_err(|e| format!("Failed to read {}: {e}", candidate_sha.display()))?;
            let hash = content
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim()
                .to_lowercase();
            if !hash.is_empty() {
                return Ok(Some(hash));
            }
            None
        } else {
            None
        }
    } else {
        None
    };

    if let Some(m_file) = manifest_file_to_check {
        let content = std::fs::read_to_string(&m_file)
            .map_err(|e| format!("Failed to read manifest {}: {e}", m_file.display()))?;

        // Attempt JSON parsing
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) {
            let comp_name = component_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let comp_stem = component_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();

            if let Some(artifacts) = json_val.get("artifacts").and_then(|a| a.as_array()) {
                for art in artifacts {
                    let wasm_file = art.get("wasm_file").and_then(|v| v.as_str()).unwrap_or("");
                    let comp_id = art
                        .get("component_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if wasm_file == comp_name.as_ref()
                        || comp_id == comp_stem.as_ref()
                        || artifacts.len() == 1
                    {
                        if let Some(sha) = art.get("sha256").and_then(|v| v.as_str()) {
                            return Ok(Some(sha.trim().to_lowercase()));
                        }
                    }
                }
            }
            if let Some(sha) = json_val.get("sha256").and_then(|v| v.as_str()) {
                return Ok(Some(sha.trim().to_lowercase()));
            }
        }

        // Attempt SHA256SUMS line parsing: "<hash>  <filename>"
        let comp_name = component_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let hash = parts[0];
                let file = parts[1].trim_start_matches('*');
                if file == comp_name.as_ref() || file.ends_with(comp_name.as_ref()) {
                    return Ok(Some(hash.trim().to_lowercase()));
                }
            }
        }
    }

    Ok(None)
}

/// Resolves the signature file path from explicit args or adjacent convention.
#[must_use]
pub fn resolve_signature_path(
    component_path: &Path,
    explicit_sig_path: Option<&Path>,
    auto_discover: bool,
) -> Option<PathBuf> {
    if let Some(explicit) = explicit_sig_path {
        return Some(explicit.to_path_buf());
    }

    if auto_discover {
        let adjacent_sig = PathBuf::from(format!("{}.minisig", component_path.display()));
        if adjacent_sig.exists() {
            return Some(adjacent_sig);
        }
    }

    None
}

/// Parses CLI string argument into typed WebAssembly Component Model [`Val`].
fn parse_arg_to_val(s: &str) -> Val {
    if let Ok(b) = s.parse::<bool>() {
        return Val::Bool(b);
    }
    if let Ok(n) = s.parse::<i32>() {
        return Val::S32(n);
    }
    if let Ok(n) = s.parse::<u32>() {
        return Val::U32(n);
    }
    if let Ok(n) = s.parse::<i64>() {
        return Val::S64(n);
    }
    if let Ok(n) = s.parse::<u64>() {
        return Val::U64(n);
    }
    if let Ok(f) = s.parse::<f32>() {
        return Val::Float32(f);
    }
    if let Ok(f) = s.parse::<f64>() {
        return Val::Float64(f);
    }
    Val::String(s.to_string())
}

/// Converts a [`Val`] to a JSON-compatible [`serde_json::Value`].
fn val_to_json(val: &Val) -> serde_json::Value {
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

/// Executes the `run` command with optional verification and execution limits.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn execute_run(
    component_path: &Path,
    function_name: &str,
    args: &[String],
    verify_opts: &RunVerificationOptions<'_>,
    limits_opts: &RunExecutionLimits,
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

    // Pre-execution validation of limit options (AN-047)
    if let Some(fuel) = limits_opts.fuel {
        if fuel == 0 {
            return emit_error(
                format,
                "Validation error: --fuel must be greater than zero",
                CliExitCode::UsageOrIoError,
            );
        }
    }

    if let Some(mem) = limits_opts.max_memory_bytes {
        if mem == 0 {
            return emit_error(
                format,
                "Validation error: --max-memory-bytes must be greater than zero",
                CliExitCode::UsageOrIoError,
            );
        }
    }

    if let Some(tables) = limits_opts.max_table_elements {
        if tables == 0 {
            return emit_error(
                format,
                "Validation error: --max-table-elements must be greater than zero",
                CliExitCode::UsageOrIoError,
            );
        }
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

    // Pre-execution verification (AN-041 & AN-042)
    let is_verifying = verify_opts.verify
        || verify_opts.expected_sha256.is_some()
        || verify_opts.manifest_path.is_some()
        || verify_opts.signature_path.is_some()
        || verify_opts.public_key.is_some();

    let mut verification_summary: Option<VerificationSummary> = None;

    if is_verifying {
        let computed_sha = compute_sha256_hex(&component_bytes);

        // Resolve expected SHA-256
        let expected_sha = match resolve_expected_sha256(
            component_path,
            verify_opts.expected_sha256,
            verify_opts.manifest_path,
            verify_opts.verify,
        ) {
            Ok(s) => s,
            Err(err) => {
                return emit_error(
                    format,
                    &format!("Integrity verification error: {err}"),
                    CliExitCode::DomainFailure,
                );
            }
        };

        let mut integrity_ok = false;
        if let Some(ref expected) = expected_sha {
            if computed_sha != *expected {
                return emit_error(
                    format,
                    &format!(
                        "Integrity verification FAILED for component {}: computed SHA-256 ({computed_sha}) does not match expected ({expected})",
                        component_path.display()
                    ),
                    CliExitCode::DomainFailure,
                );
            }
            integrity_ok = true;
        }

        // Cryptographic signature check (AN-042)
        let mut signature_ok = false;
        let mut resolved_sig_path = None;

        if let Some(key_str) = verify_opts.public_key {
            let public_key = match load_minisign_public_key(key_str) {
                Ok(pk) => pk,
                Err(err) => {
                    return emit_error(
                        format,
                        &format!("Public key error: {err}"),
                        CliExitCode::DomainFailure,
                    );
                }
            };

            let Some(sig_file_path) = resolve_signature_path(
                component_path,
                verify_opts.signature_path,
                verify_opts.verify,
            ) else {
                return emit_error(
                    format,
                    &format!(
                        "Signature file not found: expected Minisign signature for {}",
                        component_path.display()
                    ),
                    CliExitCode::DomainFailure,
                );
            };

            if !sig_file_path.exists() {
                return emit_error(
                    format,
                    &format!(
                        "Signature file not found: expected Minisign signature at {}",
                        sig_file_path.display()
                    ),
                    CliExitCode::DomainFailure,
                );
            }

            let signature = match load_minisign_signature(&sig_file_path) {
                Ok(s) => s,
                Err(err) => {
                    return emit_error(
                        format,
                        &format!("Signature read error: {err}"),
                        CliExitCode::DomainFailure,
                    );
                }
            };

            if let Err(err) = public_key.verify(&component_bytes, &signature, false) {
                return emit_error(
                    format,
                    &format!(
                        "Cryptographic signature verification FAILED for component {}: {err}",
                        component_path.display()
                    ),
                    CliExitCode::DomainFailure,
                );
            }

            signature_ok = true;
            resolved_sig_path = Some(sig_file_path.display().to_string());
        } else if expected_sha.is_none() && verify_opts.verify {
            return emit_error(
                format,
                &format!(
                    "Verification FAILED: --verify was requested, but no expected hash (--expected-sha256/--manifest), signature (--signature), or public key (--public-key) was provided, and no adjacent manifest or signature file could be found for {}",
                    component_path.display()
                ),
                CliExitCode::DomainFailure,
            );
        }

        verification_summary = Some(VerificationSummary {
            integrity_verified: integrity_ok,
            computed_sha256: computed_sha,
            expected_sha256: expected_sha,
            signature_verified: signature_ok,
            signature_path: resolved_sig_path,
        });
    }

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

    // Construct execution limits (AN-047)
    let mut limits = ExecutionLimits::unmetered();
    if let Some(fuel) = limits_opts.fuel {
        limits = limits.with_fuel(fuel);
    }
    if let Some(mem) = limits_opts.max_memory_bytes {
        limits = limits.with_max_memory_bytes(mem);
    }
    if let Some(tables) = limits_opts.max_table_elements {
        let mut il = limits.instance_limits.unwrap_or_default();
        il.max_table_elements = tables;
        limits = limits.with_instance_limits(il);
    }

    let parsed_args: Vec<Val> = args.iter().map(|s| parse_arg_to_val(s)).collect();

    // Try direct invocation first, then common Component Model qualifier candidates if bare function name passed
    let mut invoke_result = component.invoke_with_limits(function_name, &parsed_args, &limits);
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
            let candidate_res = component.invoke_with_limits(&candidate, &parsed_args, &limits);
            if !matches!(candidate_res, Err(RuntimeError::FunctionNotFound(_))) {
                invoke_result = candidate_res;
                break;
            }
        }
    }

    match invoke_result {
        Ok(report) => {
            let results_json: Vec<serde_json::Value> =
                report.values.iter().map(val_to_json).collect();
            let results_formatted: Vec<String> =
                report.values.iter().map(|v| format!("{v:?}")).collect();

            let out = RunCommandOutput {
                success: true,
                component_path: component_path.display().to_string(),
                function_name: function_name.to_string(),
                results: results_json,
                results_formatted,
                fuel_consumed: report.fuel_consumed,
                verification: verification_summary,
                error: None,
            };

            emit_result(
                format,
                &out,
                |o| {
                    let mut s = String::new();
                    if let Some(ref v) = o.verification {
                        if v.integrity_verified {
                            s.push_str(&format!(
                                "[VERIFIED] SHA-256 integrity check passed ({})\n",
                                v.computed_sha256
                            ));
                        }
                        if v.signature_verified {
                            s.push_str(&format!(
                                "[VERIFIED] Cryptographic signature check passed ({})\n",
                                v.signature_path.as_deref().unwrap_or("signature")
                            ));
                        }
                    }
                    if o.results_formatted.is_empty() {
                        s.push_str(&format!(
                            "Function '{}' executed successfully (no return value)",
                            o.function_name
                        ));
                    } else {
                        s.push_str(&format!(
                            "Function '{}' returned {} value(s):\n",
                            o.function_name,
                            o.results_formatted.len()
                        ));
                        for (i, res) in o.results_formatted.iter().enumerate() {
                            s.push_str(&format!("  [{i}]: {res}\n"));
                        }
                    }
                    if let Some(fuel) = o.fuel_consumed {
                        s.push_str(&format!("\nFuel consumed: {fuel} units"));
                    }
                    s.trim_end().to_string()
                },
                CliExitCode::Success,
            )
        }
        Err(err) => {
            let err_msg = match &err {
                RuntimeError::FuelExhausted { limit } => {
                    format!(
                        "Execution failed: Computational fuel exhausted (budget: {limit} units)"
                    )
                }
                RuntimeError::MemoryLimitExceeded {
                    requested_bytes,
                    max_bytes,
                } => {
                    format!(
                        "Execution failed: Memory limit exceeded (requested: {requested_bytes} bytes, limit: {max_bytes} bytes)"
                    )
                }
                _ => err.to_string(),
            };

            let out = RunCommandOutput {
                success: false,
                component_path: component_path.display().to_string(),
                function_name: function_name.to_string(),
                results: Vec::new(),
                results_formatted: Vec::new(),
                fuel_consumed: None,
                verification: verification_summary,
                error: Some(err_msg),
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
