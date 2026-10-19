<p align="center">
  <img src="assets/agent-native-banner.png" alt="Agent Native — Agentes de IA para un mundo real" width="100%">
</p>

# Agent-Native

> Sistema open source para convertir especificaciones funcionales y cambios incrementales en **componentes WebAssembly ejecutables, verificables y versionados** — con IA en tiempo de construcción y ejecución determinista sin LLM.

[![CI](https://github.com/Kaddo-kdd/agent-native/actions/workflows/ci.yml/badge.svg)](https://github.com/Kaddo-kdd/agent-native/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Kaddo-kdd/agent-native?color=blue)](https://github.com/Kaddo-kdd/agent-native/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Estado del proyecto

🟢 **M3 alcanzado (2026-10-19) — En desarrollo activo.** Hitos publicados:

- [`v0.0.4-m3`](https://github.com/Kaddo-kdd/agent-native/releases/tag/v0.0.4-m3) — **M3 Runtime autónomo con cobertura regresiva + línea base** (RM-03 completo). Cierre de INI-003 bajo los 7 criterios reformulados por decisión humana (D1/D2/D3).
- [`v0.0.3-hardening`](https://github.com/Kaddo-kdd/agent-native/releases/tag/v0.0.3-hardening) — **Hardening pass** post Technical Review (AN-014 end-to-end v0.5 Component Model + AN-015 políticas WIT granulares + AN-016 `CompileContext` sin dependencia del cwd).
- [`v0.0.2-m1`](https://github.com/Kaddo-kdd/agent-native/releases/tag/v0.0.2-m1) — **M1 Baseline arquitectónico** (RM-01 completo).

Lo que ya funciona:

- Workspace Rust (4 crates) + CI matrix en Linux/Windows/macOS (**75 tests** verdes).
- Modelo Semantic IR v0.5 + verifier (6 tests en `crates/ir/tests/verifier.rs`).
- Compilador IR → WASM Component Model determinista con hash SHA-256 (13 tests: 3 `compile_context` + 2 `determinism` + 8 pruebas de regresión POC en `crates/compiler/tests/poc_regression.rs`).
- Runtime autónomo: carga + invocación tipada (Core WASM y Component Model dentro del alcance documentado, lookup WIT `interface#operation`), `HostPolicy` deny-by-default pre-instantiate a 3 niveles (core, WIT interface, WIT operation), propagación tipada de errores (`RuntimeError`), `Resolver` content-addressed con método `reset()` para limpieza de caché (56 tests runtime).
- Executor YAML declarativo de aceptación: 32 tests (`crates/runtime/tests/executor_yaml.rs`), contrato normativo ([`knowledge/tech/poc-yaml-contract.md`](knowledge/tech/poc-yaml-contract.md)), 3 kinds de stub (`StateReader`, `StateWriter`, `Noop`), supresión de efectos de stubs posteriores tras miss en `StateReader` y reporte final estructurado con `CaseStatus::Error`.
- Matriz de equivalencia POC formalizada: 0 equivalencias exactas, 6 coberturas parciales y 4 aplazadas ([`knowledge/tech/poc-equivalence-matrix.md`](knowledge/tech/poc-equivalence-matrix.md)).
- Línea base técnica publicada en [`knowledge/tech/baselines/runtime-baseline.md`](knowledge/tech/baselines/runtime-baseline.md): tabla segregada por path (Core WASM / Component Model) × etapa (Engine / Load / Cold-run / Warm-run / Cache-hit-run) sobre fixtures `identity` y `point_sum`.

Próximos bloques del roadmap:

- **RM-04 (Specification, Agents & Build Pipeline):** Intérprete narrativa Markdown → `SystemSpec` (INIT-04-01), Adaptador LLM (INIT-04-02), evolución incremental y validación esclusas v1→v2 (INIT-04-03), pipeline de publicación atómica con rollback y store local de artefactos (INIT-04-05), trazabilidad Work Items (INIT-04-04).
- **RM-05 (Standalone CLI & Runtime API):** Subcomandos funcionales en `cli` (`spec`, `build`, `verify`, `test`, `run`, `publish`) (INIT-05-01), SDK Rust público reutilizable del runtime (INIT-05-02).
- **RM-06 (Runtime Security & Memory):** Policy loader manifest-driven con check al cargar (INIT-06-01, diferida por D2), límites de CPU/fuel y medición de memoria por instancia (INIT-06-02, diferida por D3), ciclo de vida multi-tenant (INIT-06-03).
- **RM-07 (Console I/O & WASI):** Baseline WASI preview2 (INIT-07-01), IR v0.6 (INIT-07-02), stubs funcionales Component Model y composición multi-componente (INIT-07-03).
- **RM-08 (Performance & Concurrency):** Harness y benchmarks formales cross-OS (INIT-08-01..02), warm pool (INIT-08-03), concurrencia host-side (INIT-08-04). Ver el [roadmap](knowledge/delivery/roadmap.md) para la ruta completa.

> La POC histórica vive en [`Trycatch-tv/poc-agent-native`](https://github.com/Trycatch-tv/poc-agent-native) (commit `26b6a18`) y, en una clonación local, bajo `_reference/`. Es **sólo lectura** — ver [`knowledge/tech/reference-policy.md`](knowledge/tech/reference-policy.md).

## Qué resuelve Agent-Native

Un LLM puede escribir código, pero esa salida no es un **contrato ejecutable** por sí misma. Agent-Native cierra la distancia entre *intención funcional* y *software ejecutable* en tres pasos verificables:

1. **Especificación** — una narrativa Markdown + requisitos se interpretan como `SystemSpec`.
2. **Construcción (con IA)** — un agente LLM propone Semantic IR; el **verifier** la valida; el **compilador** la convierte a WASM Component Model con WIT y hash SHA-256.
3. **Ejecución (sin IA)** — un runtime autónomo carga el componente publicado y lo ejecuta de forma determinista, con fallos explícitos ante dependencias faltantes o permisos no autorizados.

**Invariantes de diseño** (ADRs vinculantes):

- El runtime **no depende** del compilador ni del LLM ([ADR-002](knowledge/tech/decisions/ADR-002-runtime-independiente.md)).
- Los imports no autorizados fallan explícitamente — nunca con un valor por defecto ([ADR-004](knowledge/tech/decisions/ADR-004-seguridad-minima-transversal.md)).
- Toda compilación produce bytes determinísticos y un hash SHA-256 reproducible cross-OS.
- Interoperabilidad con componentes WIT-compatibles independientes del origen ([ADR-001](knowledge/tech/decisions/ADR-001-arquitectura-inicial.md)).

## Quick start

Requisitos: `cargo` 1.85+ (`rust-toolchain.toml` lo pinnea).

```bash
git clone https://github.com/Kaddo-kdd/agent-native
cd agent-native

cargo build --workspace
cargo test --workspace     # 75 tests (6 ir + 13 compiler + 56 runtime)
cargo bench -p agent-native-runtime --bench baseline -- --quick   # opcional
```

El binario `agent-native` existe como scaffolding de empaquetado; los subcomandos funcionales están planificados para RM-05 (ver INIT-05-01 en el roadmap).

## Estructura del repositorio

```text
agent-native/
├── assets/           # Imágenes y recursos estáticos del proyecto
├── crates/
│   ├── ir/           # Semantic IR v0.5: modelo + verifier + schemas
│   ├── compiler/     # IR → WASM Component Model + hash SHA-256
│   ├── runtime/      # carga + ejecución Wasmtime, HostPolicy, Resolver (sin LLM)
│   └── cli/          # bin agent-native (scaffold: subcomandos planeados en INIT-05-01)
├── knowledge/        # Conocimiento del proyecto gestionado por Kaddo
│   ├── business/     # Problema, usuarios, objetivos de negocio
│   ├── product/      # Visión, journeys, capacidades
│   ├── tech/         # Arquitectura, decisiones (ADRs), matriz de extracción
│   └── delivery/     # Roadmap, iniciativas, Work Items
├── .github/workflows/ # CI matrix (Linux/Windows/macOS)
└── _reference/       # POC histórica (ignorada por .gitignore; solo lectura)
```

## Desarrollo

Comandos del día a día:

```bash
cargo fmt --all                                        # formato
cargo clippy --workspace --all-targets -- -D warnings  # lint
cargo test --workspace                                 # tests
cargo build --workspace --release                      # release
```

La CI corre estos cuatro comandos en Linux, Windows y macOS en cada push y PR. Un paso adicional del workflow verifica mecánicamente las invariantes de ADR-001/002 vía `cargo tree`.

### Convenciones

- **Idioma del conocimiento:** español. Código, nombres de archivo, comandos y claves de configuración en inglés.
- **Edition Rust:** 2024.
- **Clippy pedantic** activo a nivel workspace. Los módulos portados desde la POC (`verifier.rs`, `codegen.rs`) tienen un `allow` acotado y documentado; código nuevo sigue bajo pedantic estricto.
- **`unsafe_code = deny`** a nivel workspace.

## Metodología

El desarrollo sigue **Kaddo** (Knowledge-Driven Development) como fuente de verdad del conocimiento:

- Cada decisión arquitectónica es un **ADR** materializado en [`knowledge/tech/decisions/`](knowledge/tech/decisions/).
- Cada incremento se materializa como un **Work Item** en [`knowledge/delivery/work-items/`](knowledge/delivery/work-items/) con criterios de aceptación verificables.
- El [**roadmap**](knowledge/delivery/roadmap.md) organiza el trabajo en 8 bloques (RM-01..RM-08) y ~29 iniciativas trazables.

Kaddo no es dependencia del producto — gestiona el **desarrollo** de Agent-Native, no sus artefactos ejecutables.

### Enlaces de conocimiento

- [Visión del producto](knowledge/product/product.md)
- [Capacidades planeadas](knowledge/product/capabilities.md)
- [Estado técnico actual](knowledge/tech/current-state.md)
- [Mapa del codebase](knowledge/tech/codebase.md)
- [Matriz de extracción POC → productos](knowledge/tech/extraction-matrix.md)
- [Decisiones arquitectónicas (ADRs)](knowledge/tech/decisions/)
- [Roadmap completo](knowledge/delivery/roadmap.md)

## Fuera del alcance inicial

Explícitamente diferidos ([Not Now del roadmap](knowledge/delivery/roadmap.md#not-now)):

- Generación completa de frontend/desktop end-to-end.
- SDKs para TypeScript/Python/Go.
- Servicio HTTP persistente.
- Registro OCI remoto como medio primario de distribución.
- Primitivas asíncronas en Semantic IR.
- Dependencia obligatoria de Kaddo en runtime, compiler o CLI.
- Soporte arbitrario para cualquier módulo WASM Core.

Esto **no** significa "nunca": significa "no antes de alcanzar los hitos M2..M8 con evidencia".

## Contribuir

Antes de abrir una PR revisa:

1. La [política de referencia](knowledge/tech/reference-policy.md) sobre uso de `_reference/`.
2. Los [ADRs activos](knowledge/tech/decisions/) que puedan afectar tu cambio.
3. El [roadmap](knowledge/delivery/roadmap.md) para entender el bloque/iniciativa correspondiente.

Las PRs no deben romper las invariantes auditadas por CI (`cargo tree` lint ADR-001/002, `clippy -D warnings`, tests).

## Licencia

[MIT](LICENSE).
