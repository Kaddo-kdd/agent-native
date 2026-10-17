//! Agent-Native CLI entry point.
//!
//! Per ADR-001, this binary wires together the public APIs of
//! `agent-native-ir`, `agent-native-compiler` and `agent-native-runtime`;
//! it never concentrates domain logic. Orchestration (narrative → spec,
//! LLM adapter, publishing) lives here as internal modules until the
//! extraction criteria of ADR-001 §Decision are met.
//!
//! Subcommands are materialized by INIT-05-01.

fn main() {
    println!("agent-native CLI — scaffolding (see roadmap INIT-05-01)");
}
