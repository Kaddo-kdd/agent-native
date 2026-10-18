---
id: AN-018
type: chore
status: completed
completed_at: 2026-10-18
title: Convención de WIT (public / internal / fixtures) sin reorganización física
knowledge_level: K2
suggested_knowledge_level: K2
scope_confidence: high
code:
  - wit/README.md
source_roadmap_initiative: INIT-02-03
source_work_item_candidate:
  - WI-CANDIDATE-02-03-01
  - WI-CANDIDATE-02-03-02
related_domain: Compilation
domains:
  - Compilation
related_capabilities:
  - Component Compilation
expected_value: >-
  Establece la convención `wit/public/`, `wit/internal/`, `wit/fixtures/` a
  nivel workspace mediante un `wit/README.md` canónico, sin mover archivos
  todavía. Cierra INI-002 (RM-02) con convención documentada mientras la
  reorganización física queda diferida hasta que RM-05 introduzca contratos
  públicos del producto.
risks:
  - >-
    R-WIT-01: dejar la convención como documento sin lint CI puede llevar a
    drift. Mitigado: el documento declara explícitamente que la primera vez que
    algo entre a wit/public/ se añade un lint CI dedicado.
dependencies:
  - AN-014
  - AN-016
source_signals:
  - >-
    Roadmap INIT-02-03: separar fixtures (kaddo:canal) de contratos públicos del
    producto para evitar que fixtures de prueba se vuelvan contratos de dominio
  - 'Technical Review: fixtures POC como contratos públicos (R-08)'
  - >-
    AN-014 Learning: la ubicación de las fixtures WIT no afecta el output del
    compilador si CompileContext apunta al root correcto
  - >-
    Realidad actual: no existen contratos públicos del producto todavía — RM-05
    (CLI) los introducirá vía subcomandos; RM-07 vía WASI stdio
decision_candidates: []
related_decisions:
  - ADR-003
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-18T00:00:00.000Z
ready_at: '2026-10-18'
---

# AN-018 — Convención de WIT (public / internal / fixtures)

**Actor y resultado:**
Un contribuidor que crea un archivo `.wit` nuevo lee `wit/README.md` y sabe dónde colocarlo: `wit/public/` para interfaces estables del producto, `wit/internal/` para WIT embebido en crates (como `crates/compiler/wit/order.wit`, fallback v0.4), `wit/fixtures/` para fixtures de prueba. La reorganización física queda diferida hasta que RM-05 cree el primer contrato público.

**Current behavior:**
- No existe `wit/` a nivel workspace.
- `crates/compiler/wit/order.wit`: recurso embebido vía `include_str!` para fallback IR v0.4. Correcto en su ubicación — es contenido del crate, no del producto.
- `crates/compiler/tests/fixtures/wit/{nested-records,probe-use-types,program-005}.wit`: fixtures de test co-ubicadas con los IR fixtures bajo `crates/compiler/tests/fixtures/`. Correcto en su ubicación — son fixtures del crate de tests.
- Sin convención declarada → riesgo R-08 (fixtures como contratos públicos implícitos).

**Target behavior:**
- `wit/README.md` nuevo con:
  - Las 3 categorías (`public/`, `internal/`, `fixtures/`) y qué entra en cada una.
  - Reglas para añadir un `.wit` nuevo (dónde ponerlo según su naturaleza).
  - Nota explícita de que la convención está en vigor **aunque algunas categorías estén vacías hoy**.
  - Momento en que se activará un lint CI dedicado: cuando `wit/public/` reciba su primer archivo (típicamente al materializar INIT-05-01 o INIT-07-01).
  - Enlace a ADR-003 (Semantic IR v0.5 baseline) y la nota de interoperabilidad del roadmap.
- No se mueve ningún archivo existente.
- No se crea aún lint CI (R-WIT-01 declarado explícitamente como seguimiento).

**Problem:**
INIT-02-03 queda abierto bloqueando el cierre de INI-002 por una convención que puede documentarse sin trabajo invasivo. Postergar la reorganización física es coherente con el estado del producto (no hay contratos públicos aún).

**Scope confidence:** high. Es un chore documental de 1 archivo.

## Acceptance criteria

1. **`wit/README.md` existe** y declara las 3 categorías con reglas de uso.
2. **Zero cambios en código** de crates o tests.
3. **Fixtures existentes documentadas** en el README como `wit/fixtures/` conceptualmente (aunque viven bajo `crates/compiler/tests/fixtures/wit/`).
4. **Momento del lint CI** declarado: se añade cuando `wit/public/` reciba su primer archivo real.
5. **INI-002 cerrado** (status del archivo `completed`) tras marcar INIT-02-03 como `completed, materialized_as: AN-018`.
6. **Tests existentes (43) siguen verdes.** CI remota verde 3 OS.
7. **Lint y fmt verdes.** `cargo fmt`, `clippy pedantic -D warnings`.

## Out of scope

- Mover físicamente archivos `.wit` existentes a `wit/fixtures/`.
- Crear contratos `wit/public/` nuevos.
- Implementar el lint CI (diferido hasta que haya `wit/public/`).
- Modificar `crates/compiler/wit/order.wit` (es recurso embebido del crate, no un fixture).

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                           # 43 tests inalterados
ls wit/README.md                                  # existe
```

## Definition of Done

- `wit/README.md` creado con las 3 categorías + reglas + momento del lint.
- INI-002 marcado `completed` tras actualizar INIT-02-03.
- CI remota verde.
- WI pasa a `completed` tras `kaddo verify`.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-02 |
| Roadmap iniciativa | INIT-02-03 — Contratos WIT públicos vs internos |
| ADRs | ADR-003 |
| Prerrequisitos | AN-014, AN-016 (fixtures ya establecidas bajo `crates/compiler/tests/`) |

## Learning

**Lo que salió como se esperaba:**

- Convención declarada en 1 archivo (`wit/README.md`), 0 cambios en código, 43 tests inalterados.
- INI-002 RM-02 cerrado 5/5 — primera iniciativa de RM-02 → RM-08 completamente cerrada.

**Lo inesperado:** ninguno. El scope minimal fue correcto; no apareció necesidad de mover archivos.

**Lo que esta iniciativa establece:**

- **RM-05 / INIT-05-01**: cuando introduzca el primer contrato público del CLI, debe (a) colocarlo en `wit/public/`, (b) añadir ADR, (c) activar el lint CI documentado en §4 del README.
- **RM-07 / INIT-07-01**: idem para capacidades WASI (típicamente `wasi:cli/stdio`).
- La convención queda estable; el próximo WI que la toque es el primero en poblar `wit/public/`.

### Evidencias

- `cargo test --workspace` → 43 pruebas verdes.
- `cargo clippy pedantic -D warnings` → OK.
- `cargo fmt --check` → OK.
- `kaddo verify AN-018 --json`: `"readiness": "READY_TO_COMPLETE"`, 0 blockers, 0 warnings.
- `kaddo initiative list`: `INI-002 completed 5/5`.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-018-wit-fixtures-convention.md
wit/README.md (nuevo)
knowledge/delivery/initiatives/INI-002-...md (INIT-02-03 completed, INI-002 completed)

Next: AN-019 (INIT-03-03 executor YAML → M3).
────────────────────────
