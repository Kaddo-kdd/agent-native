//! Deprecated legacy alias binary for Agent Native -> Wasma.
//!
//! Enabled only when the `legacy-alias` feature is explicitly compiled.
//! Emits a deprecation warning to stderr and forwards execution to Wasma.

use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!(
        "[WARN] 'agent-native' está obsoleto y será retirado en Wasma v0.2.0. Use 'wasma' en su lugar."
    );
    wasma_cli::run().into()
}
