//! Negative tests: components with declared imports or caller type
//! mismatches surface typed `RuntimeError` variants without executing
//! guest code (ADR-004).

use serde_json::json;
use wasma_compiler::compile_ir_to_wasm;
use wasma_runtime::{Engine, RuntimeError, Val};

/// IR v0.3 program with declared host imports. The compiler lowers it to a
/// Core WASM module whose `imports` section lists `host::kv_get` et al.
/// The runtime must reject it pre-instantiate with `UnauthorizedImport`.
#[test]
fn core_component_with_declared_import_is_unauthorized() {
    let ir_text = include_str!("../../compiler/tests/fixtures/ir/program-003.ir.json");
    let ir: serde_json::Value = serde_json::from_str(ir_text).expect("valid JSON");

    let wasm = compile_ir_to_wasm(&ir).expect("compile must succeed");
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    // program-003 exports functions that call host imports; invoking any
    // of them must trigger UnauthorizedImport pre-instantiate.
    let err = component
        .invoke("process_order", &[Val::S32(1)])
        .unwrap_err();
    match err {
        RuntimeError::UnauthorizedImport { module, name } => {
            assert_eq!(module, "host");
            assert!(
                name.starts_with("kv_") || name.starts_with("check_") || name.starts_with("emit_"),
                "expected a known host import name, got '{name}'"
            );
        }
        other => panic!("expected UnauthorizedImport, got {other:?}"),
    }
}

/// Calling `identity(bool)` on a function that expects `i32` must surface
/// `ArgTypeMismatch`.
#[test]
fn arg_type_mismatch_detected_pre_call() {
    let ir = json!({
        "ir_version": "0.1",
        "spec_version": 1,
        "functions": [
            {
                "name": "identity",
                "export": true,
                "params": [{ "name": "value", "type": "i32" }],
                "returns": "i32",
                "requirements": ["SMOKE-001"],
                "body": { "op": "param", "name": "value" }
            }
        ]
    });

    let wasm = compile_ir_to_wasm(&ir).expect("compile must succeed");
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    // Core WASM i32 param, caller passes component-level S64 → mismatch.
    let err = component.invoke("identity", &[Val::S64(42)]).unwrap_err();
    match err {
        RuntimeError::ArgTypeMismatch {
            function,
            index,
            expected,
            found,
        } => {
            assert_eq!(function, "identity");
            assert_eq!(index, 0);
            assert_eq!(found, "s64");
            assert!(
                expected.contains("I32") || expected.contains("i32"),
                "expected I32-like description, got '{expected}'"
            );
        }
        other => panic!("expected ArgTypeMismatch, got {other:?}"),
    }
}

/// Arity mismatch remains correctly reported (regression guard for AN-010
/// not accidentally shadowing AN-009's error path).
#[test]
fn arity_mismatch_still_reported() {
    let ir = json!({
        "ir_version": "0.1",
        "spec_version": 1,
        "functions": [
            {
                "name": "identity",
                "export": true,
                "params": [{ "name": "value", "type": "i32" }],
                "returns": "i32",
                "requirements": ["SMOKE-001"],
                "body": { "op": "param", "name": "value" }
            }
        ]
    });

    let wasm = compile_ir_to_wasm(&ir).expect("compile must succeed");
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    let err = component.invoke("identity", &[]).unwrap_err();
    assert!(matches!(err, RuntimeError::ArityMismatch { .. }));
}
