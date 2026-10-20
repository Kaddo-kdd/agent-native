---
id: AN-035
type: feature
status: in-progress
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
    R-FUEL-03: Interrupción por agotamiento de fuel en Windows MSVC (Wasmtime 27).
    En Windows x64 MSVC, se observa que al agotarse el fuel el runtime desencadena `panic_cannot_unwind`
    abortando el proceso anfitrión (`0xc0000409`). Se plantea como hipótesis técnica asociada al issue
    upstream #10923 de Bytecode Alliance que el desenrollado de excepciones (`RtlUnwindEx`) durante `longjmp`
    interactúa con marcos no preparados para unwinding. No obstante, no se asume que una convención de llamada
    específica sea la causa única o suficiente.
    Estado actual: En Linux y macOS la recuperación opera con normalidad (`RuntimeError::FuelExhausted`).
    En Windows, la medición y el presupuesto operan según lo esperado, pero el agotamiento crítico aborta a nivel de proceso
    hasta que se evalúe y resuelva mediante AN-038 (comparación de versiones de Wasmtime o worker de proceso aislado).
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

## Estado del Work Item: `in-progress`

La implementación base de fuel accounting, configuración de presupuesto por invocación, reporte de métricas y retrocompatibilidad no restringida se encuentra integrada en `crates/runtime` (commit `890bdf6`).

Sin embargo, este Work Item permanece formalmente en estado **`in-progress`** debido a que la garantía de contención segura del host ante agotamiento de CPU no está satisfecha en Windows MSVC:
- En **Linux y macOS**, el agotamiento de fuel es contenido limpiamente por Wasmtime y mapeado a `RuntimeError::FuelExhausted { limit }`.
- En **Windows MSVC**, el agotamiento de fuel produce la terminación no controlada de todo el proceso anfitrión con código `0xc0000409` (`panic_cannot_unwind`).

Esta brecha multiplataforma se aborda de forma dedicada en **AN-038**.

---

## Contexto y Motivación

Al ejecutar componentes generados por agentes autónomos de IA o provenientes de terceros no confiables, existe el riesgo de que el código guest contenga bucles infinitos o algoritmos de complejidad no acotada que saturen la CPU del host.

Wasmtime 27 ofrece un mecanismo de instrumentación denominado **fuel** que descuenta unidades de cómputo por cada instrucción o bloque básico ejecutado. Cuando el contador llega a cero, el runtime interrumpe la ejecución con un trap determinista.

## Invariantes Arquitectónicas y de Diseño

1. **Aislamiento por Store (ADR-005):** Cada invocación corre en un `wasmtime::Store` nuevo. El presupuesto de fuel se inicializa al arrancar la invocación y el Store se descarta al finalizar. No hay acumulación ni filtración de fuel entre invocaciones.
2. **Determinismo vs. Tiempo Real:** El fuel mide unidades computacionales de WebAssembly, no milisegundos de pared. Una misma entrada en una misma arquitectura consumirá exactamente el mismo fuel, garantizando reproducibilidad independiente de la carga del sistema operativo. Las mediciones de tiempo de reloj se registran exclusivamente como métricas observadas, no como garantías universales.
3. **Fail-explicit (ADR-004):** Si el fuel se agota, el runtime aborta de inmediato y retorna la variante tipada de primer nivel `RuntimeError::FuelExhausted { limit: u64 }`, inspeccionada estructuradamente mediante `downcast_ref::<wasmtime::Trap>()`, nunca por parsing de cadenas de texto.
4. **Retrocompatibilidad de API:** Las llamadas existentes a `Component::invoke(name, args)` mantienen compatibilidad utilizando un presupuesto no restringido (`UNMETERED_FUEL_BUDGET = u64::MAX`), mientras que `invoke_with_limits(...)` permite control explícito de cuotas.
5. **Comportamiento en Windows MSVC vs. POSIX:** En plataformas POSIX (Linux, macOS), `Trap::OutOfFuel` es capturado transparentemente a través de `invoke_wasm_and_catch_traps` y traducido a `RuntimeError::FuelExhausted`. En Windows MSVC, la interacción entre `RtlUnwindEx` y el libcall `out_of_gas` activa `panic_cannot_unwind` abortando el proceso anfitrión.

## Contratos de API Implementados

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

Variante de error en `RuntimeError`:
```rust
#[error("execution fuel exhausted: limit was {limit} units")]
FuelExhausted { limit: u64 },
```

## Alcance Técnico Implementado

- `consume_fuel(true)` y `native_unwind_info(true)` habilitados en `Engine::new()`.
- Inicialización explícita de fuel en cada Store (`store.set_fuel(...)`).
- Cálculo exacto de `fuel_consumed = budget - remaining_fuel` en ejecuciones exitosas.
- Detección estructurada de `wasmtime::Trap::OutOfFuel` en el mapeador de errores.
- Suite de pruebas de regresión y determinismo en `crates/runtime/tests/fuel_accounting.rs`.

## Condiciones Pendientes para Cierre Definitivo

1. Evaluar y resolver la contención de agotamiento de fuel en Windows MSVC mediante la comparación con versiones posteriores de Wasmtime o, si el problema persiste, mediante un worker de proceso aislado (sujeto a los resultados de la POC de AN-038).
2. Garantizar que en Windows MSVC ningún agotamiento de fuel termine el proceso anfitrión que aloja el runtime.
