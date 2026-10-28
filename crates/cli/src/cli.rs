//! Command-line interface definitions and dispatcher using `clap`.
//!
//! Per ADR-001, this module is strictly a boundary adapter: it parses CLI
//! arguments and flags, and forwards execution to the appropriate command handlers.
//! No domain or compiler logic lives here.

use clap::{Args, Parser, Subcommand};
use std::ffi::OsString;
use std::path::PathBuf;

use crate::commands::{
    RunExecutionLimits, RunVerificationOptions, execute_build_with_options, execute_publish,
    execute_run, execute_test, execute_verify_with_options,
};
use crate::output::{CliExitCode, OutputFormat};

/// Wasma CLI primary interface.
#[derive(Debug, Parser)]
#[command(
    name = "wasma",
    about = "Wasma CLI — compile, verify, test, run and publish WebAssembly components",
    version
)]
pub struct Cli {
    /// Output format for stdout.
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = OutputFormat::Human,
        help = "Output format: 'human' for plain text or 'json' for machine-readable output"
    )]
    pub format: OutputFormat,

    /// Primary subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Compiles a specification (.md) or Semantic IR (.json) to a WASM component.
    Build(BuildArgs),

    /// Verifies static correctness of Semantic IR (.json) or a specification (.md).
    Verify(VerifyArgs),

    /// Executes acceptance test suites (.yaml) against a compiled WASM component.
    Test(TestArgs),

    /// Runs an exported function on a compiled WASM component.
    Run(RunArgs),

    /// Packages and publishes components to an atomic versioned store with manifest.
    Publish(PublishArgs),
}

/// Arguments for `build` subcommand.
#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Path to input specification (.md) or Semantic IR (.json).
    #[arg(help = "Path to input specification (.md) or Semantic IR (.json)")]
    pub input: PathBuf,

    /// Destination path for the compiled .wasm component.
    #[arg(short, long, help = "Output path for the compiled .wasm component")]
    pub output: Option<PathBuf>,

    /// Target component ID to compile (defaults to first component in specification).
    #[arg(short, long, help = "Target component ID to compile")]
    pub component: Option<String>,

    /// Generation mode ('synthesizer', 'fixtures', or 'real').
    #[arg(
        short,
        long,
        default_value = "synthesizer",
        help = "Generation mode: 'synthesizer', 'fixtures', or 'real'"
    )]
    pub mode: String,

    /// Enable experimental Semantic IR v0.6 features (`string`, `result<T, E>`).
    #[arg(
        long,
        help = "Enable experimental Semantic IR v0.6 features (string, result<T, E>)"
    )]
    pub experimental: bool,
}

/// Arguments for `verify` subcommand.
#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// Path to Semantic IR (.json) or specification (.md) to verify.
    #[arg(help = "Path to Semantic IR (.json) or specification (.md) to verify")]
    pub input: PathBuf,

    /// Enable experimental Semantic IR v0.6 features (`string`, `result<T, E>`).
    #[arg(
        long,
        help = "Enable experimental Semantic IR v0.6 features (string, result<T, E>)"
    )]
    pub experimental: bool,
}

/// Arguments for `test` subcommand.
#[derive(Debug, Args)]
pub struct TestArgs {
    /// Path to the acceptance test suite (.yaml).
    #[arg(short, long, help = "Path to the acceptance test suite (.yaml)")]
    pub suite: PathBuf,

    /// Path to the compiled .wasm component artifact.
    #[arg(short, long, help = "Path to the compiled .wasm component artifact")]
    pub component: PathBuf,
}

/// Arguments for `run` subcommand.
#[derive(Debug, Args)]
pub struct RunArgs {
    /// Path to the compiled .wasm component artifact.
    #[arg(short, long, help = "Path to the compiled .wasm component artifact")]
    pub component: PathBuf,

    /// Exported function name to invoke.
    #[arg(short, long, help = "Exported function name to invoke")]
    pub function: String,

    /// Arguments to pass to the function.
    #[arg(help = "Positional arguments to pass to the function")]
    pub args: Vec<String>,

    /// Verify component integrity and/or signature before execution.
    #[arg(
        long,
        help = "Verify component integrity and/or signature before execution"
    )]
    pub verify: bool,

    /// Expected SHA-256 hex digest for component integrity verification.
    #[arg(
        long,
        help = "Expected SHA-256 hex digest for component integrity verification"
    )]
    pub expected_sha256: Option<String>,

    /// Optional path to manifest (.json) declaring component hash.
    #[arg(
        short,
        long,
        help = "Optional path to manifest (.json) declaring component hash"
    )]
    pub manifest: Option<PathBuf>,

    /// Path to Minisign signature (.minisig) file.
    #[arg(long, help = "Path to Minisign signature (.minisig) file")]
    pub signature: Option<PathBuf>,

    /// Minisign Ed25519 public key (inline base64 string or path to .pub file).
    #[arg(
        long,
        help = "Minisign Ed25519 public key (inline base64 string or path to .pub file)"
    )]
    pub public_key: Option<String>,

    /// Computational fuel budget limit.
    #[arg(long, help = "Computational fuel budget limit")]
    pub fuel: Option<u64>,

    /// Maximum linear memory limit in bytes.
    #[arg(long, help = "Maximum linear memory limit in bytes")]
    pub max_memory_bytes: Option<usize>,

    /// Maximum table elements allowed.
    #[arg(long, help = "Maximum table elements allowed")]
    pub max_table_elements: Option<u32>,
}

/// Arguments for `publish` subcommand.
#[derive(Debug, Args)]
pub struct PublishArgs {
    /// Path to the input specification (.md).
    #[arg(short, long, help = "Path to the input specification (.md)")]
    pub spec: PathBuf,

    /// Root directory of the publication store.
    #[arg(short, long, help = "Root directory of the publication store")]
    pub dist_dir: PathBuf,

    /// Work Item IDs that originated this version (repeatable).
    #[arg(long, help = "Work Item ID that originated this version (repeatable)")]
    pub source_wi: Vec<String>,

    /// Git commit SHA of the source code.
    #[arg(long, help = "Git commit SHA of the source code")]
    pub git_commit: Option<String>,

    /// Generation mode ('synthesizer', 'fixtures', or 'real').
    #[arg(
        short,
        long,
        default_value = "synthesizer",
        help = "Generation mode: 'synthesizer', 'fixtures', or 'real'"
    )]
    pub mode: String,
}

/// Dispatches parsed CLI commands to their respective implementations.
#[must_use]
pub fn dispatch(cli: Cli) -> CliExitCode {
    match cli.command {
        Commands::Build(args) => execute_build_with_options(
            &args.input,
            args.output.as_deref(),
            args.component.as_deref(),
            &args.mode,
            args.experimental,
            cli.format,
        ),
        Commands::Verify(args) => {
            execute_verify_with_options(&args.input, args.experimental, cli.format)
        }
        Commands::Test(args) => execute_test(&args.suite, &args.component, cli.format),
        Commands::Run(args) => {
            let verify_opts = RunVerificationOptions {
                verify: args.verify,
                expected_sha256: args.expected_sha256.as_deref(),
                manifest_path: args.manifest.as_deref(),
                signature_path: args.signature.as_deref(),
                public_key: args.public_key.as_deref(),
            };
            let limits_opts = RunExecutionLimits {
                fuel: args.fuel,
                max_memory_bytes: args.max_memory_bytes,
                max_table_elements: args.max_table_elements,
            };
            execute_run(
                &args.component,
                &args.function,
                &args.args,
                &verify_opts,
                &limits_opts,
                cli.format,
            )
        }
        Commands::Publish(args) => execute_publish(
            &args.spec,
            &args.dist_dir,
            &args.source_wi,
            args.git_commit.as_deref(),
            &args.mode,
            cli.format,
        ),
    }
}

fn is_json_format_requested<I, T>(args: I) -> bool
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let mut prev_is_format = false;
    for arg in args {
        let s = arg.into().to_string_lossy().to_string();
        if prev_is_format && s.eq_ignore_ascii_case("json") {
            return true;
        }
        if s == "--format=json" || s.starts_with("--format=json") {
            return true;
        }
        prev_is_format = s == "--format" || s == "-f";
    }
    false
}

/// Parses CLI arguments from environment and executes the CLI.
#[must_use]
pub fn run() -> CliExitCode {
    let raw_args: Vec<OsString> = std::env::args_os().collect();
    run_with_args(raw_args)
}

/// Parses custom arguments and executes the CLI (useful for in-process integration testing).
#[must_use]
pub fn run_with_args<I, T>(args: I) -> CliExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let items: Vec<T> = args.into_iter().collect();
    let json_requested = is_json_format_requested(items.iter().cloned());

    match Cli::try_parse_from(items) {
        Ok(cli) => dispatch(cli),
        Err(err) => {
            if err.use_stderr() {
                if json_requested {
                    let err_str = err.to_string();
                    let clean_msg = err_str.lines().next().unwrap_or(&err_str).trim();
                    let val = serde_json::json!({
                        "success": false,
                        "error": clean_msg,
                        "exit_code": CliExitCode::UsageOrIoError as u8
                    });
                    if let Ok(json) = serde_json::to_string_pretty(&val) {
                        println!("{json}");
                    }
                } else {
                    let _ = err.print();
                }
                CliExitCode::UsageOrIoError
            } else {
                let _ = err.print();
                CliExitCode::Success
            }
        }
    }
}
