<p align="center">
  <img src="assets/agent-native-banner.png" alt="Agent Native — Agentes de IA para un mundo real" width="100%">
</p>

# Agent-Native

> Sistema open source para convertir especificaciones funcionales y cambios incrementales en **componentes WebAssembly ejecutables, verificables y versionados** — con IA en tiempo de construcción y ejecución determinista sin LLM.

[![CI](https://github.com/Kaddo-kdd/agent-native/actions/workflows/ci.yml/badge.svg)](https://github.com/Kaddo-kdd/agent-native/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Kaddo-kdd/agent-native?color=blue)](https://github.com/Kaddo-kdd/agent-native/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Estado del proyecto

🟢 **M7 alcanzado (2026-10-20) — En desarrollo activo hacia M8.** Hitos publicados:

- [`v0.0.7-m7`](https://github.com/Kaddo-kdd/agent-native/releases/tag/v0.0.7-m7) — **M7 Console I/O & WASI** (RM-07 completo). Adaptador WASI Preview 2 determinista en memoria con streams acotados (`RuntimeError::IoLimitExceeded`, AN-044), autorización granular e independiente de interfaces `wasi:cli/*` e `io/*` bajo `Requested ∩ Granted` (AN-045), Semantic IR v0.6 experimental (`string` y `result<T, E>` bajo `--experimental`, AN-048), y stubs deterministas de consola (`stdin`, `expected_stdout`, `expected_stderr`) en el executor YAML sin TTY (AN-049).
- [`v0.0.6-m6`](https://github.com/Kaddo-kdd/agent-native/releases/tag/v0.0.6-m6) — **M6 Runtime Security & Memory** (RM-06 completo). Límites de fuel deterministas y memoria (`ResourceLimiter`), contención en Windows MSVC (Wasmtime 29.0.1), aislamiento multi-tenant con Store efímero (ADR-005), verificación de procedencia (SHA-256) y firmas criptográficas Minisign Ed25519 100% offline, y suite de fixtures extremos.
- [`v0.0.4-m3`](https://github.com/Kaddo-kdd/agent-native/releases/tag/v0.0.4-m3) — **M3 Runtime autónomo con cobertura regresiva + línea base** (RM-03 completo). Cierre de INI-003 bajo los 7 criterios reformulados por decisión humana (D1/D2/D3).
- [`v0.0.3-hardening`](https://github.com/Kaddo-kdd/agent-native/releases/tag/v0.0.3-hardening) — **Hardening pass** post Technical Review (AN-014 end-to-end v0.5 Component Model + AN-015 políticas WIT granulares + AN-016 `CompileContext` sin dependencia del cwd).
- [`v0.0.2-m1`](https://github.com/Kaddo-kdd/agent-native/releases/tag/v0.0.2-m1) — **M1 Baseline arquitectónico** (RM-01 completo).

Lo que ya funciona:

- **Workspace Rust robusto (4 crates)** + CI matrix en Linux/Windows/macOS (**162 tests** verdes).
- **CLI Unificado `agent-native` (RM-05):** Subcomandos funcionales `spec`, `build`, `verify`, `test`, `run`, `publish`.
- **Integración LLM en tiempo de construcción (RM-04 / AN-046):** Generación asistida con modos `fixtures`, `synthesizer` y `real` (adaptador HTTP síncrono para Anthropic y OpenAI con validación estricta de Semantic IR v0.5 y sanitización de credenciales).
- **Control de Recursos y Límites en CLI (RM-06 / AN-047):** Invocación con `--fuel`, `--max-memory-bytes` y `--max-table-elements`, con reporte de `fuel_consumed` en JSON/texto y captura limpia de errores tipados.
- **Verificación Criptográfica y Procedencia (RM-06 / AN-041, AN-042):** Validación de integridad mediante hashes SHA-256 (`--manifest`, `--expected-sha256`) y verificación de firmas Minisign Ed25519 offline (`--signature`, `--public-key`).
- **Runtime Autónomo y Seguro (Wasmtime 29.0.1):** Aislamiento multi-tenant estricto con ciclo de vida de Store efímero (ADR-005), `HostPolicy` deny-by-default manifest-driven y contención probada frente a traps de CPU y desbordamiento de memoria.
- **Console I/O & WASI Preview 2 (RM-07 / AN-044, AN-045):** Adaptador determinista en memoria (`wasmtime-wasi = "=29.0.1"`) con autorización granular por interfaz (`allow_wasi_stdout`, `allow_wasi_stderr`, `allow_wasi_stdin`, `allow_wasi_stdio`) y límites de salida por stream (`RuntimeError::IoLimitExceeded`).
- **Semantic IR v0.5 (estable) + v0.6 (`--experimental` / AN-048):** Validación formal de tipos, signaturas WIT, imports de interfaces/operaciones, y soporte experimental de `string` y `result<T, E>` bajo `--experimental`.
- **Compilador Determinista IR → WASM:** Generación reproducible de Component Model con hash SHA-256 idéntico bit a bit cross-platform.
- **Executor YAML declarativo (RM-03 / RM-07 / AN-049):** Batería de pruebas de aceptación con contratos de stubs (`StateReader`, `StateWriter`, `Noop`), redirección determinista de `stdin`/`expected_stdout`/`expected_stderr` sin TTY, y soporte de `string`/`result<T, E>`.

Bloques del roadmap:

- ✅ **RM-01 a RM-07 Completados:** Baseline arquitectónico, compilador determinista, runtime autónomo, pipeline generativo con LLMs, CLI unificada, seguridad de recursos y Console I/O WASI Preview 2.
- 🟡 **RM-08 (Performance & Concurrency) — EN CURSO:** Harness y recolección cross-OS implementados. La [publicación medida candidata](docs/benchmarks/README.md) separa medianas y percentiles individuales, conserva resultados inestables y espera certificación humana. Caché / warm pool y concurrencia host-side siguen en el [roadmap](knowledge/delivery/roadmap.md).

> La POC histórica vive en [`Trycatch-tv/poc-agent-native`](https://github.com/Trycatch-tv/poc-agent-native) (commit `26b6a18`) y bajo `_reference/`. Es **sólo lectura** — ver [`knowledge/tech/reference-policy.md`](knowledge/tech/reference-policy.md).

## Qué resuelve Agent-Native

Un LLM puede escribir código, pero esa salida no es un **contrato ejecutable** por sí misma. Agent-Native cierra la distancia entre *intención funcional* y *software ejecutable* en tres pasos verificables:

1. **Especificación** — una narrativa Markdown + requisitos se interpretan como `SystemSpec`.
2. **Construcción (con IA)** — un agente LLM propone Semantic IR; el **verifier** la valida; el **compilador** la convierte a WASM Component Model con WIT y hash SHA-256.
3. **Ejecución (sin IA)** — un runtime autónomo carga el componente publicado y lo ejecuta de forma determinista, con cuotas de fuel, límites de memoria y permisos explícitos.

**Invariantes de diseño** (ADRs vinculantes):

- El runtime **no depende** del compilador ni del LLM ([ADR-002](knowledge/tech/decisions/ADR-002-runtime-independiente.md)).
- Los imports no autorizados fallan explícitamente — nunca con un valor por defecto ([ADR-004](knowledge/tech/decisions/ADR-004-seguridad-minima-transversal.md)).
- Toda invocación se ejecuta sobre un **Store efímero** descartado incondicionalmente tras cada llamada ([ADR-005](knowledge/tech/decisions/ADR-005-store-efimero-invariantes-aislamiento.md)).
- Toda compilación produce bytes determinísticos y un hash SHA-256 reproducible cross-OS.
- Interoperabilidad con componentes WIT-compatibles independientes del origen ([ADR-001](knowledge/tech/decisions/ADR-001-arquitectura-inicial.md)).

## Quick start

Requisitos: `cargo` 1.85+ (`rust-toolchain.toml` lo pinnea).

```bash
git clone https://github.com/Kaddo-kdd/agent-native
cd agent-native

cargo build --workspace
cargo test --workspace     # 162 tests verdes (18 ir + 17 compiler + 73 runtime + 54 cli)
```

### Uso de la CLI

Construir un componente desde una especificación:

```bash
agent-native build --spec spec.md --mode synthesizer --out ./dist/calculator.wasm
```

Verificar integridad y firmas criptográficas:

```bash
agent-native verify --component ./dist/calculator.wasm --manifest ./dist/manifest.json
```

Ejecutar con cuotas deterministas de fuel y límites de memoria:

```bash
agent-native run \
  --component ./dist/calculator.wasm \
  --function calculate \
  --fuel 100000 \
  --max-memory-bytes 16777216 \
  10 20
```

## Estructura del repositorio

```text
agent-native/
├── assets/           # Imágenes y recursos estáticos del proyecto
├── crates/
│   ├── ir/           # Semantic IR v0.5: modelo AST + verifier + schemas
│   ├── compiler/     # IR → WASM Component Model determinista + hash SHA-256
│   ├── runtime/      # Runtime Wasmtime 29, Tenant, HostPolicy deny-by-default, límites de recursos
│   └── cli/          # CLI unificada (spec, build, verify, test, run, publish) y agentes LLM
├── knowledge/        # Conocimiento del proyecto gestionado por Kaddo
│   ├── business/     # Problema, usuarios, objetivos de negocio
│   ├── product/      # Visión, journeys, capacidades
│   ├── tech/         # Arquitectura, decisiones (ADRs), línea base, guías CLI
│   └── delivery/     # Roadmap, iniciativas, Work Items (completed y draft)
├── .github/workflows/ # CI matrix (Linux/Windows/macOS) y release packaging
└── _reference/       # POC histórica (ignorada por .gitignore; solo lectura)
```

## Desarrollo

Comandos del día a día:

```bash
cargo fmt --all                                        # formato
cargo clippy --workspace --all-targets -- -D warnings  # lint
cargo test --workspace                                 # 162 tests
cargo build --workspace --release                      # binario optimizado
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
- [Guía y referencia de la CLI](knowledge/tech/cli-guide.md)
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

Esto **no** significa "nunca": significa "no antes de alcanzar los hitos correspondientes con evidencia".

## Contribuir

Antes de abrir una PR revisa:

1. La [política de referencia](knowledge/tech/reference-policy.md) sobre uso de `_reference/`.
2. Los [ADRs activos](knowledge/tech/decisions/) que puedan afectar tu cambio.
3. El [roadmap](knowledge/delivery/roadmap.md) para entender el bloque/iniciativa correspondiente.

Las PRs no deben romper las invariantes auditadas por CI (`cargo tree` lint ADR-001/002, `clippy -D warnings`, tests).

## Licencia

[MIT](LICENSE).
