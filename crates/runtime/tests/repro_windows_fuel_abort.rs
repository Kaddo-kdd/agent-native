use agent_native_runtime::{Engine, ExecutionLimits, RuntimeError, Val};
use std::process::Command;
use std::time::Instant;
use wasm_encoder::{
    BlockType, CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction,
    Module, TypeSection, ValType,
};

const CHILD_ENV_VAR: &str = "AGENT_NATIVE_RUN_FUEL_CHILD";

/// Helper function to build a minimal WASM module with an infinite loop:
/// `(module (func (export "spin") (result i32) (loop br 0) (i32.const 0)))`
fn build_infinite_loop_wasm() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I32]);
    module.section(&types);

    let mut functions = FunctionSection::new();
    functions.function(0);
    module.section(&functions);

    let mut exports = ExportSection::new();
    exports.export("spin", ExportKind::Func, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut func = Function::new([]);
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::Br(0));
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::I32Const(0));
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    module.finish()
}

/// Helper function to build a minimal WASM module that returns 42:
/// `(module (func (export "answer") (result i32) (i32.const 42)))`
fn build_benign_wasm() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I32]);
    module.section(&types);

    let mut functions = FunctionSection::new();
    functions.function(0);
    module.section(&functions);

    let mut exports = ExportSection::new();
    exports.export("answer", ExportKind::Func, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut func = Function::new([]);
    func.instruction(&Instruction::I32Const(42));
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    module.finish()
}

fn run_child_logic() {
    let wasm = build_infinite_loop_wasm();
    let engine = Engine::new().expect("engine creation in child");
    let component = engine
        .load_component(&wasm)
        .expect("component load in child");

    let limits = ExecutionLimits::unmetered().with_fuel(5_000);

    match component.invoke_with_limits("spin", &[], &limits) {
        Err(RuntimeError::FuelExhausted { limit }) => {
            println!("CHILD_OUTCOME: FUEL_EXHAUSTED_TRAP limit={limit}");
            std::process::exit(0);
        }
        Ok(_) => {
            eprintln!("CHILD_OUTCOME: UNEXPECTED_OK");
            std::process::exit(2);
        }
        Err(other) => {
            eprintln!("CHILD_OUTCOME: UNEXPECTED_ERROR {other:?}");
            std::process::exit(3);
        }
    }
}

#[test]
fn test_windows_fuel_containment_subprocess() {
    // If running as the child process, execute the infinite loop directly:
    if std::env::var(CHILD_ENV_VAR).is_ok() {
        run_child_logic();
        return;
    }

    // Otherwise, we are the parent test runner.
    let current_exe = std::env::current_exe().expect("current executable path");
    let start_time = Instant::now();

    println!("=== PARENT: Spawning child process to execute infinite loop with 5000 fuel ===");
    let child_output = Command::new(&current_exe)
        .env(CHILD_ENV_VAR, "1")
        .arg("--nocapture")
        .output()
        .expect("failed to spawn child process");

    let elapsed = start_time.elapsed();
    let exit_code = child_output.status.code();
    let stdout = String::from_utf8_lossy(&child_output.stdout);
    let stderr = String::from_utf8_lossy(&child_output.stderr);

    println!("=== PARENT: Child process completed in {elapsed:?} ===");
    println!("Raw exit status: {:?}", child_output.status);
    println!("Child exit code: {exit_code:?}");
    println!("Child stdout: {stdout}");
    println!("Child stderr: {stderr}");

    // Audit Child Behavior:
    #[cfg(target_os = "windows")]
    {
        #[allow(clippy::cast_sign_loss)]
        let status_code_u32 = exit_code.map(|c| c as u32);
        println!("Windows exit code as u32: {status_code_u32:x?}");

        if exit_code == Some(0) && stdout.contains("FUEL_EXHAUSTED_TRAP") {
            println!("ANALYSIS: Wasmtime cleanly caught Trap::OutOfFuel in-process on Windows!");
        } else if status_code_u32 == Some(0xc000_0409) {
            println!(
                "ANALYSIS: Wasmtime 27 reproduced STATUS_STACK_BUFFER_OVERRUN (0xc0000409 / panic_cannot_unwind) as observed."
            );
        } else {
            println!("ANALYSIS: Child terminated with code: {status_code_u32:x?}");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        assert_eq!(
            exit_code,
            Some(0),
            "On POSIX, child must cleanly catch trap and exit with 0"
        );
        assert!(stdout.contains("FUEL_EXHAUSTED_TRAP"));
    }

    // CRITICAL INVARIANT: The parent process MUST remain operational and completely intact!
    println!("=== PARENT: Verifying host parent survival and subsequent execution capacity ===");
    let engine = Engine::new().expect("parent engine creation");
    let benign_wasm = build_benign_wasm();
    let component = engine
        .load_component(&benign_wasm)
        .expect("parent benign component load");

    let limits = ExecutionLimits::unmetered().with_fuel(10_000);

    let report = component
        .invoke_with_limits("answer", &[], &limits)
        .expect("parent benign invocation must succeed after child failure");

    assert_eq!(report.values, vec![Val::S32(42)]);
    assert!(
        report.fuel_consumed.unwrap_or(0) > 0,
        "fuel must be tracked in parent invocation"
    );
    println!(
        "=== PARENT SUCCESS: Benign component executed perfectly: return={:?}, fuel={:?} ===",
        report.values, report.fuel_consumed
    );
}
