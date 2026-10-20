#![allow(clippy::doc_markdown)]

//! Extended runtime benchmarks for INIT-08-01 (AN-051).
//!
//! Complements `baseline.rs` (same `<stage>_<path>_<fixture>` convention) with:
//!
//! - `warm_run_component_model_wasi_echo` / `cold_run_component_model_wasi_echo`:
//!   WASI 0.2 in-memory console I/O (AN-044/AN-045) through `invoke_with_wasi`.
//! - `warm_run_component_model_v06_greet` / `warm_run_component_model_v06_safe_div`:
//!   experimental IR v0.6 `string` and `result<T, E>` (AN-048).
//! - `warm_run_core_identity_fuel`: fuel-metered invocation (AN-035..AN-038),
//!   to quantify the cost of metering against `warm_run_core_identity`.

use agent_native_compiler::{CompileContext, compile_ir_to_wasm, compile_ir_to_wasm_with_context};
use agent_native_runtime::{Engine, ExecutionLimits, HostPolicy, Val, WasiIoConfig};
use criterion::{Criterion, criterion_group, criterion_main};
use serde_json::json;
use std::hint::black_box;

const WASI_ECHO_WAT: &str = include_str!("fixtures/wasi_console_echo.wat");

fn wasi_echo_wasm() -> Vec<u8> {
    wat::parse_str(WASI_ECHO_WAT).expect("wasi echo fixture must be valid WAT")
}

fn stdio_policy() -> HostPolicy {
    let mut policy = HostPolicy::deny_all();
    policy.allow_wasi_stdio();
    policy
}

fn v06_wasm() -> Vec<u8> {
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

fn identity_wasm() -> Vec<u8> {
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

fn bench_wasi_echo(c: &mut Criterion) {
    let wasm = wasi_echo_wasm();
    let policy = stdio_policy();
    let limits = ExecutionLimits::unmetered();

    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component_with_policy(&wasm, &policy)
        .expect("load");
    c.bench_function("warm_run_component_model_wasi_echo", |b| {
        b.iter(|| {
            let io = WasiIoConfig::capturing().with_stdin(b"hello agent-native".to_vec());
            let report = component
                .invoke_with_wasi(
                    &policy,
                    &limits,
                    "echo-and-log",
                    &[Val::S32(64), Val::S32(0)],
                    io,
                )
                .expect("invoke");
            black_box(report);
        });
    });

    c.bench_function("cold_run_component_model_wasi_echo", |b| {
        b.iter(|| {
            let engine = Engine::new().expect("engine");
            let component = engine
                .load_component_with_policy(black_box(&wasm), &policy)
                .expect("load");
            let io = WasiIoConfig::capturing().with_stdin(b"hello agent-native".to_vec());
            let report = component
                .invoke_with_wasi(
                    &policy,
                    &limits,
                    "echo-and-log",
                    &[Val::S32(64), Val::S32(0)],
                    io,
                )
                .expect("invoke");
            black_box(report);
        });
    });
}

fn bench_v06(c: &mut Criterion) {
    let wasm = v06_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    c.bench_function("warm_run_component_model_v06_greet", |b| {
        b.iter(|| {
            let result = component
                .invoke("greet", &[Val::String("world".to_string())])
                .expect("invoke");
            black_box(result);
        });
    });
    c.bench_function("warm_run_component_model_v06_safe_div", |b| {
        b.iter(|| {
            let result = component
                .invoke("safe-div", &[Val::S32(42), Val::S32(6)])
                .expect("invoke");
            black_box(result);
        });
    });
}

fn bench_fuel_metered(c: &mut Criterion) {
    let wasm = identity_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");
    let limits = ExecutionLimits::unmetered().with_fuel(1_000_000);

    c.bench_function("warm_run_core_identity_fuel", |b| {
        b.iter(|| {
            let report = component
                .invoke_with_limits("identity", &[Val::S32(42)], &limits)
                .expect("invoke");
            black_box(report);
        });
    });
}

criterion_group!(benches, bench_wasi_echo, bench_v06, bench_fuel_metered);
criterion_main!(benches);
