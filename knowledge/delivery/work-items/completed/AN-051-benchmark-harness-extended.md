---
id: AN-051
type: feature
status: completed
created_at: 2026-10-20T00:00:00.000Z
completed_at: 2026-10-20T00:00:00.000Z
title: Harness de benchmarks reproducible (WASI I/O, IR v0.6, fuel) y ficha de entorno
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/benches/extended.rs
  - crates/runtime/benches/fixtures/wasi_console_echo.wat
  - crates/runtime/examples/bench_env.rs
  - knowledge/tech/baselines/benchmark-methodology.md
source_roadmap_initiative: INIT-08-01
initiative: INI-008
source_work_item_candidate:
  - WI-CANDIDATE-08-01-01
  - WI-CANDIDATE-08-01-02
domains:
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Generalizar el harness `criterion` de AN-017 a las capacidades de RM-06/RM-07
  (WASI I/O, IR v0.6 string/result, fuel) con una ficha de entorno automática y
  una metodología documentada, como base de INIT-08-02.
---

# AN-051 — Harness de benchmarks reproducible

## Entregado

- Bench `extended` con la convención `<stage>_<path>_<fixture>`:
  `warm_run_component_model_wasi_echo`, `cold_run_component_model_wasi_echo`,
  `warm_run_component_model_v06_greet`, `warm_run_component_model_v06_safe_div`,
  `warm_run_core_identity_fuel` (coste del medido de fuel frente a
  `warm_run_core_identity`).
- Fixture WASI versionado (`wasi_console_echo.wat`), sin depender del target
  `wasm32-wasip2`.
- Ejemplo `bench_env`: ficha de entorno en JSON (OS, arquitectura, CPUs lógicas,
  `rustc`, `cargo`, Wasmtime), sin identificadores de hardware.
- Metodología y criterio de varianza en
  `knowledge/tech/baselines/benchmark-methodology.md`.

## Criterios de aceptación

- AC-01: `cargo bench -p agent-native-runtime --bench extended` ejecuta los 5 benches.
- AC-02: `cargo run -p agent-native-runtime --example bench_env` emite JSON válido.
- AC-03: `baseline.rs` queda intacto (sin renombrar benches existentes).
- AC-04: `cargo clippy --workspace --all-targets -- -D warnings` verde.
- AC-05: la metodología define cómo repetir y qué varianza es aceptable.
