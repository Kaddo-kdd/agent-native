---
type: adr
id: ADR-002
status: draft
date: 2026-10-17
created_from: knowledge/tech/discovery/decision-candidates.md
related_work_items:
  - AN-003
code:
  - crates/runtime/
---

# ADR-002 — Runtime independiente del LLM y del compiler

## Context

El propuesta de valor de Agent-Native documentada en `business.md` (goal #4) y `product.md` separa el **costo de construcción** del **costo de operación**: la IA interviene únicamente durante el build; los componentes publicados conservan su comportamiento operativo sin necesitar el agente que los generó.

La POC cumple parcialmente: `runner` depende de `irtool` por `path = "../irtool"` (ver [extraction-matrix.md](../extraction-matrix.md) C-07) y requiere Cargo para operar. El producto nuevo debe hacer esta independencia **técnica y auditable**, no un principio aspiracional.

## Options Considered

### Opción A — Runtime embebe un cliente LLM mínimo para "autoreparar" ejecuciones

Pros: narrativa comercial atractiva.
Contras: rompe la separación build/operación; introduce costo e inestabilidad por invocación; convierte al runtime en inútil sin credenciales de proveedor. **Rechazada** por `business.md` goal #4.

### Opción B — Runtime depende del compiler para reutilizar código de carga y metadatos

Pros: simplifica el share de utilidades (lectura de manifiestos, hash).
Contras: cualquier cambio en el compiler recompila/invalida el runtime; impide distribuir el runtime sin la toolchain del compiler; rompe INIT-05-02 (SDK Rust).

### Opción C — Runtime autosuficiente, consume `ir` para metadatos y `wasmtime` para ejecución; no referencia `compiler` ni LLM

Pros: ejecución distribuible sin Rust ni acceso a LLM; auditable por lint CI y por `cargo metadata`.
Contras: metadatos del artefacto (manifiesto, hash) deben especificarse como contrato estable en `ir` o en un crate de metadatos, no derivarse del `compiler`.

## Decision

Se adopta la **Opción C**:

- **Grafo de dependencias prohibidas:** `runtime` **no** puede declarar como dependencia Cargo (ni directa ni transitiva por `path`) a `compiler`, a ningún adaptador LLM, ni a crates de Kaddo.
- **Grafo permitido:** `runtime` depende únicamente de `ir` (para contratos de metadatos), `wasmtime` y utilidades de bajo nivel (serde, thiserror).
- **Contrato de metadatos:** cualquier información del artefacto que el runtime necesita leer (hash, versión, firma) se define en `ir::manifest::` o en un submódulo claramente nombrado dentro de `ir`.
- **Carga runtime de componentes externos:** el runtime acepta componentes WIT-compatibles independientes de su origen (incluidos los no generados por Agent-Native), siempre que respeten el manifiesto. Esta propiedad es parte del contrato del runtime, no una característica opcional.
- **Lint CI (obligatorio):** un paso de CI falla si `cargo tree -p runtime` lista `compiler`, cualquier adaptador LLM o cualquier crate de Kaddo. Este lint se materializa en INIT-01-01 junto con el workspace.

## Consequences

**Positivas:**
- INIT-05-02 (SDK Rust) es trivial: `cargo add agent-native-runtime` sin arrastrar el compiler.
- INIT-05-03 (release pipeline) puede construir binarios de `cli` que ejecutan componentes sin tooling de generación.
- La narrativa "ejecución sin IA" queda verificable mecánicamente.

**Negativas:**
- El contrato de manifiesto en `ir` debe mantenerse estable; cualquier cambio invalida publicaciones.
- La utilidad de impresión WAT (`print_wasm_to_wat`) de la POC vive en `compiler`, no en `runtime`; para depurar un runtime en producción hay que usar el `cli` del compilador por separado.

**Riesgos del roadmap mitigados:**
- R-03 (acoplamiento runtime→compiler).
- Business risk: ejecución dependiente del LLM.

**Seguimientos abiertos:**
- `[open]` Diseñar el lint CI concreto para enforcement — INIT-01-01 lo materializa.
- `[open]` Definir el crate exacto donde vive `manifest::` (dentro de `ir` o en un crate dedicado `metadata`) — se decide al abrir INIT-02-01.

## Related Capabilities

- Independent Runtime Execution.
- CLI as Primary Interface.

## Related Work Items

- AN-003 (este ADR).
- Precondición de: INIT-03-01, INIT-05-02, INIT-05-03.
