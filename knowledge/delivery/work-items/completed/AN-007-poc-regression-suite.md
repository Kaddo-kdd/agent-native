---
id: AN-007
type: chore
status: completed
completed_at: 2026-10-17
title: Suite integral de regresión POC (8 tests originales)
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/compiler/tests/poc_regression.rs
source_roadmap_initiative: INIT-02-01
source_work_item_candidate:
  - WI-CANDIDATE-02-01-04
related_domain: Semantic IR & Verification
domains:
  - Semantic IR & Verification
  - Compilation
related_capabilities:
  - Semantic IR Verification
  - Component Compilation
expected_value: >-
  Cerrar INIT-02-01 reproduciendo las 8 pruebas unitarias originales de la POC
  (irtool) que combinan verify_ir + compile_ir_to_wasm + validate_wasm_bytes +
  sha256_hex en el workspace nuevo.
risks:
  - >-
    R-REG-01: una prueba podría fallar por diferencias de salida WASM entre
    toolchains; mitigado por el test de determinismo de AN-008 ya verde
dependencies:
  - AN-005
  - AN-006
  - AN-008
source_signals:
  - 'Matriz de extracción C-01 (parte tests), Y-01..Y-05'
  - 'Roadmap INIT-02-01 exit criterion: 8 pruebas irtool POC reproducidas'
  - 'AN-006 Learning: AN-007 reagendado tras AN-008'
decision_candidates: []
related_decisions:
  - ADR-003
  - ADR-004
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-18'
---

# AN-007 — Suite integral de regresión POC (8 tests originales)

**Actor and outcome:**
Los 8 unit tests originales de `_reference/poc-agent-native/crates/irtool/src/lib.rs` (líneas 2360-2748) corren verdes como tests de integración en el workspace nuevo contra `agent_native_ir::verify_ir` + `agent_native_compiler::{compile_ir_to_wasm, validate_wasm_bytes, sha256_hex}`.

**Current behavior:**
`crates/ir` tiene 6 tests verifier-only (AN-006). `crates/compiler` tiene 2 tests de determinismo (AN-008). Las 8 pruebas integrales POC aún no se reproducen en el workspace nuevo.

**Target behavior:**
- `crates/compiler/tests/poc_regression.rs` reproduce las 8 pruebas: `valid_smoke_ir_passes_and_compiles_reproducibly`, `valid_v0_2_smoke_ir_passes_and_compiles`, `detects_v0_2_semantic_error_codes`, `validates_and_compiles_v0_3_smoke_module`, `detects_v0_3_capability_and_import_diagnostics`, `validates_and_compiles_v0_4_smoke_module`, `detects_v0_4_wit_contract_mismatch`, `test_compiles_v0_4_to_component_model`.
- Los tests importan de `agent_native_ir` y `agent_native_compiler` directamente — nada de `super::*` del binario POC.
- Si alguna prueba falla por diferencias legítimas del workspace nuevo (nombres de API, baseline Wasmtime), se documenta en Learning y se adapta la prueba sin alterar su intención semántica.

**Problem:**
Sin la suite integral, INIT-02-01 queda abierto y el gate "equivalencia funcional con POC" para RM-02 permanece sin evidencia reproducible.

**Expected result:**
INIT-02-01 cerrado 3/3. RM-02 pasa a 3/5 iniciativas completas (AN-005, AN-006, AN-007, AN-008 + INIT-02-04 cubierto parcialmente por AN-008).

**Scope confidence:** high (código ya portado; sólo falta activar las pruebas).

## Acceptance criteria

1. **8 pruebas migradas.** `crates/compiler/tests/poc_regression.rs` contiene las 8 funciones `#[test]` con el mismo nombre y contenido semántico que la POC.
2. **Dependencias correctas.** Los tests importan de `agent_native_ir` y `agent_native_compiler` únicamente.
3. **Todas pasan.** `cargo test -p agent-native-compiler --test poc_regression` → 8 passed, 0 failed.
4. **Workspace completo verde.** `cargo test --workspace` suma 16 tests pasando (6 ir + 2 compiler determinism + 8 poc regression).
5. **Lint y fmt.** `cargo fmt --check` y `cargo clippy --workspace --all-targets -- -D warnings` sin errores.
6. **Cualquier adaptación documentada.** Si una prueba requiere ajuste (p. ej. nombres de capability WIT, versiones de IR), el cambio se cita en Learning con antes/después.

## Out of scope

- Suites YAML de aceptación `tests/*.yaml` del POC — pertenecen a INIT-03-03 (executor YAML en runtime).
- Benchmarks de rendimiento — RM-08.
- Agregar casos nuevos más allá de los 8 originales.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-native-compiler --test poc_regression
cargo test --workspace
```

Esperado: 16 tests pass en el workspace.

CI remota verde en 3 OS.

## Definition of Done

- `poc_regression.rs` creado con las 8 pruebas.
- Todas pasan localmente y en CI remota (3 OS).
- Workspace sigue bajo clippy pedantic verde.
- WI pasa a `completed`.

**Open questions:**

- OQ-AN-007-01: ¿cualquier prueba POC usa helpers privados que necesitarían exportarse? Propuesta: ninguno — las pruebas sólo usan APIs públicas (`verify_ir`, `compile_ir_to_wasm`, `validate_wasm_bytes`, `sha256_hex`); los helpers (`sha256_hex` especialmente) ya están re-exportados.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-02 |
| Roadmap iniciativa | INIT-02-01 (parte 3 de 3; cierra la iniciativa) |
| WI candidato | WI-CANDIDATE-02-01-04 (suite de regresión IR) |
| Prerrequisitos | AN-005, AN-006, AN-008 |
| ADRs | ADR-003, ADR-004 |

## Learning

**Lo que salió como se esperaba:**

- **8/8 tests pasan a la primera** después de resolver 3 problemas mecánicos (imports, fixtures `include_str!`, dedent). Esto valida que AN-005+AN-006+AN-008 preservan semánticas 1:1 con el POC.
- Las 3 fixtures IR (`smoke-v3`, `smoke-v4`, `program-004`) copiadas a `crates/compiler/tests/fixtures/ir/` resuelven `include_str!` sin romper la política `_reference/`.

**Lo inesperado:**

- `tail -n -1` + `sed` dejó el archivo sin su `}` final. Ajuste trivial.
- La POC `use super::*;` ocultaba el uso de `serde_json::Value` como tipo público — había que importarlo explícitamente al salir del módulo.
- `print_wasm_to_wat` no se usa en ninguno de los 8 tests; se removió del `use` para pasar `unused_imports`.
- Clippy pedantic marcó 2 lints en las fixtures (`too_many_lines` por el test monolítico `detects_v0_3_capability_and_import_diagnostics` de 120 líneas, y `doc_markdown` por un `JSON` sin backticks en un comentario). Resueltos con `#![allow]` acotado al archivo de tests.

**Riesgos materializados:** ninguno. R-REG-01 (diferencias WASM entre toolchains) no se materializó — el determinismo de AN-008 ya lo cubría.

**Deuda y seguimientos:**

- Las fixtures en `crates/compiler/tests/fixtures/ir/` son copias del POC. INIT-02-03 (reorganización `wit/`) decidirá si fixtures IR también se consolidan en `tests/fixtures/ir/` a nivel workspace.
- El test `detects_v0_3_capability_and_import_diagnostics` tiene 120 líneas — refactor a sub-funciones es un follow-up menor.

**Lo que esta iniciativa establece para las siguientes:**

- **INIT-02-01 CERRADO 3/3.** El exit criterion "8 pruebas POC reproducidas" está satisfecho.
- **Red de seguridad** para refactorizar `verifier.rs` y `codegen.rs`: cualquier refactor que cambie semánticas rompe estos 8 tests.
- **INIT-03-01** (runtime) puede empezar: tiene el pipeline completo verify+compile+hash como base para probar carga y ejecución.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-007-poc-regression-suite.md
crates/compiler/tests/poc_regression.rs (8 tests, 400 LOC)
crates/compiler/tests/fixtures/ir/{smoke-v3,smoke-v4,program-004}.ir.json

Next:
INIT-02-03 (reorganizar wit/) o INIT-03-01 (runtime)
────────────────────────
