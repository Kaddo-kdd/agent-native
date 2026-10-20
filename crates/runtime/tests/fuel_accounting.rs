//! AN-035: Fuel accounting and CPU budget tests.

use agent_native_compiler::{CompileContext, compile_ir_to_wasm, compile_ir_to_wasm_with_context};
use agent_native_runtime::{Engine, ExecutionLimits, HostPolicy, RuntimeError, Val};
use serde_json::json;
use std::path::PathBuf;

fn sample_math_wasm() -> Vec<u8> {
    let ir = json!({
        "ir_version": "0.1",
        "spec_version": 1,
        "functions": [
            {
                "name": "add_numbers",
                "export": true,
                "params": [{ "name": "a", "type": "i32" }, { "name": "b", "type": "i32" }],
                "returns": "i32",
                "requirements": ["MATH-001"],
                "body": {
                    "op": "add",
                    "left": { "op": "param", "name": "a" },
                    "right": { "op": "param", "name": "b" }
                }
            },
            {
                "name": "identity",
                "export": true,
                "params": [{ "name": "val", "type": "i32" }],
                "returns": "i32",
                "requirements": ["MATH-002"],
                "body": { "op": "param", "name": "val" }
            }
        ]
    });
    compile_ir_to_wasm(&ir).expect("compile must succeed")
}

fn compiler_crate_ctx() -> CompileContext {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.push("compiler");
    CompileContext::new(root)
}

fn probe_cm_wasm() -> Vec<u8> {
    let text = include_str!("../../compiler/tests/fixtures/ir/probe-use-types.ir.json");
    let ir: serde_json::Value = serde_json::from_str(text).expect("valid JSON");
    compile_ir_to_wasm_with_context(&ir, &compiler_crate_ctx()).expect("compile must succeed")
}

const PROBE_TYPES_IFACE: &str = "an:probe/types@0.1.0";
const EXPORTED_OPS_IFACE: &str = "an:probe/ops@0.1.0";

#[test]
fn test_fuel_consumption_and_determinism_on_core_wasm() {
    let wasm = sample_math_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    let limits = ExecutionLimits::unmetered().with_fuel(10_000);

    // Run 1
    let report1 = component
        .invoke_with_limits("add_numbers", &[Val::S32(15), Val::S32(25)], &limits)
        .expect("invoke must succeed");
    assert_eq!(report1.values, vec![Val::S32(40)]);
    let consumed1 = report1.fuel_consumed.expect("fuel must be reported");
    assert!(consumed1 > 0, "fuel consumed must be > 0");
    assert!(
        consumed1 <= 10_000,
        "fuel consumed must be within the 10_000 limit"
    );

    // Run 2: Exact same input must consume the exact same fuel (determinism)
    let report2 = component
        .invoke_with_limits("add_numbers", &[Val::S32(15), Val::S32(25)], &limits)
        .expect("invoke must succeed");
    assert_eq!(report2.values, vec![Val::S32(40)]);
    let consumed2 = report2.fuel_consumed.expect("fuel must be reported");
    assert_eq!(
        consumed1, consumed2,
        "fuel consumption must be bit-for-bit deterministic"
    );
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "Wasmtime 27 upstream issue #10923: libcall out_of_gas is declared as extern \"C\" instead of extern \"C-unwind\", causing MSVC RtlUnwindEx to abort on Windows"
)]
fn test_fuel_exhaustion_on_core_wasm() {
    let wasm = sample_math_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    // Allocate an impossibly tiny fuel budget (e.g. 1 fuel unit)
    let tight_limits = ExecutionLimits::unmetered().with_fuel(1);

    let err = component
        .invoke_with_limits("add_numbers", &[Val::S32(10), Val::S32(20)], &tight_limits)
        .expect_err("execution must run out of fuel");

    match err {
        RuntimeError::FuelExhausted { limit } => {
            assert_eq!(limit, 1);
        }
        other => panic!("expected FuelExhausted, got {other:?}"),
    }
}

#[test]
fn test_fuel_consumption_and_determinism_on_component_model() {
    let wasm = probe_cm_wasm();
    let engine = Engine::new().expect("engine");
    let mut policy = HostPolicy::deny_all();
    policy.allow_wit_interface(PROBE_TYPES_IFACE);

    let component = engine
        .load_component_with_policy(&wasm, &policy)
        .expect("load");

    let limits = ExecutionLimits::unmetered().with_fuel(100_000);

    let report = component
        .invoke_with_limits(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(7), Val::S32(8)],
            &limits,
        )
        .expect("invocation must succeed");

    assert!(matches!(report.values[0], Val::Record(_)));
    let consumed = report.fuel_consumed.expect("fuel consumed reported");
    assert!(consumed > 0);
    assert!(consumed <= 100_000);
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "Wasmtime 27 upstream issue #10923: libcall out_of_gas is declared as extern \"C\" instead of extern \"C-unwind\", causing MSVC RtlUnwindEx to abort on Windows"
)]
fn test_fuel_exhaustion_on_component_model() {
    let wasm = probe_cm_wasm();
    let engine = Engine::new().expect("engine");
    let mut policy = HostPolicy::deny_all();
    policy.allow_wit_interface(PROBE_TYPES_IFACE);

    let component = engine
        .load_component_with_policy(&wasm, &policy)
        .expect("load");

    // Insufficient fuel budget for component model invocation
    let tight_limits = ExecutionLimits::unmetered().with_fuel(1);

    let err = component
        .invoke_with_limits(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(7), Val::S32(8)],
            &tight_limits,
        )
        .expect_err("must run out of fuel");

    assert!(
        matches!(err, RuntimeError::FuelExhausted { limit: 1 }),
        "expected FuelExhausted, got {err:?}"
    );
}

#[test]
fn test_unmetered_invoke_backward_compatibility() {
    let wasm = sample_math_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    // Standard invoke() without limits arguments succeeds transparently
    let res = component
        .invoke("add_numbers", &[Val::S32(100), Val::S32(250)])
        .expect("invoke must succeed");
    assert_eq!(res, vec![Val::S32(350)]);
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "Wasmtime 27 upstream issue #10923: libcall out_of_gas is declared as extern \"C\" instead of extern \"C-unwind\", causing MSVC RtlUnwindEx to abort on Windows"
)]
fn test_consecutive_store_fuel_isolation_adr005() {
    let wasm = sample_math_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    // Invocation 1: Runs out of fuel
    let tight_limits = ExecutionLimits::unmetered().with_fuel(1);
    let err = component
        .invoke_with_limits("add_numbers", &[Val::S32(1), Val::S32(2)], &tight_limits)
        .expect_err("must run out of fuel");
    assert!(matches!(err, RuntimeError::FuelExhausted { .. }));

    // Invocation 2: Immediately on the SAME Component, invoke with generous fuel
    let generous_limits = ExecutionLimits::unmetered().with_fuel(10_000);
    let report = component
        .invoke_with_limits("add_numbers", &[Val::S32(1), Val::S32(2)], &generous_limits)
        .expect("must succeed: fresh Store per invocation, no fuel leakage");
    assert_eq!(report.values, vec![Val::S32(3)]);

    // Invocation 3: Again with tight limits fails again
    let err2 = component
        .invoke_with_limits("add_numbers", &[Val::S32(1), Val::S32(2)], &tight_limits)
        .expect_err("must run out of fuel again");
    assert!(matches!(err2, RuntimeError::FuelExhausted { .. }));
}
