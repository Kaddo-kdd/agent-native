//! Subcommand modules and dispatcher for Agent-Native CLI.

pub mod build;
pub mod publish;
pub mod run;
pub mod test;
pub mod verify;

pub use build::execute_build;
pub use publish::execute_publish;
pub use run::{RunVerificationOptions, execute_run};
pub use test::execute_test;
pub use verify::execute_verify;
