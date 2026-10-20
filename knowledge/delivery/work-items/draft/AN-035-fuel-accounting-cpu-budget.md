---
id: AN-035
type: feature
status: draft
created_at: 2026-10-19T00:00:00.000Z
title: Fuel accounting y presupuesto de CPU por invocación
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/src/limits.rs
  - crates/runtime/src/engine.rs
  - crates/runtime/src/invoke.rs
  - crates/runtime/src/error.rs
  - crates/runtime/tests/fuel_accounting.rs
source_roadmap_initiative: INIT-06-02
source_work_item_candidate:
  - WI-CANDIDATE-06-02-01
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Resource Contention & Limits
  - Independent Runtime Execution
expected_value: >-
  Permite acotar deterministamente la cantidad de trabajo computacional (instrucciones)
  que un componente guest puede ejecutar por invocación, abortando bucles infinitos
  y operaciones de cómputo voraces con un trap tipado `RuntimeError::FuelExhausted`,
  y midiendo el consumo efectivo de fuel por llamada.
risks:
  - >-
    R-FUEL-01: Confusión entre unidades de fuel y tiempo de reloj real (milisegundos).
    Mitigación: Documentación explícita de que fuel mide instrucciones/bloques abstractos
    de WebAssembly de forma determinista, no tiempo físico de CPU dependiente de hardware.
  - >-
    R-FUEL-02: Impacto de rendimiento al habilitar fuel accounting en Wasmtime.
    Mitigación: Medición de overhead en benchmarks existentes para cuantificar el costo.
  - >-
    R-FUEL-03: Interrupción por agotamiento de fuel en Windows MSVC (Wasmtime 27 / Issue #10923).
    El libcall interno `out_of_gas` de Wasmtime 27 está declarado con ABI `extern "C"` en vez de
    `extern "C-unwind"`. En Windows x64 MSVC, `longjmp` delega en `RtlUnwindEx`, lo cual activa
    la comprobación nounwind de Rust 1.81+ abortando el proceso (`0xc0000409`).
    Mitigación: En Linux y macOS la recuperación opera con normalidad (`RuntimeError::FuelExhausted`).
    En Windows, la medición y el presupuesto operan al 100%, pero el agotamiento crítico aborta a nivel de proceso
    hasta que Wasmtime upstream incorpore la declaración `extern "C-unwind"`. Las pruebas negativas de agotamiento
    se ejecutan en CI para Linux y macOS, y quedan documentadas y marcadas con ignore en Windows.
dependencies:
  - AN-012
  - AN-029
  - AN-033
source_signals:
  - 'Roadmap INIT-06-02: Fuel accounting + InstanceLimits con trap verificable'
  - 'ADR-004: fail-explicit, error tipado predecible'
  - 'ADR-005: Store fresco por invocación'
decision_candidates: []
related_decisions:
  - ADR-004
  - ADR-005
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
---

# AN-035 — Fuel accounting y presupuesto de CPU por invocación

## Contexto y Motivación

Al ejecutar componentes generados por agentes autónomos de IA o provenientes de terceros no confiables, existe el riesgo de que el código guest contenga bucles infinitos o algoritmos de complejidad no acotada que saturen la CPU del host.

Wasmtime 27 ofrece un mecanismo de instrumentación denominado **fuel** que descuenta unidades de cómputo por cada instrucción o bloque básico ejecutado. Cuando el contador llega a cero, el runtime interrumpe la ejecución con un trap determinista.

## Invariantes Arquitectónicas y de Diseño

1. **Aislamiento por Store (ADR-005):** Cada invocación corre en un `wasmtime::Store` nuevo. El presupuesto de fuel se inicializa al arrancar la invocación y el Store se descarta al finalizar. No hay acumulación ni filtración de fuel entre invocaciones.
2. **Determinismo vs. Tiempo Real:** El fuel mide unidades computacionales de WebAssembly, no milisegundos de pared. Una misma entrada en una misma arquitectura consumirá exactamente el mismo fuel, garantizando reproducibilidad independiente de la carga del sistema operativo.
3. **Fail-explicit (ADR-004):** Si el fuel se agota, el runtime aborta de inmediato y retorna la variante tipada de primer nivel `RuntimeError::FuelExhausted { limit: u64 }`, nunca un código de salida opaco o valor por defecto.
4. **Retrocompatibilidad de API:** Las llamadas existentes a `Component::invoke(name, args)` mantienen compatibilidad (modo sin límite estricto o con default seguro), mientras que se expone `invoke_with_limits(...)` o `Component::with_limits(...)` para control explícito de cuotas.
5. **Comportamiento en Windows MSVC vs. POSIX (Issue #10923):** En plataformas POSIX (Linux, macOS), `Trap::OutOfFuel` es capturado transparentemente a través de `invoke_wasm_and_catch_traps` y traducido a `RuntimeError::FuelExhausted`. En Windows MSVC, debido a la declaración interna de libcalls con `extern "C"` en Wasmtime 27, el unwinding de SEH (`RtlUnwindEx`) gatilla la guarda de seguridad `panic_cannot_unwind` de Rust terminando el proceso anfitrión. El accounting determinista y presupuestos suficientes operan con paridad total en todas las plataformas.

## Contratos de API Propuestos

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionLimits {
    pub fuel_budget: Option<u64>,
    pub max_memory_bytes: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct InvocationReport {
    pub values: Vec<Val>,
    pub fuel_consumed: Option<u64>,
}

impl Component {
    pub fn invoke_with_limits(
        &self,
        func_name: &str,
        args: &[Val],
        limits: &ExecutionLimits,
    ) -> Result<InvocationReport, RuntimeError>;
}
```

## Alcance Técnico

- Habilitar `consume_fuel(true)` en `wasmtime::Config` dentro de `Engine::new()`.
- Agregar `ExecutionLimits` con campo `fuel_budget: Option<u64>`.
- Inyectar `store.set_fuel(budget)` antes de la invocación.
- Al retornar de la llamada:
  - En caso de éxito: calcular `fuel_consumed = budget - store.get_fuel().unwrap_or(0)` y retornarlo en `InvocationReport`.
  - En caso de error: detectar si la causa raíz es `wasmtime::Trap::OutOfFuel` y transformarlo en `RuntimeError::FuelExhausted { limit: budget }`.
- Cobertura tanto para Component Model (`Payload::Component`) como para Core WASM (`Payload::Module`).

## Criterios de Aceptación

1. Componentes con cómputo finito dentro del presupuesto de fuel completan exitosamente y reportan con precisión el `fuel_consumed > 0`.
2. Componentes con bucles infinitos o cómputo superior al presupuesto se abortan deterministamente retornando `RuntimeError::FuelExhausted`.
3. Invocaciones sucesivas demuestran que el consumo de fuel no persiste ni contamina invocaciones posteriores (Store fresco verificado).
4. La API pública existente `Component::invoke` sigue funcionando sin errores ni regresiones.
