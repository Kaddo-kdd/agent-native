//! Runtime error type.
//!
//! Per ADR-004, every failure path returns a typed [`RuntimeError`]. The
//! runtime never falls back to a default value (`0`, `false`, …) on a missing
//! export, type mismatch or trap.

use thiserror::Error;

/// All failure modes exposed by the Agent-Native runtime.
#[derive(Debug, Error)]
pub enum RuntimeError {
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
}
