---
type: initiative
id: INI-004
title: 'RM-04 — Specification, Agents & Build Pipeline'
status: planned
knowledge_level: K4
domains:
  - Specification & Knowledge Ingestion
  - Validation & Evolution
related_capabilities:
  - Specification Interpretation
  - LLM-driven Implementation
  - Versioned Publication & Evolution
roadmap_block: RM-04
roadmap_initiatives:
  - INIT-04-01
  - INIT-04-02
  - INIT-04-03
  - INIT-04-05
  - INIT-04-04
created_at: '2026-10-17'
external_links:
  - knowledge/delivery/roadmap.md
candidates:
  - id: INIT-04-01
    title: Intérprete narrativa Markdown → SystemSpec
    status: completed
    materialized_as: AN-023
  - id: INIT-04-02
    title: Adaptador LLM pluggable y modos de generación
    status: completed
    materialized_as: AN-024
  - id: INIT-04-03
    title: Evolución incremental de especificaciones
    status: completed
    materialized_as: AN-025
  - id: INIT-04-05
    title: 'Publicación atómica, versionamiento e integridad'
    status: completed
    materialized_as: AN-026
  - id: INIT-04-04
    title: Trazabilidad artefacto ↔ Work Item / Spec
    status: candidate
---

# RM-04 — Specification, Agents & Build Pipeline

## Goal

Cerrar el flujo "requisito → componente publicado": narrativa Markdown → `SystemSpec` → IR (vía LLM) → WASM verificado → publicación atómica versionada. La publicación vive en la capa de orquestación, **no** en el crate `compiler`.

## Expected Value

Convierte las piezas técnicas (RM-02/RM-03) en producto utilizable: un desarrollador escribe narrativa, obtiene un componente publicado y una suite de aceptación aprobada. Habilita RM-05 (distribución).

## Scope

- Parser de narrativa Markdown + `SystemSpec` serializable.
- Trait `LlmProvider` con tres modos explícitos: **real**, **fixtures**, **synthesizer** — con documentación clara de qué garantiza cada uno.
- Evolución incremental (v_n → v_{n+1}) con detección de regresión antes de publicar.
- Store de artefactos filesystem, manifiesto versionado, puntero activo, rollback atómico.
- Metadatos `source_wi` / `source_spec_hash` en el manifiesto (sin dependencia obligatoria hacia Kaddo).

## Out of Scope

- OCI/registry remoto (diferido).
- Firma criptográfica (RM-05/RM-06).
- Crate `orchestrator` separado — se evalúa aquí según ADR-001; implementación inicial queda dentro de la capa de orquestación sin crear un crate adicional sin justificación.

## Success Criteria

1. Las 9 pruebas `orchestrator` de la POC se reproducen verdes.
2. El escenario esclusas v1→v2 (5+5 pruebas) pasa íntegro: reconciliación, publicación, rollback.
3. Modo `real`, `fixtures` y `synthesizer` tienen documentación de garantías; `fixtures` y `synthesizer` **no** se presentan como la capacidad generativa.
4. Lint CI verifica que `compiler` no referencia store de artefactos.
5. Manifiesto incluye hash SHA-256 y origen WI/Spec verificables sin ejecutar Kaddo.

## Dependencies

- INIT-02-01 (IR) e INIT-02-04 (hash determinista) completados.
- INIT-04-02 → INIT-04-03 → INIT-04-05 secuencialmente.

## Work Item Candidates

Ver `candidates:`.

## Open Questions

- OQ L143 (API pública `SystemSpec`) — se cierra al materializar INIT-04-01.
- Riesgo R-12 (confundir offline con generación real) — mitigado por INIT-04-02.

## Learning

_Pendiente._
