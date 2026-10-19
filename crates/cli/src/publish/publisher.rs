//! Component build, packaging, and atomic publishing pipeline.
//!
//! Orchestrates code generation through [`LlmProvider`], compiles Semantic IR
//! to WASM via `agent-native-compiler`, verifies digests with SHA-256 (ADR-004),
//! stages files in `.staging/`, writes immutable manifests under `versions/`, and
//! atomically promotes `active.json`.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use agent_native_compiler::{CompileContext, compile_ir_to_wasm_with_context, sha256_hex};
use agent_native_ir::verify_ir;

use crate::agent::provider::LlmProvider;
use crate::publish::atomic::atomic_replace_file;
use crate::publish::error::PublishError;
use crate::publish::store::{
    ActivePointer, BuildArtifact, BuildFailPoint, BuildSummary, VersionManifest,
};
use crate::spec::model::SystemSpec;

fn current_timestamp_iso() -> String {
    // Generate deterministic or system timestamp in ISO 8601 format
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", now.as_secs(), now.subsec_millis())
}

/// Recovers, cleans up broken staging states, and verifies the integrity of the active version.
pub fn recover_and_verify_dist_state(
    out_dir: &Path,
) -> Result<Option<(String, VersionManifest)>, PublishError> {
    let temp_staging = out_dir.join(".staging");
    if temp_staging.exists() {
        let _ = std::fs::remove_dir_all(&temp_staging);
    }
    let active_temp = out_dir.join(".active.json.tmp");
    if active_temp.exists() {
        let _ = std::fs::remove_file(&active_temp);
    }

    let active_final = out_dir.join("active.json");
    if !active_final.exists() {
        return Ok(None);
    }

    let active_content = std::fs::read_to_string(&active_final)?;
    let active_pointer: ActivePointer = serde_json::from_str(&active_content)?;

    let manifest_path = out_dir.join(&active_pointer.manifest_path);
    if !manifest_path.exists() {
        return Err(PublishError::NoActiveVersion(format!(
            "Active manifest file missing at {}",
            manifest_path.display()
        )));
    }

    let manifest_content = std::fs::read_to_string(&manifest_path)?;
    let manifest: VersionManifest = serde_json::from_str(&manifest_content)?;

    // Verify SHA-256 integrity for every artifact listed
    for art in &manifest.artifacts {
        let wasm_path = out_dir.join(&art.wasm_file);
        let bytes = std::fs::read(&wasm_path).map_err(|e| {
            PublishError::Io(format!(
                "Artifact binary missing at {}: {e}",
                wasm_path.display()
            ))
        })?;
        let actual_sha = sha256_hex(&bytes);
        if actual_sha != art.sha256 {
            return Err(PublishError::IntegrityMismatch(
                wasm_path.display().to_string(),
                art.sha256.clone(),
                actual_sha,
            ));
        }
    }

    Ok(Some((active_pointer.active_version, manifest)))
}

/// Resolves the absolute path to a component's `.wasm` binary through `active.json`.
pub fn resolve_active_component_artifact(
    out_dir: &Path,
    component_id: &str,
) -> Result<PathBuf, PublishError> {
    let (_, manifest) = recover_and_verify_dist_state(out_dir)?
        .ok_or_else(|| PublishError::NoActiveVersion(out_dir.display().to_string()))?;

    for art in &manifest.artifacts {
        if art.component_id == component_id {
            return Ok(out_dir.join(&art.wasm_file));
        }
    }

    Err(PublishError::ArtifactNotFound(component_id.to_string()))
}

struct StagedItem {
    component_id: String,
    bytes_len: usize,
    sha256: String,
    ir_sha256: String,
}

/// Compiles and atomically publishes all components of a [`SystemSpec`] to `out_dir`.
#[allow(clippy::too_many_lines)]
pub fn build_and_publish_components<P: LlmProvider>(
    provider: &P,
    spec: &SystemSpec,
    out_dir: &Path,
    wit_root: Option<&Path>,
    fail_point: Option<BuildFailPoint>,
) -> Result<BuildSummary, PublishError> {
    let temp_staging = out_dir.join(".staging");
    if temp_staging.exists() {
        let _ = std::fs::remove_dir_all(&temp_staging);
    }
    std::fs::create_dir_all(&temp_staging)?;
    std::fs::create_dir_all(out_dir)?;

    let mut staged_items = Vec::new();
    let compile_context = wit_root.map(CompileContext::new).unwrap_or_default();

    for comp in &spec.components {
        let comp_id = &comp.id;

        // Propose IR using the pluggable LlmProvider
        let proposed_ir = provider
            .propose_ir(comp_id, spec, &[])
            .map_err(|e| PublishError::Synthesis(comp_id.clone(), e.to_string()))?;

        // Verify IR semantics
        let diags = verify_ir(&proposed_ir);
        let error_diags: Vec<_> = diags.iter().filter(|d| d.code.starts_with('E')).collect();
        if !error_diags.is_empty() {
            let _ = std::fs::remove_dir_all(&temp_staging);
            return Err(PublishError::IrVerification(
                comp_id.clone(),
                format!("{error_diags:?}"),
            ));
        }

        // Compile IR to WASM Component Model
        let wasm_bytes = compile_ir_to_wasm_with_context(&proposed_ir, &compile_context)
            .map_err(|diags| PublishError::Compilation(comp_id.clone(), format!("{diags:?}")))?;

        let staged_wasm_path = temp_staging.join(format!("{comp_id}.wasm"));
        std::fs::write(&staged_wasm_path, &wasm_bytes)?;

        let staged_ir_path = temp_staging.join(format!("{comp_id}.ir.json"));
        let ir_bytes = serde_json::to_vec_pretty(&proposed_ir)?;
        std::fs::write(&staged_ir_path, &ir_bytes)?;

        let sha256 = sha256_hex(&wasm_bytes);
        let ir_sha256 = sha256_hex(&ir_bytes);

        staged_items.push(StagedItem {
            component_id: comp_id.clone(),
            bytes_len: wasm_bytes.len(),
            sha256,
            ir_sha256,
        });
    }

    // Version ID based on monotonic timestamp
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let version_id = format!("build-{now_nanos}");
    let versions_dir = out_dir.join("versions");
    let version_path = versions_dir.join(&version_id);
    std::fs::create_dir_all(&version_path)?;

    // Copy staged artifacts to versions/<version_id>/
    let mut versioned_artifacts = Vec::with_capacity(staged_items.len());
    for (idx, staged) in staged_items.iter().enumerate() {
        if fail_point == Some(BuildFailPoint::DuringVersionCopy) && idx == 0 {
            let _ = std::fs::remove_dir_all(&temp_staging);
            return Err(PublishError::InjectedFailPoint(
                "DuringVersionCopy".to_string(),
            ));
        }

        let src_wasm = temp_staging.join(format!("{}.wasm", staged.component_id));
        let dst_wasm = version_path.join(format!("{}.wasm", staged.component_id));
        std::fs::copy(&src_wasm, &dst_wasm)?;

        let src_ir = temp_staging.join(format!("{}.ir.json", staged.component_id));
        let dst_ir = version_path.join(format!("{}.ir.json", staged.component_id));
        std::fs::copy(&src_ir, &dst_ir)?;

        versioned_artifacts.push(BuildArtifact {
            component_id: staged.component_id.clone(),
            wasm_file: format!("versions/{version_id}/{}.wasm", staged.component_id),
            bytes_len: staged.bytes_len,
            sha256: staged.sha256.clone(),
            ir_file: Some(format!(
                "versions/{version_id}/{}.ir.json",
                staged.component_id
            )),
            ir_sha256: Some(staged.ir_sha256.clone()),
            agent_metrics: provider.last_execution_metrics(),
        });
    }

    if fail_point == Some(BuildFailPoint::DuringManifestWrite) {
        let _ = std::fs::remove_dir_all(&temp_staging);
        return Err(PublishError::InjectedFailPoint(
            "DuringManifestWrite".to_string(),
        ));
    }

    let agent_metrics = provider.last_execution_metrics();

    // Write manifest.json
    let manifest = VersionManifest {
        version_id: version_id.clone(),
        created_at: current_timestamp_iso(),
        spec_version: spec.spec_version.clone(),
        spec_source: spec.metadata.source_type.clone(),
        agent_name: provider.name().to_string(),
        agent_metrics: agent_metrics.clone(),
        components_count: versioned_artifacts.len(),
        artifacts: versioned_artifacts.clone(),
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    std::fs::write(version_path.join("manifest.json"), &manifest_bytes)?;

    // Write .active.json.tmp and atomically replace active.json
    let active_pointer = ActivePointer {
        active_version: version_id.clone(),
        promoted_at: current_timestamp_iso(),
        manifest_path: format!("versions/{version_id}/manifest.json"),
        agent_name: provider.name().to_string(),
        agent_metrics: agent_metrics.clone(),
        artifacts: versioned_artifacts.clone(),
    };
    let active_temp = out_dir.join(".active.json.tmp");
    let active_final = out_dir.join("active.json");
    std::fs::write(&active_temp, serde_json::to_vec_pretty(&active_pointer)?)?;

    if fail_point == Some(BuildFailPoint::DuringActivePromote) {
        let _ = std::fs::remove_dir_all(&temp_staging);
        return Err(PublishError::InjectedFailPoint(
            "DuringActivePromote".to_string(),
        ));
    }

    atomic_replace_file(&active_temp, &active_final)?;

    // Clean up temporary staging
    let _ = std::fs::remove_dir_all(&temp_staging);

    Ok(BuildSummary {
        success: true,
        version_id,
        artifacts: versioned_artifacts,
        agent_metrics,
    })
}
