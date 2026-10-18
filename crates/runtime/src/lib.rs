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
//! - [`Engine`] — a reusable Wasmtime engine with Component Model enabled.
//! - [`Component`] — a loaded component ready for invocation.
//! - [`RuntimeError`] — the typed failure taxonomy per ADR-004.
//!
//! The `wasmtime::component::Val` type is re-exported for convenience so
//! callers do not need a direct dependency on Wasmtime for simple cases.
//!
//! INIT-03-04/05/06 will layer error propagation, deny-by-default
//! capability policy and fuel/memory limits on top of this baseline.

pub mod engine;
pub mod error;
pub mod invoke;
pub mod policy;
pub mod resolver;

pub use engine::{Component, Engine};
pub use error::RuntimeError;
pub use policy::HostPolicy;
pub use resolver::Resolver;
pub use wasmtime::component::Val;
