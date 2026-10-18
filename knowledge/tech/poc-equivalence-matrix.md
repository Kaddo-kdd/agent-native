---
title: POC Equivalence Matrix — runner unit tests → Agent-Native crates
owner: AN-019
status: baseline-2026-10-18
scope: knowledge/tech
---

# POC Equivalence Matrix

Rastreabilidad **1:1** entre las 10 pruebas unitarias de
`_reference/poc-agent-native/crates/runner/src/main.rs` y los mecanismos
reproducidos en el workspace nuevo. **Esta matriz, no el conteo de cases,
es la evidencia de cobertura regresiva para el cierre de M3.**

Convención de status:

- **ported** — reproducido con un test o case explícito en el workspace nuevo.
- **superseded** — ya reproducido por un WI previo (AN-007, AN-010, AN-014),
  cubierto como regresión en su propia suite.
- **out-of-scope-M3** — requiere capacidades fuera del alcance de M3
  (host provider real, esclusas v1→v2 integrales, composición de
  componentes); se reserva para RM-04/06/07.

| # | POC test (`runner::main.rs`) | Nuevo mecanismo | Status |
|---|---|---|---|
| 1 | `executes_all_ir_v0_1_operators_in_wasmtime` (L1432) | `crates/runtime/tests/smoke.rs::identity_component_returns_argument` + `crates/runtime/tests/executor_yaml.rs::smoke_suite_passes_three_identity_cases` (suite `smoke.yaml`, 3 cases) | ported |
| 2 | `executes_v0_2_record_and_call_in_wasmtime` (L1522) | `crates/compiler/tests/poc_regression.rs::valid_v0_2_record_and_call_programs` (AN-007 RM-02 regression) | superseded |
| 3 | `executes_v0_3_imports_and_audits_in_wasmtime` (L1594) | `crates/runtime/tests/executor_yaml.rs::kv_read_suite_passes_three_state_reader_cases` + `kv_write_suite_passes_two_state_writer_cases` + `noop_suite_passes_zero_arg_stub` (suites `kv-read.yaml`, `kv-write.yaml`, `noop.yaml`, 6 cases con `host_stubs`) y `crates/runtime/tests/host_errors.rs::core_component_with_declared_import_is_unauthorized` | ported |
| 4 | `executes_v0_4_component_in_wasmtime` (L1622) | `crates/runtime/tests/e2e_v05.rs` (AN-013 Component Model baseline) | superseded |
| 5 | `executes_v0_5_composed_components_in_wasmtime` (L1664) | pendiente — composición multi-componente queda para RM-07 | out-of-scope-M3 |
| 6 | `executes_reservation_coordinator_in_wasmtime` (L1703) | `crates/runtime/tests/nested_records.rs` (AN-014 nested records) parcial; coordinator completo requiere host provider | out-of-scope-M3 |
| 7 | `test_cumulative_reservation_evolution_v1_v2_v3` (L1739) | esclusas v1→v2 integrales en M4 (INIT-04-03/04-05) | out-of-scope-M3 |
| 8 | `test_dependency_failures_never_fallback_to_zero` (L1819) | `crates/runtime/src/invoke.rs::core_results_to_component` + `crates/runtime/tests/host_errors.rs` — AN-015 convirtió el fallback silencioso en `RuntimeError::ExecutionFailed` tipado | superseded |
| 9 | `test_unseen_llm_domain_canal_lock_v1_and_v2` (L1919) | validation agent + probe-set de IR v0.5 no-reachable (AN-014 learning) cubren la invariante en el verificador; la propiedad end-to-end entra en M4 | out-of-scope-M3 |
| 10 | `test_unknown_host_capabilities_fail_explicitly` (L1992) | `crates/runtime/tests/wit_policy.rs::allow_wit_operation_is_granular` + `crates/runtime/tests/executor_yaml.rs::negative_import_not_authorized_fails_pre_instantiate` + `negative_stub_declared_for_unauthorised_import_rejects_suite` | ported |

## Resumen

- **ported**: 3 de 10 (tests 1, 3, 10) — cubiertos directamente por nuevas
  suites YAML + tests runtime del nuevo workspace.
- **superseded**: 3 de 10 (tests 2, 4, 8) — cubiertos como regresión en WIs
  anteriores (AN-007, AN-013, AN-015).
- **out-of-scope-M3**: 4 de 10 (tests 5, 6, 7, 9) — requieren
  capacidades fuera del alcance declarado de M3: composición multi-componente,
  host provider real y esclusas v1→v2 integrales. Trazados a RM-04/06/07/M4.

Las 6 pruebas marcadas como `ported` o `superseded` constituyen la
cobertura regresiva de M3. Las 4 `out-of-scope-M3` quedan explícitamente
diferidas con destino de roadmap identificado.
