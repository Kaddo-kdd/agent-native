---
id: AN-020
type: feature
status: completed
title: Cierre técnico M3 — brechas verificables de AN-019
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
code:
  - crates/runtime/src/testing/executor.rs
  - crates/runtime/src/testing/suite.rs
  - crates/runtime/src/testing/stub.rs
  - crates/runtime/tests/executor_yaml.rs
  - crates/runtime/tests/suites/
  - knowledge/tech/poc-equivalence-matrix.md
  - >-
    knowledge/delivery/initiatives/INI-003-rm-03-runtime-acceptance-extraction.md
source_roadmap_initiative: INIT-03-03
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Validation & Evolution
related_capabilities:
  - Acceptance Testing & Regression
  - Independent Runtime Execution
expected_value: >-
  Cierra las seis brechas verificables detectadas en la revisión humana
  post-AN-019 que hoy bloquean la declaración formal de M3 y el cierre de
  INI-003. Entrega: Component Model sin host imports ejecutable, records
  comparables, StateReader fail-explicit, compatibilidad real con el formato
  YAML del POC, matriz POC con equivalencia funcional estricta, y mapeo
  verificable INI-003 success-criteria → implementación.
risks:
  - >-
    R-020-01: cambiar el campo YAML de stubs/host-calls de `function` a `import`
    rompe las 5 suites internas y los 11 cases de AN-019. Mitigación aprobada
    por humano (OQ-01): `import` queda como nombre canónico; `function` se
    acepta como alias temporal durante M3→M4. Declaraciones con ambos campos a
    la vez son rechazadas en parse (fail-explicit).
  - >-
    R-020-02: `host_state` POC trae sub-maps más allá de `kv:`
    (`credit_limits:`, etc.) usados por fixtures reales. Si AN-020 sólo re-mapea
    `kv:`, suites POC v0.3+ con tipos ricos siguen sin ejecutar. Mitigación:
    declarar explícitamente qué sub-maps se soportan en MVP M3 y cuáles quedan
    como `[open]` para INIT-07-02.
  - >-
    R-020-03: `Expected::Record` requiere resolver nombres de campo contra el
    schema de la IR. El runtime no accede a la IR (ADR-002). Mitigación:
    pattern-match contra `Val::Record` de wasmtime, comparar por nombre de campo
    plano; si un campo esperado no aparece en el resultado → fallo con evidencia
    explícita.
  - >-
    R-020-04: eliminar `unwrap_or(0)` en StateReader cambia la semántica
    observable de `kv-read.yaml` (`read-missing-key-defaults-to-zero`).
    Mitigación: ese case se convierte en test negativo (`StubReaderMiss` produce
    case `Error`), documentado en la matriz.
dependencies:
  - AN-019
source_signals:
  - 'Revisión humana post-AN-019 (2026-10-18): 6 hallazgos verificables.'
  - >-
    Diff POC YAML vs schema AN-019: campo `import` vs `function`, args
    booleanos, records en expected, sub-maps en host_state.
  - >-
    INI-003 success criteria L75-L83: 5 criterios con interpretación estricta vs
    flexible pendiente de decisión humana.
decision_candidates: []
related_decisions:
  - ADR-002
  - ADR-004
  - ADR-005
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-18T00:00:00.000Z
updated_at: 2026-10-18T00:00:00.000Z
ready_at: '2026-10-18'
completed_at: '2026-10-19'
---

# AN-020 — Cierre técnico M3: brechas verificables de AN-019

**Actor y resultado:**
Un contribuidor ejecuta `cargo test -p agent-native-runtime --test executor_yaml`, el suite incluye al menos un caso Component-Model sin host imports, al menos un caso con `expected` record no trivial, y al menos un caso negativo que verifica que `StateReader` sobre una clave ausente produce fallo tipado — todos verdes. El fichero `knowledge/tech/poc-equivalence-matrix.md` distingue tres grados de cobertura (exacta / parcial / aplazada) y lista cuáles de los 5 criterios de salida de INI-003 quedan cerrados con evidencia, cuáles necesitan implementación adicional, y cuáles exigen decisión humana explícita de cambio de alcance.

## Hallazgos origen (revisión humana post-AN-019)

Los seis hallazgos verificables de la revisión del 2026-10-18, en el orden del input original:

### H1 — Component Model sin host imports funcionales no es ejecutable

**Estado actual:** `run_component_case` en `crates/runtime/src/testing/executor.rs:143-153` devuelve `HostError("Component-Model host stubs are not yet wired (see RM-07)")` incondicionalmente. Esto rechaza también los componentes v0.5 que **no declaran imports funcionales**, que podrían ejecutarse hoy con `Component::invoke` existente.

**Precisión aprobada (ajuste 1):** separar **autorización** (lo que `HostPolicy` permite) de **resolución** (lo que Wasmtime consigue linkar sin stubs). Un import funcional autorizado por policy pero sin stub registrado NO está implementado automáticamente; sigue siendo un fallo de resolución.

**Objetivo:**
- Ejecutar vía `Component::invoke` **sólo si** el componente CM cumple alguna de estas condiciones:
  - no declara imports (0 imports en `component.component_type().imports(engine)`).
  - sólo declara imports de **tipos** (`ComponentItem::Type` / `ComponentItem::Resource` resolubles por el runtime existente sin host functions).
- Un import **funcional** (`ComponentItem::ComponentFunc` o un `ComponentInstance` cuyas exportaciones incluyan funciones), aun autorizado por policy, sigue produciendo `HostError` tipado con trazado explícito a RM-07.
- La discriminación "sólo tipos" vs "funcional" se hace iterando los imports y rechazando el primer `ComponentFunc` o exportación de función dentro de un `ComponentInstance`. Resultado documentado en el error (`"functional import 'iface#op' requires a stub (RM-07)"`).
- Añadir al menos 1 case Component Model sin imports funcionales a las suites (`component-smoke.yaml`), reutilizando un fixture v0.5 existente del compiler. Si el único fixture disponible (`program-005.ir.json`) tiene imports funcionales, el AC se cumple con un IR inline minimal construido en el test binario.

### H2 — `Expected::Record` declarado pero no comparado

**Estado actual:** `crates/runtime/src/testing/suite.rs:72-77` define `Expected::Record(BTreeMap<String, i32>)`. En `executor.rs:170-176`, la rama `Expected::Record(_)` empuja evidencia "record expectations are not supported in AN-019 MVP" y marca el case como `Fail`.

**Precisión aprobada (ajuste 2):** comparación estricta por **nombre, tipo y valor**; detectar tanto campos **faltantes** como **adicionales**; documentar la normalización de identificadores IR/WIT.

**Objetivo:**
- Implementar comparación contra `Val::Record` de wasmtime iterando sus campos nombrados.
- **Nombre**: comparación exacta tras normalización explícita. Regla: el campo del YAML se compara contra el nombre tal como Wasmtime lo expone (que a su vez es el nombre kebab-case del campo WIT). Si el YAML del POC usa snake_case y la WIT emitida es kebab-case, la normalización canónica es **kebab-case** (regla única, documentada en `poc-yaml-contract.md`). No se aplica normalización implícita de case; una divergencia es un fallo explícito con evidencia del nombre esperado y del nombre encontrado.
- **Tipo**: cada campo esperado debe ser `S32`. Un campo con tipo distinto en el resultado → `CaseStatus::Fail` con evidencia del tipo encontrado (reutilizando `component_val_kind`).
- **Valor**: comparación exacta de `i32`.
- **Campo esperado ausente del resultado** → `Fail` con "expected field `<name>` missing in result record".
- **Campo del resultado no esperado por el YAML** → `Fail` con "unexpected field `<name>` in result record". (Cambio respecto al borrador v0: antes se ignoraban. Precisión 2 obliga a detectarlos.)
- Añadir al menos 1 case con `expected:` record completo (reutilizando el patrón `smoke-v2.yaml` estilo `{sum: 30}`) + 1 test negativo con campo faltante + 1 test negativo con campo adicional.

### H3 — `StateReader` cae a 0 en claves ausentes (fallback silencioso)

**Estado actual:** `crates/runtime/src/testing/executor.rs:200-210`:
```rust
data.kv.get(&arg0).copied().unwrap_or(0)
```
Contradice ADR-004 (fail-explicit). El case actual `kv-read.yaml::read-missing-key-defaults-to-zero` **documenta** este fallback como comportamiento esperado — es exactamente la deuda a cerrar.

**Objetivo:**
- Reemplazar `unwrap_or(0)` por una vía explícita: cuando la clave no existe, retornar un `wasmtime::Error` desde el closure (via `anyhow::anyhow!`) que el runtime surface como `ExecutionFailed` tipado, convertido por el executor a `CaseStatus::Error` con evidencia "state_reader miss: key=<arg0>".
- Migrar `kv-read.yaml::read-missing-key-defaults-to-zero` a `kv-read.yaml::read-missing-key-fails-explicit` con `expected` eliminado y aserción cambiada a verificar `status: error` + mensaje.
- Añadir test negativo adicional en `executor_yaml.rs` que verifique la variante exacta de error.

### H4 — Incompatibilidad real con el formato YAML del POC

**Estado actual verificado contra `_reference/poc-agent-native/tests/*.yaml`:**

| # | Campo / estructura | POC | AN-019 | Impacto |
|---|---|---|---|---|
| 1 | `expected_host_calls[*].import` | `import: check_credit` | `function: host.kv_get` | **Breaking**: suites POC no parsean en AN-019. |
| 2 | Nombre corto vs calificado | `import: check_credit` (sólo operation) | call registrada como `host.check_credit` (interface + "." + operation) | **Breaking semántico**: aunque se renombre el campo, la comparación falla si no se define una regla explícita de normalización. |
| 3 | `args` booleans | `args: [-1, true]` (public.yaml) | sólo `Vec<i32>` | **Breaking**: cases con bool rechazados en parse. |
| 4 | `expected` record | `{status_code: 1, subtotal_cents: 0, ...}` | Declarado, no implementado (H2). | Cubierto por H2. |
| 5 | `host_state.kv` | Soportado por el runner POC. | Soportado. | OK. |
| 6 | `host_state.credit_limits` + otros sub-maps | public-003.yaml los usa (`credit_limits`), tenant-aware fixtures añaden `tenant_status`, `quota`, etc. | Rechazado por `HostState` struct estricto. | **Breaking**: fixtures con lógica de dominio no parsean. |
| 7 | Raíz sin `suite:` + `spec_version:` | `narrative-005.yaml` empieza directo con `cases:` sin ninguno de los dos. | Ambos campos obligatorios en `Suite` (no `#[serde(default)]`). | **Breaking**: suites POC narrativas no parsean. |
| 8 | Raíz con comentarios + metadata libre | `resource-reservation.yaml` empieza con `# Tests...` + bloque comentado. | Comentarios YAML `#` son tolerados por serde_yaml2 (OK). Metadata extra en raíz (`description:`, `tags:`) sería rechazada hoy por `deny_unknown_fields` implícito si lo activáramos. | OK si se mantiene el default (`allow_unknown_fields`). |
| 9 | `acceptance.public` como sección-raíz | **No existe como clave YAML en POC.** El término `acceptance` aparece sólo en el *orchestrator* del POC (`interpreter.rs`, `lib.rs`) como campo de `acceptance_criteria` extraído de markdown. No es un formato YAML del runner. | N/A. | Clarificación documental: `acceptance.public` **no** es un formato de suite del runner POC; no se adopta en M3. |

**Precisiones aprobadas (ajustes OQ-01, OQ-02, ajuste 4):**
- **OQ-01 (nombres)**: `import` canónico; `function` como **alias temporal**. Si el YAML trae ambos campos a la vez → rechazo en parse (`InvalidYaml`), nunca merge silencioso.
- **OQ-01 (compat semántica short vs qualified)**: regla de normalización única y explícita, documentada en `poc-yaml-contract.md`:
  - Si `import` contiene punto (`host.kv_get`) → se compara literalmente contra la llamada registrada.
  - Si `import` NO contiene punto (`kv_get`) → se compara contra la **operación** de la llamada registrada, ignorando el prefijo de interfaz. Rechazo del formato ambiguo "`.op`" o "`host.`" (interfaz o operación vacía) con `InvalidYaml`.
- **OQ-02 (sub-maps extra)**: campos de `host_state` distintos de `kv:` son **rechazados en parse** (fail-explicit, ADR-004). No warning. No ignore. Cualquier fixture POC que los use queda clasificado en H5 como `aplazado` con destino INIT-07-02.
- **Raíz sin `suite` + `spec_version`** (ajuste 4): mantener ambos como **obligatorios** en el schema AN-020. Las suites POC narrativas que los omitan quedan clasificadas en H5 como `aplazado` o requieren adaptación mínima documentada (el POC no es canónico; el AN-020 schema sí). Esta decisión se documenta explícitamente en `poc-yaml-contract.md`.

**Objetivo:**
- Renombrar `ExpectedHostCall.function` → `ExpectedHostCall.import`, con `function` aceptado como alias temporal y mutuamente excluyente.
- `Case.args: Vec<ArgValue>` con `enum ArgValue { I32(i32), Bool(bool) }`; la deserialización YAML distingue `true`/`false` de números enteros nativos.
- Sub-maps extra en `host_state` → `InvalidYaml` con mensaje "unsupported host_state key `<name>` (see knowledge/tech/poc-yaml-contract.md)".
- Publicar `knowledge/tech/poc-yaml-contract.md` con 4 secciones:
  1. **Campos idénticos**: igual nombre + semántica (`id`, `requirement`, `function`, `args` i32, `host_state.kv`, `expected` primitivo, `expected` record simple).
  2. **Campos renombrados o normalizados**: `expected_host_calls[*].import` (alias `function`), regla de short-vs-qualified.
  3. **Campos deliberadamente omitidos (fail-explicit)**: `host_state.credit_limits` y análogos, booleans como sentinel en records, nested records.
  4. **Estructuras-raíz no soportadas**: suites sin `suite` + `spec_version`, `acceptance.public` (que ni siquiera es un formato de suite POC).
- Cada sección incluye ejemplo YAML y destino de roadmap (INIT-07-02 / RM-07 / out-of-scope) para los puntos no soportados.

### H5 — Matriz POC con equivalencia funcional estricta

**Estado actual:** `knowledge/tech/poc-equivalence-matrix.md` usa tres estados (`ported` / `superseded` / `out-of-scope-M3`). La revisión humana identificó que esta clasificación es demasiado permisiva: por ejemplo, "cubre una capacidad relacionada" no equivale a "reproduce las aserciones del test".

**Objetivo:** sustituir la columna `Status` por una clasificación de tres grados:

- **`equivalencia-exacta`**: el test POC tiene un reemplazo directo (unit test o YAML case) que ejecuta la misma IR/componente y verifica el mismo output + mismo call-order + mismos args. Reproduce las aserciones, no sólo las capacidades.
- **`cobertura-parcial`**: el reemplazo ejercita el pipeline relevante pero no reproduce todas las aserciones del test POC (p. ej. verifica el error pero no la cadena completa de llamadas, o verifica un subconjunto de requirements). Debe listar explícitamente qué aserciones quedan sin cubrir.
- **`aplazado`**: no se reproduce en M3. Destino en roadmap (RM-04/06/07/M4) obligatorio, no opcional.

Re-evaluar las 10 filas actuales con esta rúbrica. Esperado tras la re-evaluación (sujeto a revisión humana en el entregable final):

- Test 1 (v0.1 operators): `equivalencia-exacta` si todos los operators quedan ejercitados; `cobertura-parcial` en caso contrario con lista de operators pendientes.
- Test 3 (v0.3 imports+audits): `cobertura-parcial` casi seguro — las nuevas suites ejercitan `kv_get`/`kv_set`/`ping` pero no la combinación completa del POC (audit log con sub-maps).
- Test 10 (unknown host caps fail explicitly): `equivalencia-exacta` o `cobertura-parcial` según granularidad verificada.
- Tests 5, 6, 7, 9: siguen `aplazado` con destino explícito.

Entregable final: matriz re-evaluada por humano, no por el agente.

### H6 — Mapeo verificable INI-003 success criteria → implementación actual

**Estado actual:** INI-003 lista 5 criterios de salida (líneas 75-83 de `INI-003-rm-03-runtime-acceptance-extraction.md`). AN-019 Learning afirma "INIT-03-03 cumplido" sin contrastarlos uno a uno con evidencia.

**Precisiones aprobadas:**
- **OQ-03**: la decisión final sobre los criterios INI-003 se toma en una **revisión humana posterior** a AN-020. AN-020 produce evidencia y propuesta, no modifica unilateralmente los criterios originales ni declara M3.
- **Ajuste 6**: el criterio **3** sobre "capability no listada en el manifiesto falla al cargar" NO se satisface literalmente hoy. El runtime verifica `HostPolicy` en **pre-instantiate** (dentro de `invoke`), no durante **`load_component`**. Un componente con imports no autorizados se carga sin error; el error surge al invocar. "Manifest-driven policy at load-time" queda **fuera de alcance de AN-020** (no es trivial y arrastra INIT-06-01).

**Objetivo:** para cada criterio INI-003, producir evidencia verificable y una **clasificación propuesta** (no decisión). La decisión final queda para revisión humana posterior.

| # | Criterio INI-003 (textual) | Evidencia disponible | Clasificación propuesta |
|---|---|---|---|
| 1 | "Las 10 pruebas `runner` de la POC se reproducen verdes en el nuevo repo." | Matriz H5 re-evaluada. | **Requiere-decisión-humana**: si tras H5 quedan filas `cobertura-parcial` o `aplazado`, el criterio NO se cumple literalmente. Propuesta a decidir: reinterpretar como "cobertura regresiva documentada en matriz 1:1 con cada aplazamiento justificado y con destino de roadmap". AN-020 presenta la matriz, no la decisión. |
| 2 | "Imports no autorizados y capabilities faltantes producen `RuntimeError` tipado — nunca `0`/`false` por defecto." | AN-015 (`core_results_to_component` sin fallback a 0), AN-019 `UnauthorizedImport`, tests negativos. H3 cierra la última grieta (StateReader miss). | **Cerrable-con-evidencia** tras implementar H3. |
| 3 | "Deny-by-default verificado: capability no listada en el manifiesto falla al cargar." | AN-011/15 `HostPolicy::deny_all`, `wit_policy.rs`, `negative_import_not_authorized_fails_pre_instantiate`. **Pero el check es pre-instantiate (invoke), NO load-time**. | **Requiere-decisión-humana**: implementación actual es más tardía que la literal. Propuesta a decidir: (a) reinterpretar como "pre-execution" (= cualquier momento antes de ejecutar guest code), aceptando el comportamiento actual; (b) exigir check en `load_component_with_policy` antes de M3 (implica abrir una sub-tarea bloqueante, fuera de AN-020); (c) aplazar explícitamente a INIT-06-01 (policy loader manifest-driven) y marcar el criterio como parcial. AN-020 presenta las tres opciones sin elegir. |
| 4 | "Línea base técnica publicada en `knowledge/tech/baselines/` con cold start, warm invocation y memoria por instancia." | AN-012/17 `runtime-baseline.md` con cold/warm/cache-hit. **Memoria por instancia NO está medida** (diferida a INIT-06-02 por AN-017 Learning y confirmada fuera de alcance de AN-020). | **Requiere-decisión-humana**: ¿reinterpretar "memoria por instancia" como diferida (criterio parcial con destino explícito en INIT-06-02), o considerar el criterio no satisfecho? AN-020 no mide memoria (fuera de alcance). |
| 5 | "Lint CI verifica que `runtime` no depende de `compiler` ni del LLM." | `cargo tree -p agent-native-runtime --prefix none --edges normal \| grep -i compiler` verificado local tras AN-019. | **Requiere-implementación-trivial**: el comando existe pero no es un paso explícito en `.github/workflows/ci.yml`. AC: añadir paso dedicado en CI. Clasificación tras ese paso → **cerrable-con-evidencia**. |

**Entregable:** sección nueva en el fichero de la iniciativa (`INI-003-rm-03-runtime-acceptance-extraction.md`) con esta tabla como append, **sin cambiar** los 5 criterios originales. La resolución humana de los criterios 1, 3 y 4 queda explícitamente fuera de AN-020.

## Target behavior

### Cambios de API

- `Expected`: sin cambios de forma, con comparación de record implementada.
- `ExpectedHostCall { import: String, args: Vec<...> }` (rename de `function`).
- `Case.args: Vec<ArgValue>` con `enum ArgValue { I32(i32), Bool(bool) }`.
- `HostStubRegistry` sin cambios.
- `StubKind::StateReader` sin cambios de forma; el closure retorna `wasmtime::Error` en vez de 0 en miss.
- `run_component_case`: ruta nueva para payloads Component-Model sin host imports o con imports totalmente autorizados por policy sin stubs declarados → delega a `Component::invoke`.

### Fixtures nuevas / migradas

- `tests/suites/component-smoke.yaml` — ≥1 case v0.5 sin host imports.
- `tests/suites/kv-read.yaml` — migrar `read-missing-key-defaults-to-zero` a `kv-read-negative.yaml::read-missing-key-fails-explicit` o inline en test binario.
- Nueva suite `tests/suites/records.yaml` con `expected:` record (reutilizando IR tipo `point_sum`).
- Las 4 suites existentes con `expected_host_calls` migradas de `function:` a `import:`.

### Documentos nuevos

- `knowledge/tech/poc-yaml-contract.md` — contrato formal del YAML AN-020 vs POC: campos idénticos, renombrados, omitidos deliberadamente, con destino de roadmap para los omitidos.

### Documentos migrados

- `knowledge/tech/poc-equivalence-matrix.md` — re-evaluado con 3 clasificaciones H5.
- `knowledge/delivery/initiatives/INI-003-...md` — tabla H6 añadida con clasificación por criterio.

## Acceptance criteria

Cada AC lleva su comando o artefacto de verificación. Si una comprobación falla, el AC no se da por cumplido.

1. **AC-01 (H1 — CM sin imports funcionales)**: `cargo test -p agent-native-runtime --test executor_yaml` incluye un test cuyo cuerpo carga un componente v0.5 sin imports funcionales (ni `ComponentFunc` ni funciones en `ComponentInstance`) y corre un suite verde. Verificación: salida del test identifica explícitamente la ruta "Component-Model, no functional imports" en una evidencia o en el nombre del test.
2. **AC-02 (H1 — rechazo de CM con imports funcionales sin stub)**: test adicional carga un componente CM con al menos un import funcional **autorizado por policy pero sin stub** y verifica que `CaseStatus::Error` lleva evidencia con el patrón `"functional import '<iface>#<op>' requires a stub (RM-07)"`.
3. **AC-03 (H2 — records por nombre+tipo+valor)**: suite verde con `expected:` record no trivial (≥2 campos). Más 3 tests negativos:
   - campo esperado ausente del resultado → `Fail` con evidencia `"expected field '<name>' missing in result record"`.
   - campo del resultado no esperado por el YAML → `Fail` con evidencia `"unexpected field '<name>' in result record"`.
   - campo con tipo distinto de `S32` → `Fail` con evidencia que incluya el tipo encontrado via `component_val_kind`.
4. **AC-04 (H3 — fail-explicit en StateReader)**: `grep -n "unwrap_or(0)" crates/runtime/src/testing/executor.rs` → 0 matches. Test negativo verifica que una clave ausente produce `CaseStatus::Error` con evidencia que contenga la clave entera entrante y el nombre del stub.
5. **AC-05 (H4a — rename import canónico + alias function)**: 
   - YAML con `import:` solo → parsea verde.
   - YAML con `function:` solo → parsea verde (alias).
   - YAML con ambos a la vez → `InvalidYaml` cuyo mensaje incluya "both `import` and `function` set".
6. **AC-06 (H4a — compat semántica short vs qualified)**: 
   - `import: host.kv_get` + call registrada `host.kv_get` → match.
   - `import: kv_get` (short) + call registrada `host.kv_get` → match (regla documentada).
   - `import: .op` o `import: host.` (ambiguo) → `InvalidYaml`.
7. **AC-07 (H4b — args booleans)**: ≥1 case con `args: [..., true]` verde y ≥1 case con `args: [..., false]` verde. Verificación: salida del test lee la IR correspondiente que declara el parámetro `bool`.
8. **AC-08 (H4 — host_state sub-maps extra rechazados)**: suite con `host_state: { credit_limits: {...} }` → `InvalidYaml` cuyo mensaje incluya "unsupported host_state key `credit_limits`" y referencia a `poc-yaml-contract.md`.
9. **AC-09 (H4 — raíz sin `suite`/`spec_version` rechazada)**: YAML POC estilo `narrative-005.yaml` (raíz = `cases:` directo) → `InvalidYaml`. El contrato documenta esta decisión en `poc-yaml-contract.md`.
10. **AC-10 (H4c — contrato publicado)**: `test -f knowledge/tech/poc-yaml-contract.md` && grep en el fichero devuelve las 4 cabeceras `## Campos idénticos`, `## Campos renombrados o normalizados`, `## Campos deliberadamente omitidos`, `## Estructuras-raíz no soportadas`.
11. **AC-11 (H5 — matriz re-evaluada con 3 grados)**: `grep -cE "equivalencia-exacta|cobertura-parcial|aplazado" knowledge/tech/poc-equivalence-matrix.md` ≥ 10. Cada una de las 10 filas incluye una celda "Evidencia" con test, path o commit identificable (no genérico). Clasificación final sujeta a revisión humana (no al agente).
12. **AC-12 (H6 — tabla INI-003 publicada como append)**: `grep -cE "cerrable-con-evidencia|requiere-implementación|requiere-decisión-humana" knowledge/delivery/initiatives/INI-003-*.md` ≥ 5. Los 5 criterios originales de INI-003 se preservan intactos en el fichero; la tabla aparece como append separado.
13. **AC-13 (regresión AN-019)**: `cargo test -p agent-native-runtime --test executor_yaml` → todos verdes, incluyendo los 17 anteriores migrados a la nueva semántica (field rename + StateReader miss como error) y los nuevos de AC-01..AC-09.
14. **AC-14 (ADR-002 preservado)**: `cargo tree -p agent-native-runtime --prefix none --edges normal | grep -i compiler` → exit 1. Paso explícito añadido a `.github/workflows/ci.yml`.
15. **AC-15 (workspace verde)**: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` → todos verdes. CI remota 3-OS verde en el commit de implementación.
16. **AC-16 (INI-003 y M3 no declarados)**: ni `knowledge/delivery/initiatives/INI-003-*.md` ni ningún otro fichero cambian el `status` de la iniciativa ni declaran M3. La tabla H6 es append; los 5 criterios originales siguen literalmente iguales.

**Nota sobre el conteo:** son 16 AC verificables; el borrador v0 planteaba 11 pero varios eran compuestos. La precisión del usuario exige evidencia verificable por AC, así que los AC compuestos H4 y H6 se desdoblan.

## Out of scope (explícito)

- Evolución integral v1→v2 (esclusas) → M4.
- Proveedores generales de host (más allá de los 3 stubs declarativos existentes) → INIT-06-01.
- **Políticas manifest-driven + verificación en `load_component`** (el criterio INI-003 #3 literal queda como decisión humana; su implementación va a INIT-06-01).
- **Medición de memoria por instancia** → INIT-06-02.
- WASI preview2 → RM-07.
- Optimizaciones de rendimiento (caché warm, instance pooling) → RM-06/08.
- `host_state` sub-maps distintos de `kv:` → INIT-07-02; AN-020 los rechaza en parse.
- Component Model host stubs reales (wiring de `wasmtime::component::Linker::func_wrap` por interface) → RM-07.
- **Declaración formal de M3 y cierre formal de INI-003**: AN-020 produce evidencia y propuestas; la revisión humana decide por separado.
- Modificación de los 5 criterios originales de INI-003 — AN-020 sólo añade append, no edita.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p agent-native-runtime --test executor_yaml
cargo tree -p agent-native-runtime --prefix none --edges normal | grep -i compiler || echo OK
# Verificación H4c:
test -f knowledge/tech/poc-yaml-contract.md
# Verificación H5:
grep -E 'equivalencia-exacta|cobertura-parcial|aplazado' knowledge/tech/poc-equivalence-matrix.md | wc -l  # debe ser 10
# Verificación H6:
grep -E 'cerrable-con-evidencia|requiere-implementación|requiere-decisión-humana' knowledge/delivery/initiatives/INI-003-*.md | wc -l  # debe ser 5
```

## Definition of Done

- 6 hallazgos H1-H6 cerrados con evidencia específica (tests + documentos).
- 17 tests previos de `executor_yaml.rs` migrados y verdes.
- CI remota verde 3 OS en el commit de implementación.
- Documentos nuevos (`poc-yaml-contract.md`) y migrados (`poc-equivalence-matrix.md`, `INI-003`) publicados.
- WI entregado para revisión humana antes de `kaddo learn`.
- INI-003 y M3 **no** declarados en este WI. Las dos decisiones humanas (crit 1 y crit 4 de INI-003) explícitamente presentadas pero no resueltas.

## Criterios de salida verificables de M3 (propuesta, sujeta a decisión humana)

Como parte de este WI se propone, pero no se declara, el siguiente set verificable para la declaración posterior de M3 — a aprobar por separado:

1. **M3-CRIT-1 (reinterpreta INI-003 crit 1):** La matriz POC publicada no tiene filas sin clasificar; cada aplazamiento tiene destino de roadmap específico y una justificación de 1-2 líneas.
2. **M3-CRIT-2 (hereda INI-003 crit 2):** Ningún fallback silencioso a 0/false en el runtime; verificable con `grep -rn 'unwrap_or(0)' crates/runtime/src/` → 0 matches en paths que ejecuten lógica host.
3. **M3-CRIT-3 (hereda INI-003 crit 3):** Al menos 3 tests independientes verifican deny-by-default a tres niveles (core module, WIT interface, WIT operation).
4. **M3-CRIT-4 (reinterpreta INI-003 crit 4):** `knowledge/tech/baselines/runtime-baseline.md` publicado con cold/warm/cache-hit por path (ya existe tras AN-017). Memoria por instancia **explícitamente diferida** a INIT-06-02 con criterio de aceptación de esa iniciativa.
5. **M3-CRIT-5 (hereda INI-003 crit 5):** CI remota incluye paso explícito `cargo tree -p agent-native-runtime --edges normal | ! grep -q agent-native-compiler`.
6. **M3-CRIT-6 (nuevo, derivado de AN-019 Learning):** Executor YAML ejecuta ≥1 suite Component-Model y ≥1 suite Core-WASM, cubriendo ambos payload-paths del runtime.
7. **M3-CRIT-7 (nuevo):** La matriz POC + `poc-yaml-contract.md` + tabla INI-003 están presentes y firmadas por el humano en commits de la rama que declara M3.

Esta propuesta se incorpora al WI como **sugerencia**, no como criterio aprobado. La decisión de adoptarlos (y su wording final) queda explícitamente fuera de AN-020.

## Open questions

Las 3 OQs del borrador v0 quedan **resueltas** por decisión humana del 2026-10-18 e incorporadas al cuerpo del WI:

- **OQ-01 (resuelta)**: `import` canónico, `function` alias temporal, ambos a la vez → rechazo. Short-vs-qualified normalizado con regla explícita (ver H4 y `poc-yaml-contract.md`).
- **OQ-02 (resuelta)**: `host_state` sub-maps extra → rechazo en parse. Sin warnings.
- **OQ-03 (resuelta)**: criterios finales INI-003 → revisión humana posterior a AN-020. Este WI entrega evidencia + propuesta, no decide.

**OQs residuales (ninguna bloqueante para iniciar):** —.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-03 |
| Roadmap iniciativa | INIT-03-03 — Executor de aceptación YAML (cierre de brechas) |
| Hito | M3 (NO declarado en este WI) |
| ADRs | ADR-002, ADR-004, ADR-005 |
| Prerrequisitos | AN-019 completado + CI verde (`5ee7a01`) |
| Fuera de alcance | M4 (esclusas v1→v2), INIT-06-01 (host providers), RM-06/08 (perf), RM-07 (WASI + Component Model stubs reales), INIT-07-02 (host state rico) |
| Origen | Revisión humana post-AN-019 (2026-10-18), 6 hallazgos verificables |

## Learning

**Lo que salió como se esperaba:**

- La separación entre **autorización** y **resolución** (precisión H1) se materializó limpiamente: `first_functional_import` itera los imports del Component Model y basta con detectar el primer `ComponentFunc` o función en un `ComponentInstance` para decidir si delegar a `Component::invoke` o devolver `HostError`. El resultado es que `probe-use-types` (sólo type import) corre con `invoke` reutilizado; `smoke-v4` (imports funcionales) se rechaza con mensaje `functional import '<iface>#<op>' requires a stub (RM-07)`.
- Las comparaciones de record por nombre + tipo + valor **con detección de campos adicionales** se implementaron con un `BTreeMap` auxiliar que se vacía campo por campo durante la iteración de `expected`; lo que quede en el map después es "adicional". Esta forma es tan explícita como la especificación y evita heurísticas.
- `import` canónico + alias `function` con rechazo mutuamente excluyente cuajó con un `#[derive(Deserialize)]` sobre `ExpectedHostCallRaw` y un `impl<'de> Deserialize` manual sobre `ExpectedHostCall` que materializa la regla. El error en caso de ambos campos lleva literalmente las palabras `both`, `import` y `function` para que el test negativo pueda verificarlas sin comparación frágil.
- La regla de normalización short-vs-qualified queda contenida en una función de ~10 líneas (`import_matches`) con rechazo explícito de formas ambiguas (`host.`, `.op`) en el deserializador. Documentada en `poc-yaml-contract.md`.
- `deny_unknown_fields` en `Suite`, `Case` y `HostState` dio la cobertura H4/H8/H9 "gratis": sub-maps extra en `host_state`, raíces sin `suite:`/`spec_version:` y campos desconocidos se rechazan con el mismo mecanismo, sin lógica ad-hoc.
- La matriz POC con 3 grados verificables (`equivalencia-exacta`, `cobertura-parcial`, `aplazado`) y la tabla append de INI-003 con 3 clasificaciones (`cerrable-con-evidencia`, `requiere-implementación`, `requiere-decisión-humana`) devuelven el control a quien debe decidirlo: el humano. AN-020 presenta evidencia, no decisiones.

**Lo inesperado:**

- **Windows + wasmtime 27 + `anyhow::Error` desde una closure de `Linker::func_wrap`** → `STATUS_STACK_BUFFER_OVERRUN` (0xC0000409), "panic in a function that cannot unwind". El trap path usa `setjmp`/`longjmp` (`raise_user_trap`) que colisiona con el unwinder de Rust en builds debug Windows x64. Fix pragmático: flag en `HostCtx::misses` desde la closure (siempre retorna `i32`) y conversión a `CaseStatus::Error` en el executor tras la invocación. Comportamiento observable idéntico; dependencia de plataforma eliminada. Documentado en `host_ctx.rs`.
- **`probe-use-types.ir.json` tiene un type import** (`an:probe/types@0.1.0::pt`), que la ruta `precheck_component_imports` existente rechaza bajo `HostPolicy::deny_all`. Resuelto autorizando la interfaz `an:probe/types@0.1.0` desde el test; el AN-014 Learning sobre "type-only imports auto-resolve" aplica a Wasmtime, no al pre-check del runtime. Esta ambigüedad quedó anotada implícitamente en la matriz POC fila 10 (criterio INI-003 #3).
- **Clippy pedantic `single_match_else`** reclamó `match data.kv.get(..) { Some(v) => v, None => ... }` como "para un único pattern usa `if let`". Patch mecánico; vale la pena recordar que `single_match_else` no acepta ni siquiera dos ramas en algunos builds.
- **Compat semántica short-vs-qualified** era menos obvia de lo esperado: la POC usa `check_credit` (sólo op) pero nuestro runtime registra `host.check_credit`. La regla única "si contiene '.', literal; si no, compara contra op" fue menos natural de deducir que de explicar. Documentarla en `poc-yaml-contract.md` **antes** de implementarla ahorró una iteración.
- El criterio INI-003 **#3** ("capability no listada en el manifiesto falla al cargar") no se cumple literalmente: el check ocurre pre-instantiate (en `invoke`), no en `load_component`. AN-019 Learning lo omitió. AN-020 lo marca explícito como decisión humana requerida (append INI-003); no es un bug sino una interpretación de "al cargar" que arrastra INIT-06-01 (policy loader manifest-driven).

**Riesgos materializados:**

- R-020-01 (rename `function`→`import` rompe suites internas): **mitigado**. Alias temporal activo; migración de las 5 suites + 11 cases en el mismo commit.
- R-020-02 (`host_state` sub-maps extra en fixtures POC): **confirmado y documentado**. Fixtures como `public-003.yaml` (`credit_limits`), `resource-reservation.yaml` (`tenant_status`, `quota`) quedan **aplazadas** en la matriz POC con destino INIT-07-02.
- R-020-03 (`Expected::Record` requiere resolver nombres contra schema IR): **no materializado**. La comparación cae directamente sobre `Val::Record` de wasmtime, cuyos nombres son los del WIT emitido. Normalización documentada: kebab-case único.
- R-020-04 (eliminar `unwrap_or(0)` cambia semántica de `kv-read.yaml`): **mitigado por rediseño**. `read-missing-key-defaults-to-zero` eliminado; reemplazado por test negativo dedicado `ac04_state_reader_miss_fails_explicit`; caso B de `isolation.yaml` ahora declara su `kv` explícitamente.

**Deuda y seguimientos:**

- **Component Model host stubs reales** → RM-07. AN-020 añade la discriminación type-only vs funcional; el wiring de `wasmtime::component::Linker::func_wrap` por interfaz WIT sigue diferido.
- **`host_state` tipos ricos** (strings, maps anidados, `credit_limits`, `tenant_status`, etc.) → INIT-07-02.
- **Argumentos no primitivos** (strings, floats, listas, enums) → INIT-07-02.
- **Policy loader + check en `load_component`** → INIT-06-01. La decisión sobre si esto bloquea M3 o se reinterpreta queda en la revisión humana de INI-003 criterio 3.
- **Memoria por instancia** → INIT-06-02. AN-020 confirma el aplazamiento del criterio INI-003 #4.
- **Esclusas v1→v2 integrales** → M4 (INIT-04-03/04-05).
- **Composición multi-componente** → RM-07.
- **Suites extendidas** (operadores v0.1 completos, bodies compuestos con varios host calls en una función) → RM-04/06. Trazadas en la matriz POC filas 1 y 3 (ambas `cobertura-parcial`).

**Lo que esta iniciativa establece:**

- **El formato YAML AN-020 tiene un contrato normativo publicado** (`poc-yaml-contract.md`). Cualquier extensión futura entra por un WI que lo amplíe; `deny_unknown_fields` enforcement en código.
- **La matriz POC es ejecutable**: cada fila lleva evidencia identificable (test, path, commit). Un revisor puede desafiar la clasificación propuesta sin leer el agente, sólo el fichero.
- **INI-003 queda listo para decisión humana** sin cambios unilaterales: 2 criterios cerrables, 3 que requieren decisión (crit 1, 3, 4). La resolución formal se toma en una revisión posterior; AN-020 no la fuerza.
- **El patrón `misses` en HostCtx** generaliza: cualquier futura verificación opcional del host (contadores, limits, trace hooks) puede añadir un campo nuevo a `HostCtx` y el executor convertirlo a evidencia tipada sin tocar la closure, evitando el camino `anyhow`-cross-FFI que falla en Windows debug.
- **AN-019 Learning se complementa con AN-020**: la afirmación "INIT-03-03 cumplido" de AN-019 era prematura en 3 de 5 criterios INI-003 cuando se contrasta con el texto literal. La disciplina de contrastar criterios literales uno a uno antes de declarar cierre queda establecida como práctica.

### Evidencias de ejecución

- Commit implementación: `56f1888` (feat(AN-020)) → push a `origin/main` (`5ee7a01..56f1888`). CI remota [#39189880092](https://github.com/Kaddo-kdd/agent-native/actions/runs/39189880092) **success** 3/3 OS en 1m47s.
- Local Windows: `cargo test -p agent-native-runtime --test executor_yaml` → 30 tests verdes (17 AN-019 migrados + 13 nuevos para AC-01..AC-10).
- Local Windows: `cargo test --workspace` → 60+ tests verdes (sin regresión).
- Local Windows: `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- Local Windows: `cargo fmt --all -- --check` → clean.
- Local Windows: `cargo tree -p agent-native-runtime --prefix none --edges normal | grep -i compiler` → exit 1 (ADR-002 preservado).
- Verificación AC-04: `grep -c "unwrap_or(0)" crates/runtime/src/testing/executor.rs` → 0.
- Verificación AC-10: `test -f knowledge/tech/poc-yaml-contract.md` → OK, con las 4 cabeceras requeridas.
- Verificación AC-11: 17 ocurrencias de las 3 etiquetas en `poc-equivalence-matrix.md` (≥10 filas con clasificación).
- Verificación AC-12: 12 ocurrencias de las 3 etiquetas en `INI-003-...md` (≥5 criterios con clasificación).

────────────────────────
Agent: work-item-agent
Produced:
  knowledge/delivery/work-items/completed/AN-020-m3-closure-gaps.md
  crates/runtime/src/testing/{suite,executor,host_ctx}.rs (edits)
  crates/runtime/tests/executor_yaml.rs (30 tests)
  crates/runtime/tests/suites/*.yaml (7 ficheros: 5 migrados + 2 nuevos)
  knowledge/tech/poc-yaml-contract.md (nuevo)
  knowledge/tech/poc-equivalence-matrix.md (re-evaluado)
  knowledge/delivery/initiatives/INI-003-rm-03-runtime-acceptance-extraction.md
    (append sin modificar criterios originales)

Pendiente de autorización humana explícita (fuera de este cierre):
  1. Decisión humana sobre los 3 criterios INI-003 marcados
     `requiere-decisión-humana` (crit 1, 3, 4).
  2. Declaración formal del hito M3 (si procede tras la decisión).
  3. Ejecución de `kaddo learn` (si procede como paso adicional).
────────────────────────
