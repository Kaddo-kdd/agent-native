//! AN-016 §1: reproducibility probe for the Technical Review's hypothesis
//! "`record.construct` anidado puede sobrescribir el puntero del registro
//! exterior".
//!
//! **Finding (reproduced):** the hypothesis is NOT reachable from
//! Semantic IR v0.5. The verifier (`wasma_ir::verify_ir`) rejects
//! records whose fields reference another record type: the grammar only
//! accepts `i32 | bool` as field types. Any attempt to build a
//! `record.construct` inside another `record.construct` is caught at
//! verification with `E_UNKNOWN_TYPE` + `E_TYPE_MISMATCH` before any
//! WASM is emitted. Therefore the pointer-overwrite class of defects
//! described in the Technical Review cannot manifest under v0.5.
//!
//! If IR v0.6 or later lifts the restriction (RM-07 / INIT-07-02), this
//! test must be revisited and extended to verify execution, not only
//! verification.

use wasma_compiler::compile_ir_to_wasm;

fn nested_record_ir() -> serde_json::Value {
    let text = include_str!("../../compiler/tests/fixtures/ir/nested-records.ir.json");
    serde_json::from_str(text).expect("valid JSON")
}

#[test]
fn ir_v05_rejects_record_fields_that_reference_another_record() {
    let ir = nested_record_ir();
    let diags = compile_ir_to_wasm(&ir).unwrap_err();

    // The verifier must emit the "unknown field type" diagnostic for the
    // inner-record reference in `types[1].fields[0]`.
    let has_unknown_type = diags.iter().any(|d| {
        d.code == "E_UNKNOWN_TYPE"
            && d.path.starts_with("$.types")
            && d.found.as_deref() == Some("inner")
            && d.expected.as_deref() == Some("i32 | bool")
    });
    assert!(
        has_unknown_type,
        "expected E_UNKNOWN_TYPE naming 'inner' with expected 'i32 | bool', got: {diags:?}"
    );
}
