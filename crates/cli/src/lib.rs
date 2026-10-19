//! Agent-Native library and orchestration modules.
//!
//! Per ADR-001, the CLI binary wires together the public APIs of
//! `agent-native-ir`, `agent-native-compiler` and `agent-native-runtime`.
//! Orchestration logic (narrative → `SystemSpec`, LLM adapter, atomic publishing)
//! lives here within `agent_native` as internal modules (`spec::`, `build::`, `publish::`)
//! until extraction criteria of ADR-001 §Decision are met.

pub mod agent;
pub mod cli;
pub mod commands;
pub mod output;
pub mod publish;
pub mod spec;

pub use cli::{Cli, Commands, dispatch, run, run_with_args};
pub use output::{CliExitCode, OutputFormat};
