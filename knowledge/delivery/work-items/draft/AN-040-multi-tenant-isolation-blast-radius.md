---
id: AN-040
type: test
status: draft
created_at: 2026-10-19T00:00:00.000Z
title: Suite de pruebas de aislamiento multi-tenant y contención de blast radius
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/tests/multi_tenant_isolation.rs
  - crates/runtime/tests/fixtures/
source_roadmap_initiative: INIT-06-03
source_work_item_candidate:
  - WI-CANDIDATE-06-03-02
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Independent Runtime Execution
  - Resource Contention & Limits
  - Host Capability Policy
expected_value: >-
  Validar formal y empíricamente que un Tenant A malicioso o con componentes que experimentan
  fallos extremos (bucles infinitos de fuel, denegación de memoria masiva, pánicos de trap,
  o intentos de acceso a capabilities no autorizadas) queda estrictamente contenido sin degradar,
  bloquear, corromper o filtrar estado hacia un Tenant B benigno que ejecuta subsecuentemente
  o concurrentemente.
risks:
  - >-
    R-ISO-01: Falsos positivos en pruebas concurrentes derivados de contención global de CPU en CI.
    Mitigación: Utilizar presupuestos de fuel deterministas para las aserciones de aislamiento,
    y separar las pruebas de secuencia cruzada (A-B-A-B) de las pruebas de concurrencia multihilo.
  - >-
    R-ISO-02: Retención implícita de estado entre llamadas a través de estructuras globales de Wasmtime.
    Mitigación: Validar la destrucción del Store y la liberación de la memoria lineal en cada ciclo.
dependencies:
  - AN-037
  - AN-039
source_signals:
  - 'Roadmap INIT-06-03: Tests de contaminación cruzada y contención de blast radius'
  - 'ADR-005: Modelo de memoria y Store fresco por invocación'
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

# AN-040 — Suite de pruebas de aislamiento multi-tenant y contención de blast radius

## Estado del Work Item: `draft`

Este borrador define la matriz experimental y la suite de pruebas automatizadas destinadas a verificar el aislamiento estricto y la contención de radio de impacto (*blast radius*) entre tenants en el runtime de Agent Native.

---

## Contexto y Motivación

El modelo de ejecución para componentes generados por agentes de IA requiere la convivencia segura de múltiples inquilinos (*multi-tenancy*) en un único runtime compartido. 

Aunque **ADR-005** estipula que cada invocación utiliza un `wasmtime::Store` fresco y efímero, es indispensable contar con una suite de pruebas de regresión que someta activamente esta premisa a escenarios adversos reales:
- Un tenant hostil o defectuoso que consuma todo su presupuesto de CPU.
- Un tenant que intente alocar memoria por encima de los límites permitidos.
- Un tenant que intente acceder a claves de estado o capabilities reservadas para otro tenant.
- Invocaciones alternadas y concurrentes entre tenants con configuraciones y cuotas dispares.

---

## Matriz de Escenarios de Prueba

La suite `crates/runtime/tests/multi_tenant_isolation.rs` cubrirá los siguientes casos de blast radius:

| # | Escenario (Tenant A - Malicioso / En Fallo) | Escenario (Tenant B - Benigno) | Criterio de Éxito y Aislamiento |
|---|---|---|---|
| **E1** | **Agotamiento de CPU:** Bucle infinito con presupuesto de fuel acotado (`fuel_budget = 5,000`). | Invocación estándar de cálculo (`sum(2, 3)`). | Tenant A falla de inmediato con `RuntimeError::FuelExhausted`. Inmediatamente después, Tenant B ejecuta en su propio Store y devuelve `5` con consumo normal de fuel y sin interferencia. |
| **E2** | **Agotamiento de Memoria:** Fixture que solicita 1 GB de memoria lineal con límite de 2 MB. | Fixture estándar de memoria benigna (`alloc_and_fill(100 KB)`). | Tenant A es denegado con `RuntimeError::MemoryLimitExceeded`. Tenant B ejecuta a continuación exitosamente, comprobando que la memoria del host no retiene la asignación fallida de A. |
| **E3** | **Violación de Capabilities:** Intento de acceso a namespace de storage no autorizado en su policy (`policy.allow("storage:tenant_a/*")`, intenta leer `storage:tenant_b/secret`). | Tenant B accede a su namespace autorizado (`storage:tenant_b/secret`). | Tenant A es rechazado con `RuntimeError::CapabilityNotGranted`. Tenant B lee su clave legítima sin fuga de credenciales ni interferencias. |
| **E4** | **Secuencia Alternada Rápida (Blast Interleaving):** Bucle de 100 iteraciones alternando llamadas que fallan (Tenant A) y llamadas que triunfan (Tenant B). | — | Cero variabilidad en resultados; ninguna llamada de Tenant B falla y ninguna de Tenant A filtra estado o bloquea el hilo. |
| **E5** | **Concurrencia Multi-Tenant:** Ejecución simultánea en hilos paralelos (`std::thread::spawn`) de Tenant A bajo saturación y Tenant B bajo carga benigna. | — | Ambos tenants completan sus flujos respectivos según sus políticas y límites, sin data races ni panics en el runtime. |

---

## Criterios de Aceptación

1. **Aislamiento Verificado de Memoria y CPU:** Se demuestra que la falla o el trap severo de un tenant no compromete la estabilidad ni altera el resultado de otro tenant en el mismo host.
2. **Aislamiento de Capabilities:** Las políticas aplicadas a un tenant se evalúan de forma estrictamente ortogonal e independiente a las del resto.
3. **Cero Fugas entre Stores:** Validación explícita de que ningún dato de retorno ni memoria residual sobrevive a la destrucción del Store de la invocación.
4. **Ejecución Multiplataforma:** La suite completa se integra al CI de GitHub Actions y pasa exitosamente en Linux, macOS y Windows MSVC.
