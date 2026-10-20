---
id: AN-037
type: test
status: draft
created_at: 2026-10-19T00:00:00.000Z
title: Suite de pruebas de límites operativos, agotamiento y regresión
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/tests/limits_negative_tests.rs
  - crates/runtime/tests/fixtures/
source_roadmap_initiative: INIT-06-02
source_work_item_candidate:
  - WI-CANDIDATE-06-02-03
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Resource Contention & Limits
  - Independent Runtime Execution
expected_value: >-
  Suite completa de pruebas negativas, de estrés y de regresión que valida el
  comportamiento determinista del runtime ante agotamiento de fuel (bucles infinitos),
  exceso de memoria (intentos de asignación masiva) y el perfecto aislamiento
  entre ejecuciones sucesivas sin filtración de estados de recursos.
risks:
  - >-
    R-TEST-02: Flakiness o asunciones de tiempo de CPU en CI lenta.
    Mitigación: Las aserciones se realizan estrictamente sobre el agotamiento determinista
    de unidades de fuel y variantes de error tipado, tratando la duración de reloj como métrica
    observada y no como aserción rígida de milisegundos.
dependencies:
  - AN-035
  - AN-036
source_signals:
  - 'Roadmap INIT-06-02: Fuel overflow produce trap; memory limit produce trap tipado'
  - 'ADR-004: fail-explicit, error tipado predecible'
  - 'ADR-005: Fresh Store por invocación y aislamiento'
decision_candidates: []
related_decisions:
  - ADR-004
  - ADR-005
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
---

# AN-037 — Suite de pruebas de límites operativos, agotamiento y regresión

## Contexto y Motivación

Para certificar que el runtime es seguro frente a código WebAssembly generado por agentes de IA, se requiere una suite de pruebas rigurosa que intente deliberadamente saturar los recursos del sistema y demuestre empíricamente que:
1. El cómputo infinito es abortado de forma determinista mediante fuel.
2. El crecimiento desmedido de memoria es contenido por `ResourceLimiter`.
3. Ninguna de estas condiciones de error afecta o contamina las ejecuciones subsiguientes en el mismo proceso.

## Invariantes a Auditar

1. **Determinismo del Fuel vs Mediciones Empíricas:**
   El número de instrucciones permitidas antes de interrumpir la ejecución depende de las unidades de fuel configuradas. Las pruebas verifican que con un presupuesto insuficiente la ejecución siempre se interrumpe con `RuntimeError::FuelExhausted`, independientemente de la velocidad de la máquina. La duración de pared (wall-clock time) se registra con propósitos de observación.
2. **Semántica de Denegación de Memoria:**
   Se prueba que los intentos de asignar memoria por encima del umbral configurado fallen sin corromper el host y se clasifiquen explícitamente como `RuntimeError::MemoryLimitExceeded`.
3. **Aislamiento Estricto entre Stores:**
   Una llamada $A$ que falle por agotamiento de fuel o memoria no debe restar fuel ni memoria a una llamada posterior $B$ ejecutada en el mismo `Component` o `Engine`.
4. **Regresión Cero en Happy Paths:**
   Todas las pruebas existentes de la suite (148 tests) deben continuar pasando sin verse afectadas por la instrumentación de fuel o límites de memoria.

## Casos de Prueba Requeridos

1. **Agotamiento de Fuel con Bucle Infinito:**
   - Componente WASM con bucle infinito (`loop { ... }`).
   - Configurado con presupuesto finito (ej. `fuel_budget = 10_000`).
   - Debe abortar y retornar `RuntimeError::FuelExhausted { limit: 10_000 }`.
2. **Medición Precisa de Fuel Consumido:**
   - Componente finito ejecuta un cálculo conocido.
   - Debe retornar éxito con `report.fuel_consumed > 0` y `fuel_consumed <= fuel_budget`.
3. **Exceso de Memoria Inicial al Instanciar:**
   - Módulo WASM que declara memoria mínima inicial de 32 páginas (2 MB).
   - Configurado con `max_memory_bytes = 1 MB` (16 páginas).
   - Debe fallar en instanciación con `RuntimeError::MemoryLimitExceeded`.
4. **Exceso de Memoria Dinámica (`memory.grow`):**
   - Módulo WASM que ejecuta `memory.grow` iterativamente hasta superar el límite del host.
   - El limiter deniega el crecimiento y el runtime retorna `MemoryLimitExceeded`.
5. **Aislamiento Multi-Store Consecutivo:**
   - Invocación 1: Ejecuta bucle infinito y agota fuel. Retorna `FuelExhausted`.
   - Invocación 2: Inmediatamente sobre el mismo `Component`, ejecuta una función legítima con presupuesto adecuado. Debe completar con éxito sin residuo de la invocación anterior.
   - Invocación 3: Intenta exceder memoria y falla.
   - Invocación 4: Ejecuta cálculo legítimo y completa con éxito.

## Criterios de Aceptación

1. 100% de los casos de prueba de estrés y límites pasan de forma reproducible en CI.
2. Todos los fallos retornan variantes tipadas de `RuntimeError` (`FuelExhausted`, `MemoryLimitExceeded`), nunca panics del host ni fallos silenciosos.
3. Se verifica empíricamente la independencia y el aislamiento total entre Stores consecutivos.
