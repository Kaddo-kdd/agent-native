---
id: AN-036
type: feature
status: completed
created_at: 2026-10-19T00:00:00.000Z
completed_at: 2026-10-19T00:00:00.000Z
title: Límites de memoria e instancias mediante ResourceLimiter
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/src/limits.rs
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

## Resumen de Entrega

Este Work Item implementa el control y techo de memoria WebAssembly a nivel de instancia mediante el trait `wasmtime::ResourceLimiter` integrado en el ciclo de vida del `wasmtime::Store`. Permite configurar límites sobre memoria lineal máxima, elementos de tabla y límites estructurales, interceptando el crecimiento dinámico (`memory.grow`) y mapeando de manera tipada y determinista los excesos a `RuntimeError::MemoryLimitExceeded`.

La implementación fue validada en todas las plataformas soportadas (**Windows MSVC**, **Linux** y **macOS**) en los escenarios cubiertos por la suite de pruebas, sin abortos de proceso host.

---

## Contexto y Motivación

En el hito M3 (INIT-03-06), la decisión humana D3 difirió la implementación de medición y contención de memoria por instancia hacia INIT-06-02. Para garantizar la seguridad del host frente a componentes que intenten alocar arrays masivos o ejecutar algoritmos con fuga progresiva de memoria (Risk R-11), el runtime debe imponer un techo estricto sobre el tamaño de la memoria lineal y tablas WebAssembly.

---

## Invariantes Arquitectónicas y Alcance Real

1. **Semántica de WebAssembly (`memory.grow`):**
   - En la especificación WebAssembly, la instrucción `memory.grow` no genera un trap directo: devuelve `-1` si el crecimiento es rechazado.
   - La implementación en `StoreResourceLimiter::memory_growing()` retorna `Ok(false)` al superar el techo configurado, lo que instruye al motor Wasmtime a retornar `-1` al guest.
   - Si el guest aborta por no poder alojar memoria (típicamente mediante `unreachable`), el manejador de traps del host detecta la bandera `growth_denied` registrada en el Store y mapea el trap a `RuntimeError::MemoryLimitExceeded { requested_bytes, max_bytes }`.
   - Si un componente declara una memoria inicial mínima que excede el techo (`memory min > max`), la falla ocurre en tiempo de instanciación y se clasifica de forma inmediata como `MemoryLimitExceeded`.

2. **Alcance Real de la Contención (Límites Demostrados vs. Diferidos):**
   - **Evaluación por solicitud de crecimiento:** `max_memory_bytes` se evalúa en cada solicitud individual de crecimiento de memoria lineal WebAssembly (`ResourceLimiter::memory_growing()`) dentro del Store activo. No constituye una cuota acumulada de todas las memorias que pudieran existir en el Store, ni una cuota agregada o acumulativa entre múltiples invocaciones de un tenant. Ese nivel de gobernanza agregada corresponde a INIT-06-03.
   - **Memoria lineal vs. Memoria nativa del host:** Limita el espacio de direcciones accesible por el guest WASM dentro de los escenarios probados. No garantiza aislamiento de toda la memoria del proceso anfitrión ni limita la memoria nativa que el runtime/JIT de Wasmtime o las estructuras del host consumen fuera del guest, ni futuros buffers nativos de I/O bloqueante (RM-07).
   - **Diagnóstico heurístico de traps (`GrowthDenial`):** Cuando se registra una denegación de crecimiento, el runtime correlaciona cualquier trap subsiguiente en ese Store como `MemoryLimitExceeded`. Esta correlación opera estrictamente como una heurística: un trap posterior no demuestra de forma concluyente que la denegación haya sido su causa directa (por ejemplo, el guest podría capturar el `-1` y luego fallar por una trampa lógica independiente). Esta heurística debe refinarse en INIT-06-03 para evitar clasificaciones incorrectas.

3. **Aislamiento por Store (ADR-005):**
   El estado del `ResourceLimiter` y sus banderas de denegación residen exclusivamente en los datos contextuales del `wasmtime::Store` (`StoreData`). Al finalizar la invocación, el Store se destruye por completo, liberando la memoria lineal y garantizando cero persistencia de estado entre invocaciones.

4. **Comportamiento en Windows MSVC:**
   En Windows MSVC, en las pruebas de memoria cubiertas, el callback del `ResourceLimiter` opera según lo previsto (denegación limpia con `Ok(false)` haciendo que `memory.grow` retorne `-1` y mapeo de traps subsiguientes), a diferencia del mecanismo de interrupción por fuel que actualmente aborta el proceso anfitrión. Sin embargo, esto demuestra el comportamiento del limiter en los escenarios ejecutados y no debe interpretarse como una garantía de aislamiento total de la memoria del proceso anfitrión.

---

## Contratos de API Implementados

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

Variante tipada en `RuntimeError`:
```rust
#[error("memory limit exceeded: requested {requested_bytes} bytes, maximum allowed is {max_bytes} bytes")]
MemoryLimitExceeded {
    requested_bytes: usize,
    max_bytes: usize,
},
```

---

## Verificación y Pruebas

La suite [`crates/runtime/tests/instance_limits.rs`](crates/runtime/tests/instance_limits.rs) contiene 6 pruebas automatizadas:
1. `initial_memory_exceeding_limit_fails_at_instantiation`: Rechazo inmediato si `min_pages * 64 KiB > max_memory_bytes`.
2. `initial_memory_within_limit_succeeds`: Instanciación exitosa si la memoria inicial cumple los límites.
3. `dynamic_memory_grow_within_limit_succeeds`: Crecimiento dinámico exitoso con retorno del número previo de páginas.
4. `dynamic_memory_grow_beyond_limit_denied_cleanly`: Intercepción de `memory.grow` retornando `-1` sin trap ni aborto.
5. `dynamic_memory_grow_beyond_limit_guest_trap_classified`: Mapeo a `RuntimeError::MemoryLimitExceeded` ante aborto del guest tras denegación.
6. `memory_limits_store_isolation`: Verificación de aislamiento estricto: dos Stores consecutivos sobre el mismo componente respetan sus techos independientes sin filtración.

Todas las pruebas pasan al 100% en GitHub Actions CI (Ubuntu, Windows, macOS, AWS Lambda).
