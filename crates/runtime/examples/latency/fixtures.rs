//! Same deterministic fixtures as baseline.rs and extended.rs; no measured compilation.

use serde_json::json;
use std::path::PathBuf;
use wasma_compiler::{CompileContext, compile_ir_to_wasm, compile_ir_to_wasm_with_context};
use wasma_runtime::HostPolicy;

const WASI_ECHO_WAT: &str = include_str!("../../benches/fixtures/wasi_console_echo.wat");

pub fn wasi_echo_wasm() -> Vec<u8> {
    wat::parse_str(WASI_ECHO_WAT).expect("wasi echo fixture must be valid WAT")
}

pub fn stdio_policy() -> HostPolicy {
    let mut policy = HostPolicy::deny_all();
    policy.allow_wasi_stdio();
    policy
}

pub fn v06_wasm() -> Vec<u8> {
    let ir = json!({
        "ir_version": "0.6",
        "spec_version": 1,
        "functions": [
            {
                "name": "greet",
                "export": true,
                "params": [{ "name": "name", "type": "string" }],
                "returns": "string",
                "requirements": ["BENCH-1"],
                "body": {
                    "op": "string.concat",
                    "left": { "op": "const", "type": "string", "value": "hello, " },
                    "right": { "op": "param", "name": "name" }
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
                "requirements": ["BENCH-2"],
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
                        "value": { "op": "const", "type": "string", "value": "div0" }
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
            }
        ]
    });
    let ctx = CompileContext::cwd().expect("cwd").with_experimental(true);
    compile_ir_to_wasm_with_context(&ir, &ctx).expect("v0.6 compile must succeed")
}

pub fn identity_wasm() -> Vec<u8> {
    let ir = json!({
        "ir_version": "0.1",
        "spec_version": 1,
        "functions": [{
            "name": "identity",
            "export": true,
            "params": [{ "name": "value", "type": "i32" }],
            "returns": "i32",
            "requirements": ["SMOKE-001"],
            "body": { "op": "param", "name": "value" }
        }]
    });
    compile_ir_to_wasm(&ir).expect("identity compile must succeed")
}

fn compiler_crate_root() -> PathBuf {
    // Benches run with CARGO_MANIFEST_DIR = crates/runtime. Navigate to
    // crates/compiler where program-005's wit_file lives.
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.push("compiler");
    root
}

pub fn point_sum_wasm() -> Vec<u8> {
    let text = include_str!("../../../compiler/tests/fixtures/ir/program-005.ir.json");
    let ir: serde_json::Value =
        serde_json::from_str(text).expect("program-005 fixture must be valid JSON");
    let ctx = CompileContext::new(compiler_crate_root());
    compile_ir_to_wasm_with_context(&ir, &ctx).expect("point-sum compile must succeed")
}

pub const POINT_SUM_FN: &str = "an:test/ops@0.1.0#make-point-sum";
