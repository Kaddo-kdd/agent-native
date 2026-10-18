---
type: initiative
id: INI-002
title: RM-02 — Semantic IR & Compiler Extraction
status: planned
knowledge_level: K3
domains:
  - Semantic IR & Verification
  - Compilation
related_capabilities:
  - Semantic IR Verification
  - Component Compilation
roadmap_block: RM-02
roadmap_initiatives:
  - INIT-02-01
  - INIT-02-02
  - INIT-02-03
  - INIT-02-04
  - INIT-02-05
created_at: '2026-10-17'
external_links:
  - knowledge/delivery/roadmap.md
candidates:
  - id: INIT-02-01
    title: Crate `ir` — modelo + verifier v0.5
    status: completed
    materialized_as: AN-005, AN-006, AN-007
  - id: INIT-02-02
    title: Crate `compiler` — IR → WASM Component
    status: completed
    materialized_as: AN-008, AN-014 (E2E v0.5 Component Model)
  - id: INIT-02-03
    title: Contratos WIT públicos vs internos
    status: candidate
  - id: INIT-02-04
    title: Determinismo + verificación de integridad (SHA-256)
    status: candidate
  - id: INIT-02-05
    title: Modelo de memoria y Canonical ABI
    status: candidate
---

# RM-02 — Semantic IR & Compiler Extraction

## Goal

Núcleo técnico "requisito → artefacto": extraer de la POC un verificador de Semantic IR v0.5 estable y un compilador determinista a WASM Component Model, **sin** mezclar en el `compiler` responsabilidades de publicación o de narrativa.

## Expected Value

Sin IR verificable y compilación determinista no hay producto. Habilita RM-03 (runtime) y RM-04 (pipeline completo).

## Scope

- API estable de Semantic IR v0.5 (modelo, parser, verifier, diagnósticos localizados).
- Backend WASM Component Model + bindings WIT.
- Reorganización de `wit/` en `public/`, `internal/`, `fixtures/`.
- Determinismo cross-OS y hash SHA-256 por artefacto.
- Modelo de memoria y Canonical ABI (ADR-005).

## Out of Scope

- Publicación atómica y evolución v1→v2 (viven en RM-04 / INIT-04-05).
- Agente LLM (INIT-04-02).
- Extensiones IR `string`, streams, efectos (RM-07 / INIT-07-02).
- Firma criptográfica (RM-05/RM-06).

## Success Criteria

1. Las 8 pruebas `irtool` de la POC se reproducen verdes en el crate `ir` del repo nuevo.
2. `compile(ir)` produce el mismo hash SHA-256 en Linux, Windows y macOS para el mismo input.
3. ADR-005 (modelo de memoria) cerrado con experimentos documentados.
4. `wit/` reorganizado; fixture `kaddo:canal` fuera de `public/`.
5. Lint CI verifica que `compiler` no referencia store ni runtime.

## Dependencies

- RM-01 completo (workspace + ADRs bloqueantes).
- ADR-001 (arquitectura) y ADR-003 (modelo IR) aprobados antes de INIT-02-01.

## Work Item Candidates

Ver `candidates:` en frontmatter. Pueden materializarse en paralelo tras cerrar RM-01.

## Open Questions

- OQ L143 (API pública `SystemSpec`/IR) — se resolverá al materializar INIT-02-01.
- OQ L144 (evolución IR) — tratada en RM-07, no bloquea RM-02.
- OQ L145 (baseline Wasmtime) — ADR-003 permanece abierta hasta validar.

## Learning

_Pendiente._
