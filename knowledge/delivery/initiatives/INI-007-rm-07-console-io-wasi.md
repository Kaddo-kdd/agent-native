---
type: initiative
id: INI-007
title: RM-07 — Console I/O & WASI
status: planned
knowledge_level: K4
domains:
  - Runtime & Memory
  - Semantic IR & Verification
related_capabilities:
  - Host Capability Policy
  - Independent Runtime Execution
roadmap_block: RM-07
roadmap_initiatives:
  - INIT-07-01
  - INIT-07-02
  - INIT-07-03
created_at: '2026-10-17'
external_links:
  - knowledge/delivery/roadmap.md
candidates:
  - id: INIT-07-01
    title: Baseline WASI preview2 + stdio autorizado
    status: candidate
  - id: INIT-07-02
    title: Extensión IR para `string` y errores
    status: candidate
  - id: INIT-07-03
    title: Stubs deterministas de I/O para pruebas
    status: candidate
---

# RM-07 — Console I/O & WASI

## Goal

Habilitar programas con entrada/salida real (consola → WASI preview2), con capabilities autorizadas y pruebas deterministas. Extender Semantic IR a v0.6 con `string` y errores, bajo flag `--experimental`.

## Expected Value

Convierte el producto de "núcleo técnico" a "ejecuta programas interactivos reales": CLIs útiles generadas por Agent-Native.

## Scope

- Adaptador WASI preview2, capability `wasi:cli/stdio`.
- IR v0.6: tipo `string`, variantes de error, política de versionamiento IR.
- Mocks de `stdin`/`stdout`/`stderr` en el executor YAML para pruebas sin TTY.

## Out of Scope

- Primitivas asíncronas en IR (Not Now).
- Filesystem/HTTP/KV/mensajería como capabilities (futuro, por bloque).
- Promover v0.6 a estable antes de equivalencia funcional del MVP (principio del roadmap).

## Success Criteria

1. Programa `echo`/`cat` generado por Agent-Native ejecuta bajo policy con stdio autorizado.
2. IR v0.6 con `string` pasa verifier + compilador + runtime bajo `--experimental`.
3. Suites YAML con `stdin:` y `expected_stdout:` son deterministas y no requieren TTY real.
4. v0.5 sigue siendo el camino estable y por defecto.

## Dependencies

- INIT-06-01 (policy loader) completado — stdio es una capability.
- INIT-02-01 (IR v0.5) e INIT-04-05 (publicación) para INIT-07-02.

## Work Item Candidates

Ver `candidates:`.

## Open Questions

- OQ L144 (evolución IR: streams, efectos, errores) — INIT-07-02 cubre `string` y errores; streams/efectos quedan diferidos.
- OQ L152 (capabilities WASI del primer release) — resuelta al materializar INIT-07-01.
- Riesgo R-05 (churn WASI preview2) — mitigado por pin de versión.

## Learning

_Pendiente._
