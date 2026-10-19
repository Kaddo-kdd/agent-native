---
type: initiative
id: INI-005
title: RM-05 — Standalone CLI & Runtime API
status: planned
knowledge_level: K3
domains:
  - Distribution & DX
  - Runtime & Memory
related_capabilities:
  - CLI as Primary Interface
  - Independent Runtime Execution
roadmap_block: RM-05
roadmap_initiatives:
  - INIT-05-01
  - INIT-05-02
  - INIT-05-03
created_at: '2026-10-17'
external_links:
  - knowledge/delivery/roadmap.md
candidates:
  - id: INIT-05-01
    title: Bin `cli` con build/verify/test/run/publish
    status: completed
    materialized_as: AN-028
  - id: INIT-05-02
    title: API pública reutilizable del runtime
    status: completed
    materialized_as: AN-029, AN-030
  - id: INIT-05-03
    title: Pipeline de release para 3 triples objetivo
    status: candidate
---

# RM-05 — Standalone CLI & Runtime API

## Goal

Convertir el pipeline interno en herramienta distribuible: un desarrollador en una máquina sin Rust ni acceso a LLM puede instalar la CLI, descargar un componente publicado y ejecutarlo con una suite de aceptación.

## Expected Value

Primer **hito público** del producto (RC v0.1.0 candidato). Materializa la invariante "ejecución sin toolchain" documentada en `business.md`.

## Scope

- Bin `cli` con subcomandos `build`, `verify`, `test`, `run`, `publish`.
- Salidas estructuradas opcionales (`--format=json`).
- API pública curada del `runtime` para embeber desde otras apps.
- Pipeline de release para `x86_64-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`.
- ADR dedicada para empaquetado, firma, publicación y verificación.

## Out of Scope

- OCI, Homebrew, winget, apt (diferidos).
- Imagen Docker oficial (diferida).
- SDKs para lenguajes no-Rust (Not Now).

## Success Criteria

1. `agent-native --help` lista los 5 subcomandos y sus opciones estables.
2. Smoke E2E en 3 OS: instalación, descarga, ejecución con resultado verificable.
3. Suites automatizadas de CI no parsean texto humano del CLI (consumen `--format=json`).
4. Ejemplo `embed_runtime.rs` compila y ejecuta un componente publicado desde Rust externo.
5. ADR de empaquetado/firma cerrada; artefactos firmados descargables con verificación documentada.

## Dependencies

- INIT-03-03 (executor YAML) e INIT-04-05 (publicación) completados.

## Work Item Candidates

Ver `candidates:`.

## Open Questions

- OQ-05-01: minisign vs cosign — resuelta en INIT-05-03 a favor de Minisign (ADR-006).
- OQ-05-02: política de versionado semántico de la CLI vs runtime vs IR — a definir.

## Learning

_Pendiente._
