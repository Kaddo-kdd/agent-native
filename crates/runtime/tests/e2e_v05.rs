//! AN-014: End-to-end regression for Semantic IR v0.5 → Component Model →
//! runtime execution.
//!
//! Closes the gap between "compiler produces WASM" and "runtime executes
//! operations and record returns under a WIT contract" without introducing
//! host authorisation changes (those belong to AN-015).
//!
//! Scope of this file:
//! 1. Primitive invocation through a WIT-exported interface.
//! 2. Record return construction and field verification.
//! 3. Negative path: lookup of a non-existent operation inside the
//!    interface surfaces `FunctionNotFound`.
//! 4. Negative path: an IR declaring an inexistent `wit_contract.wit_file`
//!    is rejected during compilation with a diagnostic.

use agent_native_compiler::compile_ir_to_wasm;
use agent_native_runtime::{Engine, RuntimeError, Val};
use serde_json::Value;

fn program_005_ir() -> Value {
    let text = include_str!("../../compiler/tests/fixtures/ir/program-005.ir.json");
    serde_json::from_str(text).expect("fixture must be valid JSON")
}

/// `double_value(21) -> 42` through the WIT interface `an:test/ops@0.1.0`.
#[test]
fn v05_component_model_primitive_invoke_through_wit_interface() {
    let ir = program_005_ir();
    let wasm = compile_ir_to_wasm(&ir).expect("compile must succeed");
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    let results = component
        .invoke("an:test/ops@0.1.0#double-value", &[Val::S32(21)])
        .expect("invoke must succeed");

    assert_eq!(results, vec![Val::S32(42)]);
}

/// `make_point_sum(3, 4) -> { sum: 7, doubled: 14 }` — exercises the record
/// return path end-to-end.
#[test]
fn v05_component_model_record_return_fields_verified() {
    let ir = program_005_ir();
    let wasm = compile_ir_to_wasm(&ir).expect("compile must succeed");
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    let results = component
        .invoke(
            "an:test/ops@0.1.0#make-point-sum",
            &[Val::S32(3), Val::S32(4)],
        )
        .expect("invoke must succeed");

    assert_eq!(results.len(), 1);
    let Val::Record(fields) = &results[0] else {
        panic!("expected a record, got {:?}", results[0]);
    };
    let get_s32 = |name: &str| -> i32 {
        let (_, v) = fields
            .iter()
            .find(|(fname, _)| fname == name)
            .unwrap_or_else(|| panic!("field '{name}' missing"));
        match v {
            Val::S32(n) => *n,
            other => panic!("field '{name}' is not S32: {other:?}"),
        }
    };
    assert_eq!(get_s32("sum"), 7, "sum field");
    assert_eq!(get_s32("doubled"), 14, "doubled field");
}

/// A lookup for a non-existent operation inside the exported interface
/// surfaces `RuntimeError::FunctionNotFound` — never a silent fallback.
#[test]
fn missing_operation_inside_wit_interface_is_not_found() {
    let ir = program_005_ir();
    let wasm = compile_ir_to_wasm(&ir).expect("compile must succeed");
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    let err = component
        .invoke("an:test/ops@0.1.0#does-not-exist", &[])
        .unwrap_err();
    match err {
        RuntimeError::FunctionNotFound(ref name) => {
            assert_eq!(name, "an:test/ops@0.1.0#does-not-exist");
        }
        other => panic!("expected FunctionNotFound, got {other:?}"),
    }
}

/// An IR referencing an inexistent `wit_contract.wit_file` is rejected
/// during compilation with the `E_WIT_NOT_FOUND` diagnostic. This exercises
/// the compiler's wit-resolution path from the runtime side.
#[test]
fn missing_wit_file_rejected_during_compilation() {
    let mut ir = program_005_ir();
    ir["wit_contract"]["wit_file"] =
        Value::String("crates/compiler/tests/fixtures/wit/does-not-exist.wit".to_string());

    let diags = compile_ir_to_wasm(&ir).unwrap_err();
    assert!(
        diags.iter().any(|d| d.code == "E_WIT_NOT_FOUND"),
        "expected E_WIT_NOT_FOUND, got: {diags:?}"
    );
}
