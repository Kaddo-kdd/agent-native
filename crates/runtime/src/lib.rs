//! Agent-Native runtime — loads WASM Component Model artifacts and executes
//! them without requiring Cargo, Rust toolchain or an LLM at runtime.
//!
//! # Design invariants
//!
//! Per ADR-001 and ADR-002, this crate must depend only on `agent-native-ir`
//! and Wasmtime. It must not reach into `agent-native-compiler`, any LLM
//! adapter or Kaddo. The workflow lint `cargo tree -p agent-native-runtime
//! --edges normal` enforces this invariant.
//!
//! Per ADR-005 (baseline operativa), each invocation runs in a fresh
//! `wasmtime::Store`; instances are not reused across tenants until
//! INIT-06-03 provides isolation guarantees.
//!
//! # Public API
//!
//! The public surface provides six primary entry points:
//!
//! - [`Engine`] — a reusable Wasmtime engine configured for Component Model execution.
//! - [`Component`] — a loaded component ready for invocation (`invoke`, `invoke_with_host_ctx`).
//! - [`HostPolicy`] — declarative deny-by-default capability policy for host imports.
//! - [`RuntimeError`] — typed failure taxonomy per ADR-004.
//! - [`Resolver`] — content-addressed artifact cache keyed by SHA-256 for repeated loads.
//! - [`Val`] — re-export of `wasmtime::component::Val` for typed argument passing and return lifting.
//!
//! # Usage Example
//!
//! ```no_run
//! use agent_native_runtime::{Engine, HostPolicy, RuntimeError, Val};
//!
//! fn run_artifact(wasm_bytes: &[u8]) -> Result<Vec<Val>, RuntimeError> {
//!     let engine = Engine::new()?;
//!     let mut policy = HostPolicy::deny_all();
//!     // Allow specific host interfaces if required:
//!     // policy.allow_wit_interface("an:host/logging@0.1.0");
//!
//!     let component = engine.load_component_with_policy(wasm_bytes, &policy)?;
//!     let results = component.invoke("ops#calculate", &[Val::S32(21)])?;
//!     Ok(results)
//! }
//! ```

pub mod engine;
pub mod error;
pub mod invoke;
pub mod manifest;
pub mod policy;
pub mod resolver;
pub mod testing;

pub use engine::{Component, Engine};
pub use error::{PolicyError, RuntimeError};
pub use manifest::{
    CURRENT_MANIFEST_VERSION, ComponentManifest, CoreImport, ManifestCapabilities, WitOperation,
};
pub use policy::HostPolicy;
pub use resolver::Resolver;
pub use wasmtime::component::Val;
