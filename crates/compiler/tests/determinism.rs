//! Determinism smoke test for the IR → WASM Component compiler.
//!
//! Covers the invariant from ADR-004: compiling the same IR document twice
//! must produce byte-identical WASM output (hence identical SHA-256). This
//! is a smoke test, not the full regression suite — the complete 8-test POC
//! suite migrates in AN-007.

use agent_native_compiler::{compile_ir_to_wasm, sha256_hex, validate_wasm_bytes};
use serde_json::json;

#[test]
fn same_ir_produces_byte_identical_wasm_and_hash() {
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

    let wasm1 = compile_ir_to_wasm(&ir).expect("first compile must succeed");
    let wasm2 = compile_ir_to_wasm(&ir).expect("second compile must succeed");

    assert_eq!(wasm1, wasm2, "WASM bytes must be byte-identical");
    assert_eq!(
        sha256_hex(&wasm1),
        sha256_hex(&wasm2),
        "SHA-256 hashes must match"
    );
    assert!(
        validate_wasm_bytes(&wasm1).is_ok(),
        "compiled WASM must validate"
    );
}

#[test]
fn invalid_ir_returns_diagnostics_not_wasm() {
    let ir = json!({
        "spec_version": 1,
        "functions": []
    });

    let result = compile_ir_to_wasm(&ir);
    assert!(
        result.is_err(),
        "IR without ir_version must fail compilation"
    );
    let diags = result.unwrap_err();
    assert!(
        diags.iter().any(|d| d.code == "E_SCHEMA_INVALID"),
        "expected E_SCHEMA_INVALID in diagnostics: {diags:?}"
    );
}
