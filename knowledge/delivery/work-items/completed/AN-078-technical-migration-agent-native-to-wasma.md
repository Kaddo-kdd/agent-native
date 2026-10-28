---
id: AN-078
type: feature
status: completed
created_at: 2026-10-27T00:00:00.000Z
ready_at: '2026-10-28'
completed_at: '2026-10-28'
title: Migración técnica de Agent Native a Wasma
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-11
priority: P0
depends_on:
  - AN-077
domains:
  - Core Runtime & Tooling
  - 'Developer Experience, Documentation & Branding'
code:
  - Cargo.toml
  - Cargo.lock
  - crates/cli/**
  - crates/compiler/**
  - crates/ir/**
  - crates/runtime/**
  - .github/workflows/**
  - website/scripts/test-docs.mjs
expected_value: >-
  Alineación consistente de manifests Cargo, lockfile, CLI binaria, imports,
  tests y CI con la identidad aprobada de Wasma (ADR-008), preservando contratos
  Semantic IR/WIT, garantías de seguridad y trazabilidad.
project_state: ai-assisted
template_version: 1
ready_at: '2026-10-28'
---

# AN-078 — Migración técnica de Agent Native a Wasma

## Contexto y problema

Tras la aprobación formal del **ADR-008** en AN-077, la identidad técnica de Wasma debe consolidarse en todo el workspace Rust antes de la release pública v0.1.0. Esto implica renombrar los crates a `wasma-ir`, `wasma-compiler`, `wasma-runtime`, el paquete CLI a `wasma-cli` (que produce exclusivamente el binario `wasma`), y actualizar los ~192 imports en código Rust, workflows de CI/CD y scripts de validación.

## Alcance

Aplicar exclusivamente los nombres y decisiones aprobados en el **ADR-008** sobre las siguientes superficies:

- `Cargo.toml`, manifiestos de crates (`wasma-ir`, `wasma-compiler`, `wasma-runtime`, `wasma-cli`) y `Cargo.lock`.
- Ejecutable de CLI: el paquete Cargo `wasma-cli` compila exclusivamente el binario `wasma` por defecto.
- Alias opcional: si se habilita `--features legacy-alias`, compilar el alias secundario `agent-native` con advertencia de deprecación en `stderr`.
- Variables de entorno: `WASMA_LLM_*` oficial con precedencia sobre `AGENT_NATIVE_LLM_*`, manejo seguro de valores vacíos y sanitización de credenciales en avisos.
- Declaraciones de dependencias internas relativas y rutas `use` afectadas en todo el workspace Rust.
- Pruebas unitarias, de integración, regresión y E2E.
- Fixtures, scripts de benchmark (`scripts/bench_collect.py`, `scripts/worker_bench_collect.py`) y ejemplos.
- GitHub Actions de CI, benchmarks y release (`.github/workflows/*.yml`).
- Validador y tutoriales del portal web (`website/scripts/test-docs.mjs`, `website/src/content/docs/`).

## Restricciones arquitectónicas

- Preservar los contratos Semantic IR y WIT existentes sin alteración.
- No renombrar interfaces externas versionadas sin una decisión independiente.
- No cambiar la semántica del compilador ni del runtime.
- Mantener las garantías de `Store` e instancia fresca (ADR-005), aislamiento de tenants y políticas de seguridad.
- Mantener los workers experimentales deshabilitados por defecto.
- No reescribir los commits, ADRs, Work Items, hashes, resultados ni artefactos históricos (AN-001..AN-067).
- No reinicializar Kaddo ni perder su identificador o trazabilidad.
- No realizar cambios funcionales que no sean estrictamente necesarios para la migración.
- En caso de contingencia o reversión, emplear commits trazables (`git revert`), descartando `git reset --hard`.

## Criterios de aceptación

- AC-01: Los nombres públicos definidos en ADR-008 están implementados consistentemente en todo el workspace Rust (`wasma-cli`, `wasma-ir`, `wasma-compiler`, `wasma-runtime`).
- AC-02: La CLI renombrada ejecuta los cinco subcomandos reales (`build`, `verify`, `run`, `test`, `publish`) respetando sus contratos existentes.
- AC-03: La instalación estándar provee únicamente el binario `wasma`. El alias `agent-native` está desacoplado bajo la feature opcional `--features legacy-alias` (deshabilitada por defecto) con mensaje de deprecación para v0.2.0.
- AC-04: Los manifiestos de Cargo y el lockfile resuelven correctamente todas las dependencias internas relativas sin errores de resolución.
- AC-05: Pasan `cargo fmt --check`, `cargo clippy`, compilación y suite completa de pruebas normales y experimentales en las plataformas soportadas.
- AC-06: Los tests E2E y suites de benchmarks (`bench_latency`) verifican que se preserva íntegramente el comportamiento de los componentes WebAssembly.
- AC-07: Documentación, tutoriales, validador web (`test-docs.mjs`) y CI utilizan los nuevos nombres, sin depender de rutas heredadas no aprobadas.
- AC-08: Las referencias históricas (ADRs, WIs previos AN-001..AN-067) permanecen intactas y consultables.
- AC-09: Se revisa y valida el diff completo antes de cualquier commit.

## Fuera de alcance

- Cambio del nombre remoto en GitHub (cubierto en AN-079).
- Configuración de cuentas externas y publicación en crates.io (cubiertos en AN-068/AN-076).
- Firma y publicación de release oficial (cubiertos en AN-069 y AN-076).

## Validación

1. `cargo check --workspace` y `cargo test --workspace` limpios bajo los nuevos nombres.
2. `cargo fmt --check` y `cargo clippy --workspace --all-targets` sin advertencias.
3. Invocación de `wasma --help` y `wasma --version`.
4. Ejecución del benchmark de latencia: `cargo test --locked -p wasma-runtime --example bench_latency`.
5. Ejecución del validador de tutoriales web: `node website/scripts/test-docs.mjs` y `pnpm --dir website test`.

## Entregable

Migración técnica reproducible y validada en el workspace Rust, con evidencia de regresión y suite de pruebas en verde.

## Evidencia y verificación

- **AC-01 a AC-09:** Verificados con estado `passed` (9/9).
- **Decisión Kaddo:** `READY_TO_COMPLETE`.
- **Validaciones:**
  - `cargo fmt --all -- --check`: passed.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
  - `cargo test --workspace --all-features`: passed (100% pruebas de unit, integration, CLI y doc-tests).
  - `cargo build --workspace --release`: passed (binario `wasma.exe` generado).
  - `pnpm --dir website test`: passed (2/2).
  - `pnpm --dir website run test:docs`: passed (5/5 comandos del tutorial con `wasma.exe`).
  - `pnpm --dir website build`: passed (13 páginas estáticas).
  - `cargo test --locked -p wasma-runtime --example bench_latency`: passed (3/3).
  - `cargo run --locked -p wasma-runtime --example bench_env`: passed (telemetría JSON reproducible).
