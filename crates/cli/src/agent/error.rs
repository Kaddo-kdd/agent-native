//! Agent and LLM provider error types.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AgentError {
    #[error("Component '{0}' not found in system specification")]
    ComponentNotFound(String),

    #[error("No pre-recorded fixture found for component '{0}' in fixtures mode")]
    FixtureNotFound(String),

    #[error(
        "Missing API key for LLM provider '{0}'. Set WASMA_LLM_KEY (or legacy AGENT_NATIVE_LLM_KEY)"
    )]
    MissingApiKey(String),

    #[error("HTTP transport error: {0}")]
    Http(String),

    #[error("Failed to parse JSON response: {0}")]
    Json(String),

    #[error("Synthesis failed for component '{0}': {1}")]
    Synthesis(String, String),

    #[error("Semantic IR validation failed for generated component '{0}': {1}")]
    InvalidIr(String, String),

    #[error("All providers in fallback chain failed")]
    AllProvidersFailed,
}
