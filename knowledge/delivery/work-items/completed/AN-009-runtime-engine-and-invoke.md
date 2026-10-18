---
id: AN-009
type: feature
status: completed
completed_at: 2026-10-17
title: Runtime engine mínimo — carga + invocación de componentes
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: medium
code:
  - crates/runtime/Cargo.toml
  - crates/runtime/src/lib.rs
  - crates/runtime/src/engine.rs
  - crates/runtime/src/invoke.rs
  - crates/runtime/src/error.rs
  - crates/runtime/tests/smoke.rs
source_roadmap_initiative: INIT-03-01
source_work_item_candidate:
  - WI-CANDIDATE-03-01-01
  - WI-CANDIDATE-03-01-02
  - WI-CANDIDATE-03-01-03
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  API Rust mínima que carga artefactos WASM Component Model producidos por
  agent-native-compiler y ejecuta una función exportada vía WIT, sin depender
  del compiler ni del LLM (ADR-002). Habilita INIT-03-03/04/05.
risks:
  - >-
    R-RT-01: API pública inicial podría cambiar al añadir policy loader
    (INIT-06-01); mitigado dejando tipos públicos mínimos
  - >-
    R-RT-02: Wasmtime baseline no fijada (ADR-003 [open]); seleccionamos 27.0
    por paridad con POC, revisable
dependencies:
  - AN-005
  - AN-006
  - AN-008
source_signals:
  - Matriz de extracción C-07 (parte carga/invocación)
  - 'ADR-001: runtime depende solo de ir, nunca de compiler'
  - 'ADR-002: runtime no depende del LLM'
  - 'ADR-003: baseline Wasmtime [open]'
decision_candidates: []
related_decisions:
  - ADR-001
  - ADR-002
  - ADR-004
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-18'
---

# AN-009 — Runtime engine mínimo

**Actor y resultado:**
`agent-native-runtime` ofrece `Engine::new()`, `Engine::load_component(bytes)`, `Component::invoke(func, args) -> Result<Vec<Val>, Error>`. Un consumidor Rust puede cargar un `.wasm` producido por `agent-native-compiler` y ejecutar una función exportada **sin** cargar el compiler ni acceder al LLM.

**Current behavior:**
`crates/runtime/src/lib.rs` tiene solo docstring (AN-004). No existe API ejecutable.

**Target behavior:**
- `src/engine.rs`: wrapper mínimo sobre `wasmtime::Engine` + `wasmtime::component::Component`, con Component Model activado.
- `src/invoke.rs`: `invoke(func_name, args) -> Result<Vec<Val>, Error>`; args/returns tipados a `wasmtime::component::Val` (los tipos primitivos `S32`/`Bool`/`Record` que la POC usa).
- `src/error.rs`: enum `RuntimeError` tipado para fallos (función no encontrada, aridad, tipo de arg, trap).
- `src/lib.rs`: re-exporta API pública.
- `Cargo.toml`: añade `wasmtime` 27.0 (paridad POC) con features `cranelift`, `runtime`, `component-model`; `thiserror` para el enum.
- Smoke test: compilar IR `identity` con `agent-native-compiler` en el harness de test, cargar y ejecutar con arg `42`, esperar `42`.

**Problem:**
Sin runtime no se puede ejecutar lo que el compiler produce. Es prerrequisito duro de RM-05 (CLI distribuible) y de RM-03 completo.

**Expected result:**
Runtime ejecuta un componente real producido por el compiler, con fallos tipados.

**Scope confidence:** medium (nueva superficie; mitigada con smoke test E2E).

## Acceptance criteria

1. **API pública mínima**: `Engine::new`, `Engine::load_component`, `Component::invoke`, `RuntimeError`.
2. **Component Model habilitado** en la configuración de `wasmtime::Engine`.
3. **Smoke test E2E verde**: compilar IR mínimo + cargar + invocar + comparar resultado. Al menos 2 casos:
   - `identity(42)` → `[Val::S32(42)]`.
   - Función inexistente → `RuntimeError::FunctionNotFound`.
4. **ADR-002 enforced**: `cargo tree -p agent-native-runtime` **no** lista `agent-native-compiler` ni ningún crate LLM.
5. **El test E2E** usa `agent-native-compiler` como **dev-dependency** únicamente (el crate `runtime` no lo lista en `[dependencies]`).
6. **Lint y fmt verdes**: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`.
7. **CI remota verde** en los 3 OS.

## Out of scope

- Policy loader de capabilities / deny-by-default (INIT-03-05).
- Executor YAML (INIT-03-03).
- Propagación de errores host completos (INIT-03-04 — este WI cubre solo errores de invocación).
- Fuel accounting, instance limits (RM-06).
- Caché de componentes, warm pool (RM-08).
- Resolver reutilizable de dependencias (INIT-03-02).
- Carga de imports arbitrarios — este WI acepta solo componentes sin imports o con imports vacíos. Imports no triviales llegan en INIT-03-04/05.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-native-runtime --test smoke
cargo test --workspace

# ADR-002 enforcement
cargo tree -p agent-native-runtime --prefix none --edges normal | grep -i "agent-native-compiler" && exit 1 || echo "OK"
```

Esperado: 18 tests pass en el workspace (16 previos + 2 nuevos).

## Definition of Done

- API mínima implementada y re-exportada.
- Smoke test E2E verde local y en CI 3 OS.
- `cargo tree` confirma ADR-002.
- WI `completed`.

**Open questions:**

- OQ-AN-009-01: ¿`agent-native-compiler` como dev-dep del runtime viola el espíritu de ADR-002? Propuesta: **no** — `[dev-dependencies]` no entra al grafo de artefactos distribuibles; el runtime se puede publicar sin él. El lint CI valida solo `--edges normal`.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-03 |
| Roadmap iniciativa | INIT-03-01 |
| WIs consolidados | WI-CANDIDATE-03-01-01, 03-01-02, 03-01-03 |
| Prerrequisitos | AN-005, AN-006, AN-008 |
| ADRs | ADR-001, ADR-002, ADR-004 |

## Learning

**Lo que salió como se esperaba:**

- API mínima (`Engine::new`, `Engine::load_component`, `Component::invoke`, `RuntimeError`) suficiente para el smoke E2E.
- Compiler como `[dev-dependencies]` del runtime: ADR-002 enforced (lint CI con `cargo tree --edges normal` sigue verde).
- Smoke test E2E compila IR → carga → invoca → verifica resultado en el mismo test.

**Lo inesperado:**

- **IR v0.1-v0.3 compilan a Core WASM**, no Component Model. El runtime tuvo que soportar ambos: intenta `wasmtime::component::Component::new` primero, cae al `wasmtime::Module` si falla. Añadió ~70 LOC a `invoke.rs` para el path Core (con conversión primitiva Val ↔ wasmtime::Val). Documentado en el módulo.
- **`encoding_rs@0.8.42`** (dep transitiva de Wasmtime 27) requiere **Rust 1.88**, pero el toolchain pinneado es 1.85. Pinneé `encoding_rs` a 0.8.35 vía `cargo update --precise`. Como `.gitignore` excluía `Cargo.lock`, el pin no sobrevivía al push — **`Cargo.lock` ahora se commitea** (estándar para workspace con binary; necesario para release pipeline futuro, INIT-05-03). `.gitignore` actualizado.
- Wasmtime 27 fue elegido por paridad con POC. Revisable cuando ADR-003 cierre la selección definitiva.

**Riesgos materializados:**

- R-RT-01 parcial: la API quedó más grande de lo previsto por el dual-path Core/Component. Mitigable: la API pública (`invoke`) no cambia; es solo detalle interno.
- R-RT-02 nulo por ahora (Wasmtime 27 funciona).

**Deuda y seguimientos:**

- Imports arbitrarios en componentes **no se cargan** (política explícita del WI). INIT-03-04/05 abordan allow-list y deny-by-default.
- La conversión Val ↔ wasmtime::Val cubre solo `S32` y `Bool`; records (si llegan en Component Model) requieren implementación en INIT-03-03 o posterior.

**Lo que esta iniciativa establece para las siguientes:**

- **INIT-03-03** (executor YAML) tiene `Engine::load_component` + `Component::invoke` como primitivas.
- **INIT-03-04** (errores host) añade variantes a `RuntimeError` para imports no autorizados.
- **INIT-03-05** (deny-by-default) reemplaza el `Linker<()>` vacío por un policy loader.
- **INIT-05-01** (CLI `run`) puede llamar directamente a `Engine::load_component`.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-009-runtime-engine-and-invoke.md
crates/runtime/src/{engine,invoke,error,lib}.rs
crates/runtime/tests/smoke.rs (2 tests)
Cargo.lock (newly tracked)

Next:
INIT-03-03 (executor YAML) o INIT-03-04 (errores host)
────────────────────────
