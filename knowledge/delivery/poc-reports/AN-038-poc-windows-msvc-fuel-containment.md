# Informe de POC: Contención Segura del Agotamiento de Fuel en Windows MSVC (AN-038)

**Fecha:** 2026-10-19
**Rama de Promoción:** `feat/an-038-wasmtime-29-promotion`
**Autor:** Antigravity (Pair Programming con Human Lead)
**Objetivo:** Evaluar empíricamente si la actualización de Wasmtime resuelve el aborto no controlado por agotamiento de fuel en `x86_64-pc-windows-msvc` o si se requiere una arquitectura de worker aislado por proceso.

---

## 1. Diseño del Arnés de Prueba por Subproceso

Para garantizar que un posible aborto no controlado del runtime no termine el proceso del test runner ni afecte pruebas subsiguientes, se diseñó un harness basado en subprocesos en [`crates/runtime/tests/repro_windows_fuel_abort.rs`](crates/runtime/tests/repro_windows_fuel_abort.rs):

```
+-----------------------------------------------------------------+
| PROCESO PADRE (Test Runner)                                     |
| 1. Controla timeout, captura exit code, stdout y stderr         |
| 2. Spawns -> PROCESO HIJO (Wasmtime Runtime)                   |
|              Ejecuta WASM sintético con bucle infinito           |
|              Presupuesto finito: fuel_budget = 5,000            |
|                                                                 |
|              [Resultado en Hijo: Trap Limpio vs. Aborto 0xc0000409]
|                                                                 |
| 3. Verifica supervivencia absoluta del Proceso Padre            |
| 4. Invoca componente benigno ("answer" -> 42) inmediatamente    |
+-----------------------------------------------------------------+
```

---

## 2. Matriz Comparativa de Resultados Empíricos

Se evaluaron tres versiones fijadas de Wasmtime bajo el toolchain oficial de Rust (1.85.0) en `x86_64-pc-windows-msvc`:

| Métrica / Comportamiento | Wasmtime 27.0.0 (Línea Base) | Wasmtime 28.0.1 | Wasmtime 29.0.1 |
|---|---|---|---|
| **Código de salida del proceso hijo** | `ExitStatus(3221226505)` (`0xc0000409`) | `ExitStatus(3221226505)` (`0xc0000409`) | **`ExitStatus(0)` (ÉXITO)** |
| **Excepción / Aborto en Windows** | `STATUS_STACK_BUFFER_OVERRUN` | `STATUS_STACK_BUFFER_OVERRUN` | **Ninguno (0 abortos)** |
| **Mensaje en `stderr` del hijo** | `panic in a function that cannot unwind. thread caused non-unwinding panic. aborting.` | `panic in a function that cannot unwind. thread caused non-unwinding panic. aborting.` | **Limpio (vacío)** |
| **Captura en el guest (`stdout`)** | Ninguna (terminación abrupta antes de retornar al host) | Ninguna | `CHILD_OUTCOME: FUEL_EXHAUSTED_TRAP limit=5000` |
| **Tiempo de interrupción del bucle** | ~2.57 s | ~2.40 s | **33.1 ms** |
| **Supervivencia del Proceso Padre** | ✅ Preservado (ejecuta benigno) | ✅ Preservado (ejecuta benigno) | ✅ Preservado (ejecuta benigno) |
| **Ejecución in-process de `limits_negative_tests`** | Requiere `#[ignore]` en Windows | Requiere `#[ignore]` en Windows | **Pasa 100% in-process sin `#[ignore]` (0.01 s)** |
| **Suite completa del workspace (148+ tests)** | Pasa (con 4 ignorados en Windows) | Requiere ajuste de API `func.params()` | **100% Aprobada (0 fallos, 0 ignorados, 0 regresiones)** |
| **Benchmarks (`benches/baseline.rs`)** | 9/9 Éxito | No probado | **9/9 Éxito verificado** |
| **Alineación con ADR-002 (sin compilador)** | Cumplido | Cumplido | **Cumplido** |

---

## 3. Análisis Técnico del Cambio Upstream (PR #9710 vs. v27/v28)

1. **Causa y Mecanismo Upstream (PR #9710):**
   La explicación técnica precisa de la mejora reside en el **[PR #9710 de Bytecode Alliance — Migrate libcalls to checking for traps](https://github.com/bytecodealliance/wasmtime/pull/9710)** (integrado en diciembre de 2024 e incluido en la versión 29.0.0).
   - En Wasmtime 27 y 28, los libcalls internos (como el control de fuel agotado) provocaban traps directamente saliendo mediante `longjmp`. En Windows MSVC, esto activaba el desenrollado de excepciones (`RtlUnwindEx`), violando las restricciones de Rust sobre marcos `nounwind` y gatillando `core::panicking::panic_cannot_unwind` (`0xc0000409`).
   - Con el PR #9710, los libcalls pasaron a devolver explícitamente un indicador booleano/estado de si ocurrió un trap, en lugar de invocar `longjmp` directamente desde el libcall. El código generado por Cranelift comprueba ese estado de retorno y realiza la salida de forma controlada y compatible con la convención de llamadas.
2. **Resultado en Wasmtime 29.0.1:**
   El trap `OutOfFuel` es comunicado limpiamente al runtime sin desenrollados forzados no compatibles con SEH, permitiendo que `invoke_wasm_and_catch_traps` capture el trap estructuradamente y lo retorne como `RuntimeError::FuelExhausted { limit }` en el propio hilo host.

---

## 4. Adaptaciones Requeridas para la Actualización a Wasmtime 29

La actualización a Wasmtime 29 es mínima y localizada:
- **`crates/runtime/src/invoke.rs`:** `func.params(&store)` en Wasmtime 28+ devuelve `&[(String, Type)]` en lugar de `&[Type]` (incorporando el identificador nominal del parámetro según el estándar WIT/Component Model). El iterador simplemente desestructura `((_name, expected_ty), provided)`.
- **Cero cambios en dependencias del workspace:** Solo `crates/runtime/Cargo.toml` depende de `wasmtime`.

---

## 5. Recomendación y Opciones para la Dirección Técnica

### Opción A (Aprobada para Promoción): Adoptar Wasmtime 29.0.1
* **Ventajas:**
  - Resuelve la brecha de fuel en Windows de forma nativa e in-process sin abortos anfitriones.
  - Elimina la necesidad de un worker de subproceso permanente (reduciendo latencia y complejidad operacional).
  - Permite desmarcar todas las exclusiones `#[ignore]` en `fuel_accounting.rs` y `limits_negative_tests.rs`, logrando paridad multiplataforma total y permitiendo cerrar **AN-035** y completar **INIT-06-02**.
  - La suite completa de 148+ pruebas y todos los benchmarks compilan y pasan sin regresión alguna.
* **Consideración de Mantenimiento y Seguridad (Roadmap posterior):**
  - Wasmtime 29.0.1 fue publicado en enero de 2025. Hacia octubre de 2026, la línea principal de Wasmtime ha avanzado significativamente (versiones 49.x / 50.x).
  - Se adopta Wasmtime 29.0.1 como la versión objetivo estable para resolver la brecha actual y consolidar INIT-06-02.
  - Se registrará como deuda técnica / seguimiento planificado la evaluación y actualización hacia una versión moderna activamente mantenida de Wasmtime antes del hito de producción (RM-08).

### Opción B: Mantener Wasmtime 27 y Adoptar Worker de Proceso Aislado
* **Ventajas:** Aislamiento total a nivel de OS (un fallo de hardware o segfault nativo no tumba el host).
* **Desventajas:** Complejidad de arquitectura IPC (pipes, serialización, gestión de ciclo de vida de procesos) y penalización de latencia en cada invocación. Descartada frente a la efectividad comprobada de la Opción A.
