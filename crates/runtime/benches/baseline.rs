//! Runtime baseline benchmarks for INIT-03-06.
//!
//! Measures the three primitives that any future optimization (resolver
//! caching in INIT-03-02, warm pool in INIT-08-03) must beat:
//!
//! - `cold_start_identity`: engine construction + component load + first
//!   invocation of a trivial identity function.
//! - `warm_invoke_identity`: subsequent invocations on a pre-loaded
//!   component.
//! - `load_only`: engine construction + component load without invoking —
//!   isolates the parsing/validation cost from execution.
//!
//! The identity IR is intentionally minimal so the measurement reflects the
//! runtime pipeline, not domain logic. Numbers are hardware-specific; see
//! `knowledge/tech/baselines/runtime-baseline.md` for the recorded local
//! run and reproduction steps.

use agent_native_compiler::compile_ir_to_wasm;
use agent_native_runtime::{Engine, Resolver, Val};
use criterion::{Criterion, criterion_group, criterion_main};
use serde_json::json;
use std::hint::black_box;

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

fn bench_cold_start(c: &mut Criterion) {
    let wasm = identity_wasm();
    c.bench_function("cold_start_identity", |b| {
        b.iter(|| {
            let engine = Engine::new().expect("engine");
            let component = engine.load_component(black_box(&wasm)).expect("load");
            let result = component
                .invoke("identity", &[Val::S32(42)])
                .expect("invoke");
            black_box(result);
        });
    });
}

fn bench_warm_invoke(c: &mut Criterion) {
    let wasm = identity_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");
    c.bench_function("warm_invoke_identity", |b| {
        b.iter(|| {
            let result = component
                .invoke("identity", &[Val::S32(42)])
                .expect("invoke");
            black_box(result);
        });
    });
}

fn bench_load_only(c: &mut Criterion) {
    let wasm = identity_wasm();
    c.bench_function("load_only", |b| {
        b.iter(|| {
            let engine = Engine::new().expect("engine");
            let component = engine.load_component(black_box(&wasm)).expect("load");
            black_box(component);
        });
    });
}

fn bench_cached_cold_start(c: &mut Criterion) {
    let wasm = identity_wasm();
    let mut resolver = Resolver::new().expect("resolver");
    // Prime the cache so iterations measure the hit path.
    let _ = resolver.load(&wasm).expect("prime");

    c.bench_function("cached_cold_start_identity", |b| {
        b.iter(|| {
            let component = resolver.load(black_box(&wasm)).expect("load");
            let result = component
                .invoke("identity", &[Val::S32(42)])
                .expect("invoke");
            black_box(result);
        });
    });
}

criterion_group!(
    benches,
    bench_cold_start,
    bench_warm_invoke,
    bench_load_only,
    bench_cached_cold_start
);
criterion_main!(benches);
