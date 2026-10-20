//! Runtime error type.
//!
//! Per ADR-004, every failure path returns a typed [`RuntimeError`]. The
//! runtime never falls back to a default value (`0`, `false`, …) on a missing
//! export, type mismatch or trap.

use thiserror::Error;

/// Specific failure modes for capability manifest parsing, validation, and policy checks.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PolicyError {
    /// Deserialization failed due to syntax or schema mismatch.
    #[error("failed to parse capability manifest: {0}")]
    ParseError(String),

    /// The manifest specifies an unsupported schema version.
    #[error("unsupported manifest version '{0}' (expected '0.1')")]
    UnsupportedVersion(String),

    /// A declared capability identifier is malformed or invalid.
    #[error("invalid capability identifier: {0}")]
    InvalidIdentifier(String),

    /// A capability was declared multiple times in the manifest.
    #[error("duplicate capability declaration: {0}")]
    DuplicateDeclaration(String),

    /// Wildcard capabilities are strictly forbidden in declarative manifests.
    #[error("wildcard capabilities are forbidden in declarative manifests: '{0}'")]
    WildcardForbidden(String),
}

/// All failure modes exposed by the Agent-Native runtime.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// A capability manifest or policy rule failed validation or parsing.
    #[error("policy error: {0}")]
    Policy(#[from] PolicyError),

    /// The WASM payload could not be loaded as a Component Model artifact.
    #[error("failed to load component: {0}")]
    LoadFailed(String),

    /// The requested exported function does not exist in the component.
    #[error("exported function '{0}' not found")]
    FunctionNotFound(String),

    /// The exported function expects a different number of arguments.
    #[error("function '{name}' expects {expected} args, got {got}")]
    ArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },

    /// Instantiation of the component failed.
    #[error("instantiation failed: {0}")]
    InstantiationFailed(String),

    /// Execution of the function trapped or returned a Wasmtime error.
    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    /// Execution ran out of allocated computational fuel.
    #[error("execution ran out of fuel (budget: {limit})")]
    FuelExhausted { limit: u64 },

    /// A WebAssembly instance requested or grew linear memory or tables beyond
    /// the allowed limit.
    #[error(
        "memory limit exceeded: requested {requested_bytes} bytes, maximum allowed is {max_bytes} bytes"
    )]
    MemoryLimitExceeded {
        requested_bytes: usize,
        max_bytes: usize,
    },

    /// The component declares a host import that the runtime did not
    /// provide. Detected pre-instantiate per ADR-004 (fail-explicit).
    #[error("unauthorized or missing host import: {module}::{name}")]
    UnauthorizedImport { module: String, name: String },

    /// The caller supplied an argument whose type does not match the
    /// function signature.
    #[error("argument {index} of function '{function}' expected {expected}, found {found}")]
    ArgTypeMismatch {
        function: String,
        index: usize,
        expected: String,
        found: String,
    },
}
