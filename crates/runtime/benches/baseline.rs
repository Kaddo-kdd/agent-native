#![allow(clippy::doc_markdown)]

//! Runtime baseline benchmarks for INIT-03-06 and AN-017.
//!
//! Each bench isolates a stage of the pipeline so future optimisations
//! (caching in INIT-03-02, warm pool in INIT-08-03, memory tuning in
//! INIT-02-05) can make targeted comparisons without conflating steps.
//!
//! # Naming convention (AN-017)
//!
//! ```text
//!   <stage>_<path>_<fixture>
//! ```
//!
//! - `<stage>` ∈ {`engine_construct`, `load`, `cold_run`, `warm_run`,
//!   `cache_hit_run`}:
//!   - `engine_construct` — construct the Wasmtime engine only.
//!   - `load` — construct the engine and parse/validate the artifact; no
//!     invocation.
//!   - `cold_run` — construct the engine, load the artifact and invoke
//!     once. Pays the full first-time cost.
//!   - `warm_run` — engine and artifact are pre-constructed; each
//!     iteration invokes once. Per ADR-005 baseline operativa the
//!     wasmtime::Store is still fresh per invocation; what is cached is
//!     the parsed artifact, not the instance.
//!   - `cache_hit_run` — engine is pre-constructed and the Resolver's
//!     content-addressed cache is primed; each iteration calls
//!     Resolver::load (hit) and then invokes. Mirrors `cold_run` but
//!     exercises the cached path.
//! - `<path>` ∈ {`core`, `component_model`}: which wasmtime loader is
//!   exercised. Agent-Native IR v0.1–v0.3 produces Core WASM; v0.4+
//!   produces Component Model artifacts.
//! - `<fixture>` ∈ {`identity`, `point_sum`}: the shape of the function
//!   being invoked. `identity` is a trivial `(i32) -> i32`; `point_sum`
//!   returns a record via the WIT interface `an:test/ops@0.1.0`.
//!
//! Numbers published in `knowledge/tech/baselines/runtime-baseline.md`
//! use the names below verbatim.

use criterion::{Criterion, criterion_group, criterion_main};
use serde_json::json;
use std::hint::black_box;
use std::path::PathBuf;
use wasma_compiler::{CompileContext, compile_ir_to_wasm, compile_ir_to_wasm_with_context};
use wasma_runtime::{Engine, Resolver, Val};

// -------- Fixtures --------

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

fn point_sum_wasm() -> Vec<u8> {
    let text = include_str!("../../compiler/tests/fixtures/ir/program-005.ir.json");
    let ir: serde_json::Value =
        serde_json::from_str(text).expect("program-005 fixture must be valid JSON");
    let ctx = CompileContext::new(compiler_crate_root());
    compile_ir_to_wasm_with_context(&ir, &ctx).expect("point-sum compile must succeed")
}

const POINT_SUM_FN: &str = "an:test/ops@0.1.0#make-point-sum";

// -------- Engine stage --------

fn bench_engine_construct(c: &mut Criterion) {
    c.bench_function("engine_construct", |b| {
        b.iter(|| {
            let engine = Engine::new().expect("engine");
            black_box(engine);
        });
    });
}

// -------- Load stage --------

fn bench_load_core_identity(c: &mut Criterion) {
    let wasm = identity_wasm();
    c.bench_function("load_core_identity", |b| {
        b.iter(|| {
            let engine = Engine::new().expect("engine");
            let component = engine.load_component(black_box(&wasm)).expect("load");
            black_box(component);
        });
    });
}

fn bench_load_component_model_point_sum(c: &mut Criterion) {
    let wasm = point_sum_wasm();
    c.bench_function("load_component_model_point_sum", |b| {
        b.iter(|| {
            let engine = Engine::new().expect("engine");
            let component = engine.load_component(black_box(&wasm)).expect("load");
            black_box(component);
        });
    });
}

// -------- Cold run (engine + load + invoke) --------

fn bench_cold_run_core_identity(c: &mut Criterion) {
    let wasm = identity_wasm();
    c.bench_function("cold_run_core_identity", |b| {
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

fn bench_cold_run_component_model_point_sum(c: &mut Criterion) {
    let wasm = point_sum_wasm();
    c.bench_function("cold_run_component_model_point_sum", |b| {
        b.iter(|| {
            let engine = Engine::new().expect("engine");
            let component = engine.load_component(black_box(&wasm)).expect("load");
            let result = component
                .invoke(POINT_SUM_FN, &[Val::S32(3), Val::S32(4)])
                .expect("invoke");
            black_box(result);
        });
    });
}

// -------- Warm run (artifact pre-loaded, fresh Store per invoke) --------

fn bench_warm_run_core_identity(c: &mut Criterion) {
    let wasm = identity_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");
    c.bench_function("warm_run_core_identity", |b| {
        b.iter(|| {
            let result = component
                .invoke("identity", &[Val::S32(42)])
                .expect("invoke");
            black_box(result);
        });
    });
}

fn bench_warm_run_component_model_point_sum(c: &mut Criterion) {
    let wasm = point_sum_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");
    c.bench_function("warm_run_component_model_point_sum", |b| {
        b.iter(|| {
            let result = component
                .invoke(POINT_SUM_FN, &[Val::S32(3), Val::S32(4)])
                .expect("invoke");
            black_box(result);
        });
    });
}

// -------- Cache-hit run (Resolver primed) --------

fn bench_cache_hit_run_core_identity(c: &mut Criterion) {
    let wasm = identity_wasm();
    let mut resolver = Resolver::new().expect("resolver");
    let _ = resolver.load(&wasm).expect("prime");

    c.bench_function("cache_hit_run_core_identity", |b| {
        b.iter(|| {
            let component = resolver.load(black_box(&wasm)).expect("load");
            let result = component
                .invoke("identity", &[Val::S32(42)])
                .expect("invoke");
            black_box(result);
        });
    });
}

fn bench_cache_hit_run_component_model_point_sum(c: &mut Criterion) {
    let wasm = point_sum_wasm();
    let mut resolver = Resolver::new().expect("resolver");
    let _ = resolver.load(&wasm).expect("prime");

    c.bench_function("cache_hit_run_component_model_point_sum", |b| {
        b.iter(|| {
            let component = resolver.load(black_box(&wasm)).expect("load");
            let result = component
                .invoke(POINT_SUM_FN, &[Val::S32(3), Val::S32(4)])
                .expect("invoke");
            black_box(result);
        });
    });
}

criterion_group!(
    benches,
    bench_engine_construct,
    bench_load_core_identity,
    bench_load_component_model_point_sum,
    bench_cold_run_core_identity,
    bench_cold_run_component_model_point_sum,
    bench_warm_run_core_identity,
    bench_warm_run_component_model_point_sum,
    bench_cache_hit_run_core_identity,
    bench_cache_hit_run_component_model_point_sum,
);
criterion_main!(benches);
