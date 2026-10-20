---
id: AN-039
type: feature
status: completed
created_at: 2026-10-19T00:00:00.000Z
completed_at: 2026-10-19T00:00:00.000Z
title: API Tenant, gobernanza y unificación del ciclo de vida bajo límites homogéneos
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
code:
  - crates/runtime/src/tenant.rs
  - crates/runtime/src/invoke.rs
  - crates/runtime/src/limits.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/tests/tenant_lifecycle.rs
source_roadmap_initiative: INIT-06-03
source_work_item_candidate:
  - WI-CANDIDATE-06-03-01
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Independent Runtime Execution
  - Host Capability Policy
  - Resource Contention & Limits
expected_value: >-
  Proveer una abstracción ergonómica `Tenant` y `TenantConfig` que encapsule políticas de
  capabilities (`HostPolicy`), presupuestos computacionales (`ExecutionLimits`) y límites de
  recursos (`InstanceLimits`), unificando el pipeline de instanciación e invocación (tanto estándar
  como con contexto de host `HostContext`) bajo una invariante estricta de cero reutilización
  de instancias entre tenants (Store efímero por invocación según ADR-005) y refinando el diagnóstico
  de errores de memoria para desacoplar denegaciones previas de traps no relacionados.
risks:
  - >-
    R-TEN-01: Fugas de memoria o acumulación de estado mutable entre invocaciones sucesivas de un tenant.
    Mitigación: Descarte incondicional del `wasmtime::Store` al finalizar cada invocación. Las instancias
    nunca se almacenan ni se reúsan en la estructura `Tenant`.
  - >-
    R-TEN-02: Divergencia entre invocaciones estándar y con contexto de host (`HostContext`).
    Mitigación: Canalizar `invoke` e `invoke_with_host_ctx` a través de un único pipeline interno de instanciación
    que aplique homogéneamente policy, fuel y límites de memoria.
  - >-
    R-TEN-03: Falso diagnóstico de traps de memoria tras un rechazo previo de `memory.grow` manejado por el guest.
    Mitigación: Inspeccionar estructuradamente el tipo de trap y refinar la bandera `growth_denied` para evitar
    atribuir erróneamente excepciones lógicas ajenas a fallos de asignación de memoria.
dependencies:
  - AN-033
  - AN-035
  - AN-036
  - AN-038
source_signals:
  - 'Roadmap INIT-06-03: Ciclo de vida de instancias y aislamiento multi-tenant'
  - 'ADR-004: Fail-explicit y errores tipados predecibles'
  - 'ADR-005: Store fresco por invocación y prohibición de reutilización de instancias'
  - 'knowledge/tech/discovery/multi-tenant-isolation-design.md'
decision_candidates: []
related_decisions:
  - ADR-004
  - ADR-005
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
---

# AN-039 — API Tenant, gobernanza y unificación del ciclo de vida bajo límites homogéneos

## Estado del Work Item: `completed`

La API `Tenant` y la unificación del ciclo de vida bajo límites homogéneos se encuentran plenamente implementadas, verificadas y fusionadas en `main` (PR #2, commit `b3df809`).

---

## Contexto y Motivación

En **INIT-06-01** (AN-033, AN-034) y en **INIT-06-02** (AN-035, AN-036, AN-037, AN-038), el runtime incorporó controles de seguridad esenciales:
1. `HostPolicy`: Allow-list de capabilities por componente.
2. `ExecutionLimits`: Presupuesto y contabilidad determinista de fuel/CPU por invocación.
3. `InstanceLimits`: Límites estrictos de memoria lineal y tablas WebAssembly mediante `wasmtime::ResourceLimiter`.

Sin embargo, estos mecanismos operan actualmente de forma dispersa:
- `Component::invoke_with_limits()` aplica presupuestos y limiter, pero no valida capabilities mediante `HostPolicy`.
- `Component::invoke_with_host_ctx()` valida capabilities, pero corre bajo presupuesto ilimitado (`UNMETERED_FUEL_BUDGET`) y sin límites estrictos de memoria.
- No existe una abstracción que represente la identidad (`TenantId`) y configuración consolidada (`TenantConfig`) de un inquilino multi-tenant.
- Si el guest maneja una denegación de `memory.grow` (que retorna `-1`), un trap subsiguiente no relacionado (por ejemplo, `IntegerOverflow` o `BadConversionToInteger`) puede ser clasificado incorrectamente como `MemoryLimitExceeded` debido a la bandera heurística `growth_denied`.

**AN-039** unifica estas capacidades en una API cohesiva y segura.

---

## Invariantes de Diseño

1. **Estrictamente cero reutilización de instancias entre tenants (ADR-005):**
   - La estructura `Tenant` **no** retiene instancias de `wasmtime::component::Instance` ni `wasmtime::Store`.
   - Cada invocación instancia el componente en un `Store` fresco y efímero que se destruye al salir de la función.
   - Toda la memoria lineal del guest se libera al concluir la llamada; los valores devueltos cruzan la Canonical ABI copiados al host.
2. **Homogeneidad de garantías:**
   - Tanto `tenant.invoke()` como `tenant.invoke_with_host_ctx()` aplican de manera uniforme la política de capabilities (`HostPolicy`), el presupuesto de fuel (`ExecutionLimits`) y los límites de memoria (`InstanceLimits`).
3. **Diagnóstico desacoplado de traps:**
   - La correlación de denegación de memoria se refina para requerir evidencia estructurada de que el trap final fue consecuencia directa del agotamiento de memoria, o proporcionar información causal detallada.

---

## Contrato de API Propuesto

```rust
/// Identificador inmutable de un tenant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenantId(pub String);

impl TenantId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Configuración integral de un Tenant.
#[derive(Debug, Clone)]
pub struct TenantConfig {
    pub id: TenantId,
    pub policy: HostPolicy,
    pub limits: ExecutionLimits,
    pub instance_limits: InstanceLimits,
}

impl TenantConfig {
    pub fn new(id: TenantId) -> Self {
        Self {
            id,
            policy: HostPolicy::default(),
            limits: ExecutionLimits::default(),
            instance_limits: InstanceLimits::default(),
        }
    }

    pub fn with_policy(mut self, policy: HostPolicy) -> Self {
        self.policy = policy;
        self
    }

    pub fn with_execution_limits(mut self, limits: ExecutionLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_instance_limits(mut self, instance_limits: InstanceLimits) -> Self {
        self.instance_limits = instance_limits;
        self
    }
}

/// Contexto de ejecución de un Tenant.
/// Garantiza aislamiento estricto y Store efímero por invocación.
pub struct Tenant {
    config: TenantConfig,
}

impl Tenant {
    pub fn new(config: TenantConfig) -> Self {
        Self { config }
    }

    pub fn id(&self) -> &TenantId {
        &self.config.id
    }

    pub fn config(&self) -> &TenantConfig {
        &self.config
    }

    /// Invoca una función de un componente bajo las políticas y límites del tenant.
    pub fn invoke(
        &self,
        component: &Component,
        func_name: &str,
        args: &[Val],
    ) -> Result<InvocationReport, RuntimeError>;

    /// Invoca una función con contexto de host bajo las políticas y límites del tenant.
    pub fn invoke_with_host_ctx<T: HostContext>(
        &self,
        component: &Component,
        func_name: &str,
        args: &[Val],
        ctx: &mut T,
    ) -> Result<InvocationReport, RuntimeError>;
}
```

---

## Criterios de Aceptación

1. **Unificación de Límites:** `Tenant::invoke` e `Tenant::invoke_with_host_ctx` aplican `HostPolicy`, `ExecutionLimits` e `InstanceLimits` con errores tipados correspondientes (`CapabilityNotGranted`, `FuelExhausted`, `MemoryLimitExceeded`).
2. **Cero Reutilización de Instancias:** Cada llamada dentro de un mismo tenant o entre diferentes tenants crea y destruye su propio `Store`. Verificado mediante asserts de aislamiento de punteros y memoria.
3. **Desacoplamiento de Diagnóstico:** Una llamada que maneje internamente una denegación de memoria y luego trapée por otra causa retorna el trap causal adecuado en lugar de asumir `MemoryLimitExceeded`.
4. **Retrocompatibilidad:** La API pública existente de `Component::invoke` e `invoke_with_limits` se mantiene intacta para casos de uso que no requieran gobernanza de tenant.
5. **Multiplataforma:** La suite completa de pruebas pasa en Linux, macOS y Windows MSVC sin atributos `#[ignore]`.


---

## Verificación y Cierre Técnico

1. **API y Gobernanza:** Abstracciones `TenantId`, `TenantConfig`, `Tenant` implementadas en `crates/runtime/src/tenant.rs` y exportadas en el root del crate (`agent_native_runtime`).
2. **Cero Reutilización de Instancias (ADR-005):** Validado que cada llamada a `tenant.invoke()` y `tenant.invoke_core_with_host_ctx()` ejecuta en un `wasmtime::Store` fresco y efímero que se destruye al salir, sin retención de instancias ni acumulación de fuel.
3. **Desacoplamiento de Diagnóstico de Traps:** En `map_execution_error`, los traps explícitos (`OutOfFuel`, `IntegerOverflow`, `IntegerDivisionByZero`, `BadConversionToInteger`, `TableOutOfBounds`, `StackOverflow`) tienen precedencia sobre eventos previos de `growth_denied`, evitando clasificaciones falsas de `MemoryLimitExceeded` cuando el código guest maneja `-1` en memoria y luego falla por causas aritméticas o computacionales.
4. **Retrocompatibilidad Preservada:** Todas las firmas existentes (`invoke`, `invoke_with_limits`, `invoke_core_with_host_ctx`, `invoke_component_with_host_ctx`) se mantienen operativas sin breaking changes.
5. **Calidad y CI:**
   - Suite `tenant_lifecycle.rs` con 6 tests específicos pasando.
   - `public_api_audit.rs` auditando reexportaciones de `Tenant`.
   - `cargo fmt --check` limpio.
   - `cargo clippy --workspace --all-targets -- -D warnings` limpio.
   - CI de GitHub Actions (run `39366330410`) 100% verde en Linux, macOS, Windows MSVC y AWS Lambda.
