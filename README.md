# Agent-Native

> Sistema open source para convertir especificaciones funcionales y cambios incrementales en **componentes WebAssembly ejecutables, verificables y versionados** — con IA en tiempo de construcción y ejecución determinista sin LLM.

[![CI](https://github.com/Kaddo-kdd/agent-native/actions/workflows/ci.yml/badge.svg)](https://github.com/Kaddo-kdd/agent-native/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Estado del proyecto

🟡 **En desarrollo activo — fase temprana.** El baseline arquitectónico (hito **M1**) está aprobado y publicado como [`v0.0.2-m1`](https://github.com/Kaddo-kdd/agent-native/releases/tag/v0.0.2-m1). Lo que ya funciona:

- Workspace Rust (4 crates) + CI matrix en Linux/Windows/macOS.
- Modelo Semantic IR v0.5 + verifier portado desde la POC con tests verifier-only.
- Compilador IR → WASM Component Model determinista con hash SHA-256.

Lo que **todavía no**: runtime autónomo, CLI usable, pipeline de publicación atómica, I/O WASI, línea base de rendimiento. Ver el [roadmap](knowledge/delivery/roadmap.md) para la ruta completa.

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
cargo test --workspace     # 8 tests actualmente (6 verifier + 2 compiler determinism)
```

El binario `agent-native` existe pero todavía no tiene subcomandos funcionales (ver INIT-05-01 en el roadmap).

## Estructura del repositorio

```text
agent-native/
├── crates/
│   ├── ir/           # Semantic IR v0.5: modelo + verifier + schemas
│   ├── compiler/     # IR → WASM Component Model + hash SHA-256
│   ├── runtime/      # (planeado) carga + ejecución Wasmtime, sin LLM
│   └── cli/          # bin agent-native (planeado: build/verify/test/run/publish)
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
