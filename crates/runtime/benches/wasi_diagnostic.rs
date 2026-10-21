#![allow(clippy::doc_markdown)]

//! Diagnostic decomposition for `cold_run_component_model_wasi_echo` (AN-062).
//!
//! The historical benchmark in `extended.rs` remains unchanged. These three
//! measurements are separate observations that localise variability; their
//! values must not be added together or used to rewrite historical evidence.

use agent_native_runtime::{Engine, ExecutionLimits, HostPolicy, Val, WasiIoConfig};
use criterion::{Criterion, criterion_group, criterion_main};
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

fn bench_wasi_diagnostic(c: &mut Criterion) {
    let wasm = wasi_echo_wasm();
    let policy = stdio_policy();
    let limits = ExecutionLimits::unmetered();

    // Stage 1: only Engine construction, matching the first operation in the
    // historical cold path.
    c.bench_function("diagnostic_wasi_engine_construct", |b| {
        b.iter(|| black_box(Engine::new().expect("engine")));
    });

    // Stage 2: the Engine exists before timing; only policy-aware component
    // loading is timed here.
    let load_engine = Engine::new().expect("engine");
    c.bench_function("diagnostic_wasi_component_load_with_policy", |b| {
        b.iter(|| {
            let component = load_engine
                .load_component_with_policy(black_box(&wasm), &policy)
                .expect("load");
            black_box(component);
        });
    });

    // Stage 3: Engine and component exist before timing. Keep the same WASI
    // input, export, arguments and limits as the historical cold benchmark.
    let invoke_engine = Engine::new().expect("engine");
    let component = invoke_engine
        .load_component_with_policy(&wasm, &policy)
        .expect("load");
    c.bench_function("diagnostic_wasi_invoke_with_wasi", |b| {
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
}

criterion_group!(benches, bench_wasi_diagnostic);
criterion_main!(benches);
