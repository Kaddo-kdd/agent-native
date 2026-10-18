//! Agent-Native compiler — Semantic IR to WASM Component Model.
//!
//! Per ADR-001 and ADR-002, this crate depends only on `agent-native-ir` and
//! WASM toolchain crates (`wasm-encoder`, `wasmparser`, `wasmprinter`,
//! `wit-component`, `sha2`). It must not depend on `agent-native-runtime` nor
//! on any LLM adapter.
//!
//! # Public API
//!
//! - [`compile_ir_to_wasm`] — compiles a verified IR document to deterministic
//!   WASM Component Model bytes. Backwards-compatible shortcut that uses the
//!   current working directory as WIT root.
//! - [`compile_ir_to_wasm_with_context`] — explicit [`CompileContext`]
//!   variant that eliminates the cwd dependency (AN-016).
//! - [`CompileContext`] — carries the WIT resolution root.
//! - [`validate_wasm_bytes`] — validates a WASM payload against the component
//!   model.
//! - [`print_wasm_to_wat`] — pretty-prints WASM bytes as WAT for debugging.
//! - [`sha256_hex`] — canonical SHA-256 digest for artifact integrity
//!   (ADR-004).

pub mod codegen;
pub mod context;
pub mod hash;

pub use codegen::{
    compile_ir_to_wasm, compile_ir_to_wasm_with_context, print_wasm_to_wat, validate_wasm_bytes,
};
pub use context::{CompileContext, ResolveOutcome};
pub use hash::sha256_hex;
