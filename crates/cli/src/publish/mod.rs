//! Atomic versioned publication and artifact store.
//!
//! # Architecture & ADR-001 Invariants
//!
//! Per ADR-001 and ADR-004, artifact storage, manifest versioning, and atomic
//! active-pointer promotion live inside the orchestration/CLI library (`crates/cli/src/publish/`),
//! ensuring that `agent-native-compiler` remains completely independent of the filesystem store.

pub mod atomic;
pub mod error;
pub mod publisher;
pub mod store;

pub use atomic::atomic_replace_file;
pub use error::PublishError;
pub use publisher::{
    PublishOptions, build_and_publish_components, build_and_publish_components_with_options,
    recover_and_verify_dist_state, resolve_active_component_artifact,
};
pub use store::{
    ActivePointer, BuildArtifact, BuildFailPoint, BuildSummary, SpecTraceability, VersionManifest,
};
