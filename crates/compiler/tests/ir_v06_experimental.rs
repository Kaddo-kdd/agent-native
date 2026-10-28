//! Verification and compilation tests for experimental Semantic IR v0.6 (AN-048).

use serde_json::{Value, json};
use wasma_compiler::{
    CompileContext, compile_ir_to_wasm, compile_ir_to_wasm_with_context, sha256_hex,
    validate_wasm_bytes,
};
use wasma_ir::{VerifyOptions, verify_ir, verify_ir_with_options};

#[allow(clippy::too_many_lines)]
fn sample_v06_ir() -> Value {
    json!({
      "ir_version": "0.6",
      "spec_version": 1,
      "functions": [
        {
          "name": "greet",
          "export": true,
          "params": [
            { "name": "name", "type": "string" }
          ],
          "returns": "string",
          "requirements": ["REQ-1"],
          "body": {
            "op": "string.concat",
            "left": { "op": "const", "type": "string", "value": "hello, " },
            "right": { "op": "param", "name": "name" }
          }
        },
        {
          "name": "byte_len",
          "export": true,
          "params": [
            { "name": "text", "type": "string" }
          ],
          "returns": "i32",
          "requirements": ["REQ-2"],
          "body": {
            "op": "string.len",
            "value": { "op": "param", "name": "text" }
          }
        },
        {
          "name": "is_admin",
          "export": true,
          "params": [
            { "name": "role", "type": "string" }
          ],
          "returns": "bool",
          "requirements": ["REQ-3"],
          "body": {
            "op": "eq",
            "left": { "op": "param", "name": "role" },
            "right": { "op": "const", "type": "string", "value": "admin" }
          }
        },
        {
          "name": "safe_div",
          "export": true,
          "params": [
            { "name": "a", "type": "i32" },
            { "name": "b", "type": "i32" }
          ],
          "returns": "result<i32, string>",
          "requirements": ["REQ-4"],
          "body": {
            "op": "if",
            "condition": {
              "op": "eq",
              "left": { "op": "param", "name": "b" },
              "right": { "op": "const", "type": "i32", "value": 0 }
            },
            "then": {
              "op": "result.err",
              "type": "result<i32, string>",
              "value": { "op": "const", "type": "string", "value": "division by zero" }
            },
            "else": {
              "op": "result.ok",
              "type": "result<i32, string>",
              "value": {
                "op": "div_s",
                "left": { "op": "param", "name": "a" },
                "right": { "op": "param", "name": "b" }
              }
            }
          }
        },
        {
          "name": "describe_div",
          "export": true,
          "params": [
            { "name": "a", "type": "i32" },
            { "name": "b", "type": "i32" }
          ],
          "returns": "string",
          "requirements": ["REQ-5"],
          "body": {
            "op": "if",
            "condition": {
              "op": "result.is_ok",
              "value": {
                "op": "call",
                "function": "safe_div",
                "args": [
                  { "op": "param", "name": "a" },
                  { "op": "param", "name": "b" }
                ]
              }
            },
            "then": {
              "op": "const",
              "type": "string",
              "value": "ok"
            },
            "else": {
              "op": "result.unwrap_err",
              "value": {
                "op": "call",
                "function": "safe_div",
                "args": [
                  { "op": "param", "name": "a" },
                  { "op": "param", "name": "b" }
                ]
              },
              "default": {
                "op": "const",
                "type": "string",
                "value": "unexpected ok"
              }
            }
          }
        },
        {
          "name": "div_or_default",
          "export": true,
          "params": [
            { "name": "a", "type": "i32" },
            { "name": "b", "type": "i32" },
            { "name": "fallback", "type": "i32" }
          ],
          "returns": "i32",
          "requirements": ["REQ-6"],
          "body": {
            "op": "result.unwrap_ok",
            "value": {
              "op": "call",
              "function": "safe_div",
              "args": [
                { "op": "param", "name": "a" },
                { "op": "param", "name": "b" }
              ]
            },
            "default": {
              "op": "param",
              "name": "fallback"
            }
          }
        }
      ]
    })
}

#[test]
fn ir_v06_requires_experimental_flag_in_verifier_and_compiler() {
    let ir = sample_v06_ir();

    let diags = verify_ir(&ir);
    assert!(
        diags.iter().any(|d| d.code == "E_EXPERIMENTAL_IR_VERSION"),
        "expected E_EXPERIMENTAL_IR_VERSION without allow_experimental, got: {diags:?}"
    );

    let compile_err =
        compile_ir_to_wasm(&ir).expect_err("compile_ir_to_wasm without experimental must fail");
    assert!(
        compile_err
            .iter()
            .any(|d| d.code == "E_EXPERIMENTAL_IR_VERSION"),
        "expected E_EXPERIMENTAL_IR_VERSION in compiler error, got: {compile_err:?}"
    );
}

#[test]
fn ir_v05_rejects_string_and_result_types() {
    let v05_with_string = json!({
      "ir_version": "0.5",
      "spec_version": 1,
      "functions": [
        {
          "name": "bad",
          "export": true,
          "params": [{ "name": "s", "type": "string" }],
          "returns": "string",
          "requirements": ["REQ-1"],
          "body": { "op": "param", "name": "s" }
        }
      ]
    });
    let diags = verify_ir_with_options(
        &v05_with_string,
        &VerifyOptions {
            allow_experimental: true,
        },
    );
    assert!(
        diags.iter().any(|d| d.code == "E_UNKNOWN_TYPE"),
        "expected E_UNKNOWN_TYPE for string in v0.5, got: {diags:?}"
    );

    let v05_with_result = json!({
      "ir_version": "0.5",
      "spec_version": 1,
      "functions": [
        {
          "name": "bad_res",
          "export": true,
          "params": [{ "name": "x", "type": "i32" }],
          "returns": "result<i32, i32>",
          "requirements": ["REQ-1"],
          "body": {
            "op": "result.ok",
            "type": "result<i32, i32>",
            "value": { "op": "param", "name": "x" }
          }
        }
      ]
    });
    let diags_res = verify_ir_with_options(
        &v05_with_result,
        &VerifyOptions {
            allow_experimental: true,
        },
    );
    assert!(
        diags_res.iter().any(|d| d.code == "E_UNKNOWN_TYPE"),
        "expected E_UNKNOWN_TYPE for result<T, E> in v0.5, got: {diags_res:?}"
    );
}

#[test]
fn ir_v06_compiles_deterministically_with_experimental_enabled() {
    let ir = sample_v06_ir();
    let diags = verify_ir_with_options(
        &ir,
        &VerifyOptions {
            allow_experimental: true,
        },
    );
    assert!(
        diags.is_empty(),
        "v0.6 IR should verify cleanly with allow_experimental, got: {diags:?}"
    );

    let ctx = CompileContext::cwd()
        .expect("cwd should resolve")
        .with_experimental(true);
    let wasm1 = compile_ir_to_wasm_with_context(&ir, &ctx).expect("first v0.6 compile must pass");
    let wasm2 = compile_ir_to_wasm_with_context(&ir, &ctx).expect("second v0.6 compile must pass");

    assert_eq!(wasm1, wasm2, "v0.6 compilation must be deterministic");
    assert_eq!(sha256_hex(&wasm1), sha256_hex(&wasm2));
    validate_wasm_bytes(&wasm1).expect("compiled v0.6 component must pass wasmparser validation");
}
