//! Prints a reproducible environment sheet (AN-051 / INIT-08-01) as JSON.
//!
//! Run before a benchmark session so results can be tied to the machine and
//! toolchain that produced them: `cargo run -p agent-native-runtime --example bench_env`.
//! No external crates and no hardware identifiers (serial numbers, hostnames).

use std::process::Command;

fn command_output(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map_or_else(
            || "unknown".to_string(),
            |o| String::from_utf8_lossy(&o.stdout).trim().to_string(),
        )
}

fn main() {
    let cpus = std::thread::available_parallelism().map_or(0, std::num::NonZero::get);
    let sheet = serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "logical_cpus": cpus,
        "rustc": command_output("rustc", &["--version"]),
        "cargo": command_output("cargo", &["--version"]),
        "wasmtime": "29.0.1",
        "profile": "release (criterion)",
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&sheet).expect("serialisable sheet")
    );
}
