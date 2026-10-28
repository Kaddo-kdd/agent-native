//! End-to-end execution tests for Semantic IR v0.6 `string` and `result<T, E>` (AN-048).

use serde_json::{Value, json};
use wasma_compiler::{CompileContext, compile_ir_to_wasm_with_context};
use wasma_runtime::{Engine, Val};

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
fn ir_v06_compiles_and_executes_strings_and_results_end_to_end() {
    let ir = sample_v06_ir();
    let ctx = CompileContext::cwd()
        .expect("cwd should resolve")
        .with_experimental(true);
    let wasm_bytes =
        compile_ir_to_wasm_with_context(&ir, &ctx).expect("v0.6 component should compile");

    let engine = Engine::new().expect("engine should initialize");
    let component = engine
        .load_component(&wasm_bytes)
        .expect("compiled v0.6 component should load in Wasmtime");

    // 1. greet("world") -> "hello, world"
    let out = component
        .invoke("greet", &[Val::String("world".to_string())])
        .expect("greet should execute");
    assert_eq!(out, vec![Val::String("hello, world".to_string())]);

    // 2. byte-len("hola") -> 4
    let len = component
        .invoke("byte-len", &[Val::String("hola".to_string())])
        .expect("byte-len should execute");
    assert_eq!(len, vec![Val::S32(4)]);

    // 3. is-admin("admin") -> true, is-admin("guest") -> false
    assert_eq!(
        component
            .invoke("is-admin", &[Val::String("admin".to_string())])
            .expect("is-admin(admin)"),
        vec![Val::Bool(true)]
    );
    assert_eq!(
        component
            .invoke("is-admin", &[Val::String("guest".to_string())])
            .expect("is-admin(guest)"),
        vec![Val::Bool(false)]
    );

    // 4. safe-div(20, 4) -> Ok(5), safe-div(20, 0) -> Err("division by zero")
    let ok_val = component
        .invoke("safe-div", &[Val::S32(20), Val::S32(4)])
        .expect("safe-div(20, 4)");
    assert_eq!(ok_val, vec![Val::Result(Ok(Some(Box::new(Val::S32(5)))))]);

    let err_val = component
        .invoke("safe-div", &[Val::S32(20), Val::S32(0)])
        .expect("safe-div(20, 0)");
    assert_eq!(
        err_val,
        vec![Val::Result(Err(Some(Box::new(Val::String(
            "division by zero".to_string()
        )))))]
    );

    // 5. describe-div(20, 4) -> "ok", describe-div(20, 0) -> "division by zero"
    assert_eq!(
        component
            .invoke("describe-div", &[Val::S32(20), Val::S32(4)])
            .expect("describe-div(20, 4)"),
        vec![Val::String("ok".to_string())]
    );
    assert_eq!(
        component
            .invoke("describe-div", &[Val::S32(20), Val::S32(0)])
            .expect("describe-div(20, 0)"),
        vec![Val::String("division by zero".to_string())]
    );

    // 6. div-or-default(20, 4, -1) -> 5, div-or-default(20, 0, -1) -> -1
    assert_eq!(
        component
            .invoke("div-or-default", &[Val::S32(20), Val::S32(4), Val::S32(-1)])
            .expect("div-or-default(20, 4, -1)"),
        vec![Val::S32(5)]
    );
    assert_eq!(
        component
            .invoke("div-or-default", &[Val::S32(20), Val::S32(0), Val::S32(-1)])
            .expect("div-or-default(20, 0, -1)"),
        vec![Val::S32(-1)]
    );
}

#[test]
fn ir_v06_record_with_string_fields_roundtrips_correctly() {
    let ir = json!({
      "ir_version": "0.6",
      "spec_version": 1,
      "types": [
        {
          "name": "UserProfile",
          "kind": "record",
          "fields": [
            { "name": "id", "type": "i32" },
            { "name": "username", "type": "string" }
          ]
        }
      ],
      "functions": [
        {
          "name": "make_and_extract_name",
          "export": true,
          "params": [
            { "name": "id", "type": "i32" },
            { "name": "username", "type": "string" }
          ],
          "returns": "string",
          "requirements": ["REQ-REC-STR"],
          "body": {
            "op": "record.get",
            "field": "username",
            "target": {
              "op": "record.construct",
              "type": "UserProfile",
              "fields": {
                "id": { "op": "param", "name": "id" },
                "username": { "op": "param", "name": "username" }
              }
            }
          }
        }
      ]
    });

    let ctx = CompileContext::cwd()
        .expect("cwd should resolve")
        .with_experimental(true);
    let wasm_bytes =
        compile_ir_to_wasm_with_context(&ir, &ctx).expect("v0.6 record+string should compile");

    let engine = Engine::new().expect("engine should initialize");
    let component = engine
        .load_component(&wasm_bytes)
        .expect("component should load");

    let out = component
        .invoke(
            "make-and-extract-name",
            &[Val::S32(42), Val::String("alice".to_string())],
        )
        .expect("make-and-extract-name should execute");
    assert_eq!(out, vec![Val::String("alice".to_string())]);
}
