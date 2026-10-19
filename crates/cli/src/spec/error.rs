//! Specification parsing and interpretation errors.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpecError {
    #[error(
        "Narrative contains no recognizable components. Expected at least one component heading e.g. '## Componente (`id`)'."
    )]
    MissingComponents,

    #[error(
        "Narrative contains no recognizable functional requirements. Expected bullet points with stable IDs e.g. '- **ID-001**: statement'."
    )]
    MissingRequirements,

    #[error("Failed to parse specification: {0}")]
    ParseError(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
