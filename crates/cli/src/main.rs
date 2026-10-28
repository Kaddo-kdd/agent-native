//! Wasma CLI binary entry point.
//!
//! Per ADR-001, this binary wires together the public APIs of
//! `wasma-ir`, `wasma-compiler` and `wasma-runtime`;
//! it never concentrates domain logic. Orchestration (narrative → spec,
//! LLM adapter, publishing) lives as internal modules.
//!
//! Subcommands are materialized by INIT-05-01 / AN-028.

use std::process::ExitCode;

fn main() -> ExitCode {
    wasma_cli::run().into()
}
