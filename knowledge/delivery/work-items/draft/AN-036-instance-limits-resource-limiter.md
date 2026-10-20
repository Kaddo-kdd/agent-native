---
id: AN-036
type: feature
status: draft
created_at: 2026-10-19T00:00:00.000Z
title: Límites de memoria e instancias mediante ResourceLimiter
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/src/limits.rs
  - crates/runtime/src/limiter.rs
  - crates/runtime/src/engine.rs
  - crates/runtime/src/invoke.rs
  - crates/runtime/src/error.rs
  - crates/runtime/tests/instance_limits.rs
source_roadmap_initiative: INIT-06-02
source_work_item_candidate:
  - WI-CANDIDATE-06-02-02
  - WI-CANDIDATE-06-02-04
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Resource Contention & Limits
  - Independent Runtime Execution
expected_value: >-
  Implementa contención estricta de memoria lineal y elementos de tabla por instancia
  mediante `wasmtime::ResourceLimiter`, impidiendo que un componente agote la memoria RAM
  del host, distinguiendo la semántica de rechazo de `memory.grow` y retornando errores
  tipados `RuntimeError::MemoryLimitExceeded`.
risks:
  - >-
    R-MEM-01: Falsa clasificación de errores de memoria. En WASM, `memory.grow` retorna `-1`
    en lugar de emitir un trap inmediato si el guest maneja el error internamente.
    Mitigación: Monitoreo en el limiter del evento de denegación (`growth_denied`) para
    correlacionar fallos subsiguientes del guest y distinguir claramente OOM por límite del host
    frente a corrupciones de punteros o desbordamientos aritméticos.
  - >-
    R-MEM-02: Límites por defecto demasiado restrictivos que rompan componentes válidos.
    Mitigación: Establecer defaults sanos (ej. 16 MB para micro-componentes, ampliable por config).
dependencies:
  - AN-012
  - AN-029
  - AN-035
source_signals:
  - 'Roadmap INIT-06-02: InstanceLimits configurables + ResourceLimiter diferido de M3 (D3)'
  - 'ADR-005: Modelo de memoria y Store fresco por invocación'
decision_candidates: []
related_decisions:
  - ADR-004
  - ADR-005
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
---

# AN-036 — Límites de memoria e instancias mediante ResourceLimiter

## Contexto y Motivación

En el hito M3 (INIT-03-06), la decisión humana D3 difirió la implementación de medición y contención de memoria por instancia hacia INIT-06-02. Para garantizar la seguridad del host frente a componentes que intenten alocar arrays masivos o ejecutar algoritmos con fuga progresiva de memoria (Risk R-11), el runtime debe imponer un techo estricto sobre el tamaño de la memoria lineal y tablas WebAssembly.

## Invariantes Arquitectónicas y de Diseño

1. **Semántica de WebAssembly (`memory.grow`):**
   En la especificación WebAssembly, la instrucción `memory.grow` no genera un trap por sí misma: devuelve `-1` si el crecimiento es rechazado. La denegación impuesta por `ResourceLimiter` hace que `memory.grow` retorne `-1`. Si el código guest no puede recuperarse de la falta de memoria y entra en trap (abort o `unreachable`), o si el componente solicita de inicio más páginas que el límite configurado (`memory min > max`), el runtime debe identificar con precisión que la causa raíz fue el techo de memoria configurado y retornar `RuntimeError::MemoryLimitExceeded`.
2. **Aislamiento por Store (ADR-005):**
   El estado del `ResourceLimiter` se adjunta al Store de la invocación. No existe estado de memoria compartido entre invocaciones ni entre componentes distintos.
3. **Alcance Real del Aislamiento:**
   Este mecanismo contiene la memoria lineal WebAssembly del Store (`memory (pages * 64KiB)` y elementos de tabla). Se debe documentar explícitamente que no contiene la memoria nativa que el propio proceso de Wasmtime (compilador JIT, metadatos) consume fuera del guest, ni memoria consumida en futuros host stubs de I/O bloqueante (RM-07).
4. **Defaults Sanos y Retrocompatibilidad:**
   Se definen límites por omisión generosos pero seguros para ejecuciones no confiables (ej. 16 MB de memoria lineal máxima, 10,000 elementos de tabla), permitiendo que ejecuciones de confianza puedan especificar cuotas personalizadas o ilimitadas.

## Contratos de API Propuestos

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstanceLimits {
    pub max_memory_bytes: usize,
    pub max_table_elements: u32,
    pub max_instances: usize,
    pub max_tables: usize,
    pub max_memories: usize,
}

impl Default for InstanceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 16 * 1024 * 1024, // 16 MB
            max_table_elements: 10_000,
            max_instances: 10,
            max_tables: 10,
            max_memories: 5,
        }
    }
}
```

Integrado en `RuntimeError`:
```rust
#[error("memory limit exceeded: requested {requested_bytes} bytes, maximum allowed is {max_bytes} bytes")]
MemoryLimitExceeded {
    requested_bytes: usize,
    max_bytes: usize,
},
```

## Alcance Técnico

- Implementación del trait `wasmtime::ResourceLimiter` en una estructura interna `HostResourceLimiter`.
- Registro del limiter en el `wasmtime::Store` mediante `store.limiter(...)`.
- Intercepción de `memory_growing` y verificación de `desired_pages * 65536 <= max_memory_bytes`.
- Registro de banderas internas en caso de denegación para clasificar adecuadamente traps consecuentes.
- Exposición de configuración de límites en `ExecutionLimits` / `Engine` / `Component`.

## Criterios de Aceptación

1. Componentes que solicitan una memoria inicial superior a `max_memory_bytes` fallan en tiempo de instanciación con `RuntimeError::MemoryLimitExceeded`.
2. Componentes que intentan crecer dinámicamente mediante `memory.grow` más allá de `max_memory_bytes` reciben denegación del host y, de producir trap, son clasificados limpiamente como `MemoryLimitExceeded`.
3. Invocaciones sucesivas demuestran que la memoria liberada al finalizar el Store no afecta a la siguiente invocación.
4. Componentes con consumo normal (menor al límite) operan con total normalidad sin degradación funcional.
