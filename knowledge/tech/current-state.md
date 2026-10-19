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

- **Dirección confirmada:** Rust 2024 (1.85) + workspace Cargo, WebAssembly Component Model vía Wasmtime (declarada en rama compatible 27.0, resuelta por Cargo.lock), contratos WIT, generación LLM solo en build-time, runtime independiente sin LLM.
- **Workspace operativo (4 crates):** `crates/ir`, `crates/compiler`, `crates/runtime`, `crates/cli` (bin). Manifest del workspace (`Cargo.toml`) y `Cargo.lock` consolidados. CI multi-OS (Linux, Windows, macOS) activa y verificada en verde. Binarios y tests de integración compilados y operativos en CI.
- **Modelo de memoria:** la asunción inicial de dlmalloc fue retirada y formalmente resuelta en ADR-005: la estrategia productiva es Canonical ABI con copy-on-return y un Store nuevo por invocación; dlmalloc no es una elección arquitectónica obligatoria.
- **Referencia histórica:** `_reference/poc-agent-native/` (commit `26b6a18e`), preservada como fuente histórica de solo lectura excluida del build y de Git (política canónica: `knowledge/tech/reference-policy.md`).

## Modules

Estado y nivel de confianza de cada módulo basado en la implementación real verificable del workspace:

### `crates/ir` — Semantic IR + Verifier

- **Estado:** Implementado (M2 — AN-004, AN-005).
- **Responsabilidad:** modelo de IR v0.5, parser, inferencia de tipos, verificador semántico y diagnósticos localizados.
- **Dependencias salientes:** ninguna del producto (no depende de LLM, runtime ni Kaddo; solo `serde`, `serde_json`).
- **Verificación:** 6 pruebas de regresión unitaria del verificador en `crates/ir/tests/verifier.rs` (`verify_ir` en aislamiento, diagnósticos estructurados y validación de schemas). Las ocho pruebas POC de compilación y regresión corresponden a `crates/compiler` (`crates/compiler/tests/poc_regression.rs` — AN-007).
- **Confianza:** High.

### `crates/compiler` — IR → WASM Component

- **Estado:** Implementado (M2 — AN-006, AN-007, AN-008, AN-016).
- **Responsabilidad:** compilación determinista de documentos Semantic IR v0.5 verificados a binarios WASM Component Model con contratos WIT (`compile_ir_to_wasm`, `compile_ir_to_wasm_with_context`), validación de bytes (`validate_wasm_bytes`), utilidades de inspección WAT (`print_wasm_to_wat`) y cálculo de hash canónico SHA-256 (`sha256_hex`).
- **Fronteras arquitectónicas (ADR-001/ADR-002):** el compilador **no** incluye orquestación, agentes LLM de implementación ni publicación atómica. Estas capacidades pertenecen a la capa de orquestación (módulos `spec::`, `build::`, `publish::` previstos para RM-04, albergados temporalmente en `cli` o en un futuro crate `orchestrator`) y a INIT-04-05.
- **Dependencias:** `crates/ir` y toolchain WASM (`wasm-encoder`, `wasmparser`, `wasmprinter`, `wit-component`, `wit-parser`, `sha2`). Sin dependencia de adaptadores LLM ni de `runtime` (auditado por CI con `cargo tree`).
- **Verificación:** 8 pruebas de regresión del pipeline de compilación de la POC reproducidas y verificadas en CI (`crates/compiler/tests/poc_regression.rs` — AN-007).
- **Confianza:** High.

### `crates/runtime` — Ejecución y pruebas de aceptación

- **Estado:** Implementado dentro del alcance de M3 (M3 alcanzado 2026-10-19 — AN-009..AN-020).
- **Responsabilidad:** carga de componentes Core WASM y Component Model vía Wasmtime (rama compatible 27.0), soporte de contratos WIT e invocación Component Model dentro del alcance documentado (la composición multi-componente sigue diferida a RM-07), resolución y caché content-addressed de artefactos (`Resolver` con método `reset()`), autorización `HostPolicy` deny-by-default pre-instantiate a 3 niveles (core, WIT interface, WIT operation), y ejecutor de suites de aceptación YAML con contrato normativo (`knowledge/tech/poc-yaml-contract.md`).
- **Capacidades operativas:**
  - Carga Core WASM + Component Model. En Component Model: imports sin dependencias funcionales delegan a `invoke`; imports funcionales producen `HostError` tipado diferido a RM-07.
  - Fail-explicit sin fallback a `0`/`false` en lifting de resultados.
  - Executor YAML con 7 submódulos y 3 kinds de stub declarativos (`StateReader`, `StateWriter`, `Noop`). En caso de miss en `StateReader`: detección obligatoria, registro del miss, envenenamiento del contexto, supresión de efectos observables posteriores y fallo final con `CaseStatus::Error`.
  - Línea base de latencia documentada en `knowledge/tech/baselines/runtime-baseline.md` (cold start, warm invocation, cache-hit).
- **Fuera de alcance de M3 (diferido según resolución humana D1/D2/D3 e INI-003):** stubs funcionales de Component Model (RM-07), composición multi-componente (RM-07), host providers reales (INIT-06-01), validación manifest-driven en `load_component` (INIT-06-01, decisión D2), medición de memoria por instancia (INIT-06-02, decisión D3), soporte WASI preview2 (RM-07), esclusas v1→v2 integrales (M4 — INIT-04-03/04-05), interrupción inmediata / trap en guest ante callback error (investigación post-M3; issue experimental de wasmtime 27 documentado en `host_ctx.rs`).
- **Equivalencia POC:** documentada en `knowledge/tech/poc-equivalence-matrix.md` con 3 grados verificables: **0 `equivalencia-exacta` + 6 `cobertura-parcial` + 4 `aplazado`**. No se afirma reproducción literal de las 10 pruebas unitarias del runner de la POC (decisión humana D1).
- **Dependencias:** `wasmtime` (rama 27.0), `crates/ir` (metadatos), `serde`, `serde_yaml2`, `sha2`, `thiserror`. **No** depende de `compiler` ni de adaptadores LLM (enforced en CI).
- **Confianza:** High para el alcance certificado de M3; Medium para extensiones diferidas.

### `crates/cli` (bin) — Interfaz pública

- **Estado:** Scaffolding inicial implementado; subcomandos funcionales previstos para RM-05 (INIT-05-01).
- **Responsabilidad actual:** punto de entrada ejecutable (`crates/cli/src/main.rs`) que enlaza las dependencias del workspace y valida el empaquetado del binario.
- **Responsabilidad planificada:** subcomandos `spec`, `build`, `verify`, `test`, `run` y `publish`. Per ADR-001, la CLI nunca concentra lógica de dominio; albergará internamente los módulos de orquestación previstos para RM-04 (`spec::`, `build::`, `publish::`) hasta que se cumplan las condiciones para su extracción a un crate `orchestrator` separado.
- **Dependencias:** APIs públicas de `ir`, `compiler`, `runtime`.
- **Confianza:** High (scaffolding verificado); Medium (subcomandos y orquestación futura).

### `wit/` — Contratos WIT y fixtures

- **Estado:** Convenciones documentadas en `wit/README.md`.
- **Responsabilidad:** declara las convenciones de ubicación (`wit/public/`, `wit/internal/`, `wit/fixtures/`). Los contratos públicos del producto todavía no se han materializado (su primer archivo requerirá ADR y lint CI asociado). Los fixtures WIT existentes permanecen co-ubicados con sus respectivos crates (`crates/compiler/wit/order.wit`, `crates/compiler/tests/fixtures/wit/`).
- **Confianza:** High — convención normativa establecida.

### `tests/` y `examples/`

- **Estado:** Parcialmente implementado.
- **Responsabilidad:** suites de pruebas unitarias y de integración por crate (60+ pruebas en workspace), fixtures y suites declarativas YAML bajo contrato normativo. Las pruebas de integración E2E completas y ejemplos ejecutables corresponden a RM-04/RM-05. La equivalencia con la POC se rige por la matriz re-evaluada (decisión D1) y el escenario de esclusas v1→v2 se valida en M4 (INIT-04-03/INIT-04-05).
- **Confianza:** High (cobertura actual M3); Medium (escenarios E2E futuros).

### `_reference/poc-agent-native/` (solo lectura)

- **Estado:** Congelada en commit `26b6a18e`.
- **Responsabilidad:** fuente histórica de solo lectura, excluida de build y de Git. No se modifica ni se importa como dependencia.
- **Política canónica:** [reference-policy.md](reference-policy.md) (WI AN-001).
- **Confianza:** High.

## Capacidades: Implementadas, Parcialmente implementadas y Planificadas

Para dar visibilidad clara sobre el estado real del producto al cierre de M3:

### 1. Capacidades Implementadas (verificadas en código y CI)

- **Workspace Rust 2024 unificado:** 4 crates (`ir`, `compiler`, `runtime`, `cli`), `Cargo.lock` congelado, toolchain pinneado (Rust 1.85.0).
- **CI multi-OS completa:** GitHub Actions en Linux, Windows y macOS ejecutando `fmt`, `clippy -D warnings`, `test`, `build --release` y lint automático de aislamiento de dependencias ADR-001/ADR-002 (`cargo tree`).
- **Semantic IR v0.5 (`crates/ir`):** modelo semántico, parser, inferencia de tipos, verificador y diagnósticos estructurados (6 pruebas unitarias en `crates/ir/tests/verifier.rs`).
- **Compilador WASM Component Model determinista (`crates/compiler`):** compilación IR v0.5 a WASM con contratos WIT (`compile_ir_to_wasm`, `compile_ir_to_wasm_with_context`), validación de bytes (`validate_wasm_bytes`), inspección WAT (`print_wasm_to_wat`), hash canónico SHA-256 (`sha256_hex`) y 8 pruebas de regresión del compilador de la POC reproducidas en CI (`crates/compiler/tests/poc_regression.rs` — AN-007). Independiente de LLM y runtime.
- **Runtime Wasmtime independiente (`crates/runtime`):** motor Wasmtime (declarada en rama compatible `27.0`, resuelta a `27.0.0` por `Cargo.lock`) sin dependencia de compilador ni LLM (ADR-002). Carga Core WASM y Component Model (type-only / sin imports funcionales).
- **Seguridad deny-by-default pre-instantiate:** control de acceso a host capabilities a 3 niveles (core, WIT interface, WIT operation) con fallo explícito (`RuntimeError::UnauthorizedImport`) antes de ejecutar guest code.
- **Executor YAML declarativo con contrato normativo:** soporte de suites YAML (7 submódulos), 3 kinds de stub (`StateReader`, `StateWriter`, `Noop`), records por nombre+tipo+valor, fail-explicit sin fallback a `0`/`false` en lifting de resultados, supresión de efectos de stubs posteriores tras miss en `StateReader` y reporte final con `CaseStatus::Error`.
- **Caché content-addressed:** `Resolver` de dependencias y componentes en runtime para evitar recompilaciones / recargas redundantes, con método `reset()` para limpieza de caché.
- **Línea base de latencia documentada:** benchmarks publicados en `knowledge/tech/baselines/runtime-baseline.md` cubriendo cold start, warm invocation y cache-hit por path (Core WASM y Component Model).

### 2. Capacidades Parcialmente Implementadas

- **Crate `crates/cli`:** Scaffolding inicial ejecutable (`main.rs`) compilado y probado en CI; subcomandos funcionales de dominio (`build`, `verify`, `test`, `run`, `publish`) planificados para RM-05.
- **Suites de aceptación de la POC:** Cobertura documentada en matriz de equivalencia 1:1 con 3 grados (**0 exactas, 6 parciales, 4 aplazadas**; decisión D1).
- **Component Model en runtime:** Funcional para componentes autónomos o con imports de tipos; componentes con imports funcionales producen error tipado explícito (`HostError`) cuyo soporte completo está planificado para RM-07.

### 3. Capacidades Planificadas (Roadmap post-M3)

- **RM-04 (Specification, Agents & Build Pipeline):** Intérprete narrativa Markdown → `SystemSpec` (INIT-04-01), Adaptador LLM con modos real/fixtures/synthesizer (INIT-04-02), evolución incremental de especificaciones y validación integral esclusas v1→v2 (INIT-04-03), publicación atómica con rollback y store local de artefactos (INIT-04-05), trazabilidad artefacto ↔ Work Item (INIT-04-04).
- **RM-05 (Standalone CLI & Runtime API):** Subcomandos funcionales en `cli` con formato JSON opcional (INIT-05-01), SDK Rust público reutilizable del runtime (INIT-05-02), pipeline de release en 3 triples con firma (INIT-05-03).
- **RM-06 (Runtime Security & Memory):** Policy loader manifest-driven con check al cargar (INIT-06-01, diferida de M3 por D2), límites de CPU/fuel y medición de memoria por instancia (INIT-06-02, diferida de M3 por D3), ciclo de vida y aislamiento multi-tenant (INIT-06-03), verificación de integridad `--verify` (INIT-06-04), fixtures de ejecución extrema (INIT-06-05).
- **RM-07 (Console I/O & WASI):** Baseline WASI preview2 y stdio autorizado (INIT-07-01), extensión IR v0.6 (`string`, errores) (INIT-07-02), stubs deterministas de I/O, stubs funcionales CM y composición multi-componente (INIT-07-03).
- **RM-08 (Performance & Concurrency):** Harness de benchmark cross-OS (INIT-08-01), línea base pública cross-OS (INIT-08-02), warm pool cuantificado vs línea base (INIT-08-03), concurrencia host-side (INIT-08-04).

## Dependencies and Integrations

### Internas del workspace

Se distingue explícitamente entre los módulos implementados hoy (M3) y la arquitectura prevista para fases posteriores:

#### Módulos implementados hoy (M3)

```text
cli (scaffolding) ──▶ compiler ──▶ ir
       │
       └────────────▶ runtime ──▶ ir
```

#### Arquitectura prevista (RM-04 / RM-05)

La orquestación (narrativa → spec → LLM → publicación) reside en `cli` (o en un futuro crate `orchestrator` separado según las reglas de extracción de ADR-001) y consume el adaptador LLM durante el build para generar el IR antes de pasarlo al compilador. El crate `compiler` nunca depende del adaptador LLM:

```text
cli (orquestación / futuro orchestrator)
 │
 ├──▶ Adaptador LLM (build-time) ──▶ [genera IR]
 ├──▶ compiler ──▶ ir
 └──▶ runtime ──▶ ir
```

Reglas invariantes:

- `ir` no depende de nada del producto.
- `compiler` depende únicamente de `ir`.
- `runtime` no depende de `compiler` ni del LLM (enforced en CI).
- `cli` no concentra lógica de dominio; es orquestación de APIs.

### Externas

| Dependencia | Rol | Estado / Versión | Notas |
| --- | --- | --- | --- |
| `wasmtime` | Runtime WASM Component Model | Rama compatible `27.0` (resuelta a `27.0.0` por `Cargo.lock`) | Configurada sin features por defecto; activa `cranelift`, `runtime`, `component-model` |
| `wit-component` / `wit-parser` | Manipulación de componentes WIT | Pinned: `0.219.2` | En `crates/compiler` |
| `wasm-encoder` / `wasmparser` / `wasmprinter` | Manipulación y validación WASM | Pinned: `0.221.0` | En `crates/compiler` |
| `serde` / `serde_json` / `serde_yaml2` / `sha2` / `thiserror` | Infraestructura de datos, errores y hash | Integrado | Versiones consolidadas en workspace |
| Proveedor LLM (p. ej. Anthropic/OpenAI) | Generación de IR en build (RM-04) | Planificado (Medium) | Pluggable; solo build-time; sin credenciales en CI por defecto |
| `clap` | Infraestructura CLI (RM-05) | Planificado (High) | Para subcomandos de INIT-05-01 |
| GitHub Releases (tarball firmado) | Distribución (RM-05) | Planificado (Medium) | 3 triples objetivo (INIT-05-03) |

### Integraciones opcionales

- **Kaddo:** fuente de Work Items y conocimiento; **no es dependencia** del runtime ni del compilador. Se consume a través de archivos (`knowledge/`, `.kaddo/`), no mediante API embebida.

## Data Stores

- **Store de artefactos publicados:** filesystem local en MVP (ruta por configurar), con manifiestos JSON + SHA-256 por versión. OCI/registry remoto queda **diferido**.
- **Fixtures y suites YAML:** archivos versionados bajo `tests/` y `examples/`.
- **Contexto Kaddo:** `.kaddo/` y `knowledge/` como fuente de verdad del dominio, no como base de datos operacional.
- **Secretos de proveedor LLM:** nunca en el repositorio; se consumen vía variables de entorno o secretos de CI.

**No hay base de datos relacional ni almacén KV en el MVP.** La persistencia entre invocaciones del runtime queda explícitamente fuera de alcance inicial.

## Infrastructure

- **Lenguaje:** Rust 2024 (1.85.0 pinneado en `rust-toolchain.toml`).
- **Build:** `cargo build --workspace --release`.
- **CI operativa (.github/workflows/ci.yml):** activa en Linux (`ubuntu-latest`), Windows (`windows-latest`) y macOS (`macos-latest`). Verifica en cada push/PR:
  - Formato: `cargo fmt --all -- --check`.
  - Lints: `cargo clippy --workspace --all-targets -- -D warnings`.
  - Pruebas: `cargo test --workspace --all-targets`.
  - Compilación release: `cargo build --workspace --release`.
  - Lint de arquitectura: script que verifica que `agent-native-runtime` no dependa de `compiler` ni viceversa (ADR-001/ADR-002).
- **Distribución:** GitHub Releases con tarball/zip firmado para `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin` (planificado RM-05).
- **Hosting:** no se asume ningún servicio hosted en MVP (producto es CLI standalone).
- **Observabilidad:** stdout/stderr estructurado como punto de partida; métricas y tracing diferidos.
- **Baseline runtime:** [`knowledge/tech/baselines/runtime-baseline.md`](baselines/runtime-baseline.md) — tabla segregada por path (Core WASM / Component Model) × etapa (Engine / Load / Cold-run / Warm-run / Cache-hit-run), fixtures `identity` + `point_sum` (v0.5 con WIT), 9 benches publicados por AN-017. No es SLA. INIT-08-01 formaliza la matrix cross-OS.

## Architectural Decisions (ADR-001..005 materializadas)

Las decisiones iniciales del proyecto fueron formalizadas y aprobadas bajo `knowledge/tech/decisions/` durante M1, M2 y M3:

1. **ADR-001 (Aprobada):** Arquitectura inicial del workspace con 4 crates (`ir`, `compiler`, `runtime`, `cli`), reglas de dependencia unidireccionales y orquestación alojada temporalmente en `cli`.
2. **ADR-002 (Aprobada):** Runtime independiente del compilador y del LLM, verificado mecánicamente por CI.
3. **ADR-003 (Aprobada):** Modelo de Semantic IR v0.5 como baseline normativa y política de compatibilidad hacia v0.6.
4. **ADR-004 (Aprobada):** Estrategia de seguridad mínima transversal (fail-explicit, deny-by-default, SHA-256).
5. **ADR-005 (Aprobada):** Modelo de memoria y Canonical ABI (la estrategia productiva es Canonical ABI con copy-on-return y un Store nuevo por invocación; dlmalloc no es una elección arquitectónica obligatoria, retirando el supuesto original).

Candidatos a ADR futuros (pendientes de materializar por `decision-agent` según el roadmap):
- **Candidato ADR (RM-04):** Publicación atómica + preservación de versión anterior y rollback.
- **Candidato ADR (RM-05):** Empaquetado, firma y verificación de releases para los 3 triples objetivo.

## Open Questions

Heredadas de `knowledge/tech/codebase.md`, con su estado actual post-M3:

- [open] ¿Qué partes del modelo `System Spec` y del contrato Semantic IR v0.5 son API pública versionada? (codebase.md L143 — RM-04)
- [open] Estrategia de evolución de Semantic IR para `string`, streams, efectos secuenciados, errores y estructuras adicionales. (L144 — RM-07)
- [resuelto en M2/M3] Baseline soportada de Wasmtime/WIT: declarada en rama compatible `27.0` en `crates/runtime/Cargo.toml`, con versión concreta `27.0.0` resuelta por `Cargo.lock`.
- [open] Caché de componentes y ciclo de vida de instancias sin fuga de memoria ni estado entre tenants. (L146 — RM-06)
- [resuelto en M3] Resolver reutilizable para dependencias: implementado en `crates/runtime/src/resolver.rs` (AN-013).
- [open] Modelo de permisos, límites de recursos, procedencia y firma antes de habilitar I/O real o multiusuario. (L149 — RM-06)
- [open] Imports externos, múltiples dependencias y versionamiento de contratos de componentes de terceros. (L150 — objetivo diferido)
- [open] Capacidades estándar de consola/WASI del primer release y estrategia de stub en pruebas. (L152 — RM-07)
- [open] Umbrales de rendimiento (p50/p95/p99, throughput, memoria) una vez completada la línea base y benchmarks. (L153 — RM-08)
- [resuelto en M1-M3] Artefactos de la POC migrados vs históricos: política de solo lectura en `reference-policy.md`, matriz de extracción y matriz de equivalencia POC (0 exactas + 6 parciales + 4 aplazadas).

## Areas Requiring Human Validation

### Decisiones humanas consolidadas (M1..M3)

1. **Estructura de crates del MVP:** Consolidada en 4 crates (`ir`, `compiler`, `runtime`, `cli`) bajo ADR-001.
2. **Estrategia de memoria:** Resuelta en ADR-005: la estrategia productiva es Canonical ABI con copy-on-return y un Store nuevo por invocación; dlmalloc no es una elección arquitectónica obligatoria.
3. **Rol del orchestrator:** Alojado como submódulos en `cli` (`spec::`, `build::`, `publish::` previstos para RM-04, no como implementaciones existentes) hasta cumplir criterios objetivos de extracción según ADR-001; nunca fusionado en `compiler`.
4. **Baseline de Wasmtime:** Declarada en la rama compatible `27.0` en `Cargo.toml`, con versión concreta `27.0.0` resuelta por `Cargo.lock`.
5. **Criterios de salida de M3 y equivalencia POC (Resoluciones D1/D2/D3 del 2026-10-19):**
   - **D1:** Aceptación de la matriz de equivalencia con 3 grados (**0 exactas, 6 parciales, 4 aplazadas**), descartando la afirmación de reproducción literal de las 10 pruebas unitarias del runner de la POC.
   - **D2:** Deny-by-default pre-instantiate a 3 niveles verificado antes de ejecutar guest code; validación manifest-driven en `load_component` diferida formalmente a INIT-06-01.
   - **D3:** Línea base de latencia publicada; medición de memoria por instancia diferida formalmente a INIT-06-02.

### Áreas que requerirán validación humana en fases posteriores

1. **Matriz de plataformas de release:** Validar si `aarch64-unknown-linux-gnu` o Windows ARM64 se incorporan tras el release inicial (RM-05).
2. **Modelo de distribución del store de artefactos:** Filesystem local en MVP vs soporte de registro OCI (RM-04 / RM-05).
3. **Umbrales definitivos de rendimiento:** Fijar SLAs de p50/p95/p99 tras los benchmarks de INIT-08-01 e INIT-08-02.

### Project Resource candidates

Sistemas externos detectados en el conocimiento, propuestos como candidatos:

- **candidate:** GitHub repository `Kaddo-kdd/agent-native` — host del código y los releases firmados. Interface: Git + GitHub Releases.
- **candidate:** Proveedor LLM (p. ej. Anthropic/OpenAI) — generación de IR en build (RM-04). Interface: API HTTP + secretos.
- **candidate:** Registro OCI (futuro, diferido) — distribución alternativa de componentes y binarios.

────────────────────────
Agent: architecture-agent

Produced:
knowledge/tech/current-state.md

Next:
decision-agent
roadmap-agent
────────────────────────
