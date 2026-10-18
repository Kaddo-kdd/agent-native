---
id: AN-014
type: feature
status: in-progress
title: Recuperación E2E de Semantic IR v0.5 y Component Model
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: medium
code:
  - crates/compiler/tests/fixtures/ir/
  - crates/runtime/src/invoke.rs
  - crates/runtime/tests/e2e_v05.rs
source_roadmap_initiative: INIT-02-02
cross_initiatives:
  - INIT-03-01
  - INIT-03-03
source_work_item_candidate:
  - Technical Review 2026-10-17 (ref 06d270d) — WI-A
related_domain: Compilation
domains:
  - Compilation
  - Runtime & Memory
related_capabilities:
  - Component Compilation
  - Independent Runtime Execution
expected_value: >-
  Demostrar end-to-end que una especificación Semantic IR v0.5 real llega a
  ejecución correcta sobre el runtime, incluyendo retorno de `record`. Cierra el
  gap entre "el compilador produce bytes WASM" y "el runtime los ejecuta
  correctamente".
risks:
  - >-
    R-A-01: la resolución de funciones en interfaces WIT del Component Model
    puede exigir cambios en invoke_component; esfuerzo real no acotable hasta
    tener un fixture reproduciendo el escenario
  - >-
    R-A-02: construir un v0.5 IR sin host imports puede ser restrictivo; si no
    es posible, la prueba E2E necesita una policy allow más una implementación
    stub
dependencies:
  - AN-005
  - AN-006
  - AN-008
  - AN-009
  - AN-013
source_signals:
  - >-
    Technical Review: la suite actual no demuestra ejecución de
    operaciones/interfaces/estructuras de Semantic IR v0.5
  - 'Roadmap principio #1: recuperar antes de innovar'
  - 'ADR-003: v0.5 es baseline estable'
decision_candidates: []
related_decisions:
  - ADR-003
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-18'
---

# AN-014 — Recuperación E2E de Semantic IR v0.5 y Component Model

**Actor y resultado:**
Un contribuidor obtiene evidencia reproducible de que una especificación Semantic IR **v0.5** real pasa por verify → compile → load → invoke en el workspace nuevo, incluyendo una función que **retorna un `record`** y usa una interfaz WIT exportada. La prueba E2E vive en `crates/runtime/tests/e2e_v05.rs` y corre en CI 3 OS.

**Current behavior:**
- `crates/compiler/tests/poc_regression.rs` ejerce `verify_ir + compile_ir_to_wasm + validate_wasm_bytes` sobre v0.1–v0.4, pero **ninguna** prueba carga un componente **v0.5** en el runtime y verifica su ejecución.
- `crates/runtime/tests/smoke.rs` usa IR v0.1 (Core WASM).
- `crates/runtime/tests/host_errors.rs` usa IR v0.3 (Core WASM con imports).
- `crates/runtime/tests/policy.rs` igual.
- Ninguna prueba del runtime construye un Component Model y le invoca una función que retorne record.

**Target behavior:**
- `crates/compiler/tests/fixtures/ir/program-005.ir.json`: fixture v0.5 válido con `wit_contract`, al menos una función exportada que retorne un `record` de 2+ campos, sin host imports (o con imports vacíos autorizables).
- `crates/compiler/tests/fixtures/wit/program-005.wit`: contrato WIT del componente.
- `crates/runtime/tests/e2e_v05.rs`: compila el fixture, lo carga en el runtime, invoca una operación exportada vía interfaz WIT y verifica cada campo del record retornado.
- Si `invoke_component` actual no resuelve funciones dentro de interfaces WIT anidadas (`instance.get_func(func_name)` falla), extenderlo con lookup por `(interface_id, operation_name)`.
- Mantener pruebas negativas: función inexistente dentro de interfaz, firma incompatible, contrato WIT inválido.

**Problem:**
Sin esta evidencia, el camino crítico v0.5 → Component Model → ejecución queda sin cobertura. Es el gate técnico más importante antes de abrir INIT-03-03 (executor YAML) y la base del hito M3.

**Scope confidence:** medium. El fixture v0.5 es manual (no hay generador); la resolución de funciones en interfaces WIT puede requerir cambios no triviales en `invoke_component`.

## Acceptance criteria

1. **Fixture v0.5 válido** en `crates/compiler/tests/fixtures/ir/program-005.ir.json` + contrato WIT asociado.
2. **Verify + compile:** `verify_ir(ir)` devuelve 0 diagnósticos; `compile_ir_to_wasm(ir)` produce un Component Model que `validate_wasm_bytes` aprueba.
3. **Load:** `Engine::load_component` lo carga como `Payload::Component` (no cae a Core WASM).
4. **Invoke con record:** una prueba invoca una función exportada que retorna un `record` de 2+ campos y verifica cada campo con `assert_eq!`.
5. **Resolución WIT:** si `instance.get_func(func_name)` no encuentra la función porque está anidada en una interfaz, la prueba usa `instance.get_export(interface_id).get_func(op_name)` o equivalente — el soporte queda en `invoke_component` o en una función pública nueva.
6. **Contrato inválido:** una prueba carga un IR con `wit_contract.world` inexistente y espera `RuntimeError::LoadFailed` o un diagnóstico de compile explícito.
7. **Tests existentes (29) siguen verdes.**
8. **CI remota verde** 3 OS.

## Out of scope

- WASI preview2 / stdio (INIT-07-01).
- Nuevos tipos de Semantic IR (string, errores) — RM-07.
- Concurrencia (RM-08).
- Publicación remota del artefacto (INIT-05-03 / INIT-04-05).
- Reorganización global de `wit/` (INIT-02-03 / AN-xxx dedicado).

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-native-runtime --test e2e_v05
cargo test --workspace
```

Esperado: ≥30 tests pass (29 + al menos 1 E2E v0.5; típicamente 3-4 nuevos).

## Definition of Done

- Fixture v0.5 + WIT presentes.
- Prueba E2E con record retornado verde en local.
- Si fue necesario: `invoke_component` extendido con resolución WIT y documentado.
- CI remota verde.
- WI `completed`.

**Open questions:**

- OQ-AN-014-01: ¿se puede construir un v0.5 IR válido **sin** host imports? Si la gramática lo exige, la prueba E2E necesita (a) una policy `allow_module("host")` **más** (b) una implementación stub mínima para evitar `InstantiationFailed`. Alternativa (preferida): fixture con imports vacíos.
- OQ-AN-014-02: ¿`instance.get_func` en Component Model busca sólo exports top-level, o también anidados en interfaces? Confirmación requiere prototipo pequeño antes de redactar la prueba.
- OQ-AN-014-03: ¿el contrato WIT se migra ad-hoc dentro de `crates/compiler/tests/fixtures/wit/` o se espera a INIT-02-03? Propuesta: ad-hoc; INIT-02-03 formaliza la reorganización global.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-02 + RM-03 |
| Roadmap iniciativa | INIT-02-02 (principal), INIT-03-01, INIT-03-03 (atraviesa) |
| ADRs | ADR-003 |
| Technical Review | WI-A (ref 06d270d) |
| Prerrequisitos | AN-005, AN-006, AN-008, AN-009, AN-013 |

────────────────────────
Agent: work-item-agent
Produced: knowledge/delivery/work-items/draft/AN-014-e2e-semantic-ir-v05-component-model.md
Next: human review → kaddo ready
────────────────────────
