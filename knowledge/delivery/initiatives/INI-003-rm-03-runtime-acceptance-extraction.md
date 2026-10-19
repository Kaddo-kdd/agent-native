---
type: initiative
id: INI-003
title: RM-03 — Runtime & Acceptance Extraction
status: in-progress
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
    materialized_as: AN-009, AN-014 (WIT interface resolution)
  - id: INIT-03-03
    title: Executor de aceptación YAML
    status: candidate
  - id: INIT-03-04
    title: Propagación de errores host + imports
    status: completed
    materialized_as: AN-010, AN-015 (hardening + WIT-granular policy)
  - id: INIT-03-05
    title: Controles mínimos de ejecución (deny-by-default)
    status: completed
    materialized_as: AN-011, AN-015 (operation-level granularity)
  - id: INIT-03-06
    title: Línea base técnica del runtime
    status: completed
    materialized_as: AN-012, AN-017 (consolidación + Component Model)
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

---

## Append AN-020 — Evidencia por criterio de salida (sin decisión)

**Producido por AN-020 (hallazgo H6).** Esta sección es un append
informativo: **no modifica** los 5 criterios de salida originales (líneas
75-83 arriba) ni declara la iniciativa cerrada. Su único propósito es
que una revisión humana posterior pueda decidir qué criterios cerrar con
evidencia, qué criterios requieren implementación adicional y qué
criterios necesitan una resolución explícita de alcance.

### Clasificación por criterio

- **`cerrable-con-evidencia`** — la evidencia reunida satisface el
  criterio literal; AN-020 no deja grietas.
- **`requiere-implementación`** — el criterio no se cumple hoy, pero su
  cierre es un trabajo acotado y mapeable a un WI; AN-020 lo deja
  propuesto, no implementado.
- **`requiere-decisión-humana`** — el criterio no se cumple literalmente
  y la resolución exige un juicio humano (reinterpretar, aplazar
  formalmente a otra iniciativa, o abrir una sub-tarea bloqueante).

### Tabla

| # | Criterio INI-003 (textual, L75-L83) | Evidencia disponible | Clasificación propuesta | Decisión humana requerida |
|---|---|---|---|---|
| 1 | "Las 10 pruebas `runner` de la POC se reproducen verdes en el nuevo repo." | `knowledge/tech/poc-equivalence-matrix.md` (re-evaluada por AN-020 con 3 grados): 3 exactas + 3 parciales + 4 aplazadas. | `requiere-decisión-humana` | Reinterpretar como "cobertura regresiva documentada en matriz 1:1 con cada aplazamiento justificado" (opción A), o bloquear M3 hasta reducir el conjunto `aplazado` (opción B). AN-020 presenta la matriz, no la decisión. |
| 2 | "Imports no autorizados y capabilities faltantes producen `RuntimeError` tipado — nunca `0`/`false` por defecto." | AN-015 (`core_results_to_component` sin fallback a 0), AN-019 `UnauthorizedImport`, AN-020 H3 (`StateReader` miss → `CaseStatus::Error` con evidencia `state_reader '<name>' miss: key=<n>`). | `cerrable-con-evidencia` | — |
| 3 | "Deny-by-default verificado: capability no listada en el manifiesto falla al cargar." | AN-011/15 `HostPolicy::deny_all`, `wit_policy.rs`, `negative_import_not_authorized_fails_pre_instantiate` + AN-020 `ac02`. **Pero**: el check ocurre pre-instantiate (dentro de `invoke`), **no durante `load_component`**. La frase literal exige el check al cargar. | `requiere-decisión-humana` | Opciones no exclusivas: (a) reinterpretar "falla al cargar" como "falla antes de ejecutar guest code" aceptando el comportamiento actual; (b) exigir el check en `load_component_with_policy` como pre-condición de M3 (sub-tarea bloqueante, fuera de alcance de AN-020); (c) aplazar explícitamente a INIT-06-01 (policy loader manifest-driven) y marcar este criterio como parcial. AN-020 no decide. |
| 4 | "Línea base técnica publicada en `knowledge/tech/baselines/` con cold start, warm invocation y memoria por instancia." | `knowledge/tech/baselines/runtime-baseline.md` (AN-012/17) con cold/warm/cache-hit. **Memoria por instancia NO está medida**; AN-017 Learning la difiere a INIT-06-02. AN-020 no mide memoria (fuera de alcance). | `requiere-decisión-humana` | Reinterpretar "memoria por instancia" como diferida con destino INIT-06-02 (criterio parcial), o considerar el criterio no satisfecho. AN-020 no mide. |
| 5 | "Lint CI verifica que `runtime` no depende de `compiler` ni del LLM." | `cargo tree -p agent-native-runtime --prefix none --edges normal \| grep -i compiler` verificado local tras AN-019 y AN-020. AN-020 añade paso explícito a `.github/workflows/ci.yml` (ver AC-14). | `cerrable-con-evidencia` tras AC-14 de AN-020 | — |

### Resumen propuesto

- **`cerrable-con-evidencia`**: criterios 2 y 5 (ambos con evidencia
  concreta en el repo).
- **`requiere-decisión-humana`**: criterios 1, 3 y 4 (los tres no se
  cumplen literalmente y la resolución exige un juicio sobre
  reinterpretación o aplazamiento formal).
- **`requiere-implementación`**: ninguno (el criterio 5 contaba como
  tal hasta que AN-020 añadió el paso explícito a CI).

### Lo que AN-020 **no** hace

AN-020 **no**:

- modifica los 5 criterios originales de INI-003;
- declara ningún criterio cerrado;
- declara INI-003 completada ni cambia su campo `status`;
- declara el hito M3 alcanzado.

La decisión final sobre los 3 criterios con `requiere-decisión-humana`
queda explícitamente para una revisión humana separada, posterior a la
revisión de los entregables de AN-020.
