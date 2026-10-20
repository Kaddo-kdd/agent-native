---
id: AN-043
type: test
status: completed
created_at: 2026-10-20T00:00:00.000Z
completed_at: 2026-10-20T00:00:00.000Z
title: Fixtures de ejecución extrema (recursión controlada, allocación masiva y trampas inducidas)
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/tests/limits_negative_tests.rs
  - crates/runtime/src/invoke.rs
source_roadmap_initiative: INIT-06-05
source_work_item_candidate:
  - WI-CANDIDATE-06-05-01
  - WI-CANDIDATE-06-05-02
  - WI-CANDIDATE-06-05-03
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Acceptance Testing & Regression
  - Resource Contention & Limits
  - Independent Runtime Execution
expected_value: >-
  Proveer fixtures WebAssembly de bajo nivel que sometan al runtime a condiciones extremas
  de cómputo (recursión profunda con control de pila/fuel), estrés de memoria (asignación dinámica
  y estática masiva) y trampas de ejecución inducidas (división por cero, instrucción unreachable),
  verificando de forma determinista y reproducible que el motor contiene cualquier fallo sin abortar
  el proceso ni corromper ejecuciones posteriores.
risks:
  - >-
    R-EXT-01: Falsos positivos por diferencias de límites de pila del sistema operativo.
    Mitigación: Las pruebas de recursión controlada se acotan mediante unidades abstractas
    de fuel deterministas en lugar de confiar en stack overflows no portables.
dependencies:
  - AN-035
  - AN-036
  - AN-037
  - AN-038
source_signals:
  - 'Roadmap INIT-06-05: Fixtures de ejecución extrema'
  - 'ADR-004: Fail-explicit y contención estricta de errores'
  - 'ADR-005: Aislamiento por Store efímero'
decision_candidates: []
related_decisions:
  - ADR-004
  - ADR-005
---

# AN-043 — Fixtures de ejecución extrema (recursión controlada, allocación masiva y trampas inducidas)

## Resumen de Entrega

Este Work Item formaliza e implementa la suite completa de fixtures de ejecución extrema a bajo nivel en [`crates/runtime/tests/limits_negative_tests.rs`](crates/runtime/tests/limits_negative_tests.rs), cerrando definitivamente la iniciativa **INIT-06-05** y culminando el bloque **RM-06** (*Runtime Security & Memory*).

Los fixtures están construidos directamente en bytecode WebAssembly mediante `wasm-encoder`, sin depender de compiladores externos ni de extensiones no soportadas de Semantic IR v0.5.

## Acceptance Criteria Verificados

- **AC-01 (Fixture de Recursión Controlada - WI-CANDIDATE-06-05-01):**
  - Módulo WASM con función recursiva de bajo nivel `recurse(n: i32) -> i32` (`if n <= 0 { 1 } else { recurse(n - 1) + 1 }`).
  - Validación de camino feliz: con recursión somera (`depth = 20`) y presupuesto suficiente (`50,000` unidades), retorna determinísticamente `21` y reporta métricas de fuel consumido.
  - Validación de contención: con recursión profunda (`depth = 5,000`) y presupuesto acotado (`500` unidades), se agota el fuel de forma determinista retornando `RuntimeError::FuelExhausted { limit: 500 }` sin desbordamiento de pila del host ni abortos.

- **AC-02 (Fixture de Allocación Masiva - WI-CANDIDATE-06-05-02):**
  - Validación estática: rechazo inmediato en instanciación ante módulos que declaran memorias iniciales que superan el techo (`32 MB` vs límite de `16 MB`), retornando `RuntimeError::MemoryLimitExceeded`.
  - Validación dinámica: bucle de crecimiento dinámico (`memory.grow`) que al superar el techo de memoria configurado (`256 KiB`) recibe `-1`, entra en trampa controlada y es clasificado determinísticamente como `RuntimeError::MemoryLimitExceeded`.
  - Validación de crecimiento de tablas: intento de crecimiento de tablas de funciones más allá del límite devuelve `-1` de forma segura.

- **AC-03 (Fixture de Trampas Inducidas - WI-CANDIDATE-06-05-03):**
  - Módulo WASM con trampas explícitas de ejecución:
    1. `div_zero`: ejecución de división entera con divisor 0 (`i32.div_s 1 / 0`). Retorna determinísticamente `RuntimeError::ExecutionFailed` con diagnóstico descriptivo de división por cero.
    2. `trap_unreachable`: ejecución explícita de instrucción WebAssembly `unreachable`. Retorna limpiamente `RuntimeError::ExecutionFailed` con diagnóstico de trampa sin abortar el proceso anfitrión.
  - Refinamiento en `map_execution_error` para garantizar que `Trap::UnreachableCodeReached` sin denegación previa de memoria se clasifique explícitamente con su causa raíz.
