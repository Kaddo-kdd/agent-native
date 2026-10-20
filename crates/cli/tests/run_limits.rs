//! Integration tests for configurable execution limits in `agent-native run` (AN-047).
//!
//! # Invariants Verified
//! - `--fuel <u64>` enforces finite computational fuel budget.
//! - Successful invocation with `--fuel` reports `fuel_consumed > 0` in human and JSON formats.
//! - Exhaustion of fuel returns typed `RuntimeError::FuelExhausted` and exit code 1.
//! - `--max-memory-bytes <usize>` enforces linear memory limits.
//! - Exceeding memory quota returns typed `RuntimeError::MemoryLimitExceeded` and exit code 1.
//! - Unmetered invocations (without limit flags) preserve full backward compatibility and omit `fuel_consumed`.
//! - Input validation rejects 0 or invalid arguments with exit code 2.
//! - Works predictably across platforms, including Windows MSVC (Wasmtime 29.0.1 containment).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, MemorySection,
    MemoryType, Module, TypeSection, ValType,
};

fn cli_binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_agent-native"))
}

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let temp = std::env::temp_dir().join(format!(
            "agent_native_limits_test_{name}_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("must create test temp dir");
        Self { path: temp }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

const VALID_IDENTITY_IR: &str = r#"{
    "ir_version": "0.1",
    "spec_version": 1,
    "functions": [
        {
            "name": "identity",
            "export": true,
            "params": [{ "name": "value", "type": "i32" }],
            "returns": "i32",
            "requirements": ["LIMIT-001"],
            "body": { "op": "param", "name": "value" }
        }
    ]
}"#;

fn build_test_component(dir: &TestDir, name: &str) -> PathBuf {
    let ir_file = dir.path().join(format!("{name}.json"));
    let wasm_file = dir.path().join(format!("{name}.wasm"));
    fs::write(&ir_file, VALID_IDENTITY_IR).expect("must write IR file");

    let build_out = Command::new(cli_binary_path())
        .arg("build")
        .arg(&ir_file)
        .arg("-o")
        .arg(&wasm_file)
        .output()
        .expect("must execute build");
    assert_eq!(build_out.status.code(), Some(0), "build must succeed");
    wasm_file
}

fn build_wasm_with_memory(pages: u64) -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I32]);
    module.section(&types);

    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);

    let mut memories = MemorySection::new();
    memories.memory(MemoryType {
        minimum: pages,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memories);

    let mut exports = ExportSection::new();
    exports.export("get_val", ExportKind::Func, 0);
    exports.export("memory", ExportKind::Memory, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut func = Function::new([]);
    func.instruction(&Instruction::I32Const(42));
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    module.finish()
}

#[test]
fn test_run_with_sufficient_fuel_succeeds_and_reports_fuel_consumed() {
    let dir = TestDir::new("fuel_success");
    let wasm_file = build_test_component(&dir, "identity");

    // 1. Human format test
    let output = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("42")
        .arg("--fuel")
        .arg("100000")
        .output()
        .expect("must execute run");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Function 'identity' returned 1 value(s)"));
    assert!(
        stdout.contains("Fuel consumed:"),
        "Must output fuel consumed"
    );

    // 2. JSON format test
    let output_json = Command::new(cli_binary_path())
        .arg("--format=json")
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("42")
        .arg("--fuel")
        .arg("100000")
        .output()
        .expect("must execute run");

    assert_eq!(output_json.status.code(), Some(0));
    let stdout_json = String::from_utf8_lossy(&output_json.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout_json).expect("valid JSON expected");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["function_name"], "identity");
    assert_eq!(parsed["results"], serde_json::json!([42]));
    assert!(
        parsed.get("fuel_consumed").is_some(),
        "JSON must contain fuel_consumed"
    );
    let fuel = parsed["fuel_consumed"].as_u64().expect("fuel as u64");
    assert!(fuel > 0, "Consumed fuel must be greater than 0");
}

#[test]
fn test_run_with_fuel_exhaustion_fails_with_typed_error() {
    let dir = TestDir::new("fuel_exhausted");
    let wasm_file = build_test_component(&dir, "identity");

    // Executing with fuel budget = 1 unit must exhaust
    let output = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("42")
        .arg("--fuel")
        .arg("1")
        .output()
        .expect("must execute run");

    assert_eq!(output.status.code(), Some(1));
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("Computational fuel exhausted (budget: 1 units)"),
        "Error message must specify fuel exhaustion: {combined}"
    );

    // JSON format error
    let output_json = Command::new(cli_binary_path())
        .arg("--format=json")
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("42")
        .arg("--fuel")
        .arg("1")
        .output()
        .expect("must execute run");

    assert_eq!(output_json.status.code(), Some(1));
    let stdout_json = String::from_utf8_lossy(&output_json.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout_json).expect("valid JSON expected");
    assert_eq!(parsed["success"], false);
    let err_str = parsed["error"].as_str().unwrap_or("");
    assert!(err_str.contains("Computational fuel exhausted (budget: 1 units)"));
}

#[test]
fn test_run_with_max_memory_bytes_success_and_exceeded() {
    let dir = TestDir::new("memory_limits");
    let wasm_bytes = build_wasm_with_memory(2); // 2 pages = 131,072 bytes
    let wasm_file = dir.path().join("mem_test.wasm");
    fs::write(&wasm_file, wasm_bytes).expect("write wasm");

    // 1. Generous memory quota (256 KiB >= 131,072 bytes) succeeds
    let output_ok = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("get_val")
        .arg("--max-memory-bytes")
        .arg("262144")
        .output()
        .expect("must execute run");

    assert_eq!(output_ok.status.code(), Some(0));

    // 2. Insufficient memory quota (64 KiB < 131,072 bytes) fails with MemoryLimitExceeded
    let output_exceeded = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("get_val")
        .arg("--max-memory-bytes")
        .arg("65536")
        .output()
        .expect("must execute run");

    assert_eq!(output_exceeded.status.code(), Some(1));
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output_exceeded.stdout),
        String::from_utf8_lossy(&output_exceeded.stderr)
    );
    assert!(
        combined.contains("Memory limit exceeded"),
        "Must report memory limit exceeded: {combined}"
    );
}

#[test]
fn test_run_unmetered_backward_compatibility() {
    let dir = TestDir::new("unmetered");
    let wasm_file = build_test_component(&dir, "identity");

    let output = Command::new(cli_binary_path())
        .arg("--format=json")
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("99")
        .output()
        .expect("must execute run");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON expected");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["results"], serde_json::json!([99]));
    assert!(
        parsed.get("fuel_consumed").is_none(),
        "Unmetered invocation must omit fuel_consumed"
    );
}

#[test]
fn test_run_invalid_limit_arguments_rejected_early() {
    let dir = TestDir::new("invalid_args");
    let wasm_file = build_test_component(&dir, "identity");

    // --fuel 0 must fail with exit code 2
    let out_fuel_0 = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("42")
        .arg("--fuel")
        .arg("0")
        .output()
        .expect("must execute run");

    assert_eq!(out_fuel_0.status.code(), Some(2));
    let combined_fuel = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out_fuel_0.stdout),
        String::from_utf8_lossy(&out_fuel_0.stderr)
    );
    assert!(combined_fuel.contains("--fuel must be greater than zero"));

    // --max-memory-bytes 0 must fail with exit code 2
    let out_mem_0 = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("42")
        .arg("--max-memory-bytes")
        .arg("0")
        .output()
        .expect("must execute run");

    assert_eq!(out_mem_0.status.code(), Some(2));
    let combined_mem = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out_mem_0.stdout),
        String::from_utf8_lossy(&out_mem_0.stderr)
    );
    assert!(combined_mem.contains("--max-memory-bytes must be greater than zero"));
}
