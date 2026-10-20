---
id: AN-038
type: chore
status: draft
created_at: 2026-10-19T00:00:00.000Z
title: Contención segura del agotamiento de fuel en Windows MSVC
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
code:
  - crates/runtime/Cargo.toml
  - crates/runtime/src/engine.rs
  - crates/runtime/src/invoke.rs
  - crates/runtime/tests/limits_negative_tests.rs
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
  - 'Bytecode Alliance Wasmtime Issue #10923 (refactorización de trampolines y excepciones)'
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

## Contexto del Problema

Durante la implementación de AN-035 y AN-037, se identificó un fallo reproducible en la plataforma `x86_64-pc-windows-msvc` con Wasmtime 27.0.0: cuando un componente WebAssembly agota su presupuesto de fuel, el proceso anfitrión finaliza abruptamente con código de excepción `0xc0000409` (`panic_cannot_unwind`).

La explicación técnica precisa de este fallo y su posterior corrección reside en el **[PR #9710 de Bytecode Alliance — Migrate libcalls to checking for traps](https://github.com/bytecodealliance/wasmtime/pull/9710)** (integrado en diciembre de 2024 e incluido en la versión 29.0.0). En versiones anteriores (v27/v28), los libcalls como `out_of_gas` salían invocando directamente `longjmp`, lo que en Windows MSVC activa `RtlUnwindEx`, violando las restricciones `nounwind` de Rust e induciendo el aborto forzoso. A partir de Wasmtime 29, los libcalls devuelven explícitamente un estado de trampa comprobado por el código generado, evitando desenrollados no controlados.

Mientras que en Linux y macOS el trap siempre se interceptó como `RuntimeError::FuelExhausted`, en Windows se requería actualizar a Wasmtime 29 para contener el fallo y evitar comprometer la integridad del proceso que aloja el runtime.

---

## Criterio Primario de Aceptación

> **Invariante de Contención:** Un componente WebAssembly que ejecute un bucle infinito con un presupuesto finito de fuel nunca debe terminar el proceso anfitrión. Debe retornar de forma limpia y controlada la variante tipada `RuntimeError::FuelExhausted { limit }`.

Para auditar este criterio de forma segura, el test de verificación **debe ejecutarse en un subproceso aislado** (`std::process::Command` ejecutando un binario o test harness auxiliar), de tal manera que si ocurre un fallo o aborto no controlado (`0xc0000409`), el proceso hijo termine pero el proceso de prueba padre permanezca intacto y reporte la falla de contención con detalle.

---

## Estrategia de Evaluación Técnica

Se evaluarán dos alternativas técnicas en orden secuencial:

### Alternativa A: Actualización de Wasmtime (Camino Embebido)
1. **Hipótesis:** En versiones posteriores de Wasmtime (por ejemplo Wasmtime 28.x, 29.x o 30.x), los mantenedores upstream de Bytecode Alliance refactorizaron trampolines y manejo de excepciones en Windows. Se evalúa empíricamente si alguna de estas versiones contiene el trap de fuel sin abortar el proceso anfitrión en Windows MSVC.
2. **Procedimiento:**
   - Crear una prueba de concepto en una rama aislada actualizando las dependencias de `wasmtime`, `wasmtime-wasi`, etc.
   - Ejecutar la prueba de bucle infinito en Windows MSVC mediante el harness de subproceso.
   - Verificar si el error se traduce limpiamente a `RuntimeError::FuelExhausted` sin terminar el proceso.
   - Si funciona: verificar compatibilidad estricta con las 148+ pruebas, benchmarks y la regla ADR-002 (sin compilador).

### Alternativa B: Aislamiento por Proceso Worker (Out-of-Process Fallback)
1. **Condición de activación:** Si Wasmtime en sus versiones posteriores mantiene el mismo comportamiento en Windows MSVC, o si la actualización introduce incompatibilidades o dependencias no deseadas.
2. **Diseño:**
   - En Windows, la ejecución de componentes no confiables o con límites finitos de fuel se delega a un proceso worker hijo subordinado (`agent-native-worker`).
   - El proceso host se comunica mediante IPC tipado (pipes/shared memory).
   - Si el worker termina anormalmente (código `0xc0000409`), el proceso host lo detecta de inmediato, contiene el blast radius, y traduce el fallo a un `RuntimeError::FuelExhausted` o `RuntimeError::ProcessAborted` sin que el anfitrión sufra impacto alguno.

---

## Entregables Previstos

1. **Harness de prueba en subproceso (`crates/runtime/tests/repro_windows_fuel_abort.rs`):**
   Binario / test harness que invoca el componente con bucle infinito en un proceso hijo e inspecciona el código de salida.
2. **Reporte de evaluación Wasmtime 27 vs versión evaluada:**
   Evidencia reproducible con logs, versiones de Rust y Wasmtime, y códigos de salida.
3. **Decisión de arquitectura o actualización de dependencias:**
   Propuesta formal para consolidar el cierre técnico de AN-035 e INIT-06-02.
