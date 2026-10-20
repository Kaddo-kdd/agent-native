---
id: AN-033
type: feature
status: completed
completed_at: '2026-10-19'
created_at: 2026-10-19T00:00:00.000Z
title: Policy loader de capabilities manifest-driven
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/src/manifest.rs
  - crates/runtime/src/policy.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/engine.rs
  - crates/runtime/src/resolver.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/tests/manifest_policy.rs
source_roadmap_initiative: INIT-06-01
source_work_item_candidate:
  - WI-CANDIDATE-06-01-01
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Host Capability Policy
  - Independent Runtime Execution
expected_value: >-
  Permite definir y cargar políticas de capabilities declarativamente mediante
  un manifiesto (YAML/JSON) con granularidad por interfaz WIT, operación WIT o
  par Core WASM, aplicando deny-by-default y un modelo de intersección estricto
  (requested ∩ granted) sin permitir auto-concesión de privilegios.
risks:
  - >-
    R-POL-02: Ambigüedad entre capacidades solicitadas por el componente y
    capacidades concedidas por el host. Mitigación: modelo formal de intersección
    donde el host mantiene el control de admisión final.
  - >-
    R-POL-03: Deserialización de manifiestos no confiables o corruptos.
    Mitigación: validación estricta con error tipado `PolicyError` sin panics.
dependencies:
  - AN-011
  - AN-015
  - AN-029
source_signals:
  - 'Roadmap INIT-06-01: Policy loader con allow-list por componente, deny-by-default'
  - 'ADR-004: fail-explicit y deny-by-default'
decision_candidates: []
related_decisions:
  - ADR-004
  - ADR-005
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
---

# AN-033 — Policy loader de capabilities manifest-driven

## Contexto y Motivación

En RM-02 y RM-03 (AN-011, AN-015) se introdujo el mecanismo base de autorización `HostPolicy` en `agent-native-runtime`. Anteriormente, la configuración de políticas requería invocaciones programáticas imperativas en Rust (`allow_core`, `allow_wit_interface`, `allow_wit_operation`).

Para ejecutar componentes generados por agentes de IA o empaquetados por terceros en escenarios multi-tenant o no confiables (RM-06), es imprescindible:
1. Declarar políticas de seguridad mediante manifiestos legibles por humanos y máquinas (YAML/JSON).
2. Distinguir las capacidades declaradas/solicitadas por el componente (`requested`) de las capacidades autorizadas por el host (`granted`).
3. Calcular la política efectiva como la intersección estricta: `effective = requested ∩ granted`.

## Invariantes Arquitectónicas

1. **Separación de Autoridad y Declaración:** El manifiesto del componente es metadato no confiable; representa únicamente una *declaración de capacidades requeridas*, **no** una prueba de procedencia ni de integridad criptográfica (aspecto reservado a INIT-06-04).
2. **No auto-concesión (No self-granting):** Un componente no puede concederse permisos a sí mismo. La autorización efectiva depende exclusivamente de la política autorizada por el host bajo la regla:
   \[ \forall i \in \text{Imports}(WASM): i \in \text{Requested} \land i \in \text{Granted} \]
3. **Autorizar no implica implementar (Authorize != Implement):** Una interfaz WIT permitida por la política cuya implementación no sea provista por el host falla explícitamente antes de la ejecución (`fail-explicit` / `InstantiationFailed`), distinguiéndose claramente de una denegación de permisos (`UnauthorizedImport`).
4. **Validación pre-ejecución y puntos de entrada explícitos:** La validación ocurre antes de instanciar o ejecutar código del componente. Para validación estricta al cargar, se proporcionan `Engine::load_component_enforcing_policy`, `Engine::load_component_with_manifest` y sus equivalentes en `Resolver`.
5. **Deny-by-default y sin comodines:** Cualquier import no explícitamente autorizado y solicitado es rechazado con error tipado. Los comodines (`*` o `allow_module`) quedan estrictamente fuera del esquema de manifiestos declarativos.

## Alcance Técnico

- **Esquema del Manifiesto (`ComponentManifest`):**
  - Formato declarativo versionado (`version: "0.1"`).
  - Listas de capacidades:
    - `wit_interfaces`: Nombres canónicos completos de interfaces WIT (ej. `an:test/ops@0.1.0`).
    - `wit_operations`: Pares estructurados `(interface, operation)` para granularidad fina.
    - `core_imports`: Pares estructurados `(module, name)` para módulos Core WASM.
- **Parsing y Validación:**
  - Deserialización segura desde YAML/JSON usando `serde` y `serde_json` como dependencias normales.
  - Validación sintáctica y semántica (campos no vacíos, identificadores WIT válidos, sin comodines).
  - Tipos de error dedicados: `PolicyError::{ParseError, InvalidIdentifier, UnsupportedVersion, DuplicateDeclaration, WildcardForbidden}` integrados en `RuntimeError`.
- **Integración con `HostPolicy`:**
  - Constructores `HostPolicy::from_manifest()`, `HostPolicy::from_manifest_yaml()`, `HostPolicy::from_manifest_json()`.
  - Método de intersección: `ComponentManifest::effective_policy(&self, granted: &HostPolicy) -> HostPolicy` y `HostPolicy::effective(requested, granted)`.
- **Compatibilidad de API:**
  - Preservar la API existente de `Engine::load_component` y `Engine::load_component_with_policy`.
  - Puntos de entrada para validación en tiempo de carga: `load_component_enforcing_policy`, `load_component_with_manifest` en `Engine` y `Resolver`.

## Criterios de Aceptación Cumplidos

1. `ComponentManifest` se deserializa correctamente desde representaciones YAML y JSON estándar.
2. `HostPolicy::from_manifest_yaml` y `from_manifest_json` construyen instancias de `HostPolicy` que reflejan exactamente las reglas declaradas.
3. El cálculo de política efectiva respeta la regla de intersección: solo se conceden las capacidades presentes en AMBOS manifiestos/políticas (`requested ∩ granted`).
4. Cualquier formato de manifiesto mal formado, versión no soportada o duplicados retorna un error tipado `PolicyError` descriptivo y no genera panics.
5. Se conserva total retrocompatibilidad con la API programática de `HostPolicy`.
