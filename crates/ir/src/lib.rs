//! Agent-Native Semantic IR — model, verifier and schemas.
//!
//! This crate hosts the Semantic IR **v0.5** baseline (see ADR-003). It owns
//! the data types that describe an IR document, the verifier (migrated in
//! AN-006) and the regression suite (migrated in AN-007).
//!
//! # Design invariants
//!
//! Per ADR-001 and ADR-002:
//!
//! - `agent-native-ir` has no runtime, compiler or LLM dependency.
//! - Only `serde` and `serde_json` are allowed as external crates; WASM
//!   toolchain crates (`wasm-encoder`, `wasmparser`, `wasmprinter`,
//!   `wit-component`) belong to `agent-native-compiler`.
//! - Hash computation (`sha2`) belongs to `agent-native-compiler` as well.
//!
//! # Modules
//!
//! - [`diagnostic`] — the `Diagnostic` type returned by the verifier.
//! - [`model`] — IR data types (`IrType`, `ImportSignature`, `RecordDef`,
//!   `WitContractDef`, `FuncSignature`).
//! - [`context`] — the `Context` built while verifying an IR document.
//!
//! # Schemas
//!
//! JSON schemas live under `schemas/`:
//!
//! - `schemas/ir.schema.json` — Semantic IR v0.5 (baseline, ADR-003).
//! - `schemas/history/ir.schema.v{2,3,4}.json` — historical lineage, kept
//!   for auditability. These are **not** public API.

pub mod context;
pub mod diagnostic;
pub mod model;

pub use context::Context;
pub use diagnostic::Diagnostic;
pub use model::{FuncSignature, ImportSignature, IrType, RecordDef, WitContractDef};
