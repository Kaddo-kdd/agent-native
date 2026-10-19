//! Publish and artifact store error definitions.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PublishError {
    #[error("IO error during publication: {0}")]
    Io(String),

    #[error("Agent synthesis error for component '{0}': {1}")]
    Synthesis(String, String),

    #[error("Semantic IR verification failed for component '{0}': {1}")]
    IrVerification(String, String),

    #[error("Component compilation failed for component '{0}': {1}")]
    Compilation(String, String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Active pointer not found in {0}")]
    NoActiveVersion(String),

    #[error("Integrity check failed for artifact {0}: expected {1}, found {2}")]
    IntegrityMismatch(String, String, String),

    #[error("Artifact not found in active manifest: {0}")]
    ArtifactNotFound(String),

    #[error("Injected failure at fail point '{0}'")]
    InjectedFailPoint(String),
}

impl From<std::io::Error> for PublishError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for PublishError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}
