---
id: AN-038
type: chore
status: completed
created_at: 2026-10-19T00:00:00.000Z
completed_at: 2026-10-19T00:00:00.000Z
title: Contención segura del agotamiento de fuel en Windows MSVC
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
code:
  - crates/runtime/Cargo.toml
  - crates/runtime/src/engine.rs
  - crates/runtime/src/invoke.rs
  - crates/runtime/tests/fuel_accounting.rs
  - crates/runtime/tests/limits_negative_tests.rs
  - crates/runtime/tests/repro_windows_fuel_abort.rs
  - knowledge/delivery/poc-reports/AN-038-poc-windows-msvc-fuel-containment.md
source_roadmap_initiative: INIT-06-02
source_work_item_candidate:
  - WI-CANDIDATE-06-02-05
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Resource Contention & Limits
  - Independent Runtime Execution
expected_value: >-
  Investigar y validar una solución de contención para el agotamiento de fuel en
  Windows MSVC (`x86_64-pc-windows-msvc`), comparando Wasmtime 27 con versiones
  posteriores compatibles o evaluando aislamiento por proceso, garantizando que un
  componente no confiable con cómputo infinito nunca pueda terminar el proceso anfitrión.
risks:
  - >-
    R-WIN-01: Incompatibilidad de API o regresiones al evaluar versiones superiores de Wasmtime.
    Mitigación: Mantener la evaluación aislada en una rama/experimento sin alterar `main` hasta
    comprobar que compila y pasa la suite completa de 148+ pruebas.
  - >-
    R-WIN-02: Aborto del runner de pruebas durante la verificación del fallo.
    Mitigación: El harness de prueba para verificar contención ante bucles infinitos en Windows
    debe ejecutarse en un proceso hijo separado (`std::process::Command`), comprobando el código
    de salida y stdout/stderr sin arriesgar el runner de `cargo test`.
dependencies:
  - AN-035
  - AN-036
  - AN-037
source_signals:
  - 'Fallo reproducible en Windows MSVC: panic_cannot_unwind (0xc0000409) al agotar fuel'
  - 'Bytecode Alliance Wasmtime PR #9710: Migrate libcalls to checking for traps'
  - 'Principio de seguridad: Agent Native nunca debe prometer contener una ejecución que pueda terminar el host'
decision_candidates: []
related_decisions:
  - ADR-004
  - ADR-005
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
---

# AN-038 — Contención segura del agotamiento de fuel en Windows MSVC

## Resumen de Entrega

Este Work Item investigó, reprodujo y resolvió la brecha de contención de CPU en la plataforma Windows MSVC (`x86_64-pc-windows-msvc`), donde un componente WebAssembly que agotaba su presupuesto de fuel provocaba la terminación forzosa de todo el proceso anfitrión con código de excepción `0xc0000409` (`panic_cannot_unwind`).

Mediante una prueba de concepto estructurada y un arnés de prueba por subproceso, se demostró empíricamente que:
1. **Wasmtime 27 y 28:** Ambos reproducen el fallo no controlado (`0xc0000409`).
2. **Wasmtime 29.0.1:** Resuelve el problema de forma nativa e in-process, interceptando limpiamente el trap y retornando `RuntimeError::FuelExhausted { limit }` en 33 ms sin abortos.

La solución fue promovida al repositorio mediante la actualización de `wasmtime = "29.0.1"`, la eliminación de todas las exclusiones `#[ignore]` en Windows, y la incorporación del arnés de prueba automatizado por subproceso [`repro_windows_fuel_abort.rs`](crates/runtime/tests/repro_windows_fuel_abort.rs).

---

## Causa Técnica Upstream

La causa técnica del fallo y su posterior corrección reside en el **[PR #9710 de Bytecode Alliance — Migrate libcalls to checking for traps](https://github.com/bytecodealliance/wasmtime/pull/9710)** (integrado en diciembre de 2024 e incluido en Wasmtime 29.0.0):
- En Wasmtime 27 y 28, los libcalls internos de bajo nivel (como `out_of_gas`) salían directamente mediante `longjmp`. En Windows MSVC, esto activaba el desenrollado estructurado del sistema operativo (`RtlUnwindEx`), violando la suposición `nounwind` de Rust y disparando `panic_cannot_unwind`.
- Con el PR #9710, los libcalls devuelven un indicador booleano de si ocurrió una trampa en vez de forzar una salida por `longjmp`. El código compilado por Cranelift comprueba ese estado de retorno y realiza la salida de forma controlada y compatible con las convenciones de llamadas de Windows.

---

## Evidencia y Verificación

1. **Arnés por Subproceso ([`repro_windows_fuel_abort.rs`](crates/runtime/tests/repro_windows_fuel_abort.rs)):**
   - Ejecuta un componente con bucle infinito en un proceso hijo bajo un presupuesto finito de 5,000 unidades de fuel.
   - En Wasmtime 29.0.1, el hijo termina limpiamente con código de salida `0` y mensaje `CHILD_OUTCOME: FUEL_EXHAUSTED_TRAP limit=5000`.
   - El proceso padre verifica su propia integridad ejecutando de forma inmediata un segundo componente benigno que completa con éxito exacto.
2. **Eliminación Total de Exclusiones Windows:**
   - Se removieron todas las anotaciones `#[ignore]` / `target_os = "windows"` en `crates/runtime/tests/fuel_accounting.rs` (3 pruebas) y `crates/runtime/tests/limits_negative_tests.rs` (1 prueba).
   - El 100% de las pruebas negativas y de agotamiento se ejecutan ahora in-process en Windows.
3. **CI Multiplataforma (Pull Request #1 / Run 39362495281):**
   - 100% de éxito verde en las 4 plataformas: Windows MSVC (`windows-latest`), Linux (`ubuntu-latest`), macOS (`macos-latest`), y AWS Lambda offline.
   - Preservación estricta de la regla arquitectónica ADR-002 (el crate `runtime` no depende del compilador).
   - 9 de 9 benchmarks en `benches/baseline.rs` verificados exitosamente.

---

## Consideraciones de Mantenimiento y Deuda Técnica

- Wasmtime 29.0.1 data de enero de 2025. Hacia octubre de 2026, las versiones publicadas de Wasmtime han avanzado (versiones 49.x / 50.x).
- Se adopta Wasmtime 29.0.1 como la versión mínima que resuelve de forma estable y verificada la brecha de contención de fuel.
- Se deja registrado como seguimiento técnico para RM-08 (Performance & Concurrency) la evaluación y actualización hacia una versión activamente mantenida de Wasmtime antes del release de producción.
