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

    #[error(
        "Reconciliation conflict in {0}: Requirement '{1}' is version {2}, but update proposes lower or identical version {3}"
    )]
    ReconciliationConflict(String, String, u32, u32),

    #[error(
        "Broken reference in {0}: Work item attempts to replace non-existent requirement '{1}'"
    )]
    BrokenReference(String, String),
}
