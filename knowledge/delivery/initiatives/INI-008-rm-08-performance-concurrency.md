---
type: initiative
id: INI-008
title: RM-08 — Performance & Concurrency
status: planned
knowledge_level: K4
domains:
  - Performance
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
roadmap_block: RM-08
roadmap_initiatives:
  - INIT-08-01
  - INIT-08-02
  - INIT-08-03
  - INIT-08-04
created_at: '2026-10-17'
external_links:
  - knowledge/delivery/roadmap.md
candidates:
  - id: INIT-08-01
    title: Harness de benchmarks reproducible
    status: candidate
  - id: INIT-08-02
    title: Línea base pública cross-OS
    status: candidate
  - id: INIT-08-03
    title: Caché de componentes y warm pool
    status: candidate
  - id: INIT-08-04
    title: Concurrencia host-side (workers)
    status: candidate
---

# RM-08 — Performance & Concurrency

## Goal

Caracterizar el runtime con benchmarks reproducibles cross-OS, publicar una línea base objetiva, y optimizar (caché, warm pool, workers) **sobre** mediciones previas — nunca a ciegas.

## Expected Value

Materializa el business goal "decisiones basadas en evidencia" y permite publicar afirmaciones de rendimiento defendibles.

## Scope

- Harness `criterion` con hardware baseline documentado y varianza acotada.
- Línea base pública (cold start, warm invocation, memoria por instancia, throughput, p50/p95/p99) en `docs/benchmarks/`.
- Module cache y warm pool con wipe entre tenants.
- Worker pool host-side y soak test.

## Out of Scope

- Primitivas asíncronas dentro del guest (Not Now).
- Comparaciones directas con Rust/Go/TypeScript nativos — se documentan solo si la metodología lo permite.
- Afirmaciones de superioridad frente a otros runtimes.

## Success Criteria

1. Dos corridas consecutivas del harness producen varianza < Y% (Y documentado).
2. Línea base publicada con metodología, hardware y resultados por operación/OS.
3. Caché mejora warm start en Z% medible vs línea base (Z fijado por INIT-08-02).
4. Throughput escala con N workers hasta saturar; sin estado compartido entre tenants.
5. Soak test de M horas sin degradación perceptible ni fuga de memoria.

## Dependencies

- INIT-03-06 (línea base técnica del runtime) completada — es la entrada mínima.
- INIT-05-01 (CLI) completado para orquestar las corridas.
- INIT-06-03 (aislamiento multi-tenant) para INIT-08-03.

## Work Item Candidates

Ver `candidates:`.

## Open Questions

- OQ L153 (umbrales de rendimiento) — se cierra al publicar INIT-08-02; los números concretos salen de la medición, no de supuestos.
- Riesgo R-10 (afirmaciones de rendimiento sin medición) — mitigado por este bloque entero.

## Learning

_Pendiente._
