---
title: POC Equivalence Matrix — runner unit tests → Agent-Native crates
owner: AN-020
status: baseline-2026-10-19 (AN-020 patch 1 — rows 2/4/8 downgraded)
scope: knowledge/tech
---

# POC Equivalence Matrix

Rastreabilidad **1:1** entre las 10 pruebas unitarias de
`_reference/poc-agent-native/crates/runner/src/main.rs` y los mecanismos
reproducidos en el workspace nuevo. **Esta matriz, no el conteo de cases
ni el `test result: ok`, es la evidencia de cobertura regresiva para la
futura declaración de M3.**

## Clasificación

AN-020 (hallazgo H5) sustituye las etiquetas iniciales
`ported`/`superseded`/`out-of-scope-M3` por tres grados verificables:

- **`equivalencia-exacta`** — Existe un test o case que ejecuta la misma
  IR o componente que el test POC y verifica las **mismas aserciones**
  (output + orden exacto de llamadas al host + args). La propuesta de
  esta clasificación es verificable por cualquier revisor leyendo la
  evidencia enlazada.
- **`cobertura-parcial`** — El reemplazo ejercita la capacidad relevante
  del test POC, pero **no reproduce todas sus aserciones**. La columna
  "Aserciones no cubiertas" lista explícitamente lo que queda sin
  cubrir, con destino de roadmap.
- **`aplazado`** — No se reproduce en M3. La columna "Destino" lleva un
  identificador de roadmap específico (RM/INIT/M).

**La clasificación de cada fila es una propuesta de AN-020 sujeta a
revisión humana.** La columna "Evidencia" permite a un revisor
contrastar directamente la propuesta sin necesitar explicación
adicional.

## Matriz

| # | POC test (`runner::main.rs`) | Evidencia (test / commit) | Aserciones no cubiertas (si procede) | Clasificación propuesta | Destino si procede |
|---|---|---|---|---|---|
| 1 | `executes_all_ir_v0_1_operators_in_wasmtime` (L1432) | `crates/runtime/tests/smoke.rs::identity_component_returns_argument` + `crates/runtime/tests/executor_yaml.rs::regression_smoke_suite_passes` (`suites/smoke.yaml`, 3 cases). Operadores v0.1 (`param`, aritmética) ejercitados por `compile_ir_to_wasm` + `invoke`. | El test POC original cubre un conjunto más amplio de operadores v0.1 (comparaciones, `if`, aritmética) sin verificar exhaustivamente aquí. | **cobertura-parcial** | Suites extendidas con casos aritméticos y de control de flujo → RM-04/06 (expansión de regresión), no bloqueante para M3. |
| 2 | `executes_v0_2_record_and_call_in_wasmtime` (L1522) | `crates/compiler/tests/poc_regression.rs::valid_v0_2_record_and_call_programs` (AN-007 RM-02 regression; sólo verifica que la IR v0.2 del POC compila sin diagnóstico — **no la ejecuta ni comprueba output**). AN-020 `ac01` ejerce record-return pero sobre un fixture v0.5 distinto (`probe-use-types.ir.json`), **no sobre la IR v0.2 del POC**. | Ejecución end-to-end de la IR v0.2 exacta del POC + aserciones sobre output y call-order. | **cobertura-parcial** (**reclasificado 2026-10-19**; previamente `equivalencia-exacta`) | Fixture POC v0.2 cargado y ejecutado por el runtime nuevo con las mismas aserciones → RM-04/06. |
| 3 | `executes_v0_3_imports_and_audits_in_wasmtime` (L1594) | `crates/runtime/tests/executor_yaml.rs`: `regression_kv_write_suite_passes`, `regression_noop_suite_passes`, `ac06_short_name_matches_qualified_recorded_call` + `crates/runtime/tests/host_errors.rs::core_component_with_declared_import_is_unauthorized` + `ac04b_statewriter_suppressed_after_reader_miss` (chain `kv_get` + `kv_set`). | El test POC original ejecuta `kv_get` + `emit_audit` **con output semántico específico** sobre la IR v0.3 del POC completa. AN-020 ejercita stubs individualmente + un chain minimal; la combinación completa de la IR POC queda sin verificar. | **cobertura-parcial** | Fixture POC v0.3 completo bajo el executor nuevo → RM-04/06. |
| 4 | `executes_v0_4_component_in_wasmtime` (L1622) | `crates/runtime/tests/e2e_v05.rs` ejecuta componentes **v0.5**, no el componente **v0.4** exacto del POC. `crates/runtime/tests/executor_yaml.rs::ac02_component_with_functional_import_without_stub_errors` usa `smoke-v4.ir.json` pero sólo verifica el fail-explicit de imports funcionales sin stub, no la ejecución semántica. | Ejecución semántica del componente v0.4 exacto del POC con sus aserciones de output; `smoke-v4` corre sólo en modo "verifica rechazo". | **cobertura-parcial** (**reclasificado 2026-10-19**; previamente `equivalencia-exacta`) | Ejecución semántica de `smoke-v4` con stubs CM reales → RM-07 (CM linker wiring). |
| 5 | `executes_v0_5_composed_components_in_wasmtime` (L1664) | — Composición multi-componente no soportada en runtime nuevo. | Todas las aserciones del POC quedan sin cubrir. | **aplazado** | **RM-07** — composición multi-componente + WASI. |
| 6 | `executes_reservation_coordinator_in_wasmtime` (L1703) | `crates/runtime/tests/nested_records.rs` (AN-014 nested records) parcial. El coordinator completo requiere host provider con `tenant_status`, `quota` y lógica domain-specific. | Ejecución end-to-end del coordinator con stubs completos. | **aplazado** | **INIT-06-01** (host provider) + **INIT-07-02** (host state rico con `tenant_status`, `quota`). |
| 7 | `test_cumulative_reservation_evolution_v1_v2_v3` (L1739) | — Esclusas v1→v2 integrales no implementadas. | Todas las aserciones del POC. | **aplazado** | **M4** — INIT-04-03/04-05 (esclusas v1→v2 integrales). |
| 8 | `test_dependency_failures_never_fallback_to_zero` (L1819) | `crates/runtime/src/invoke.rs::core_results_to_component` sin fallback silencioso (AN-015) + `crates/runtime/tests/executor_yaml.rs::ac04_state_reader_miss_fails_explicit` + `ac04b_statewriter_suppressed_after_reader_miss`. | El test POC original ejecuta cadenas de dependencias **multi-paso** en una sola función y verifica que ningún paso cae a `0`/`false` por defecto. AN-020 cubre un miss aislado + un chain minimal de 2 host calls; cadenas más largas quedan sin verificar. **Además**: el executor nuevo detecta el miss **al final de la invocación** (via `HostCtx::poisoned`), no inmediatamente — el guest puede continuar computando con `0` provisional hasta que termine (patch AN-020 D1 opción A). El POC original interrumpe en el momento del miss. | **cobertura-parcial** (**reclasificado 2026-10-19**; previamente `equivalencia-exacta`) | Cadenas multi-paso + interrupción inmediata (Wasmtime trap desde closure sin fast-fail en Windows) → investigación post-M3 sobre propagación segura de errores de host; cadenas → RM-04/06. |
| 9 | `test_unseen_llm_domain_canal_lock_v1_and_v2` (L1919) | AN-014 Learning: el verificador v0.5 rechaza records con campos que referencian otros records por diseño; la propiedad end-to-end LLM-domain lock requiere el flujo integral orchestrator → compiler → runtime. | Esclusas semánticas v1↔v2 ejercitadas sobre componentes LLM-generados. | **aplazado** | **M4** — integración orchestrator/validation + esclusas v1→v2. |
| 10 | `test_unknown_host_capabilities_fail_explicitly` (L1992) | `crates/runtime/tests/wit_policy.rs::allow_wit_operation_is_granular` + `wit_policy.rs::same_package_type_import_is_not_implicitly_allowed` + `crates/runtime/tests/executor_yaml.rs::negative_import_not_authorized_fails_pre_instantiate` + `negative_stub_declared_for_unauthorised_import_rejects_suite`. **Pero**: el check se hace pre-instantiate durante `invoke`, no durante `load_component`; el POC original comprueba al cargar. | Momento del fail-explicit: AN-020 lo hace al invocar, POC al cargar. | **cobertura-parcial** | **INIT-06-01** — policy loader + check en `load_component` manifest-driven. Decisión humana D2 (opción C) difiere este criterio literal a INIT-06-01. |

## Resumen propuesto (sujeto a revisión humana)

**Actualizado 2026-10-19 (patch AN-020 corrige rows 2, 4, 8).**

- **`equivalencia-exacta`**: **0 de 10**. Las filas 2, 4 y 8 que
  previamente figuraban en este grado se **reclasificaron a
  cobertura-parcial** tras verificar que su evidencia no reproduce la
  misma IR/componente del POC o no cubre todas las aserciones
  históricas.
- **`cobertura-parcial`**: **6 de 10** (tests **1, 2, 3, 4, 8, 10**),
  cada uno con aserciones específicas no cubiertas y destino de
  roadmap identificado.
- **`aplazado`**: 4 de 10 (tests **5, 6, 7, 9**), cada uno con destino
  explícito en RM-07, INIT-06-01/07-02, M4.

**Interpretación de INI-003 criterio 1:** la frase literal "las 10
pruebas se reproducen verdes" no se cumple en ningún grado. La
decisión humana **D1 (opción A)** aceptó esta matriz como evidencia
de "cobertura regresiva documentada con aplazamientos justificados",
sin afirmar reproducción literal. Ver append INI-003 2026-10-19.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| WI | AN-020 — hallazgo H5 |
| Rúbrica | 3 grados verificables: equivalencia-exacta / cobertura-parcial / aplazado |
| Reemplaza | AN-019 matriz (3 grados antiguos ya no se usan) |
| Decisión final | Revisión humana posterior |
