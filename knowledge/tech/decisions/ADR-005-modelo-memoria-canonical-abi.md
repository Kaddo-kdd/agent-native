---
type: adr
id: ADR-005
status: draft
date: 2026-10-17
created_from: knowledge/tech/discovery/decision-candidates.md
related_work_items:
  - AN-003
code:
  - crates/ir/
  - crates/runtime/
---

# ADR-005 — Modelo de memoria y Canonical ABI

## Context

El supuesto inicial del roadmap (A2) fijaba dlmalloc + copia al host + reset entre tenants como estrategia por defecto. Fue retirado por el usuario por falta de sustento técnico. La POC usa un asignador incremental sin estrategia general de liberación (registrado como riesgo R-11); no se validó su comportamiento con instancias persistentes ni bajo presión de memoria.

Open Question L148 quedó `[deferred]` en `knowledge/tech/codebase.md`, con plan de resolución explícito: materializar este ADR y conducir los experimentos de INIT-02-05 antes de cerrar la estrategia definitiva.

Esta decisión impacta:
- Semánticas de retorno de registros bajo Canonical ABI.
- Ciclo de vida de instancias y reutilización entre tenants.
- Elegibilidad de componentes para `agent-native run` con instancias persistentes.

## Options Considered

### Opción A — Adoptar dlmalloc + copia al host + reset entre tenants como estrategia definitiva

Pros: patrón conocido en guests WASM.
Contras: no se midió en la POC; su comportamiento bajo instancias persistentes es desconocido. **Rechazada** por retirado del supuesto A2.

### Opción B — Instancias nuevas siempre; prohibir reutilización hasta tener garantías de aislamiento

Pros: seguro por defecto; alinea con `business.md` §Constraints "privilegio mínimo"; se puede optimizar después.
Contras: overhead por invocación; impacta p50/p95 cold start.

### Opción C — Modelo híbrido: instancias nuevas por defecto, pool con reset explícito tras validación de aislamiento

Pros: equilibra seguridad y rendimiento; permite experimentar en INIT-02-05 y validar en INIT-06-03.
Contras: más complejo; requiere diseño cuidadoso del API `Tenant`.

### Opción D — Permitir que el consumidor elija allocator vía trait object

Pros: máxima flexibilidad.
Contras: superficie de API grande para MVP; invita a decisiones no revisadas por el consumidor. **Diferida**.

## Decision

Se adopta la **Opción B como baseline operativa para MVP** con la **Opción C como ruta de optimización** gobernada por INIT-02-05/INIT-06-03:

### Baseline operativa (MVP, hasta M3 inclusive)

- Toda invocación del runtime crea una **instancia nueva**; el guest termina y libera recursos al final de cada invocación.
- Los **registros devueltos** por el componente se **copian al host** al cruzar la Canonical ABI; el runtime no retiene referencias al estado interno del guest.
- Se usa el allocator del guest **por defecto** generado por `wit-bindgen` (hoy dlmalloc, lo que **no** convierte dlmalloc en decisión; es el default del toolchain).
- **No hay reutilización de instancias entre tenants** en MVP. Ni siquiera dentro del mismo tenant si no hay evidencia de aislamiento.

### Ruta de optimización (post-M3, gobernada)

Antes de habilitar pool de instancias o reutilización entre invocaciones:

1. **INIT-02-05** documenta formalmente la semántica de Canonical ABI usada por el verifier + compiler (qué se copia, qué se mueve, qué cruza fronteras del guest).
2. **INIT-03-06** publica una línea base técnica (cold start, warm invocation, memoria por instancia) con la baseline operativa anterior.
3. **INIT-06-03** diseña `Tenant` API y pruebas de contaminación cruzada que **fallen sin aislamiento, pasen con él**.
4. Sólo entonces INIT-08-03 habilita module cache + warm pool, con wipe explícito entre tenants.

### Prohibiciones durante baseline operativa

- No introducir pool de instancias.
- No reutilizar `wasmtime::Store` entre invocaciones.
- No mantener referencias a memoria del guest en el host más allá del scope de la invocación.
- No optimizar prematuramente el allocator del guest sin evidencia.

### Qué queda explícitamente `[deferred]` tras AN-016

Las tres preguntas que originalmente quedaron `[open]` fueron ejercitadas por los experimentos de AN-016 (ver [`memory-model-notes.md`](../discovery/memory-model-notes.md) §3-§5). Ninguna se cerró con una decisión productiva, pero cada una ganó un **criterio objetivo de cierre**:

- **[deferred to: INIT-06-02]** — **Allocator alternativo** (p. ej. arena por invocación). AN-012/013 miden que el default `wit-bindgen` **no es el término dominante** del cold start (~2 µs warm vs ~389 µs cold está dominado por parsing + Cranelift, no por allocación). Criterio objetivo: antes de proponer un allocator alternativo hay que medir bytes allocados reales con `wasmtime::ResourceLimiter`, cuya integración vive en INIT-06-02 (fuel + instance limits). Hasta entonces, mantener el default.
- **[deferred to: INIT-06-03]** — **Estrategia de liberación en instancias persistentes**. El baseline operativa (fresh `Store` per invoke) hace la pregunta vacua: no hay instancias persistentes. INIT-06-03 debe decidir si levantar esa restricción con garantías de aislamiento multi-tenant; sólo entonces aplica una decisión sobre liberación.
- **[deferred to: RM-07 / INIT-07-02]** — **Comportamiento ante allocación masiva o fuga en el guest**. Los experimentos de AN-016 §3 muestran que la gramática v0.5 **impide** construir fixtures de estrés (sin loops, listas ni strings; records anidados rechazados por diseño — ver AN-016 §2 y `nested_records.rs`). El experimento se habilita cuando IR v0.6+ relaje esas restricciones. Mientras tanto, INIT-06-05 cubre fixtures de recursión a nivel WASM (fuera del dominio del compilador).

### Canonical ABI — [chosen] copy-on-return + fresh Store per invoke

Añadido por AN-016 con evidencia (`memory-model-notes.md` §4):

- **Records y primitivos retornados por el guest se copian al host** vía `wasmtime::component::Val` (confirmado con `program-005`/`make-point-sum`).
- **El `wasmtime::Store` se destruye al final de cada invocación**, liberando toda memoria lineal del guest sin acción del host.
- **Resources, strings y listas quedan fuera de alcance** de esta decisión (no cubiertos por IR v0.5).

Esta es la decisión productiva de ADR-005 sobre Canonical ABI: no quedó `[open]`; está documentada aquí.

## Consequences

**Positivas:**
- MVP seguro-por-defecto respecto a memoria: ningún estado cruza invocaciones ni tenants.
- La ruta de optimización queda explícita y gobernada por mediciones.
- R-11 (allocator incremental de la POC) no se hereda silenciosamente al producto nuevo.

**Negativas:**
- Cold start por invocación impacta p50/p95; se cuantifica en INIT-03-06.
- Rendimiento vs implementaciones nativas será necesariamente peor hasta que RM-08 habilite caché y warm pool.

**Riesgos del roadmap mitigados:**
- R-01 (fuga de estado entre tenants) — prohibida por diseño en baseline.
- R-11 (allocator incremental sin liberación general) — tratada por aislamiento por invocación.

**Seguimientos abiertos:**
- `[deferred to: INIT-06-02]` Allocator alternativo — criterio objetivo: medir bytes con `ResourceLimiter`.
- `[deferred to: INIT-06-03]` Estrategia para instancias persistentes — criterio objetivo: aislamiento multi-tenant.
- `[deferred to: RM-07 / INIT-07-02]` Comportamiento ante allocación masiva — criterio objetivo: IR relaje restricciones (loops/listas/strings).

Cerrados por AN-016 (ver `memory-model-notes.md`):
- `[chosen]` Canonical ABI: copy-on-return + fresh Store per invoke.
- `[chosen]` Eliminación de la dependencia del cwd en la resolución de WIT via `CompileContext` (impacto cruzado con el modelo de memoria vía reproducibilidad de artefactos).

## Related Capabilities

- Independent Runtime Execution.
- Host Capability Policy.
- Semantic IR Verification (sección Canonical ABI).

## Related Work Items

- AN-003 (este ADR).
- AN-016: experimentos de memoria, Canonical ABI y resolución WIT por contexto (ver `memory-model-notes.md`).
- Precondición de: INIT-02-05, INIT-03-01, INIT-06-02, INIT-06-03.
- Habilita (una vez cerrado tras experimentos): INIT-08-03.
