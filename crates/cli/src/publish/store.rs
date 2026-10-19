//! Artifact store models, manifests, and active version pointers.

use serde::{Deserialize, Serialize};

use crate::agent::provider::AgentExecutionMetrics;

/// Information about a single compiled component artifact published in a version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildArtifact {
    pub component_id: String,
    pub wasm_file: String,
    pub bytes_len: usize,
    pub sha256: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ir_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ir_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub agent_metrics: Option<AgentExecutionMetrics>,
}

/// Inmutable version manifest written to `versions/<version_id>/manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionManifest {
    pub version_id: String,
    pub created_at: String,
    pub spec_version: String,
    pub spec_source: String,
    pub agent_name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub agent_metrics: Option<AgentExecutionMetrics>,
    pub components_count: usize,
    pub artifacts: Vec<BuildArtifact>,
}

/// Active distribution pointer written to `active.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivePointer {
    pub active_version: String,
    pub promoted_at: String,
    pub manifest_path: String,
    pub agent_name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub agent_metrics: Option<AgentExecutionMetrics>,
    pub artifacts: Vec<BuildArtifact>,
}

/// Summary returned upon successful build and publication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildSummary {
    pub success: bool,
    pub version_id: String,
    pub artifacts: Vec<BuildArtifact>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub agent_metrics: Option<AgentExecutionMetrics>,
}

/// Optional fail-point injection for resilience and recovery testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildFailPoint {
    DuringVersionCopy,
    DuringManifestWrite,
    DuringActivePromote,
}
