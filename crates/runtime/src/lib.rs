//! Agent-Native runtime — loads WASM Component Model artifacts and executes
//! them without requiring Cargo, Rust toolchain or an LLM at runtime (ADR-002).
//!
//! This crate depends only on `agent-native-ir` for metadata contracts.
//! A CI lint enforces this invariant (ADR-002 §Lint CI).
//!
//! Content is materialized by INIT-03-01 and later.
