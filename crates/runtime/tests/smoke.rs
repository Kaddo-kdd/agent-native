//! End-to-end smoke test: compile a minimal IR with
//! `agent-native-compiler`, load it into the runtime and invoke the
//! exported function.
//!
//! The compiler is a dev-dependency only (ADR-002); the production runtime
//! never links to it.

use serde_json::json;
use wasma_compiler::compile_ir_to_wasm;
use wasma_runtime::{Engine, RuntimeError, Val};

fn identity_wasm() -> Vec<u8> {
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
    compile_ir_to_wasm(&ir).expect("compile must succeed")
}

#[test]
fn identity_component_returns_argument() {
    let wasm = identity_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    let results = component
        .invoke("identity", &[Val::S32(42)])
        .expect("invoke");

    assert_eq!(results, vec![Val::S32(42)]);
}

#[test]
fn missing_function_returns_function_not_found() {
    let wasm = identity_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    let err = component.invoke("nonexistent", &[]).unwrap_err();
    assert!(
        matches!(err, RuntimeError::FunctionNotFound(ref name) if name == "nonexistent"),
        "expected FunctionNotFound, got {err:?}"
    );
}
