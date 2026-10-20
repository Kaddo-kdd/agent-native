---
id: AN-034
type: test
status: completed
completed_at: '2026-10-19'
created_at: 2026-10-19T00:00:00.000Z
title: Pruebas negativas y verificación de límites de capabilities
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/tests/manifest_negative_tests.rs
  - crates/runtime/tests/manifest_policy.rs
source_roadmap_initiative: INIT-06-01
source_work_item_candidate:
  - WI-CANDIDATE-06-01-02
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Host Capability Policy
  - Independent Runtime Execution
expected_value: >-
  Suite exhaustiva de pruebas negativas que audita el policy loader frente a
  intentos de escalamiento de privilegios, auto-concesión, manifiestos
  malformados, llamadas a operaciones WIT hermanas no autorizadas, aislamiento
  del caché del resolver y violaciones de deny-by-default.
risks:
  - >-
    R-TEST-01: Falsos positivos por pruebas que verifiquen errores de Wasmtime en
    lugar del error tipado del runtime. Mitigación: aserciones estrictas sobre las
    variantes de `RuntimeError` y `PolicyError`.
dependencies:
  - AN-033
source_signals:
  - 'Roadmap INIT-06-01: Auditado por suite de pruebas negativas'
  - 'ADR-004: fail-explicit, rechazo con error tipado pre-instanciación'
decision_candidates: []
related_decisions:
  - ADR-004
  - ADR-005
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
---

# AN-034 — Pruebas negativas y verificación de límites de capabilities

## Contexto y Motivación

Para confiar en la ejecución de componentes generados por agentes autónomos de IA, no basta con probar los casos de éxito (happy path). Se requiere una suite de pruebas de seguridad y robustez que intente activamente vulnerar o eludir el sistema de capabilities y demuestre que el runtime permanece seguro bajo cualquier circunstancia anómala en todas las rutas de ejecución.

## Invariantes Auditadas y Verificadas

1. **Deny-by-default:** Todo import no concedido explícitamente es bloqueado antes de la instanciación (`test_invoke_denies_unauthorized_component_model_import`).
2. **Precheck en todas las rutas con host context:** Demostración de que `invoke_core_with_host_ctx()` e `invoke_component_with_host_ctx()` ejecutan el precheck de imports **antes** de instanciar y antes de que se configure el linker o se invoquen stubs del host (`test_invoke_core_with_host_ctx_enforces_policy_before_stub_execution`).
3. **Aislamiento en el Caché del Resolver:** Cargar un componente con una política permisiva y posteriormente el mismo artefacto byte-idéntico con una política restrictiva demuestra que no hay fuga de privilegios a través del caché (`test_resolver_cache_does_not_leak_permissions_to_subsequent_loads`).
4. **No auto-concesión (No self-granting):** Un componente que declare capacidades irrestrictas en su manifiesto no concedidas por el host produce una política efectiva vacía y es rechazado (`test_self_granting_manifest_defeated_by_effective_policy`).
5. **Semántica WIT estricta:** Permitir una operación no autorizada o no coincidente dentro de una interfaz de tipo importada bloquea la interfaz completa (`test_wit_operation_partial_authorization_rejects_missing_operations`).
6. **Distinción explícita de errores:** Permiso denegado genera `RuntimeError::UnauthorizedImport`; permiso concedido pero sin implementación provista por el host genera `RuntimeError::InstantiationFailed` (`test_distinction_unauthorized_import_vs_instantiation_failed`).

## Criterios de Aceptación Cumplidos

1. La suite de pruebas cubre el 100% de los escenarios de rechazo especificados en todas las rutas de invocación y carga.
2. Cada fallo retorna un `RuntimeError` tipado (`UnauthorizedImport`, `PolicyError`, etc.) comprobable mediante pattern matching, nunca panics ni mensajes genéricos opacos.
3. Se verificó empíricamente que no hay fugas de privilegios ni escalamiento lateral entre interfaces u operaciones o por reutilización de caché.
