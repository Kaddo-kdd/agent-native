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

## Casos de Prueba Implementados (`limits_negative_tests.rs`)

1. **Agotamiento de Fuel con Bucle Infinito (`test_infinite_loop_fuel_exhaustion`):**
   - Componente WASM con bucle infinito (`loop { br 0 }`).
   - Configurado con presupuesto finito (`fuel_budget = 5_000`).
   - En Linux/macOS valida interrupción limpia y retorno tipado de `RuntimeError::FuelExhausted { limit: 5_000 }`.
   - En Windows MSVC queda marcado con `#[cfg_attr(target_os = "windows", ignore = "...")]` documentando el riesgo upstream de Wasmtime 27.
2. **Exceso de Memoria Inicial al Instanciar (`test_initial_memory_stress_rejection`):**
   - Módulo WASM que declara memoria mínima inicial de 512 páginas (32 MB).
   - Configurado con `max_memory_bytes = 16 MB` (256 páginas).
   - Instanciación rechazada de inmediato retornando `RuntimeError::MemoryLimitExceeded { requested_bytes: 33554432, max_bytes: 16777216 }`.
3. **Exceso de Memoria Dinámica (`test_dynamic_memory_leak_growth_denial_and_trap`):**
   - Módulo WASM que ejecuta `memory.grow(1)` iterativamente en un bucle hasta intentar superar el techo de 256 KiB (4 páginas).
   - Al solicitar la 5ª página (320 KiB), el host deniega la solicitud (`memory.grow` devuelve `-1`). El guest ejecuta `unreachable` y el runtime mapea estructuradamente el trap a `RuntimeError::MemoryLimitExceeded`.
4. **Límite de Elementos de Tabla (`test_table_growth_limit_enforcement`):**
   - Módulo con tabla inicial de 5 elementos sujeta a techo de 10 elementos.
   - Intento de crecimiento dinámico (`table.grow(20)`) es denegado por `ResourceLimiter` devolviendo `-1` según la especificación WebAssembly.
5. **Aislamiento Multi-Store Consecutivo (`test_consecutive_multi_store_lifecycle_isolation`):**
   - Paso 1: Invocación con límite restrictivo de memoria (64 KiB) falla limpiamente con `MemoryLimitExceeded`.
   - Paso 2: Siguiente invocación sobre el **mismo** `Component` con presupuesto adecuado (256 KiB) ejecuta exitosamente y retorna el resultado exacto (Store nuevo, cero filtración, ADR-005).
   - Paso 3: Siguiente invocación con límite restrictivo vuelve a fallar con `MemoryLimitExceeded`.
   - Paso 4: Invocación no restringida (`invoke`) ejecuta y completa exitosamente.
6. **Contención Combinada de Fuel y Memoria (`test_combined_fuel_and_memory_contention`):**
   - Ambos límites configurados simultáneamente (`fuel_budget = 50_000`, `max_memory_bytes = 256 KiB`).
   - La ejecución exitosa reporta métricas de fuel consumido dentro de ambos límites.

## Criterios de Aceptación

1. 100% de los casos de prueba de estrés y límites pasan de forma reproducible en CI.
2. Todos los fallos retornan variantes tipadas de `RuntimeError` (`FuelExhausted`, `MemoryLimitExceeded`), nunca panics del host ni fallos silenciosos.
3. Se verifica empíricamente la independencia y el aislamiento total entre Stores consecutivos.
