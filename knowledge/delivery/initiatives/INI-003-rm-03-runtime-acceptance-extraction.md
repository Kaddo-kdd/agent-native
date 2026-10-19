---
type: initiative
id: INI-003
title: RM-03 — Runtime & Acceptance Extraction
status: completed
completed_at: '2026-10-19'
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
    status: completed
    materialized_as: AN-019 (executor + stubs + regresión POC) + AN-020 (cierre técnico M3: compat YAML POC, Component Model type-only, StateReader fail-contract, matriz re-evaluada, contrato normativo)
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

Cerrado 2026-10-19 bajo los **7 criterios M3 reformulados** acordados en la Resolución humana que figura más abajo. Los 5 criterios originales (sección "Success Criteria", líneas 75-83) se conservan literalmente como referencia histórica; INI-003 **no** declara su cumplimiento literal.

### Capacidades alcanzadas

- **Runtime independiente** (ADR-002): el crate `agent-native-runtime` no depende de `agent-native-compiler` ni de LLM. Verificado por paso CI explícito y por `cargo tree -p agent-native-runtime --prefix none --edges normal | grep -i compiler` → exit 1.
- **Ejecución Core WASM + Component Model**: `Engine::load_component_with_policy` acepta ambos payloads. Para CM, imports no funcionales (sólo tipos o resources auto-resolubles) delegan a `Component::invoke`. CM con imports funcionales queda deliberadamente fuera (RM-07).
- **Deny-by-default pre-instantiate** a tres niveles (core, WIT interface, WIT operation), con pruebas independientes. El fail-explicit ocurre antes de ejecutar guest code, aunque **no** al cargar (ver limitaciones).
- **Executor de aceptación YAML** con contrato normativo publicado (`poc-yaml-contract.md`): 7 submódulos, 3 kinds de stub declarativos (`StateReader`, `StateWriter`, `Noop`), `expected_host_calls` ordenados y comparados por `import` + args, records comparados por nombre + tipo + valor con faltantes y adicionales detectados.
- **Fail-explicit en lifting de resultados** (sin fallback silencioso a `0`/`false`): `grep -c "unwrap_or(0)" crates/runtime/src/testing/executor.rs` → 0.
- **Línea base técnica de latencia** publicada (`knowledge/tech/baselines/runtime-baseline.md`): cold start, warm invocation y cache-hit por path.
- **Caché content-addressed de componentes** (`Resolver`, AN-013).

### Decisiones humanas incorporadas

Resumidas aquí; detalladas en la sección "Resolución humana" al final de este fichero.

- **D1 — matriz POC con 3 grados verificables, no reproducción literal.** La matriz definitiva publica **0 `equivalencia-exacta` + 6 `cobertura-parcial` + 4 `aplazado`** (reclasificación 2026-10-19). INI-003 **no afirma** que las 10 pruebas unitarias de `runner/src/main.rs` del POC hayan sido reproducidas verdes.
- **D2 — validación manifest-driven diferida a INIT-06-01.** El check de policy ocurre pre-instantiate dentro de `invoke`, no durante `load_component`. INI-003 **no afirma** validación al cargar.
- **D3 — medición de memoria por instancia diferida a INIT-06-02.** La línea base de latencia es completa; la memoria se declara explícitamente no medida.

### Limitaciones conocidas (dentro del alcance declarado)

- **StateReader miss en el executor YAML no interrumpe el guest.** Opción D1-A: el stub registra el miss, marca `HostCtx::poisoned`, retorna `0` al guest y suprime los efectos observables de stubs posteriores. El executor fuerza `CaseStatus::Error` al final del caso. **No se afirma trap de Wasmtime ni interrupción inmediata del guest.** El guest puede continuar computando con el `0` provisional hasta que termine la invocación.
- **Equivalencia POC**: ninguna de las 10 pruebas unitarias de `runner/src/main.rs` del POC se reproduce con `equivalencia-exacta` en el nuevo workspace. 6 están cubiertas parcialmente, 4 explícitamente aplazadas. La POC permanece como referencia (`_reference/poc-agent-native/`) y no se modifica.
- **YAML contract**: el executor acepta sólo la forma de raíz `{ suite, spec_version, cases }`. Las otras dos formas que el runner POC admite (`acceptance.public` y secuencia-raíz directa) quedan rechazadas por parse, con razón documentada.
- **Component Model host stubs funcionales**: declarados fuera de alcance (RM-07). Un componente CM con imports funcionales produce `HostError` tipado con mensaje `functional import '<iface>#<op>' requires a stub (RM-07)`.

### Deuda diferida (fuera de alcance de M3)

| Deuda | Destino |
|---|---|
| Composición multi-componente | RM-07 |
| Host providers reales (más allá de los 3 stubs declarativos) | INIT-06-01 |
| Políticas manifest-driven con check en `load_component` | INIT-06-01 |
| Medición de memoria por instancia | INIT-06-02 |
| WASI preview2 | RM-07 |
| Evolución v1→v2 integral (esclusas) | M4 (INIT-04-03/04-05) |
| Ejecución multi-paso de dependencias con interrupción exacta | RM-04/06 + investigación post-M3 |
| Propagación inmediata de errores desde callbacks del host (trap Wasmtime desde closure) | Investigación post-M3; hallazgo experimental documentado en `host_ctx.rs` (STATUS_STACK_BUFFER_OVERRUN con wasmtime 27 en Windows) |
| `host_state` tipos ricos (strings, maps anidados, `credit_limits`, `tenant_status`) | INIT-07-02 |
| Argumentos no primitivos (strings, floats, listas, enums) | INIT-07-02 |
| Records anidados en `expected` | RM-07 si surge demanda |
| Ejecución semántica de fixtures POC v0.2/v0.3/v0.4/v0.5 exactos | RM-04/06 y RM-07 |

### Materialización por candidato de iniciativa

| Candidato | Status | Materializado como |
|---|---|---|
| INIT-03-01 — Crate runtime | completed | AN-009, AN-014 |
| INIT-03-02 — Resolver | completed | AN-013 |
| INIT-03-03 — Executor YAML | **completed (2026-10-19)** | **AN-019 + AN-020** |
| INIT-03-04 — Propagación de errores | completed | AN-010, AN-015 |
| INIT-03-05 — Deny-by-default | completed | AN-011, AN-015 |
| INIT-03-06 — Línea base técnica | completed | AN-012, AN-017 |

### Evidencia de aceptación

- Commits implementación: AN-019 `5b98d81`, AN-020 `56f1888`, patch pre-M3 `be9f7e2`.
- CI remota 3-OS verde en el commit final del patch: run #39198976200.
- Workspace tests: 60+ verdes (incluyendo 32 en `executor_yaml`).
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- Documentos de soporte: `poc-equivalence-matrix.md`, `poc-yaml-contract.md`, `baselines/runtime-baseline.md`, además de los Learnings de los WI materiales (AN-009..AN-020).

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

---

## Resolución humana 2026-10-19 — Criterios de salida de M3 reformulados

**Contexto.** Tras revisar la matriz POC re-evaluada y el append AN-020,
el usuario tomó las tres decisiones humanas pendientes. Estas
decisiones **modifican formalmente los criterios de salida aplicables
a M3**; **no** declaran cumplidos los criterios originales 1, 3 y 4
tal como estaban redactados en las líneas 75-83 de este fichero.

Los 5 criterios originales se conservan **sin edición** en su sección
`Success Criteria` arriba, con fines de trazabilidad histórica. La
declaración formal de M3 — cuando se autorice — se evaluará contra los
**Criterios M3 consolidados** enumerados más abajo, no contra los
originales en su forma literal.

Las decisiones **no abren** nuevos Work Items. Lo diferido se atiende
a través de las iniciativas de roadmap ya existentes.

### Decisiones humanas y trazabilidad

| # | Criterio INI-003 original (textual, L75-L83) | Decisión humana (2026-10-19) | Criterio M3 reformulado | Iniciativa que atiende lo diferido |
|---|---|---|---|---|
| 1 | "Las 10 pruebas `runner` de la POC se reproducen verdes en el nuevo repo." | **D1 — Opción A**: aceptar la matriz de equivalencia (en su estado definitivo tras el patch pre-M3 2026-10-19: **0 `equivalencia-exacta` + 6 `cobertura-parcial` + 4 `aplazado`**; previamente 3+3+4 antes de la reclasificación de filas 2, 4 y 8). **No** afirmar que las 10 pruebas originales fueron reproducidas. | **M3-CRIT-1**: la matriz `knowledge/tech/poc-equivalence-matrix.md` publica la cobertura regresiva con 3 grados verificables; cada fila lleva evidencia identificable (test, path o commit) y cada `aplazado` lleva destino de roadmap específico. **No se afirma reproducción literal de las 10 pruebas**. | Filas `cobertura-parcial`: expansión de regresión + ejecución semántica de fixtures POC v0.2/v0.3/v0.4/v0.5 exactos → RM-04/06 y RM-07. Filas `aplazado`: RM-07 (composición multi-componente), INIT-06-01 + INIT-07-02 (host provider + host state rico), M4 (esclusas v1→v2 integrales). |
| 2 | "Imports no autorizados y capabilities faltantes producen `RuntimeError` tipado — nunca `0`/`false` por defecto." | **Sin decisión humana requerida** — ya `cerrable-con-evidencia`. | **M3-CRIT-2** (precisión patch 2026-10-19): tres propiedades verificables dentro del alcance del executor. **(a)** El runtime no cae a `0`/`false` por defecto en el lifting de resultados: `core_results_to_component` surge `ExecutionFailed` tipado ante tipos no-`I32` (AN-015). **(b)** Imports no autorizados producen `RuntimeError::UnauthorizedImport` pre-instantiate (AN-015/AN-019). **(c)** En el executor YAML, una miss de `StateReader` es de detección obligatoria: registra el miss, marca `HostCtx::poisoned`, suprime los efectos observables de los stubs posteriores (`StateWriter` no escribe, `Noop` no retorna su valor nominal) y fuerza `CaseStatus::Error` al final del caso. **No se afirma fail-fast dentro del guest**: el guest puede continuar computando con un `0` provisional hasta que termine la invocación. Verificable: `grep -c "unwrap_or(0)" crates/runtime/src/testing/executor.rs` → 0; tests `ac04_state_reader_miss_fails_explicit` y `ac04b_statewriter_suppressed_after_reader_miss`. | — |
| 3 | "Deny-by-default verificado: capability no listada en el manifiesto falla al cargar." | **D2 — Opción C**: diferir la validación manifest-driven a INIT-06-01. Mantener la autorización obligatoria **antes de instanciar y ejecutar** el componente; **no** trasladar el check a `load_component` (modificaría el contrato de carga sin aportar una mejora clara de seguridad). | **M3-CRIT-3**: el runtime hace deny-by-default pre-instantiate: cualquier componente con imports no autorizados por `HostPolicy` falla con `RuntimeError::UnauthorizedImport` **antes de ejecutar guest code**. Verificado por 3 tests independientes a nivel core, WIT interface y WIT operation (`host_errors.rs`, `wit_policy.rs`, `executor_yaml.rs::negative_import_not_authorized_fails_pre_instantiate`). | Validación manifest-driven al cargar → **INIT-06-01** (policy loader + capability tokens). |
| 4 | "Línea base técnica publicada en `knowledge/tech/baselines/` con cold start, warm invocation y memoria por instancia." | **D3 — Opción A**: diferir la medición de memoria por instancia a INIT-06-02. Conservar la línea base de latencia actual y documentar explícitamente que la memoria todavía no se ha medido. | **M3-CRIT-4**: `knowledge/tech/baselines/runtime-baseline.md` publica cold start, warm invocation y cache-hit por path (AN-012/17). Memoria por instancia **explícitamente diferida** y documentada como no medida en M3. | Memoria por instancia → **INIT-06-02** (fuel + instance limits + `ResourceLimiter`). |
| 5 | "Lint CI verifica que `runtime` no depende de `compiler` ni del LLM." | **Sin decisión humana requerida** — ya `cerrable-con-evidencia`. | **M3-CRIT-5**: `.github/workflows/ci.yml` ejecuta el paso explícito "Enforce ADR-002 (runtime does not depend on compiler)" en 3 OS (verificado en CI de `12e3a74`). | — |

Criterios adicionales derivados del alcance real demostrado por
AN-019 + AN-020, incorporados a M3:

| # | Criterio M3 adicional | Evidencia |
|---|---|---|
| 6 | **M3-CRIT-6** — El executor YAML ejecuta Core WASM y Component Model (ambos payload-paths del runtime). | `crates/runtime/tests/executor_yaml.rs::regression_smoke_suite_passes` (Core) + `::ac01_component_model_without_functional_imports_runs_via_invoke` (CM). |
| 7 | **M3-CRIT-7** — Contrato YAML normativo publicado para el executor. | `knowledge/tech/poc-yaml-contract.md` con las 4 secciones; `deny_unknown_fields` enforcement en código. |

### Criterios M3 consolidados (aplicables a la declaración)

La declaración formal de M3 — cuando se autorice — exige **y sólo
exige**:

1. **M3-CRIT-1** — matriz POC publicada con 3 grados verificables, cada
   fila con evidencia y cada aplazamiento con destino.
2. **M3-CRIT-2** — fail-explicit en runtime (sin fallback silencioso)
   y, en el executor YAML, detección obligatoria de miss +
   `CaseStatus::Error` + supresión de efectos posteriores de los
   stubs. **No afirma fail-fast dentro del guest.** Verificable con
   `grep` + tests `ac04` / `ac04b`.
3. **M3-CRIT-3** — deny-by-default pre-instantiate (3 niveles: core,
   WIT interface, WIT operation), verificable con 3 tests
   independientes.
4. **M3-CRIT-4** — línea base latencia publicada con cold/warm/cache-hit;
   memoria por instancia documentada como diferida.
5. **M3-CRIT-5** — paso explícito en CI que verifica ADR-002.
6. **M3-CRIT-6** — executor YAML ejecuta Core WASM y Component Model.
7. **M3-CRIT-7** — contrato YAML normativo publicado.

### Enunciado consolidado de M3

**M3 certifica un runtime independiente capaz de ejecutar componentes
Core WASM y Component Model (dentro del alcance soportado), con
políticas explícitas de deny-by-default, errores tipados sin fallbacks
silenciosos, línea base de latencia publicada y pruebas de aceptación
YAML con contrato normativo documentado.**

Memoria por instancia y validación manifest-driven al cargar son
capacidades **declaradas fuera del alcance de M3** y asignadas a
INIT-06-02 e INIT-06-01 respectivamente.

### Lo que esta resolución **no** hace

- No declara INI-003 completada ni cambia su campo `status` (sigue
  `in-progress` hasta la autorización explícita del cierre).
- No declara M3 alcanzado.
- No modifica los 5 criterios originales en su sección `Success
  Criteria` (siguen literalmente iguales con fines de trazabilidad
  histórica).
- No abre AN-021 ni ningún otro Work Item nuevo.

### Pendiente antes del cierre formal de INI-003 y declaración de M3

1. Revisión humana del estado final de AN-020 y la evidencia
   consolidada en este fichero.
2. Autorización explícita para cambiar `status: in-progress` →
   `status: completed` en el frontmatter de INI-003.
3. Autorización explícita para declarar el hito M3 (y, si procede,
   actualizar `knowledge/delivery/roadmap.md` / `current-state.md`).
