---
id: AN-050
type: feature
status: draft
created_at: 2026-10-20T00:00:00.000Z
title: Soporte de binarios reales wasm32-wasip2 bajo política deny-by-default
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: medium
code:
  - crates/runtime/src/wasi.rs
  - crates/runtime/src/policy.rs
  - crates/runtime/tests/wasi_real_binaries.rs
  - crates/runtime/tests/fixtures/wasip2_hello.wasm
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

## Refinamiento y materialización (2026-10-20)

AN-050 queda materializado como un incremento de compatibilidad de runtime, no como
una ampliación de privilegios de WASI. La implementación actual registra por interfaz
solamente `wasi:cli/{stdin,stdout,stderr}` y `wasi:io/{streams,error,poll}`; por ello
un componente real no puede apoyarse en `add_to_linker_sync`, que registraría familias
fuera del alcance: filesystem, sockets, clocks y random.

### Estado del spike

El target se instaló de forma explícita para el spike y se compiló el fixture mínimo
con `rustc 1.85.0 (4d91de4e4 2025-02-17)`, `wasm32-wasip2`, edición 2024 y `-O`.
El resultado se versiona en `crates/runtime/tests/fixtures/wasip2_hello.wasm`:

- fuente: SHA-256 `bedf55efdeaa20dfee3a97e6a1b6be9703fdbdc2d9f7cfaf15e3408ea89f7287`;
- componente: 2,102,311 bytes, SHA-256
  `b4886a16712301db0a3173b55531d5428b05ed58a963a427563ff9f7ec0c0558`;
- exportación de entrada: `wasi:cli/run@0.2.0`;
- imports: `wasi:cli/{environment,exit,stdin,stdout,stderr}@0.2.0`,
  `wasi:io/{error,streams}@0.2.0`, `wasi:clocks/wall-clock@0.2.0` y
  `wasi:filesystem/{preopens,types}@0.2.0`.

La prueba `wasi_real_binaries` comprueba hashes, tamaño, imports y exportación usando
el parser de Component Model. La suite normal consume el fixture y no requiere que el
target esté instalado en cada runner.

### Decisiones de diseño fijadas

- La autorización continúa usando el identificador WIT exacto, incluida la versión.
  No se aceptan comodines ni compatibilidad implícita por `@0.2.x`.
- `wasi:cli/run` será una ruta de invocación separada y tipada. Un `Err(())` de `run`
  se mapeará a un nuevo error de runtime con código de salida, sin inspeccionar
  mensajes de Wasmtime.
- `wasi:cli/environment`, si el fixture lo requiere, recibirá argumentos y entorno
  vacíos por defecto. Una API para conceder argumentos o variables no entra aquí.
- Filesystem, sockets y HTTP permanecen sin proveedor. Una concesión de política sin
  proveedor debe terminar en fallo tipado, nunca en una capacidad funcional.
- `wall-clock` y filesystem sí aparecen en el inventario. Ambos bloquean la siguiente
  fase: requieren una decisión explícita sobre semántica determinista y permisos antes
  de registrar proveedores. Random no aparece y queda fuera de este item.

### Plan de implementación verificable

1. Añadir el fixture y su manifiesto de procedencia; comparar sus imports con el
   inventario fijado.
2. Extender `HostPolicy`, el linker por interfaz y `WasiCtxBuilder` únicamente para
   las interfaces necesarias. Mantener el registro condicional por import solicitado.
3. Añadir `invoke_wasi_command` para `wasi:cli/run`, salida tipada y captura en memoria
   reutilizando los límites de I/O existentes.
4. Probar stdout autorizado y denegado antes de instanciar, salida no cero, límite de
   I/O, fuel, memoria y dos invocaciones sin fuga de estado.
5. Ejecutar la suite de `agent-native-runtime`, `fmt`, `clippy` y CI en tres sistemas.
   `ir` y `compiler` no recibirán dependencias de WASI.

### Criterios de cierre refinados

- El inventario procede de un fixture real y tiene hash verificable.
- Cada import inventariado está enlazado de forma acotada o se rechaza antes de guest
  con `RuntimeError::UnauthorizedImport`.
- La prueba de aislamiento cubre dos llamadas consecutivas con buffers distintos.
- La documentación identifica interfaces y versión soportadas; no anuncia soporte
  general para `wasm32-wasip2`, filesystem o red.

## Siguiente gate

Antes de pasar AN-050 a `in-progress`, aprobar explícitamente si el runtime ofrecerá
un filesystem virtual vacío y un `wall-clock` determinista para este perfil de comando,
o si estos imports se rechazarán y el soporte se limitará a otro fixture/perfil.
