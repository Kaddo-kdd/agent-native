---
id: AN-047
type: feature
status: completed
created_at: 2026-10-20T00:36:00.000Z
completed_at: 2026-10-20T01:40:00.000Z
title: Límites de ejecución y métricas de fuel en agent-native run
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/cli/src/cli.rs
  - crates/cli/src/commands/run.rs
  - crates/cli/tests/run_limits.rs
  - knowledge/tech/cli-guide.md
source_roadmap_initiative: INIT-05-01
initiative: INI-005
domains:
  - Distribution & DX
  - Runtime & Memory
  - Security
related_capabilities:
  - CLI as Primary Interface
  - Independent Runtime Execution
expected_value: >-
  Exponer flags de límites de ejecución (`--fuel`, `--max-memory-bytes`, `--max-table-elements`) en el comando
  `agent-native run`, conectándolos con `ExecutionLimits` y `Component::invoke_with_limits()` del runtime.
  Reportar `fuel_consumed` en la salida JSON y de texto cuando exista presupuesto configurado, capturar y
  traducir de forma amigable los errores tipados `RuntimeError::FuelExhausted` y `RuntimeError::MemoryLimitExceeded`,
  validar los argumentos previamente y documentar las diferencias entre fuel determinista, memoria guest y tiempo CPU.
risks:
  - >-
    R-LIM-01: Confusión entre fuel determinista y tiempo de reloj de pared. Mitigación: Documentación explícita
    en ayuda y guías de que fuel mide instrucciones y bloques básicos WebAssembly independientes de la CPU física.
  - >-
    R-LIM-02: Regresión en llamadas existentes sin flags de límites. Mitigación: Preservar `ExecutionLimits::unmetered()`
    como comportamiento por defecto si no se especifican cuotas, omitiendo `fuel_consumed` en la salida.
  - >-
    R-LIM-03: Abortos en Windows MSVC por traps de fuel. Mitigación: Ya verificado y mitigado en AN-038 mediante
    la adopción de Wasmtime 29.0.1 (Bytecode Alliance PR #9710). Se añaden pruebas de regresión multiplataforma.
dependencies:
  - AN-028
  - AN-035
  - AN-036
  - AN-038
source_signals:
  - "Roadmap RM-05: INIT-05-01 Binario CLI unificado"
  - "Roadmap RM-06: INIT-06-02 Cuotas de fuel y límites de memoria"
  - "ADR-001: CLI como adaptador de frontera sin duplicar lógica de runtime"
  - "ADR-004: Seguridad fail-explicit y manejo tipado de errores"
  - "ADR-005: Store efímero e invariantes de aislamiento"
---

# AN-047 — Límites de ejecución y métricas de fuel en `agent-native run`

## Contexto y Motivación

En `crates/cli/src/commands/run.rs`, el comando `agent-native run` delegaba en `Component::invoke()`, el cual utilizaba internamente `ExecutionLimits::unmetered()`. 

Aunque el crate `agent-native-runtime` ya disponía de capacidades completas de seguridad de recursos (`ExecutionLimits`, `InstanceLimits`, `Component::invoke_with_limits()`, `InvocationReport`, `RuntimeError::FuelExhausted`, `RuntimeError::MemoryLimitExceeded`), los usuarios de la CLI no podían configurar presupuestos de fuel ni límites de memoria desde la línea de comandos, ni inspeccionar el fuel consumido por sus funciones.

Este Work Item expuso estos controles en la CLI de forma modular y sin duplicar lógica del runtime (ADR-001).

## Alcance Funcional Entregado

1. **Extensión de Argumentos CLI (`RunArgs`):**
   - `--fuel <U64>`: Presupuesto máximo de unidades de fuel computacional para la invocación.
   - `--max-memory-bytes <USIZE>`: Límite máximo de memoria lineal WebAssembly en bytes.
   - `--max-table-elements <U32>`: Límite máximo de elementos en tablas WebAssembly.
2. **Validación Previa de Argumentos:**
   - Validación de que los valores no sean cero (`> 0`) antes de inicializar el motor o cargar el componente. En caso contrario, se emite error de uso (`CliExitCode::UsageOrIoError`).
3. **Construcción de `ExecutionLimits`:**
   - Reutilización directa de las APIs del runtime:
     - `ExecutionLimits::unmetered()` si no se proporcionan flags.
     - `limits.with_fuel(fuel)` si se especifica `--fuel`.
     - `limits.with_max_memory_bytes(bytes)` si se especifica `--max-memory-bytes`.
     - `limits.with_instance_limits(instance_limits)` si se especifica `--max-table-elements`.
4. **Invocación con Límites y Soporte de Candidatos WIT:**
   - Invocación con límites mediante `component.invoke_with_limits(..., &limits)` tanto en la invocación directa como en la resolución de candidatos calificados (`ops#...`, `an:synthesized/ops@0.1.0#...`).
5. **Captura y Exposición de `fuel_consumed`:**
   - En `RunCommandOutput`, se incorporó `fuel_consumed: Option<u64>`, serializado en JSON y formateado en salida humana.
6. **Mapeo de Errores Tipados del Runtime:**
   - Formateo comprensivo de `RuntimeError::FuelExhausted` y `RuntimeError::MemoryLimitExceeded` con salida estructurada de error y código de salida 1 (DomainFailure).
7. **Documentación de Garantías:**
   - Actualización de `knowledge/tech/cli-guide.md` distinguiendo fuel determinista, límites de memoria y tiempo de reloj de pared.

## Verificación y Pruebas

La suite [`crates/cli/tests/run_limits.rs`](crates/cli/tests/run_limits.rs) valida:
- `test_cli_run_with_sufficient_fuel_succeeds_and_reports_consumption`: Ejecución con `--fuel 100000`, confirmando `success: true` y `fuel_consumed > 0`.
- `test_cli_run_fuel_exhausted_fails_with_typed_error`: Ejecución con `--fuel 1`, confirmando fallo con mensaje claro y código de salida 1.
- `test_cli_run_unmetered_backward_compatible`: Ejecución sin flags de límites, confirmando funcionamiento transparente con `fuel_consumed: None`.
- `test_cli_run_memory_limit_exceeded_fails`: Ejecución de un componente con memoria inicial superior a `--max-memory-bytes`, verificando rechazo tipado de memoria.
- `test_cli_run_invalid_limit_args_fail_fast`: Validación previa de `--fuel 0` y `--max-memory-bytes 0` retornando código 2 (UsageOrIoError) antes de cargar el binario.
