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

fn cpu_model() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(info) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in info.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    if matches!(key.trim(), "model name" | "Hardware") && !value.trim().is_empty() {
                        return value.trim().to_string();
                    }
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        command_output("sysctl", &["-n", "machdep.cpu.brand_string"])
    }
    #[cfg(target_os = "windows")]
    {
        command_output(
            "powershell.exe",
            &[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "(Get-CimInstance Win32_Processor | Select-Object -ExpandProperty Name -Unique) -join '; '",
            ],
        )
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    "unknown".to_string()
}

fn main() {
    let cpus = std::thread::available_parallelism().map_or(0, std::num::NonZero::get);
    let sheet = serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "logical_cpus": cpus,
        "cpu_model": cpu_model(),
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
