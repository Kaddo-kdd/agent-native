//! Typed errors for the YAML executor (AN-019).

use thiserror::Error;

use crate::error::RuntimeError;

/// Taxonomy of failures the YAML executor can surface.
///
/// Per AN-019 precision "deny-by-default + stubs no autorizan", every
/// branch carries enough context for the caller (test binary or future
/// CLI) to decide what to do without parsing error strings.
#[derive(Debug, Error)]
pub enum ExecutorError {
    /// The YAML text is syntactically invalid or does not match the
    /// `Suite` schema.
    #[error("invalid YAML: {0}")]
    InvalidYaml(String),

    /// The requested exported function does not exist in the component.
    #[error("operation '{0}' does not exist on the component")]
    OperationNotFound(String),

    /// The suite declares a stub for an import that the `HostPolicy`
    /// does not authorise. Stubs do NOT grant authorisation.
    #[error("stub declared for '{interface}::{operation}' but policy denies the import")]
    UnauthorizedByPolicy {
        interface: String,
        operation: String,
    },

    /// The component imports an operation for which the suite registers
    /// no stub.
    #[error("component imports '{interface}::{operation}' but no stub is registered")]
    StubMissing {
        interface: String,
        operation: String,
    },

    /// A stub's declared `kind` does not match the import's signature
    /// expected by the component.
    #[error(
        "stub '{interface}::{operation}' kind '{declared_kind}' is incompatible with the imported signature"
    )]
    SignatureMismatch {
        interface: String,
        operation: String,
        declared_kind: String,
    },

    /// Wasmtime reported a trap or runtime error while executing a
    /// stubbed host call or the guest itself.
    #[error("host error during execution: {0}")]
    HostError(String),

    /// Wrapped error from the underlying runtime for cases where the
    /// pre-check or invocation surfaces one of the AN-010 variants.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}
