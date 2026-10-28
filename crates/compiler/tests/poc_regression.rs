//! Integration regression suite: the 8 unit tests from the POC
//! (`_reference/poc-agent-native/crates/irtool/src/lib.rs`, lines 2360-2748)
//! executed against the new workspace APIs.
//!
//! Closes INIT-02-01. These tests exercise the full pipeline
//! verify_ir -> compile_ir_to_wasm -> validate_wasm_bytes -> sha256_hex.

#![allow(clippy::too_many_lines, clippy::doc_markdown)]

use serde_json::{Value, json};
use wasma_compiler::{compile_ir_to_wasm, sha256_hex, validate_wasm_bytes};
use wasma_ir::verify_ir;

#[test]
fn valid_smoke_ir_passes_and_compiles_reproducibly() {
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
    assert!(
        diags.is_empty(),
        "smoke IR must have 0 diagnostics: {diags:?}"
    );

    let wasm1 = compile_ir_to_wasm(&ir).expect("compile must succeed");
    let wasm2 = compile_ir_to_wasm(&ir).expect("compile must succeed");
    assert_eq!(sha256_hex(&wasm1), sha256_hex(&wasm2));
    assert!(validate_wasm_bytes(&wasm1).is_ok());
}

#[test]
fn valid_v0_2_smoke_ir_passes_and_compiles() {
    let ir = json!({
        "ir_version": "0.2",
        "spec_version": 1,
        "types": [
            {
                "name": "Point",
                "fields": [
                    { "name": "x", "type": "i32" },
                    { "name": "y", "type": "i32" }
                ]
            },
            {
                "name": "PointSum",
                "fields": [
                    { "name": "sum", "type": "i32" }
                ]
            }
        ],
        "functions": [
            {
                "name": "add_coords",
                "export": false,
                "params": [
                    { "name": "x", "type": "i32" },
                    { "name": "y", "type": "i32" }
                ],
                "returns": "i32",
                "requirements": [],
                "body": {
                    "op": "add",
                    "left": { "op": "param", "name": "x" },
                    "right": { "op": "param", "name": "y" }
                }
            },
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
                            "op": "call",
                            "function": "add_coords",
                            "args": [
                                { "op": "param", "name": "x" },
                                { "op": "param", "name": "y" }
                            ]
                        }
                    }
                }
            }
        ]
    });

    let diags = verify_ir(&ir);
    assert!(
        diags.is_empty(),
        "v0.2 smoke IR must have 0 diagnostics: {diags:?}"
    );

    let wasm = compile_ir_to_wasm(&ir).expect("compile must succeed");
    assert!(validate_wasm_bytes(&wasm).is_ok());
}

#[test]
fn detects_v0_2_semantic_error_codes() {
    // 1. E_UNKNOWN_FUNCTION in call
    let unknown_func = json!({
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
    assert!(
        verify_ir(&unknown_func)
            .iter()
            .any(|d| d.code == "E_UNKNOWN_FUNCTION")
    );

    // 2. E_CALL_ARITY
    let arity_err = json!({
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
                "body": { "op": "call", "function": "target", "args": [] }
            }
        ]
    });
    assert!(
        verify_ir(&arity_err)
            .iter()
            .any(|d| d.code == "E_CALL_ARITY")
    );

    // 3. E_UNKNOWN_FIELD in record.construct
    let unknown_field = json!({
        "ir_version": "0.2",
        "spec_version": 1,
        "types": [{ "name": "Item", "fields": [{ "name": "price", "type": "i32" }] }],
        "functions": [{
            "name": "test",
            "export": true,
            "params": [],
            "returns": "Item",
            "requirements": ["R1"],
            "body": {
                "op": "record.construct",
                "type": "Item",
                "fields": { "price": { "op": "const", "type": "i32", "value": 10 }, "bogus": { "op": "const", "type": "i32", "value": 1 } }
            }
        }]
    });
    assert!(
        verify_ir(&unknown_field)
            .iter()
            .any(|d| d.code == "E_UNKNOWN_FIELD")
    );

    // 4. E_MISSING_FIELD in record.construct
    let missing_field = json!({
        "ir_version": "0.2",
        "spec_version": 1,
        "types": [{ "name": "Item", "fields": [{ "name": "price", "type": "i32" }, { "name": "qty", "type": "i32" }] }],
        "functions": [{
            "name": "test",
            "export": true,
            "params": [],
            "returns": "Item",
            "requirements": ["R1"],
            "body": {
                "op": "record.construct",
                "type": "Item",
                "fields": { "price": { "op": "const", "type": "i32", "value": 10 } }
            }
        }]
    });
    assert!(
        verify_ir(&missing_field)
            .iter()
            .any(|d| d.code == "E_MISSING_FIELD")
    );
}

#[test]
fn validates_and_compiles_v0_3_smoke_module() {
    let smoke_json_str = include_str!("fixtures/ir/smoke-v3.ir.json");
    let ir: Value = serde_json::from_str(smoke_json_str).expect("smoke-v3 must be valid JSON");

    let diags = verify_ir(&ir);
    assert!(
        diags.is_empty(),
        "smoke-v3 IR must have 0 diagnostics: {diags:?}"
    );

    let wasm = compile_ir_to_wasm(&ir).expect("smoke-v3 compilation must succeed");
    assert!(
        validate_wasm_bytes(&wasm).is_ok(),
        "smoke-v3 WASM must pass wasmparser validation"
    );
}

#[test]
fn detects_v0_3_capability_and_import_diagnostics() {
    // 1. E_UNDECLARED_CAPABILITY at import declaration
    let undeclared_cap_import = json!({
        "ir_version": "0.3",
        "spec_version": 1,
        "capabilities": ["kv_store"],
        "imports": [
            {
                "module": "host",
                "name": "emit_audit",
                "capability": "audit_log",
                "params": [{ "name": "event_id", "type": "i32" }],
                "returns": "i32"
            }
        ],
        "functions": [{
            "name": "noop",
            "export": true,
            "params": [],
            "returns": "i32",
            "requirements": ["R1"],
            "body": { "op": "const", "type": "i32", "value": 0 }
        }]
    });
    let diags1 = verify_ir(&undeclared_cap_import);
    assert!(
        diags1.iter().any(|d| d.code == "E_UNDECLARED_CAPABILITY"),
        "diags: {diags1:?}"
    );

    // 2. E_UNKNOWN_IMPORT in call_import
    let unknown_import_call = json!({
        "ir_version": "0.3",
        "spec_version": 1,
        "capabilities": ["kv_store"],
        "imports": [
            {
                "module": "host",
                "name": "kv_get",
                "capability": "kv_store",
                "params": [{ "name": "key", "type": "i32" }],
                "returns": "i32"
            }
        ],
        "functions": [{
            "name": "test",
            "export": true,
            "params": [],
            "returns": "i32",
            "requirements": ["R1"],
            "body": {
                "op": "call_import",
                "import": "nonexistent_import",
                "args": [{ "op": "const", "type": "i32", "value": 1 }]
            }
        }]
    });
    let diags2 = verify_ir(&unknown_import_call);
    assert!(
        diags2.iter().any(|d| d.code == "E_UNKNOWN_IMPORT"),
        "diags: {diags2:?}"
    );

    // 3. E_CALL_ARITY in call_import
    let arity_err_call = json!({
        "ir_version": "0.3",
        "spec_version": 1,
        "capabilities": ["kv_store"],
        "imports": [
            {
                "module": "host",
                "name": "kv_get",
                "capability": "kv_store",
                "params": [{ "name": "key", "type": "i32" }],
                "returns": "i32"
            }
        ],
        "functions": [{
            "name": "test",
            "export": true,
            "params": [],
            "returns": "i32",
            "requirements": ["R1"],
            "body": {
                "op": "call_import",
                "import": "kv_get",
                "args": []
            }
        }]
    });
    let diags3 = verify_ir(&arity_err_call);
    assert!(
        diags3.iter().any(|d| d.code == "E_CALL_ARITY"),
        "diags: {diags3:?}"
    );

    // 4. E_TYPE_MISMATCH in call_import
    let type_mismatch_call = json!({
        "ir_version": "0.3",
        "spec_version": 1,
        "capabilities": ["kv_store"],
        "imports": [
            {
                "module": "host",
                "name": "kv_get",
                "capability": "kv_store",
                "params": [{ "name": "key", "type": "i32" }],
                "returns": "i32"
            }
        ],
        "functions": [{
            "name": "test",
            "export": true,
            "params": [],
            "returns": "i32",
            "requirements": ["R1"],
            "body": {
                "op": "call_import",
                "import": "kv_get",
                "args": [{ "op": "const", "type": "bool", "value": true }]
            }
        }]
    });
    let diags4 = verify_ir(&type_mismatch_call);
    assert!(
        diags4.iter().any(|d| d.code == "E_TYPE_MISMATCH"),
        "diags: {diags4:?}"
    );
}

#[test]
fn validates_and_compiles_v0_4_smoke_module() {
    let smoke_json = include_str!("fixtures/ir/smoke-v4.ir.json");
    let val: Value = serde_json::from_str(smoke_json).expect("valid JSON");
    let diags = verify_ir(&val);
    assert!(diags.is_empty(), "diags on v0.4 smoke: {diags:?}");

    let wasm = compile_ir_to_wasm(&val).expect("compile v0.4 to wasm");
    assert!(!wasm.is_empty());
    assert!(
        wasm.starts_with(b"\0asm\x0d\0\x01\0") || wasm.starts_with(b"\0asm\r\0\x01\0"),
        "v0.4 must compile to a WebAssembly Component Model binary"
    );

    let mut validator = wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all());
    validator.validate_all(&wasm).expect("valid wasm component");
}

#[test]
fn detects_v0_4_wit_contract_mismatch() {
    let smoke_json = include_str!("fixtures/ir/smoke-v4.ir.json");
    let mut val: Value = serde_json::from_str(smoke_json).expect("valid JSON");
    val["functions"][0]["wit_interface"] = json!("nonexistent-interface");
    let diags = verify_ir(&val);
    assert!(
        diags.iter().any(|d| d.code == "E_UNKNOWN_EXPORT_INTERFACE"),
        "diags: {diags:?}"
    );
}

#[test]
fn test_compiles_v0_4_to_component_model() {
    let program_json = include_str!("fixtures/ir/program-004.ir.json");
    let val: Value = serde_json::from_str(program_json).expect("valid JSON");
    let comp_bytes = compile_ir_to_wasm(&val).expect("compile to component");
    assert!(
        comp_bytes.starts_with(b"\0asm\x0d\0\x01\0") || comp_bytes.starts_with(b"\0asm\r\0\x01\0"),
        "Compiled binary must have WebAssembly Component magic [0, 61, 73, 6d, 13, 0, 1, 0]"
    );
    assert!(comp_bytes.len() > 1000);
}
