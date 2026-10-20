---
id: AN-050
type: feature
status: ready
created_at: 2026-10-20T00:00:00.000Z
title: Soporte de binarios reales wasm32-wasip2 bajo política deny-by-default
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: medium
code:
  - crates/runtime/src/wasi.rs
  - crates/runtime/src/policy.rs
  - crates/runtime/tests/wasi_real_binaries.rs
source_roadmap_initiative: INIT-07-01
initiative: INI-007
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Host Capability Policy
  - Independent Runtime Execution
depends_on:
  - AN-044
  - AN-045
expected_value: >-
  Ejecutar componentes reales compilados con `rustc --target wasm32-wasip2`
  (p. ej. un `hello world` con `println!`) bajo el modelo Requested ∩ Granted,
  sin ampliar permisos de forma implícita. Cierra la ampliación diferida de
  INI-007 (decisión 2, opción B: hasta ahora solo fixtures WIT mínimos).
risks:
  - >-
    R-WASIP2-01: un binario real importa más interfaces que stdio
    (`wasi:cli/environment`, `wasi:cli/exit`, `wasi:clocks/*`,
    `wasi:random/*`, `wasi:filesystem/*`). Mitigación: cada una se decide
    explícitamente (implementar acotada o rechazar con `UnauthorizedImport`);
    nunca stubs vacíos que aparenten capacidad.
  - >-
    R-WASIP2-02: churn de versiones `@0.2.x` entre toolchain Rust y
    `wasmtime-wasi = 29.0.1`. Mitigación: igualdad exacta en política (decisión
    3 de INI-007); el spike documenta qué versión emite el toolchain pinneado.
  - >-
    R-WASIP2-03: el target `wasm32-wasip2` no es parte del toolchain 1.85 por
    defecto en CI. Mitigación: fixtures binarios precompilados y versionados con
    su hash, o job opcional que instale el target.
---

# AN-050 — Soporte de binarios reales `wasm32-wasip2`

## Contexto

INIT-07-01 validó el adaptador WASI con fixtures WIT mínimos (AN-044, AN-045).
Los binarios reales traen imports adicionales y el comando `wasi:cli/run`. Este
Work Item los habilita sin debilitar el aislamiento de RM-06.

## Alcance

1. **Spike inicial:** compilar un `hello world` real con `wasm32-wasip2` y listar
   sus imports exactos y versiones. Registrar el resultado en este documento.
2. Decidir por interfaz (`environment`, `exit`, `clocks`, `random`, `filesystem`,
   `sockets`): implementar acotada y determinista, o rechazar explícitamente.
3. Soportar la exportación `wasi:cli/run@0.2.x#run` como punto de entrada.
4. Mapear `wasi:cli/exit` a un resultado tipado (sin comparar mensajes de texto).
5. Mantener autorización exacta por versión y comprobación previa a instanciar.

## Fuera de alcance

- Filesystem y red (siguen denegados por defecto).
- Compatibilidad por rangos de versión `@0.2.x` (requiere política explícita aparte).
- Async/WASI preview 3.

## Criterios de aceptación

- AC-01: un `hello world` real con stdout autorizado imprime el texto esperado
  capturado en memoria.
- AC-02: el mismo binario sin `wasi:cli/stdout` concedido falla con
  `RuntimeError::UnauthorizedImport` antes de ejecutar código guest.
- AC-03: cada import adicional del binario real está implementado de forma
  acotada o rechazado explícitamente; ninguno es un stub vacío.
- AC-04: `wasi:cli/exit` con código distinto de cero produce un error tipado.
- AC-05: fuel y límites de memoria siguen aplicando; el límite de I/O
  (`IoLimitExceeded`) sigue vigente.
- AC-06: aislamiento entre invocaciones (ADR-005) verificado con el binario real.
- AC-07: las pruebas no requieren el target `wasm32-wasip2` instalado en CI
  (fixture versionado con hash) y `ir`/`compiler` no ganan dependencias WASI
  (ADR-002).

## Notas

Orden recomendado: spike (alcance 1), luego decisión por interfaz y, por último,
implementación y pruebas.
