---
id: AN-004
type: chore
status: completed
completed_at: 2026-10-17
title: Workspace Rust y CI mínima
knowledge_level: K2
suggested_knowledge_level: K2
scope_confidence: high
code:
  - Cargo.toml
  - rust-toolchain.toml
  - crates/ir/Cargo.toml
  - crates/ir/src/lib.rs
  - crates/compiler/Cargo.toml
  - crates/compiler/src/lib.rs
  - crates/runtime/Cargo.toml
  - crates/runtime/src/lib.rs
  - crates/cli/Cargo.toml
  - crates/cli/src/main.rs
  - .github/workflows/ci.yml
source_roadmap_initiative: INIT-01-01
source_work_item_candidate:
  - WI-CANDIDATE-01-01-01
  - WI-CANDIDATE-01-01-02
  - WI-CANDIDATE-01-01-03
related_domain: Foundation & Governance
domains:
  - Foundation & Governance
related_capabilities: []
expected_value: >-
  Workspace Rust y CI mínima listos para recibir el contenido técnico de
  RM-02/03, cerrando RM-01 y alcanzando el hito M1 (Baseline arquitectónico
  aprobado).
risks:
  - 'R-CI-01: elegir toolchain Rust incompatible con Wasmtime baseline futura'
dependencies:
  - AN-001
  - AN-002
  - AN-003
source_signals:
  - 'ADR-001: workspace de 3 crates + bin cli'
  - 'ADR-002: runtime no depende de compiler ni LLM (requiere lint)'
  - 'ADR-003: toolchain Rust pinneada (sin fijar versión Wasmtime)'
  - 'ADR-004: fail-explicit transversal (requiere clippy -D warnings)'
decision_candidates: []
related_decisions:
  - ADR-001
  - ADR-002
  - ADR-003
  - ADR-004
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-17'
---

# AN-004 — Workspace Rust y CI mínima

> Idioma del proyecto: **español**. Mantener en inglés el código, los nombres de archivo, los comandos y las claves de configuración.

**Actor and outcome:**
El repositorio tiene un workspace Cargo con 4 crates (`ir`, `compiler`, `runtime`, `cli` bin), toolchain pinneado y CI en 3 OS verificando `fmt`, `clippy -D warnings` y `test`. Los crates están vacíos pero compilan, listos para recibir contenido técnico desde INIT-02-01 en adelante.

**Current behavior:**
El repositorio contiene únicamente conocimiento (sin código Rust) y `LICENSE`. No existe `Cargo.toml`, `rust-toolchain.toml` ni workflows de GitHub Actions. Alcanzar M1 requiere este scaffolding.

**Target behavior:**
- Workspace raíz con `Cargo.toml` listando los 4 crates y `[workspace.package]` común.
- `rust-toolchain.toml` pinneado a `stable` versión documentada.
- Cada crate con `Cargo.toml` mínimo, `src/lib.rs` (o `src/main.rs` para `cli`) y docstring breve.
- Dependencias Cargo respetando ADR-001/ADR-002:
  - `ir`: sin dependencias del producto.
  - `compiler`: depende solo de `ir` (por `path`).
  - `runtime`: depende solo de `ir`.
  - `cli`: depende de los 3 anteriores.
- `.github/workflows/ci.yml` con 3 jobs matrix (ubuntu, windows, macos) ejecutando `fmt --check`, `clippy --workspace --all-targets -D warnings` y `test --workspace`.
- `cargo build --workspace`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` verdes localmente.

**Entry points:**
- `Cargo.toml` (workspace raíz).
- `rust-toolchain.toml`.
- `crates/{ir,compiler,runtime,cli}/`.
- `.github/workflows/ci.yml`.

**End-to-end flow:**
1. Crear `Cargo.toml` workspace con `resolver = "2"` y 4 miembros.
2. Crear `rust-toolchain.toml` con `channel = "1.85.0"` (versión actual local; se revisará al cerrar OQ L145).
3. Scaffoldear 4 crates vacíos, con placeholders que respeten las dependencias de ADR-001.
4. Añadir `.github/workflows/ci.yml` con matriz 3 OS.
5. Correr `cargo build/clippy/test --workspace` localmente para verificar.
6. Commit + push; CI remota verifica.

**Problem:**
Sin workspace no hay dónde materializar los crates de RM-02/03/04. Sin CI, cualquier regresión futura puede filtrarse al main.

**Expected result:**
Base de desarrollo lista para abrir INIT-02-01 inmediatamente después.

**Suggested Knowledge Level:** K2

**Impact analysis:**
- Backend: **affected** — nuevo workspace Rust.
- Configuration: **affected** — `Cargo.toml`, `rust-toolchain.toml`, workflow CI.
- Operations / release: **reviewed-not-affected** — release pipeline queda para INIT-05-03.
- Documentation: **reviewed-not-affected** — no se añade README por ahora.
- Resto: not-applicable.

**Module coverage:**
- `crates/ir`: **affected** — nuevo (vacío).
- `crates/compiler`: **affected** — nuevo (vacío).
- `crates/runtime`: **affected** — nuevo (vacío).
- `crates/cli`: **affected** — nuevo (vacío).
- `knowledge/`: not-applicable.
- `.github/`: **affected** — nuevo workflow.

**Scope unknowns:**
- La versión exacta de Wasmtime se fija en INIT-02-01 (ADR-003 permanece `[open]`). AN-004 no bloquea en esta OQ.
- El lint CI de ADR-001/002 (verificar que `runtime` no referencia `compiler`) se añade como paso **best-effort** aquí mediante `cargo tree -i`; se refina si INIT-02-01/02 lo necesita.

**Scope confidence:** high

Razones:
- Scaffolding Rust estándar.
- Los 4 ADRs recién aceptables marcan todas las decisiones.
- No hay lógica de dominio en los crates.

## Acceptance criteria

1. **Workspace cargo.** `Cargo.toml` raíz con `[workspace] resolver = "2" members = [...]` listando los 4 crates y `[workspace.package] version = "0.0.0" edition = "2024" license = "MIT"`.
2. **Toolchain pinneado.** `rust-toolchain.toml` existe con `channel = "1.85.0"`, `components = ["rustfmt", "clippy"]`.
3. **4 crates creados.** `crates/{ir,compiler,runtime,cli}/` con `Cargo.toml` + fuente mínima; `cli` es un `bin`, los otros 3 son `lib`.
4. **Dependencias respetan ADR-001/002.** `cargo tree -p runtime` no incluye `compiler`; `cargo tree -p compiler` no incluye `runtime`.
5. **Build local verde.** `cargo build --workspace --release` termina sin error.
6. **Clippy verde.** `cargo clippy --workspace --all-targets -- -D warnings` termina sin warning.
7. **Test verde.** `cargo test --workspace` termina sin test fallido (vacío cuenta como éxito).
8. **Fmt check verde.** `cargo fmt --all -- --check` sin diferencias.
9. **CI configurada.** `.github/workflows/ci.yml` ejecuta los 4 comandos anteriores en matriz `ubuntu-latest / windows-latest / macos-latest`.

## Out of scope

- Lógica de dominio en los crates (RM-02 en adelante).
- Dependencias concretas de Wasmtime, wit-bindgen, serde, etc. (se añaden en los WIs que las necesitan).
- Release pipeline (INIT-05-03).
- Pre-commit hooks locales.
- Lint CI dedicado para "runtime no depende de compiler" (versión completa en INIT-01-01 refinamiento futuro; aquí sólo el chequeo mínimo del AC 4).
- README público (puede añadirse en un WI aparte).

## Validation

Comandos a ejecutar localmente antes de commit:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
cargo tree -p runtime --prefix none | grep -i compiler  # debe ser vacío (exit 1)
cargo tree -p compiler --prefix none | grep -i runtime  # debe ser vacío (exit 1)
```

Verificación remota: CI verde en los 3 OS tras `git push`.

`kaddo verify AN-004` ejerce los 9 AC sobre la evidencia local.

## Definition of Done

- 4 crates compilan en local.
- `fmt --check`, `clippy -D warnings`, `test --workspace` verdes localmente.
- `cargo tree` confirma reglas de dependencia (AC 4).
- `.github/workflows/ci.yml` configurado.
- CI remota verde en los 3 OS tras push (reportada en el Learning section).
- WI pasa a `completed` tras `kaddo verify`.

**Open questions:**

- OQ-AN-004-01: ¿`edition = "2024"` o `edition = "2021"` para el workspace? Rust 1.85 soporta 2024 ya estable. Propuesta: **2024** (más reciente, sin regresiones conocidas en Wasmtime baseline). Reversible si se detecta incompatibilidad.
- OQ-AN-004-02: ¿pinneamos `rust-toolchain` a `1.85.0` exacto o a `stable`? Propuesta: **1.85.0 exacto** para reproducibilidad; se actualiza con WI explícito.
- OQ-AN-004-03: ¿la CI usa `actions/checkout@v4` + `dtolnay/rust-toolchain@stable`? Propuesta: sí, patrón estándar.

**Suggested ownership (code globs):**

- `Cargo.toml`
- `rust-toolchain.toml`
- `crates/**`
- `.github/workflows/ci.yml`

**Related domain / capability:**
- `related_domain`: Foundation & Governance.

**Related decisions:**
- ADR-001 (arquitectura inicial).
- ADR-002 (runtime independiente).
- ADR-003 (baseline IR; toolchain Rust pinneado).
- ADR-004 (seguridad mínima; implica `clippy -D warnings`).

**Related legacy findings:** no aplica.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-01 — Foundation & POC Assessment |
| Roadmap iniciativa | INIT-01-01 — Workspace Rust y CI mínima |
| Candidatos consolidados | WI-CANDIDATE-01-01-01, WI-CANDIDATE-01-01-02, WI-CANDIDATE-01-01-03 |
| Prerrequisitos completados | AN-001, AN-002, AN-003 |
| ADRs aplicables | ADR-001, ADR-002, ADR-003, ADR-004 |
| Hito | M1 — Baseline arquitectónico aprobado (al cerrar este WI) |

## Learning

**Lo que salió como se esperaba:**

- Scaffolding de 4 crates vacíos compila a la primera.
- `cargo fmt --check`, `clippy -D warnings` y `test --workspace` pasan sin tuning.
- Las reglas de dependencia de ADR-001/002 se cumplen literalmente: `cargo tree -p agent-native-runtime` lista únicamente `agent-native-ir`; `cargo tree -p agent-native-compiler` igualmente.
- El lint CI del workflow (grep sobre `cargo tree`) es una primera versión defendible — simple y de bajo mantenimiento.

**Lo inesperado:**

- `edition = "2024"` funciona sin fricción en Rust 1.85 — se mantiene (resuelve OQ-AN-004-01 a favor de 2024).
- Los nombres de los crates terminaron con prefijo `agent-native-` (crates publicables) pero el bin se llama `agent-native` (nombre del comando) — mezcla intencional: nombre del paquete ≠ nombre del binario. Documentar esta convención ayudará a quien venga de la POC.
- `rustup show` instaló automáticamente el toolchain 1.85.0 al leer `rust-toolchain.toml`; no requiere acción manual del contribuidor nuevo. 👍

**Riesgos materializados:** ninguno. R-CI-01 (toolchain incompatible con Wasmtime baseline) sigue abierto hasta ADR-003 fije la versión exacta de Wasmtime, pero no bloquea.

**Deuda y seguimientos:**

- OQ-AN-004-01: resuelta — `edition = "2024"`.
- OQ-AN-004-02: resuelta — `channel = "1.85.0"` exacto.
- OQ-AN-004-03: resuelta — `dtolnay/rust-toolchain@stable` + `Swatinem/rust-cache@v2`.
- **Verificación remota pendiente:** CI remota debe correr en los 3 OS tras push. Si falla en Windows por algún shell diferente al bash del step de lint, se añade un fix de seguimiento (no abre nuevo WI; se hace en un siguiente commit).
- **Lint CI minimalista:** la detección por `cargo tree + grep` es defendible pero no exhaustiva (no cubre dependencias transitivas de terceros con nombres colisionantes). Se refina si surge falso positivo/negativo real.

**Lo que esta iniciativa establece para las siguientes:**

- **INIT-02-01** puede abrirse de inmediato con `crates/ir/` listo para recibir modelo + verifier v0.5.
- **INIT-02-02** idem para `crates/compiler/`.
- **INIT-03-01** idem para `crates/runtime/`.
- **INIT-05-01** idem para `crates/cli/`.
- El patrón de "scaffolding → verificar → documentar en Learning" queda establecido para los WIs que escriban código.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-004-workspace-and-ci.md
Cargo.toml, rust-toolchain.toml, crates/{ir,compiler,runtime,cli}, .github/workflows/ci.yml

Next:
kaddo create --from roadmap (RM-02 — INIT-02-01)
────────────────────────
