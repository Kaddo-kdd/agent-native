---
type: adr
id: ADR-003
status: draft
date: 2026-10-17
created_from: knowledge/tech/discovery/decision-candidates.md
related_work_items:
  - AN-003
code:
  - crates/ir/
  - crates/ir/schemas/
---

# ADR-003 — Semantic IR v0.5 como baseline estable y política de compatibilidad

## Context

La POC congeló Semantic IR v0.5 como la gramática validada por `verify_ir` (ver [extraction-matrix.md](../extraction-matrix.md) C-01, S-01). Soporta: tipos numéricos, booleanos, registros, aritmética, comparaciones, condicionales, llamadas e imports. Hay 4 schemas versionados (`ir.schema.json`, `v2`, `v3`, `v4`) y la POC validó la evolución entre ellos.

El roadmap identifica tres caminos de evolución (OQ L144): `string`, streams/efectos, errores. La extensión a v0.6 con `string` es necesaria para WASI preview2 (INIT-07-02).

El principio del roadmap "equivalencia funcional primero" prohíbe incorporar nuevas características antes de alcanzar paridad con la POC, salvo correcciones necesarias para compatibilidad, seguridad o integridad.

La selección exacta de la versión de Wasmtime (OQ L145) sigue abierta; cualquier cambio de runtime requiere pruebas de compatibilidad y regresión.

## Options Considered

### Opción A — Fijar v0.5 como estable y congelar schemas v2/v3/v4 como histórico

Pros: la POC validó v0.5; permite migrar las 8 pruebas `irtool`. Preserva el linaje de evolución.
Contras: ninguna significativa.

### Opción B — Rehacer la IR desde cero con nombres distintos

Pros: oportunidad de limpiar nombres y estructura.
Contras: destruye el valor de recuperación de la POC; invalida las 8 pruebas sin ganancia funcional. Rompe el principio #1 del roadmap.

### Opción C — Saltar directamente a v0.6 con `string` y errores

Pros: habilita RM-07 antes.
Contras: viola "equivalencia funcional primero"; introduce extensiones no probadas antes de validar el MVP. **Rechazada** por principio del roadmap.

## Decision

Se adopta la **Opción A** con política de evolución explícita:

- **Baseline estable:** Semantic IR **v0.5** tal como está definida en `_reference/poc-agent-native/spec/semantic-ir-v0.5-spec.md` y `semantic-ir-v0.5.schema.json`. Se migra como `crates/ir/schemas/ir.schema.json` (ver S-01 en matriz).
- **Linaje histórico:** `ir.schema.v2.json`, `v3.json`, `v4.json` se migran como histórico documentado en `crates/ir/schemas/history/`. No son API pública; existen para auditoría.
- **Política de evolución:**
  - Toda extensión a v0.6+ se materializa **bajo flag** `--ir-version=0.6` (o equivalente), nunca por defecto.
  - Las nuevas características (`string`, errores) se añaden una por una, cada una con suite de regresión propia.
  - v0.5 **sigue siendo la ruta estable y por defecto** hasta que v0.6 tenga equivalencia funcional demostrada en todos los escenarios de la POC.
- **Baseline de Wasmtime:** `[open]`. Se cerrará en INIT-02-01 tras validar: (a) soporte de Component Model y WIT en versión LTS razonable, (b) compatibilidad con v0.5 reproducida por las 8 pruebas `irtool`, (c) metodología de upgrade con regresión obligatoria. Hasta entonces, el workspace puede pinnear una versión concreta de forma provisional.
- **Compatibilidad hacia atrás:** cualquier cambio en v0.5 que no sea retrocompatible exige incrementar la mayor (v1.0) y mantener v0.5 en modo lectura.

## Consequences

**Positivas:**
- Las 8 pruebas `irtool` migran con compatibilidad 1:1.
- Los 5 programas IR fixture de la POC (ver S-02) siguen siendo válidos.
- La extensión futura `string` para RM-07 queda explícitamente separada del MVP.

**Negativas:**
- v0.6 no se promueve a estable hasta que M3 (regresión POC) esté verde.
- El pin provisional de Wasmtime puede desactualizarse rápido; requiere revisión en cada cierre de bloque.

**Riesgos del roadmap mitigados:**
- R-04 (baseline Wasmtime no fijada) — gestionado por `[open]` explícito con criterios.
- R-09 (divergencia v0.5↔v0.6) — mitigado por flag experimental.

**Seguimientos abiertos:**
- `[open]` Versión exacta de Wasmtime baseline — INIT-02-01.
- `[open]` Formato exacto del flag `--ir-version` en la CLI — INIT-05-01.
- `[open]` Política de deprecación de v0.5 cuando v0.6 esté estable (p. ej. 2 ciclos de release antes de deprecarla) — a decidir en RM-07.

## Related Capabilities

- Semantic IR Verification.
- Component Compilation.

## Related Work Items

- AN-003 (este ADR).
- AN-002 (matriz de extracción C-01, S-01, S-02).
- Precondición de: INIT-02-01, INIT-02-02, INIT-07-02.
