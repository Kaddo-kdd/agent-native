---
type: current-state
project_state: ai-assisted
generated_by: kaddo-bootstrap
refined_by: architecture-agent
template_version: 1
---

> Idioma del proyecto: **español**. Escribe este conocimiento en español. Mantén en inglés el código, los nombres de archivo, los comandos y las claves de configuración.

# Current State

Generado a partir del Context Pack de Kaddo, de `knowledge/tech/codebase.md` y de `knowledge/product/product.md`. El repositorio `Kaddo-kdd/agent-native` arrancó con sólo `LICENSE` al 2026-10-17 y ha alcanzado **M3 (Runtime autónomo con cobertura regresiva + línea base)** el 2026-10-19 bajo los 7 criterios reformulados por decisión humana (ver INI-003 Learning y `knowledge/delivery/roadmap.md`). Lo que sigue describe el estado real del workspace: cada componente lleva su etiqueta de confianza y las capacidades operativas realmente verificables.

## Hitos alcanzados

| Hito | Fecha | Evidencia |
|---|---|---|
| M1 — Baseline arquitectónico | 2026-10-17 | INI-001 cerrada; ADR-001..005 publicados |
| M2 — Compilador autónomo | 2026-10-18 | INI-002 cerrada; `agent-native-compiler` emite componentes WIT verificables |
| **M3 — Runtime autónomo con cobertura regresiva + línea base** | **2026-10-19** | **INI-003 cerrada bajo 7 criterios reformulados (D1/D2/D3). Executor YAML Core+CM dentro del alcance soportado. Matriz POC 0 exactas + 6 parciales + 4 aplazadas. Línea base latencia publicada; memoria y manifest-driven diferidos. Ver `knowledge/delivery/roadmap.md` y `knowledge/delivery/initiatives/INI-003-rm-03-runtime-acceptance-extraction.md`.** |

Hitos siguientes (M4..M8) siguen fuera de alcance; sus criterios residen en el roadmap sin modificación.

## System Overview

Agent-Native es una toolchain de línea de comandos escrita en Rust que convierte narrativas y Work Items en componentes WASM ejecutables. El pipeline conceptual, de izquierda a derecha, es:

```text
Narrativa + Work Items  ──▶  System Spec  ──▶  Semantic IR  ──▶  WASM Component (WIT)
                                           ▲                      │
                                           │                      ▼
                                 LLM Implementation          Runtime (Wasmtime)
                                 (solo build-time)                │
                                                                  ▼
                                                     Aceptación YAML + Publicación
```

- **Dirección confirmada:** Rust 2024 + workspace Cargo, WebAssembly Component Model vía Wasmtime, contratos WIT, generación LLM solo en build, runtime sin LLM.
- **Asumido (MVP):** 3 crates iniciales (`ir`, `compiler`, `runtime`) + bin `cli`; el `orchestrator` se extrae cuando la API de IR y runtime estabilice. Allocator dlmalloc del guest por defecto. Distribución inicial: tarball firmado para Linux x86_64, Windows x86_64, macOS arm64.
- **No observado todavía:** manifest del workspace, Cargo.lock, CI pipelines, binarios compilados. Toda la evidencia histórica proviene de `_reference/poc-agent-native/` (commit `26b6a18e`), que es **solo lectura**.

## Modules

Confianza: **planeada** salvo donde se indique. "Observada" significa que la responsabilidad existe hoy en la POC y se migrará.

### `crates/ir` — Semantic IR + Verifier

- **Responsabilidad:** modelo de IR v0.5, parser, inferencia de tipos, verificador semántico.
- **Dependencias salientes:** ninguna del producto (no depende de LLM, runtime ni Kaddo).
- **Observado en POC:** `crates/irtool/src/lib.rs` (8 pruebas).
- **Confianza:** High.

### `crates/compiler` — IR → WASM Component

- **Responsabilidad:** backend WASM, generación WIT, publicación atómica con hash SHA-256, agente LLM de implementación.
- **Dependencias:** `crates/ir`, `wasm-tools`/`wit-bindgen` (externas), adaptador de proveedor LLM.
- **Observado en POC:** `crates/orchestrator/src/{lib,agent}.rs` + `crates/irtool` (compilación).
- **Confianza:** Medium — la fusión "compiler + orchestrator" en un crate es la asunción del MVP; se dividirá cuando los límites justifiquen la separación.

### `crates/runtime` — Ejecución y pruebas de aceptación

- **Responsabilidad:** carga de componentes, composición WIT, resolución de dependencias, host capabilities autorizadas, ejecutor de suites YAML.
- **Dependencias:** `wasmtime`, `crates/ir` (metadatos), **no** depende de `compiler`.
- **Capacidades operativas (M3 alcanzado 2026-10-19):** carga Core WASM + Component Model con `HostPolicy` deny-by-default pre-instantiate a 3 niveles (core, WIT interface, WIT operation); invocación tipada; caché content-addressed (`Resolver`); executor YAML con contrato normativo (`knowledge/tech/poc-yaml-contract.md`), 3 kinds de stub declarativos (`StateReader` / `StateWriter` / `Noop`), records por nombre+tipo+valor con faltantes y adicionales detectados, `StateReader` miss registrado + supresión de efectos posteriores de los stubs + `CaseStatus::Error` al final del caso. Component Model dentro del alcance soportado: sin imports funcionales delega a `Component::invoke`; con imports funcionales produce `HostError` tipado deferido a RM-07.
- **Fuera de alcance de M3 (ver roadmap + INI-003 Learning):** stubs funcionales de Component Model, composición multi-componente, host providers reales, policy manifest-driven en `load_component`, memoria por instancia, WASI, esclusas v1→v2 integrales, cadenas multi-paso de dependencias con interrupción exacta, propagación inmediata de errores desde callbacks del host (hallazgo experimental documentado en `host_ctx.rs`: STATUS_STACK_BUFFER_OVERRUN con `wasmtime` 27 en Windows al devolver `Result::Err` desde closures de `Linker::func_wrap`).
- **Observado en POC:** `crates/runner/src/main.rs` (10 pruebas). Equivalencia documentada en `knowledge/tech/poc-equivalence-matrix.md` con 3 grados verificables: **0 `equivalencia-exacta` + 6 `cobertura-parcial` + 4 `aplazado`**. Reproducción literal de las 10 pruebas **no** se afirma (decisión humana D1 2026-10-19).
- **Confianza:** Medium — superficie estable para M3; stubs funcionales CM y manifest-driven loader siguen diferidos.

### `crates/cli` (bin) — Interfaz pública

- **Responsabilidad:** subcomandos `spec`, `build`, `verify`, `test`, `run`; nunca concentra lógica de dominio.
- **Dependencias:** las APIs públicas de `ir`, `compiler`, `runtime`.
- **Observado en POC:** tres CLIs separadas (`orchestrator`, `irtool`, `runner`); se unifican en un solo bin.
- **Confianza:** Medium.

### `wit/` — Contratos WIT y fixtures

- **Responsabilidad:** interfaces públicas de componentes y fixtures de prueba (p. ej. `kaddo:canal@0.1.0`).
- **Confianza:** High — formato estandarizado.

### `tests/` y `examples/`

- **Responsabilidad:** pruebas E2E, regresión offline y ejemplos ejecutables.
- **Confianza:** Medium — la equivalencia con las 27 pruebas de la POC es criterio de salida de la etapa.

### `_reference/poc-agent-native/` (solo lectura)

- **Responsabilidad:** fuente histórica, excluida de build y de Git. No se modifica ni se importa como dependencia.
- **Política canónica:** [reference-policy.md](reference-policy.md) (WI AN-001).
- **Confianza:** High (congelada en `26b6a18e`).

## Dependencies and Integrations

### Internas del workspace

```text
cli ──▶ compiler ──▶ ir
 │        │
 │        └─▶ (adaptador LLM, build-time)
 └─▶ runtime ──▶ ir
```

Reglas invariantes:

- `ir` no depende de nada del producto.
- `runtime` no depende de `compiler` ni del LLM.
- `cli` no concentra lógica; es orquestación de APIs.

### Externas (asumidas para MVP)

| Dependencia | Rol | Confianza | Notas |
| --- | --- | --- | --- |
| `wasmtime` (versión baseline pendiente) | Runtime WASM Component Model | High | Baseline exacto es open question (`codebase.md` L145) |
| `wit-bindgen` / `wasm-tools` | Generación de bindings y manipulación de componentes | High | Observado en POC |
| Proveedor LLM (p. ej. Anthropic/OpenAI) | Generación de IR en build | Medium | Debe ser pluggable; sin credenciales en CI por defecto |
| `clap` / `thiserror` / `serde` / `anyhow` | Infraestructura CLI y errores | High | Patrón estándar en Rust |
| GitHub Releases (tarball firmado) | Distribución | Medium | Asunción del MVP (`codebase.md` L151) |

### Integraciones opcionales

- **Kaddo:** fuente de Work Items y conocimiento; **no es dependencia** del runtime ni del compilador. Se consume a través de archivos (`knowledge/`, `.kaddo/`), no mediante API embebida.

## Data Stores

- **Store de artefactos publicados:** filesystem local en MVP (ruta por configurar), con manifiestos JSON + SHA-256 por versión. OCI/registry remoto queda **diferido**.
- **Fixtures y suites YAML:** archivos versionados bajo `tests/` y `examples/`.
- **Contexto Kaddo:** `.kaddo/` y `knowledge/` como fuente de verdad del dominio, no como base de datos operacional.
- **Secretos de proveedor LLM:** nunca en el repositorio; se consumen vía variables de entorno o secretos de CI.

**No hay base de datos relacional ni almacén KV en el MVP.** La persistencia entre invocaciones del runtime queda explícitamente fuera de alcance inicial.

## Infrastructure

- **Lenguaje:** Rust 2024.
- **Build:** `cargo build --workspace --release`.
- **CI objetivo** (una vez exista workspace): `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`. Jobs con proveedor LLM real se controlan por secretos y no bloquean regresiones offline.
- **Distribución:** GitHub Releases con tarball/zip firmado para `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`.
- **Hosting:** no se asume ningún servicio hosted en MVP (producto es CLI standalone).
- **Observabilidad:** stdout/stderr estructurado como punto de partida; métricas y tracing diferidos.
- **Baseline runtime:** [`knowledge/tech/baselines/runtime-baseline.md`](baselines/runtime-baseline.md) — tabla segregada por path (Core WASM / Component Model) × etapa (Engine / Load / Cold-run / Warm-run / Cache-hit-run), fixtures `identity` + `point_sum` (v0.5 con WIT), 9 benches publicados por AN-017. No es SLA. INIT-08-01 formaliza la matrix cross-OS.

## Implicit Decisions (candidates)

Candidatos a ADR (no decisiones finales — pendientes de materializar bajo `knowledge/tech/decisions/` por `decision-agent`):

1. **ADR-candidate: Workspace Rust con 3 crates iniciales.** Alternativa descartada en MVP: cinco crates desde el día uno.
2. **ADR-candidate: Runtime y compiler sin dependencia cruzada.** Impacta reutilización del runtime en entornos sin toolchain.
3. **ADR-candidate: LLM solo en build-time.** Rechaza invocar modelos durante `run`; limita costos y latencia operativa.
4. **ADR-candidate: Fallo explícito en vez de valores por defecto.** Dependencias faltantes y capacidades no autorizadas abortan la ejecución.
5. **ADR-candidate: `_reference/poc-agent-native/` como fuente histórica congelada.** Excluir de build y de edición; migración por extracción, no por import.
6. **ADR-candidate: Publicación atómica + preservación de la versión anterior.** Mantener puntero activo y rollback implícito ante fallo de publicación.
7. **ADR-candidate: Baseline de distribución = 3 triples firmados vía GitHub Releases.** Difiere OCI/paquetes nativos.
8. **ADR-candidate: Allocator dlmalloc del guest por defecto; registros devueltos se copian al host; instancias persistentes se reinician entre tenants.**

## Open Questions

Heredadas de `knowledge/tech/codebase.md` que siguen abiertas tras el pase del architecture-agent (las bloqueantes ya quedaron en `[assumed]`):

- [open] ¿Qué partes del modelo `System Spec` y del contrato Semantic IR v0.5 son API pública versionada? (codebase.md L143)
- [open] Estrategia de evolución de Semantic IR para `string`, streams, efectos secuenciados, errores y estructuras adicionales. (L144)
- [open] Baseline soportada de Wasmtime/WIT/WASI y política de upgrade. (L145)
- [open] Caché de componentes y ciclo de vida de instancias sin fuga de memoria ni estado entre tenants. (L146)
- [open] Resolver reutilizable para dependencias (reemplaza compilación/instanciación repetida de la POC). (L147)
- [open] Modelo de permisos, límites de recursos, procedencia y firma antes de habilitar I/O real o multiusuario. (L149)
- [open] Imports externos, múltiples dependencias y versionamiento de contratos de componentes de terceros. (L150)
- [open] Capacidades estándar de consola/WASI del primer release y estrategia de stub en pruebas. (L152)
- [open] Umbrales de rendimiento (p50/p95/p99, throughput, memoria) una vez medida la línea base. (L153)
- [open] Qué artefactos de la POC se migran como fuente trazable y cuáles permanecen solo en `_reference/`. (L154)

## Areas Requiring Human Validation

1. **Estructura de crates del MVP (3 vs 5):** el `[assumed]` de arrancar con 3 crates debe revisarse al materializar `AN-001` (Work Item de arquitectura).
2. **Allocator/estrategia de memoria:** la asunción de dlmalloc + copia al host necesita benchmark antes de considerarla final.
3. **Matriz de plataformas de release:** `aarch64-unknown-linux-gnu` y Windows ARM64 no están en el baseline; validar si son requisito temprano.
4. **Rol del `orchestrator`:** confirmar si se mantiene fusionado dentro de `compiler` o se extrae en un cuarto crate antes del primer release.
5. **Modelo de distribución del store de artefactos:** filesystem local vs OCI vs ambos.
6. **Baseline exacto de Wasmtime/WIT/WASI** y política de actualización con pruebas de regresión.

### Project Resource candidates

Sistemas externos detectados en el conocimiento, propuestos como candidatos (no se crean automáticamente — requieren `kaddo resources create`):

- **candidate:** GitHub repository `Kaddo-kdd/agent-native` — host del código y los releases firmados. Interface: Git + GitHub Releases.
- **candidate:** Proveedor LLM (p. ej. Anthropic/OpenAI) — generación de IR en build. Interface: API HTTP + secretos.
- **candidate:** Registro OCI (futuro, diferido) — distribución alternativa de componentes y binarios.

────────────────────────
Agent: architecture-agent

Produced:
knowledge/tech/current-state.md

Next:
decision-agent
roadmap-agent
────────────────────────
