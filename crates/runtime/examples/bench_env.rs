//! Prints a reproducible environment sheet (AN-051 / INIT-08-01) as JSON.
//!
//! Run before a benchmark session so results can be tied to the machine and
//! toolchain that produced them: `cargo run -p wasma-runtime --example bench_env`.
//! No external crates and no hardware identifiers (serial numbers, hostnames).

use std::process::Command;
use std::time::Instant;

fn optional_command(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|output| !output.is_empty())
}

fn command_output(program: &str, args: &[&str]) -> String {
    optional_command(program, args).unwrap_or_else(|| "unknown".to_string())
}

#[cfg(target_os = "linux")]
fn cpu_model() -> String {
    if let Ok(info) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in info.lines() {
            if let Some((key, value)) = line.split_once(':') {
                if matches!(key.trim(), "model name" | "Hardware") && !value.trim().is_empty() {
                    return value.trim().to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

#[cfg(target_os = "macos")]
fn cpu_model() -> String {
    command_output("sysctl", &["-n", "machdep.cpu.brand_string"])
}

#[cfg(target_os = "windows")]
fn cpu_model() -> String {
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

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn cpu_model() -> String {
    "unknown".to_string()
}

#[cfg(target_os = "linux")]
fn load_average_1m() -> Option<f64> {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|value| value.split_whitespace().next()?.parse().ok())
}

#[cfg(target_os = "macos")]
fn load_average_1m() -> Option<f64> {
    optional_command("sysctl", &["-n", "vm.loadavg"]).and_then(|value| {
        value
            .trim_matches(|character| character == '{' || character == '}')
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    })
}

#[cfg(target_os = "windows")]
fn load_average_1m() -> Option<f64> {
    optional_command(
        "powershell.exe",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average",
        ],
    )
    .and_then(|value| value.parse().ok())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn load_average_1m() -> Option<f64> {
    None
}

#[cfg(target_os = "linux")]
fn reported_cpu_frequency_mhz() -> Option<u64> {
    let cpus = std::fs::read_dir("/sys/devices/system/cpu").ok()?;
    for cpu in cpus.flatten() {
        let name = cpu.file_name();
        if !name.to_string_lossy().starts_with("cpu") {
            continue;
        }
        let path = cpu.path().join("cpufreq/scaling_cur_freq");
        if let Ok(value) = std::fs::read_to_string(path) {
            if let Ok(khz) = value.trim().parse::<u64>() {
                return Some(khz / 1_000);
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn reported_cpu_frequency_mhz() -> Option<u64> {
    optional_command("sysctl", &["-n", "hw.cpufrequency"])
        .and_then(|value| value.parse::<u64>().ok())
        .map(|hertz| hertz / 1_000_000)
}

#[cfg(target_os = "windows")]
fn reported_cpu_frequency_mhz() -> Option<u64> {
    optional_command(
        "powershell.exe",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_Processor | Measure-Object -Property CurrentClockSpeed -Average).Average",
        ],
    )
    .and_then(|value| value.parse().ok())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn reported_cpu_frequency_mhz() -> Option<u64> {
    None
}

#[cfg(target_os = "linux")]
fn frequency_source() -> &'static str {
    "sysfs scaling_cur_freq snapshot (first available CPU)"
}

#[cfg(target_os = "macos")]
fn frequency_source() -> &'static str {
    "sysctl hw.cpufrequency"
}

#[cfg(target_os = "windows")]
fn frequency_source() -> &'static str {
    "Win32_Processor CurrentClockSpeed average"
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn frequency_source() -> &'static str {
    "unavailable"
}

#[cfg(target_os = "linux")]
fn process_affinity() -> Option<String> {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("Cpus_allowed_list:")
                    .map(|value| value.trim().to_string())
            })
        })
}

#[cfg(target_os = "windows")]
fn process_affinity() -> Option<String> {
    optional_command(
        "powershell.exe",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "[System.Diagnostics.Process]::GetCurrentProcess().ProcessorAffinity.ToInt64().ToString('X')",
        ],
    )
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn process_affinity() -> Option<String> {
    None
}

fn clock_min_positive_step_ns() -> Option<u64> {
    let mut previous = Instant::now();
    let mut minimum = None;
    for _ in 0..100_000 {
        let now = Instant::now();
        let step = now.duration_since(previous).as_nanos();
        if step > 0 {
            let step = u64::try_from(step).ok()?;
            minimum = Some(minimum.map_or(step, |current: u64| current.min(step)));
        }
        previous = now;
    }
    minimum
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
        "host_telemetry": {
            "load_average_1m": load_average_1m(),
            "reported_cpu_frequency_mhz": reported_cpu_frequency_mhz(),
            "reported_cpu_frequency_source": frequency_source(),
            "process_cpu_affinity": process_affinity(),
            "available_parallelism": cpus,
            "clock_min_positive_step_ns": clock_min_positive_step_ns(),
            "clock_method": "minimum positive Instant::now step across 100000 consecutive reads",
        },
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&sheet).expect("serialisable sheet")
    );
}
