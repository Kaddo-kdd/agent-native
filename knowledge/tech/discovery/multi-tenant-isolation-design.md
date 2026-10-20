# Diseño Arquitectónico: Aislamiento Multi-Tenant y Contención de Blast Radius (INIT-06-03)

## 1. Objetivo y Principio Fundamental

El objetivo de **INIT-06-03** no es únicamente proporcionar una abstracción de tipo `Tenant` en la API pública, sino **demostrar formal y empíricamente que un componente defectuoso o malicioso perteneciente al Tenant A no puede afectar la ejecución del Tenant B**.

### Invariante Fundamental: Prohibición de Reutilización de Instancias
En consonancia con **ADR-005** y los principios de seguridad de `business.md`:
- **Estrictamente cero reutilización de instancias entre tenants.**
- Cada invocación de cualquier tenant corre en un `wasmtime::Store` fresco y efímero que se descarta inmediatamente al concluir la llamada.
- La memoria lineal del guest se libera al destruir el Store; los valores devueltos cruzan la Canonical ABI copiados al host y nunca se mantienen punteros ni referencias a la memoria interna del guest.

---

## 2. Tratamiento de Brechas y Precisiones Identificadas

A partir de la experiencia en AN-035, AN-036 y AN-037, se incorporan tres requerimientos obligatorios al diseño de INIT-06-03:

### Brecha 1: Memoria Lineal por Instancia vs. Cuota Agregada por Tenant
- **Estado actual (AN-036):** `max_memory_bytes` actúa a nivel de Store individual interceptando el callback `ResourceLimiter::memory_growing()`. Garantiza que una invocación singular no crezca más allá de $N$ MB.
- **Diseño INIT-06-03:** Un tenant puede ejecutar múltiples llamadas simultáneas o sucesivas. El diseño de `Tenant` debe incorporar un mecanismo de agregación (contabilidad acumulada o concurrente) que impida que un tenant monopolice la memoria o CPU del host mediante ráfagas de llamadas pequeñas dentro del límite individual pero abusivas en conjunto.

### Brecha 2: Unificación de Rutas de Invocación con Host Context
- **Estado actual:** `Component::invoke_with_limits()` aplica presupuestos y `ResourceLimiter`, mientras que las variantes con contexto de host (`invoke_with_host_ctx()`, `invoke_module_with_host_ctx()`) utilizan una ruta no acotada (`UNMETERED_FUEL_BUDGET` y sin limiter estricto).
- **Diseño INIT-06-03:** Toda invocación delegada a través del contexto de un `Tenant` debe canalizarse por un flujo unificado que aplique de manera homogénea:
  1. Policy de capabilities (`HostPolicy`).
  2. Presupuestos de fuel / CPU (`ExecutionLimits`).
  3. Límites de memoria y tablas (`InstanceLimits`).

### Brecha 3: Diagnóstico Estructurado vs. Correlación Heurística en Errores de Memoria
- **Estado actual:** Si `growth_denied` fue marcado en el Store tras un rechazo en `memory.grow`, cualquier trap subsiguiente en esa invocación se clasifica actualmente como `RuntimeError::MemoryLimitExceeded`.
- **Diseño INIT-06-03:** Si el guest intercepta el `-1` devuelto por `memory.grow`, se recupera o ejecuta lógica de contingencia, y posteriormente entra en un trap no relacionado (por ejemplo, desbordamiento aritmético, índice de tabla fuera de rango o `unreachable` no derivado de OOM), el manejador de errores debe distinguir entre:
  - Aborto directamente causado por agotamiento de memoria.
  - Aborto por violación lógica ajena a la memoria.

---

## 3. Modelo Conceptual de la API `Tenant`

```rust
/// Identificador inmutable de un tenant en el sistema.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenantId(String);

/// Configuración de gobernanza, cuotas y políticas asignadas a un Tenant.
#[derive(Debug, Clone)]
pub struct TenantConfig {
    pub id: TenantId,
    pub policy: HostPolicy,
    pub limits: ExecutionLimits,
    pub instance_limits: InstanceLimits,
}

/// Contexto operativo de un Tenant.
/// No almacena instancias persistentes de componentes (ADR-005).
/// Orquesta la instanciación limpia y el seguimiento de métricas por tenant.
pub struct Tenant {
    config: TenantConfig,
    engine: Engine,
}

impl Tenant {
    pub fn new(engine: &Engine, config: TenantConfig) -> Self;

    /// Invoca una función de un componente bajo las garantías del tenant.
    pub fn invoke(
        &self,
        component: &Component,
        func_name: &str,
        args: &[Val],
    ) -> Result<InvocationReport, RuntimeError>;

    /// Invoca una función con contexto de host subordinado a las cuotas del tenant.
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

## 4. Matriz de Pruebas de Contaminación Cruzada (Cross-Tenant Blast Radius)

Para validar empíricamente el aislamiento, se diseñará una suite de pruebas que simule la interacción entre dos tenants (`Tenant A` malicioso o saturado vs. `Tenant B` benigno):

| Escenario de Ataque / Saturación (Tenant A) | Comportamiento Esperado de Tenant A | Comportamiento Esperado de Tenant B |
|---|---|---|
| **Agotamiento de CPU:** Bucle infinito con presupuesto finito | Falla con `FuelExhausted` | Ejecuta inmediatamente después y completa con éxito exacto |
| **Pánico de Memoria:** Intento de alocar 1 GB de memoria lineal | Falla con `MemoryLimitExceeded` | Ejecuta inmediatamente después con memoria limpia y completa con éxito |
| **Acceso a Capability No Autorizada:** Intento de leer clave fuera de política | Rechazado con `CapabilityNotGranted` | Puede acceder a sus propias capabilities autorizadas sin restricción |
| **Ejecución Concurrente:** Tenant A en carga extrema simultánea | Contenido dentro de sus cuotas | Throughput y latencia de Tenant B operan dentro de los SLAs esperados |

---

## 5. Prerrequisitos de Transición

Antes de iniciar la codificación de INIT-06-03:
1. La brecha técnica de contención de fuel en Windows MSVC debe estar formalmente acotada o resuelta mediante **AN-038**.
2. La suite de pruebas de aislamiento multi-tenant debe ejecutarse y validar de forma reproducible los casos de prueba en las plataformas soportadas.
