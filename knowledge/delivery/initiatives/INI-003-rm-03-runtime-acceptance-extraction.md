---
type: initiative
id: INI-003
title: RM-03 — Runtime & Acceptance Extraction
status: planned
knowledge_level: K3
domains:
  - Runtime & Memory
  - Validation & Evolution
related_capabilities:
  - Independent Runtime Execution
  - Acceptance Testing & Regression
roadmap_block: RM-03
roadmap_initiatives:
  - INIT-03-01
  - INIT-03-03
  - INIT-03-04
  - INIT-03-05
  - INIT-03-06
  - INIT-03-02
created_at: '2026-10-17'
external_links:
  - knowledge/delivery/roadmap.md
candidates:
  - id: INIT-03-01
    title: Crate `runtime` — carga + ejecución Wasmtime
    status: completed
    materialized_as: AN-009
  - id: INIT-03-03
    title: Executor de aceptación YAML
    status: candidate
  - id: INIT-03-04
    title: Propagación de errores host + imports
    status: completed
    materialized_as: AN-010
  - id: INIT-03-05
    title: Controles mínimos de ejecución (deny-by-default)
    status: completed
    materialized_as: AN-011
  - id: INIT-03-06
    title: Línea base técnica del runtime
    status: completed
    materialized_as: AN-012
  - id: INIT-03-02
    title: Resolver de dependencias reutilizable
    status: completed
    materialized_as: AN-013
---

# RM-03 — Runtime & Acceptance Extraction

## Goal

Runtime autónomo capaz de cargar, componer y ejecutar componentes WASM del producto, con pruebas de regresión de la POC, controles mínimos de seguridad **desde el inicio** y una línea base técnica antes de cualquier optimización. Debe aceptar componentes WIT-compatibles independientes de su origen.

## Expected Value

Separa el runtime del compilador y del LLM (ADR-002). Habilita RM-04 (pipeline) y RM-05 (CLI distribuible). Es la mitad "ejecución" del producto.

## Scope

- API Rust de carga + invocación tipada WIT.
- Executor YAML con host-mocks por capability.
- Propagación tipada de errores de imports/capabilities (fail-explicit).
- Deny-by-default de capabilities del host, con allow-list en manifiesto.
- Micro-benchmarks de cold start, warm invocation y memoria por instancia **antes** de caché.
- Resolver reutilizable de dependencias (reemplaza compilación por invocación de la POC).

## Out of Scope

- Policy loader completo, fuel, instance limits (RM-06).
- Caché de componentes y warm pool (RM-08, depende de la línea base de INIT-03-06).
- WASI preview2 y stdio (RM-07).
- Verificación de firma (RM-06 / INIT-06-04).

## Success Criteria

1. Las 10 pruebas `runner` de la POC se reproducen verdes en el nuevo repo.
2. Imports no autorizados y capabilities faltantes producen `RuntimeError` tipado — nunca `0`/`false` por defecto.
3. Deny-by-default verificado: capability no listada en el manifiesto falla al cargar.
4. Línea base técnica publicada en `knowledge/tech/baselines/` con cold start, warm invocation y memoria por instancia.
5. Lint CI verifica que `runtime` no depende de `compiler` ni del LLM.

## Dependencies

- INIT-02-02 (compiler) e INIT-02-05 (modelo de memoria) completados.

## Work Item Candidates

Ver `candidates:`. Orden sugerido: 03-01 → 03-04 → 03-05 → 03-03 → 03-06 → 03-02.

## Open Questions

- OQ L146 (caché y ciclo de vida sin fuga) — se trata en RM-06.
- OQ L147 (resolver reutilizable) — resuelta por INIT-03-02 con la línea base de INIT-03-06 como entrada.
- OQ L152 (capabilities WASI del primer release) — se trata en RM-07.

## Learning

_Pendiente._
