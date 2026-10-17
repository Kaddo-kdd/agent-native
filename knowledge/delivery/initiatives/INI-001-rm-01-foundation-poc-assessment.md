---
type: initiative
id: INI-001
title: RM-01 — Foundation & POC Assessment
status: in-progress
knowledge_level: K3
domains:
  - Foundation & Governance
related_capabilities: []
roadmap_block: RM-01
roadmap_initiatives:
  - INIT-01-04
  - INIT-01-02
  - INIT-01-03
  - INIT-01-01
created_at: '2026-10-17'
external_links:
  - knowledge/delivery/roadmap.md
candidates:
  - id: INIT-01-04
    title: Gobierno de _reference/ y .gitignore
    status: completed
    materialized_as: AN-001
  - id: INIT-01-02
    title: Matriz de extracción POC → productos
    status: candidate
  - id: INIT-01-03
    title: ADRs bloqueantes (001..005)
    status: candidate
  - id: INIT-01-01
    title: Workspace Rust y CI mínima
    status: candidate
---

# RM-01 — Foundation & POC Assessment

## Goal

Convertir el repositorio vacío (hoy solo `LICENSE`) en una base verificable: POC excluida del build, matriz de extracción aprobada, decisiones arquitectónicas bloqueantes materializadas y workspace Rust con CI mínima construido **sobre decisiones aprobadas**.

## Expected Value

Sin esta base, cualquier incremento posterior se apoya en supuestos no revisados. Esta iniciativa es prerrequisito explícito de RM-02 (IR + compiler) y de todas las olas siguientes.

## Scope

- Exclusión y gobierno de `_reference/poc-agent-native/`.
- Catalogación de capacidades y pruebas de la POC con decisión por fila (migrate / redesign / drop).
- Materialización de los 5 ADRs bloqueantes (001..005).
- Workspace Cargo + CI básica (`fmt`, `clippy`, `test`) en Linux, Windows y macOS.

## Out of Scope

- Decisiones diferidas (publicación, empaquetado, firma, resolver, caché, concurrencia) — van a sus bloques respectivos.
- Importar código ejecutable desde `_reference/`.
- Release pipeline (INIT-05-03).

## Success Criteria

1. `_reference/` no entra al índice de Git y está documentada como solo lectura. ✅ (AN-001)
2. Existe matriz de extracción con decisión por fila y responsable candidato.
3. Los 5 ADRs bloqueantes están firmados y referenciados por los crates/módulos afectados.
4. Workspace compila en 3 OS y CI pasa `fmt/clippy/test` verdes.

## Dependencies

Ninguna aguas arriba. Es la primera ola del roadmap.

## Work Item Candidates

Ver `candidates:` en frontmatter. Orden de materialización:

1. **INIT-01-04** → **AN-001** (completed, 2026-10-17).
2. **INIT-01-02** — pendiente. Candidatos: `spike: catálogo de pruebas POC`, `chore: redactar matriz final`.
3. **INIT-01-03** — pendiente. Candidatos: un `chore` por ADR (ADR-001..005).
4. **INIT-01-01** — pendiente. Candidatos: `chore: init cargo workspace`, `chore: GitHub Actions`, `chore: rust-toolchain.toml`.

## Open Questions

- Ninguna bloqueante. Las OQs técnicas asociadas (L143..L154 en `codebase.md`) son `[open]`/`[deferred]`/`[assumed]` y no detienen RM-01.

## Learning

_Pendiente — se captura al cerrar la iniciativa._
