---
id: AN-005
type: feature
status: completed
completed_at: 2026-10-17
title: Modelo de datos IR v0.5 (tipos públicos + schemas)
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/ir/Cargo.toml
  - crates/ir/src/lib.rs
  - crates/ir/src/model.rs
  - crates/ir/src/diagnostic.rs
  - crates/ir/src/context.rs
  - crates/ir/schemas/ir.schema.json
  - crates/ir/schemas/history/ir.schema.v2.json
  - crates/ir/schemas/history/ir.schema.v3.json
  - crates/ir/schemas/history/ir.schema.v4.json
source_roadmap_initiative: INIT-02-01
source_work_item_candidate:
  - WI-CANDIDATE-02-01-01
related_domain: Semantic IR & Verification
domains:
  - Semantic IR & Verification
related_capabilities:
  - Semantic IR Verification
expected_value: >-
  Tipos públicos y schemas del Semantic IR v0.5 disponibles en crates/ir como
  fundamento para el verifier (AN-006) y la suite de regresión (AN-007).
risks:
  - >-
    R-API-01: congelar prematuramente nombres públicos; mitigado con pub(crate)
    donde dude
dependencies:
  - AN-003
  - AN-004
source_signals:
  - Matriz de extracción C-01 (parte modelo)
  - Matriz de extracción S-01 (schemas IR)
  - 'ADR-003: v0.5 como baseline estable'
decision_candidates: []
related_decisions:
  - ADR-001
  - ADR-003
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-17'
---

# AN-005 — Modelo de datos IR v0.5 (tipos públicos + schemas)

> Idioma del proyecto: **español**. Mantener en inglés el código, los nombres de archivo, los comandos y las claves de configuración.

**Actor and outcome:**
El crate `agent-native-ir` ofrece los tipos públicos del Semantic IR v0.5 (`Diagnostic`, `IrType`, `ImportSignature`, `RecordDef`, `WitContractDef`, `Context`, `FuncSignature`) con las mismas semánticas que la POC, más los 4 schemas JSON (v0.5 estable + v2/v3/v4 histórico) migrados a `schemas/`. El crate compila con `clippy -D warnings` y las APIs quedan listas para que AN-006 implemente el verifier sobre ellas.

**Current behavior:**
`crates/ir/src/lib.rs` solo tiene un docstring (scaffolding de AN-004). No existen tipos ni schemas.

**Target behavior:**
- `src/model.rs` con los tipos de datos IR: `IrType`, `ImportSignature`, `RecordDef`, `WitContractDef`, `FuncSignature`.
- `src/diagnostic.rs` con `Diagnostic`.
- `src/context.rs` con `Context` (contenedor de verificación: imports, records, functions, capabilities).
- `src/lib.rs` re-exporta las APIs públicas y documenta las decisiones (ADR-001/003).
- `schemas/ir.schema.json` = v0.5 (baseline estable, equivalente a la POC `v5`).
- `schemas/history/ir.schema.v{2,3,4}.json` = linaje histórico (no API pública).
- Dependencias en `Cargo.toml`: `serde`, `serde_json` (mínimas; `sha2` y `wasm-encoder` **no** entran — pertenecen a `compiler`).

**Entry points:**
- `crates/ir/src/{lib,model,diagnostic,context}.rs`.
- `crates/ir/schemas/ir.schema.json` + `schemas/history/`.

**End-to-end flow:**
1. Leer tipos de `_reference/poc-agent-native/crates/irtool/src/lib.rs` (líneas 1-110).
2. Portar fielmente los tipos a módulos separados sin añadir campos nuevos.
3. Copiar schemas desde `_reference/poc-agent-native/ir/*.json` a `crates/ir/schemas/` + `history/`.
4. `src/lib.rs` re-exporta API pública y añade docstrings que citan ADR-001/003.
5. Añadir `Cargo.toml` deps mínimas: `serde = { version, features = ["derive"] }`, `serde_json`.
6. `cargo build --workspace`, `cargo clippy -D warnings`, `cargo test --workspace` verdes.

**Problem:**
Sin tipos y schemas, AN-006 (verifier) no puede empezar: el verifier opera sobre esta ontología. Separarlos como PR independiente mantiene cada paso reviewable.

**Expected result:**
Crate `agent-native-ir` con API pública curada y schemas versionados, 0 lógica de verificación aún.

**Suggested Knowledge Level:** K3

**Impact analysis:**
- Backend: **affected** — `crates/ir/` recibe contenido.
- Documentation: **affected** — docstrings públicos.
- Configuration: **affected** — nuevas deps en `crates/ir/Cargo.toml`.
- Resto: not-applicable.

**Module coverage:**
- `crates/ir`: **affected**.
- `crates/compiler`: **reviewed-not-affected** — no se añaden deps hacia él.
- `crates/runtime`: **reviewed-not-affected**.
- `crates/cli`: **reviewed-not-affected**.

**Scope unknowns:**
- Si el verifier (AN-006) necesita un helper utility que debería vivir en `ir` (p. ej. `is_valid_identifier`), se añadirá al cerrar AN-006 — no se adelanta aquí.
- Los schemas se copian **tal cual** desde la POC; posibles diferencias de formato (BOM, line endings) se resuelven con `cargo fmt` que no toca JSON.

**Scope confidence:** high

Razones:
- 110 líneas de tipos ya validados por la POC.
- 4 schemas JSON de ~50-150 líneas cada uno.
- Sin lógica de verificación — solo definiciones.

## Acceptance criteria

1. **Tipos públicos migrados.** `agent-native-ir` expone `Diagnostic`, `IrType`, `ImportSignature`, `RecordDef`, `WitContractDef`, `Context`, `FuncSignature` con semánticas idénticas a `_reference/poc-agent-native/crates/irtool/src/lib.rs` (líneas 11-108).
2. **Métodos preservados.** `IrType::as_str` e `IrType::from_str` conservan comportamiento.
3. **Schemas migrados.** `crates/ir/schemas/ir.schema.json` es equivalente al baseline v0.5 de la POC; `history/ir.schema.v{2,3,4}.json` también presentes.
4. **Separación en módulos.** Al menos 3 módulos (`model`, `diagnostic`, `context`) + `lib.rs` que re-exporta.
5. **Dependencias correctas.** `agent-native-ir/Cargo.toml` tiene `serde` y `serde_json` únicamente; NO tiene `sha2`, `wasm-encoder`, `wasmparser` ni `wasmprinter` (pertenecen a `compiler`).
6. **Build + lint verdes.** `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo test --workspace` sin error ni warning.
7. **Docstrings públicos.** Cada tipo público tiene al menos 1 línea de doc. `lib.rs` documenta el crate con cita a ADR-003.
8. **No lógica de verificación.** `verify_ir`, `compile_ir_to_wasm` y afines **no** entran aquí (son de AN-006 y de `crates/compiler`).

## Out of scope

- Verifier (`verify_ir`) — AN-006.
- Diagnósticos localizados (`E_*` codes) — AN-006.
- 8 tests de regresión POC — AN-007.
- Backend WASM (`compile_ir_to_wasm`, `validate_wasm_bytes`, `print_wasm_to_wat`, `sha256_hex`) — `crates/compiler` en INIT-02-02 (fuera de AN-005/006/007).
- Helpers privados como `is_valid_identifier` — se añaden cuando el verifier los necesite.
- IR v0.6 (`string`, errores) — RM-07 / INIT-07-02.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release

# Verificar que agent-native-ir no arrastra deps WASM
cargo tree -p agent-native-ir --prefix none | grep -iE "wasm|wasmtime|wasmparser|wasmprinter|wit-component|sha2"
# → debe ser vacío (exit 1)

# Verificar que los 4 schemas están presentes
ls crates/ir/schemas/
# → ir.schema.json
ls crates/ir/schemas/history/
# → ir.schema.v2.json, ir.schema.v3.json, ir.schema.v4.json
```

CI remota verde en 3 OS tras push.

`kaddo verify AN-005` ejerce los 8 AC.

## Definition of Done

- 3+ módulos nuevos en `crates/ir/src/`.
- 4 schemas migrados.
- Build/lint/test verdes localmente y en CI remota.
- `cargo tree` confirma AC 5.
- WI pasa a `completed`.

**Open questions:**

- OQ-AN-005-01: ¿los schemas se mueven a `crates/ir/schemas/` o a `knowledge/tech/semantic-ir-v0.5/`? Propuesta: `crates/ir/schemas/` para co-ubicación con el código que los valida; la referencia **documental** en `knowledge/tech/semantic-ir-v0.5/` queda para N-01 de la matriz (post-M3).
- OQ-AN-005-02: ¿`IrType` queda como enum con `Record(String)` o se refactoriza a `enum IrType { ... Record { name: String } }`? Propuesta: **mantener tal cual la POC** para minimizar diferencias; cualquier refactor va a un ADR + WI explícito.

**Suggested ownership (code globs):**

- `crates/ir/src/model.rs`
- `crates/ir/src/diagnostic.rs`
- `crates/ir/src/context.rs`
- `crates/ir/schemas/**`

**Related domain / capability:**
- `related_domain`: Semantic IR & Verification.

**Related decisions:**
- ADR-001 (fronteras entre crates — `ir` sin deps de producto).
- ADR-003 (v0.5 baseline + histórico v2/v3/v4).

**Related legacy findings:** no aplica.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-02 — Semantic IR & Compiler Extraction |
| Roadmap iniciativa | INIT-02-01 — Crate `ir` — modelo + verifier v0.5 (parte 1 de 3) |
| WI candidato consolidado | WI-CANDIDATE-02-01-01 |
| Matriz de extracción | C-01 (parte modelo), S-01 (schemas) |
| Siguientes WIs del mismo INIT | AN-006 (verifier), AN-007 (regresión) |
| Prerrequisitos completados | AN-001..AN-004 (RM-01 completo) |

## Learning

**Lo que salió como se esperaba:**

- Los tipos públicos migraron 1:1 desde la POC sin cambios semánticos.
- Separar `diagnostic.rs`, `model.rs` y `context.rs` desde el inicio deja el crate ordenado para que el verifier (AN-006) tenga dónde escribirse sin ensanchar módulos existentes.
- `clippy pedantic` del workspace añadió `#[must_use]` a `as_str`/`from_str` sin fricción.
- `cargo tree -p agent-native-ir` confirma que las únicas deps externas son `serde` + `serde_json`. Las deps WASM/sha2 **no** aparecen — ADR-001/002 cumplidos.

**Lo inesperado:**

- La POC define `IrType::from_str(&self, s, known_records)` como función no-trait (nombre colisiona con el trait `std::str::FromStr`). Mantuve el nombre tal cual porque el verifier lo llama por nombre no-trait; cambiar a `parse` o similar sería un refactor fuera de alcance de AN-005.
- `FuncSignature` no deriva `PartialEq` en la POC. Mantuve así (en vez de añadirlo gratuitamente); si AN-006 lo necesita, se añade allí.
- El doc del lib.rs cita ADR-001/003 y la convención "schemas son co-ubicados con el código", lo que resuelve OQ-AN-005-01 sin un ADR adicional.

**Riesgos materializados:** ninguno. R-API-01 (congelar nombres) mitigado: los nombres son los de la POC validada, no inventos.

**Deuda y seguimientos:**

- OQ-AN-005-01: resuelta — schemas en `crates/ir/schemas/`.
- OQ-AN-005-02: resuelta — `IrType::Record(String)` tal cual la POC.
- Helpers privados (`is_valid_identifier`) viven aún en la POC; se migran en AN-006 cuando el verifier los invoque.

**Lo que esta iniciativa establece para las siguientes:**

- **AN-006** puede importar `agent_native_ir::{Context, Diagnostic, IrType, ...}` directamente para el verifier.
- **AN-007** apuntará las 8 pruebas POC a las APIs públicas aquí definidas.
- **INIT-02-02** (compiler) recibe `agent-native-ir` como su única dep del producto, confirmando ADR-001.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-005-ir-types-and-schemas.md
crates/ir/src/{lib,diagnostic,model,context}.rs
crates/ir/schemas/{ir.schema.json, history/*.json}

Next:
AN-006 (verifier v0.5 + diagnósticos localizados)
────────────────────────
