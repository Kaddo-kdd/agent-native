//! Verifier-only regression tests for Semantic IR v0.5.
//!
//! These tests exercise `verify_ir` in isolation (no compile step). They
//! cover the diagnostic codes most load-bearing for downstream work: schema
//! shape, unknown functions and arity mismatches. The full regression suite
//! that also exercises `compile_ir_to_wasm` + `validate_wasm_bytes` + hash
//! determinism lives in AN-007, after `agent-native-compiler` exists
//! (INIT-02-02).

use serde_json::json;
use wasma_ir::verify_ir;

#[test]
fn minimal_v0_1_ir_has_no_diagnostics() {
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

    let diags = verify_ir(&ir);
    assert!(diags.is_empty(), "expected 0 diagnostics, got: {diags:?}");
}

#[test]
fn missing_ir_version_reports_schema_invalid() {
    let ir = json!({
        "spec_version": 1,
        "functions": []
    });

    let diags = verify_ir(&ir);
    assert!(
        diags
            .iter()
            .any(|d| d.code == "E_SCHEMA_INVALID" && d.path == "$.ir_version"),
        "expected E_SCHEMA_INVALID at $.ir_version, got: {diags:?}"
    );
}

#[test]
fn call_to_unknown_function_reports_unknown_function() {
    let ir = json!({
        "ir_version": "0.2",
        "spec_version": 1,
        "functions": [{
            "name": "caller",
            "export": true,
            "params": [],
            "returns": "i32",
            "requirements": ["R1"],
            "body": { "op": "call", "function": "nonexistent", "args": [] }
        }]
    });

    let diags = verify_ir(&ir);
    assert!(
        diags.iter().any(|d| d.code == "E_UNKNOWN_FUNCTION"),
        "expected E_UNKNOWN_FUNCTION, got: {diags:?}"
    );
}

#[test]
fn wrong_call_arity_reports_call_arity() {
    let ir = json!({
        "ir_version": "0.2",
        "spec_version": 1,
        "functions": [
            {
                "name": "target",
                "export": false,
                "params": [{ "name": "a", "type": "i32" }],
                "returns": "i32",
                "requirements": [],
                "body": { "op": "param", "name": "a" }
            },
            {
                "name": "caller",
                "export": true,
                "params": [],
                "returns": "i32",
                "requirements": ["R1"],
                "body": {
                    "op": "call",
                    "function": "target",
                    "args": []
                }
            }
        ]
    });

    let diags = verify_ir(&ir);
    assert!(
        diags.iter().any(|d| d.code == "E_CALL_ARITY"),
        "expected E_CALL_ARITY, got: {diags:?}"
    );
}

#[test]
fn root_must_be_object() {
    let ir = json!([1, 2, 3]);
    let diags = verify_ir(&ir);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "E_SCHEMA_INVALID");
    assert_eq!(diags[0].path, "$");
}

#[test]
fn valid_v0_2_record_construction_has_no_diagnostics() {
    let ir = json!({
        "ir_version": "0.2",
        "spec_version": 1,
        "types": [
            {
                "name": "PointSum",
                "fields": [
                    { "name": "sum", "type": "i32" }
                ]
            }
        ],
        "functions": [
            {
                "name": "make_point_sum",
                "export": true,
                "params": [
                    { "name": "x", "type": "i32" },
                    { "name": "y", "type": "i32" }
                ],
                "returns": "PointSum",
                "requirements": ["SMOKE2-001"],
                "body": {
                    "op": "record.construct",
                    "type": "PointSum",
                    "fields": {
                        "sum": {
                            "op": "add",
                            "left": { "op": "param", "name": "x" },
                            "right": { "op": "param", "name": "y" }
                        }
                    }
                }
            }
        ]
    });

    let diags = verify_ir(&ir);
    assert!(diags.is_empty(), "expected 0 diagnostics, got: {diags:?}");
}
