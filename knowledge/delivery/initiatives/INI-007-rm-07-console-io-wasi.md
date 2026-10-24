---
type: initiative
id: INI-007
title: RM-07 — Console I/O & WASI
status: completed
knowledge_level: K4
domains:
  - Runtime & Memory
  - Semantic IR & Verification
related_capabilities:
  - Host Capability Policy
  - Independent Runtime Execution
roadmap_block: RM-07
roadmap_initiatives:
  - INIT-07-01
  - INIT-07-02
  - INIT-07-03
created_at: '2026-10-17'
external_links:
  - knowledge/delivery/roadmap.md
candidates:
  - id: INIT-07-01
    title: Baseline WASI preview2 + stdio autorizado
    status: completed
    materialized_as: AN-044 (completed), AN-045 (completed), AN-050 (completed)
  - id: INIT-07-02
    title: Extensión IR para `string` y errores
    status: completed
    materialized_as: AN-048 (completed)
  - id: INIT-07-03
    title: Stubs deterministas de I/O para pruebas
    status: completed
    materialized_as: AN-049 (completed)
---

# RM-07 — Console I/O & WASI

## Goal

Habilitar programas con entrada/salida real (consola → WASI preview2), con capabilities autorizadas bajo el modelo Requested ∩ Granted y pruebas deterministas. Extender Semantic IR a v0.6 con `string` y errores, bajo flag `--experimental`.

## Expected Value

Convierte el producto de "núcleo técnico de cálculo" a "ejecuta programas interactivos reales": CLIs útiles generadas por Agent-Native con control explícito de flujos de entrada y salida.

## Scope

- Adaptador WASI preview2 (WASI 0.2 / Component Model) en `crates/runtime` con `wasmtime-wasi = "29.0.1"`.
- Autorización granular e independiente de interfaces WIT de consola (`wasi:cli/stdin@0.2.0`, `wasi:cli/stdout@0.2.0`, `wasi:cli/stderr@0.2.0`, `wasi:io/streams@0.2.0`).
- Redirección y captura en memoria de streams (`MemoryInputPipe`, `MemoryOutputPipe`) para pruebas deterministas sin TTY.
- IR v0.6: tipo `string`, variantes de error, política de versionamiento IR (INIT-07-02).
- Mocks de `stdin`/`stdout`/`stderr` en el executor YAML para pruebas sin TTY (INIT-07-03).

## Out of Scope

- Primitivas asíncronas en IR (Not Now).
- Filesystem, sockets de red, HTTP, KV o mensajería como capabilities (permanecen estrictamente denegados por default).
- Promover v0.6 a estable antes de equivalencia funcional del MVP (principio del roadmap).
- Generación de programas `echo`/`cat` directamente desde Semantic IR (corresponde a INIT-07-02).

## Success Criteria

1. Programa con importaciones WASI 0.2 de consola ejecuta exitosamente cuando sus interfaces están autorizadas en la política del tenant.
2. Componente con interfaz de consola no concedida es rechazado determinísticamente antes de ejecutar código guest con `RuntimeError::UnauthorizedImport`.
3. IR v0.6 con `string` pasa verifier + compilador + runtime bajo `--experimental` (INIT-07-02).
4. Suites YAML con `stdin:` y `expected_stdout:` son deterministas y no requieren TTY real (INIT-07-03).
5. v0.5 sigue siendo el camino estable y por defecto.

## Dependencies

- INIT-06-01 (policy loader) completado — las interfaces WASI se gobiernan como capabilities.
- INIT-06-02 (fuel y límites de memoria) e INIT-06-03 (Store efímero) completados.
- INIT-02-01 (IR v0.5) e INIT-04-05 (publicación) para INIT-07-02.

## Work Item Candidates

Ver `candidates:`.
- `AN-044`: Adaptador WASI Preview 2 para I/O determinista en memoria (`feature`, completed).
- `AN-045`: Autorización granular de stdio y streams bajo Requested ∩ Granted (`feature`, completed).
- `AN-048`: Extensión Semantic IR v0.6 para `string`, `result<T, E>` y política `--experimental` (`feature`, completed).
- `AN-049`: Stubs deterministas de I/O (`stdin`/`expected_stdout`/`expected_stderr`) y tipos v0.6 en el executor YAML (`feature`, completed).
- `AN-050`: Soporte de binarios reales `wasm32-wasip2` bajo deny-by-default (`feature`, in-progress; ampliación posterior de INIT-07-01, no bloquea RM-08). El spike fijó dos fixtures con hash e inventario de imports; el perfil de comando conserva entorno, filesystem y reloj deterministas y acotados.

## Precisiones Arquitectónicas (INIT-07-01 / 2026-10-20)

1. **`wasi:cli/stdio` como Categoría Conceptual:** No constituye una única interfaz WIT concreta en WASI Preview 2. WASI 0.2 expone interfaces atómicas independientes (`wasi:cli/stdin`, `wasi:cli/stdout`, `wasi:cli/stderr`, `wasi:io/streams`). La autorización debe evaluar los identificadores WIT reales solicitados por el componente.
2. **Permisos Separados (Principio de Menor Privilegio):** Debe ser posible habilitar `stdout` sin habilitar automáticamente `stdin`, `stderr`, filesystem o red.
3. **Integración Controlada sin Fugas de Privilegios:** Registrar proveedores WASI en el linker no debe implicar su concesión ciega o automática al guest; el `HostPolicy` mantiene el control de admisión final.
4. **I/O Determinista en Memoria:** Inyección de `stdin` desde memoria (con EOF garantizado) y captura de `stdout`/`stderr` en búferes de memoria aislados, permitiendo pruebas deterministas sin depender de terminales interactivas.
5. **Aislamiento por Store (ADR-005):** Cada invocación construye un `WasiCtx` y una `ResourceTable` dedicados en el `Store` efímero, descartándolos incondicionalmente al concluir la ejecución, bajo las cuotas de fuel y límites de memoria de RM-06.
6. **Desacoplamiento Estricto (ADR-002):** `crates/compiler` y `crates/ir` no incorporan dependencias de `wasmtime-wasi` ni de I/O de host.
