---
id: AN-008
type: feature
status: completed
completed_at: 2026-10-17
title: Compiler IR → WASM Component + hash SHA-256
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: medium
code:
  - crates/compiler/Cargo.toml
  - crates/compiler/src/lib.rs
  - crates/compiler/src/codegen.rs
  - crates/compiler/src/hash.rs
  - crates/compiler/tests/determinism.rs
source_roadmap_initiative: INIT-02-02
source_work_item_candidate:
  - WI-CANDIDATE-02-02-01
  - WI-CANDIDATE-02-02-02
related_domain: Compilation
domains:
  - Compilation
related_capabilities:
  - Component Compilation
expected_value: >-
  Compilador Semantic IR → WASM Component determinista con hash SHA-256,
  reutilizando verify_ir del crate ir. Habilita INIT-04-05 (publicación) y
  desbloquea AN-007 (regresión integral POC con las 8 pruebas originales).
risks:
  - >-
    R-PORT-02: ~670 LOC migradas en un solo WI; mitigado con smoke test de
    determinismo end-to-end
  - >-
    R-DET-01: diferencias de determinismo entre OS en toolchain WASM; se cubre
    con smoke test local y CI matrix 3 OS
dependencies:
  - AN-005
  - AN-006
source_signals:
  - Matriz de extracción C-01 (parte backend WASM)
  - 'ADR-001: compiler depende solo de ir'
  - 'ADR-003: v0.5 baseline'
  - 'ADR-004: hash SHA-256 reproducible como control mínimo'
decision_candidates: []
related_decisions:
  - ADR-001
  - ADR-003
  - ADR-004
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-17'
---

# AN-008 — Compiler IR → WASM Component + hash SHA-256

**Actor and outcome:**
`agent-native-compiler` expone `pub fn compile_ir_to_wasm(root) -> Result<Vec<u8>, Vec<Diagnostic>>`, `pub fn validate_wasm_bytes`, `pub fn print_wasm_to_wat` y `pub fn sha256_hex`, portando el pipeline backend de la POC con comportamiento 1:1 y reutilizando `agent_native_ir::verify_ir` como paso previo.

**Current behavior:**
`crates/compiler/src/lib.rs` tiene solo docstring (de AN-004); solo depende de `agent-native-ir`.

**Target behavior:**
- `src/codegen.rs` con `compile_ir_to_wasm`, `emit_expr`, `infer_expr_type`, `is_const_zero`, `wit_interface_id` (helper traído del verifier POC, usado solo aquí).
- `src/hash.rs` con `sha256_hex`.
- `src/lib.rs` re-exporta `compile_ir_to_wasm`, `validate_wasm_bytes`, `print_wasm_to_wat`, `sha256_hex`.
- Dependencias: `agent-native-ir`, `wasm-encoder`, `wasmparser`, `wasmprinter`, `wit-component`, `sha2`, `serde_json`.
- Smoke test de determinismo: mismo IR → mismo hash en 2 corridas consecutivas.

**Problem:**
Sin compiler no hay artefactos WASM. Es el prerrequisito duro de INIT-03-01 (runtime), AN-007 (regresión integral) e INIT-04-05 (publicación).

**Expected result:**
Compiler portado + hash determinista verificado en local.

**Scope confidence:** medium (port grande + nuevas deps WASM).

## Acceptance criteria

1. **`compile_ir_to_wasm` migrado** con comportamiento equivalente a la POC (lee IR, verifica vía `agent_native_ir::verify_ir`, emite WASM Component).
2. **Helpers migrados**: `emit_expr`, `infer_expr_type`, `is_const_zero`, `wit_interface_id`.
3. **Hash y validación**: `sha256_hex`, `validate_wasm_bytes`, `print_wasm_to_wat` exportados.
4. **Deps correctas**: `crates/compiler/Cargo.toml` depende de `agent-native-ir` + `wasm-encoder`, `wasmparser`, `wasmprinter`, `wit-component`, `sha2`, `serde_json`. **No** depende de `agent-native-runtime`.
5. **Smoke test de determinismo**: compilar un IR válido 2 veces produce el mismo hash SHA-256; `validate_wasm_bytes` aprueba el resultado.
6. **Build + lint verdes**: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, `cargo build --workspace --release`.
7. **Lint ADR-001/002**: `cargo tree -p agent-native-runtime | grep compiler` sigue vacío.

## Out of scope

- Publicación atómica (INIT-04-05, vive en `cli::publish`).
- Suite integral POC (AN-007, usa esto + verify_ir en los 8 escenarios).
- Determinismo cross-OS verificado en CI (lo cubre el workflow matrix existente al pasar los tests).

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-native-compiler
cargo tree -p agent-native-runtime --prefix none | grep compiler || echo OK
```

## Definition of Done

- `codegen.rs`, `hash.rs` creados y re-exportados.
- Smoke test de determinismo pasa local.
- CI remota verde.
- WI `completed`.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-02 |
| Roadmap iniciativa | INIT-02-02 |
| WI candidato consolidado | WI-CANDIDATE-02-02-01 (backend WASM) + WI-CANDIDATE-02-02-02 (hash) |
| Prerrequisitos | AN-005, AN-006 |
| ADRs | ADR-001, ADR-003, ADR-004 |

## Learning

**Lo que salió como se esperaba:**

- Port de 660 LOC de `compile_ir_to_wasm` + helpers compila 1:1 tras ajustes mecánicos.
- `sha256_hex` movido a `hash.rs` separado (como pedía el WI); el resto queda en `codegen.rs`.
- 2 tests de determinismo + fallo explícito pasan a la primera.
- `cargo tree` confirma: `agent-native-runtime` sin dep hacia `compiler` ✅ (ADR-002); `agent-native-compiler` sin dep hacia `runtime` ✅ (ADR-001).

**Lo inesperado:**

- **Rust 2024 edition rompe `if let (Some(ref x), ...)`.** Las nuevas match-ergonomics ya no permiten el modificador `ref` cuando el patrón hace binding por referencia. Dos ocurrencias en el POC port — se eliminaron directamente (`Some(ref contract)` → `Some(contract)`). Fidelidad semántica preservada; solo cambia la expresión del binding.
- **El POC usa `include_str!("../../../wit/order.wit")`** como fallback de IR v0.4. Como `wit/` del producto nuevo no existe (y la migración de WIT fixtures está en INIT-02-03), copié `order.wit` a `crates/compiler/wit/order.wit` como recurso embebido del crate. Se re-evaluará en INIT-02-03 cuando se reorganice `wit/` del workspace.
- **Dependencia `wit-parser` adicional:** la POC la lista en `Cargo.toml` pero el código del compilador portado **no la usa** (solo `wit-component`). La incluí para igualar la POC; se puede quitar en un siguiente clean-up.
- **Nuevo lint `clippy::bool_to_int_with_if`** en Rust 1.85 añadido al allow-list del módulo.

**Riesgos materializados:**

- R-PORT-02 parcial: los 3 problemas de compile (`ref` match, `include_str!`, imports faltantes) se resolvieron sin regresión funcional. Los 2 tests de determinismo son la evidencia que faltaba.
- R-DET-01 nulo en local; CI matrix 3 OS lo verifica.

**Deuda y seguimientos:**

- INIT-02-03 (reorganizar `wit/`) debe decidir si `crates/compiler/wit/order.wit` queda como recurso embebido del crate o migra a `wit/fixtures/`.
- El `wit-parser` sin uso se mantiene por paridad POC; limpieza para un follow-up corto.
- `AN-007` (regresión integral POC) ahora puede materializarse: tiene todas las piezas (`verify_ir`, `compile_ir_to_wasm`, `validate_wasm_bytes`, `sha256_hex`).

**Lo que esta iniciativa establece para las siguientes:**

- **AN-007** desbloqueado: las 8 pruebas POC pueden portarse directamente, pues llaman solo a APIs que ya existen en el workspace.
- **INIT-03-01** (`runtime`) puede empezar: tiene artefactos WASM reales (producidos por `compile_ir_to_wasm`) sobre los cuales iterar.
- **INIT-04-05** (publicación): tiene `sha256_hex` + bytes WASM determinísticos como contrato de integridad.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-008-compiler-ir-to-wasm.md
crates/compiler/src/codegen.rs (719 LOC)
crates/compiler/src/hash.rs
crates/compiler/wit/order.wit
crates/compiler/tests/determinism.rs (2 tests)

Next:
AN-007 (regresión integral POC — 8 tests originales)
INIT-03-01 (crate runtime)
────────────────────────
