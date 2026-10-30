---
id: AN-068
type: chore
status: completed
created_at: 2026-10-26T00:00:00.000Z
ready_at: '2026-10-30'
completed_at: '2026-10-30'
title: Auditoría de preparación de release y contrato de versionamiento
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-01
priority: P0
depends_on:
  - AN-078
  - AN-079
domains:
  - Distribution & Release
code:
  - Cargo.toml
  - crates/**/Cargo.toml
  - Cargo.lock
  - CHANGELOG.md
  - docs/**
expected_value: >-
  Auditar la coherencia de versiones entre crates Rust con identidad definitiva
  Wasma, tags de Git, changelog y notas de versión para la release v0.1.0.
project_state: ai-assisted
template_version: 1
---

# AN-068 — Auditoría de preparación de release y contrato de versionamiento

## Contexto y problema

Wasma completó su núcleo técnico con 67 Work Items y consolidó su identidad técnica y repositorio en AN-078 y AN-079. Para publicar la primera versión oficial v0.1.0, se requirió establecer un contrato de versionamiento estricto sobre los paquetes definitivos, realizar una auditoría de dependencias internas (incluyendo compatibilidad `path + version`), y estructurar la política de changelog y notas de versión. La versión `0.1.0` existente del sitio web no constituía evidencia de que la CLI tuviera esa misma versión.

## Alcance

- Auditar todos los crates con sus nombres definitivos de Wasma post-migración (`wasma-cli`, `wasma-ir`, `wasma-compiler`, `wasma-runtime`).
- Establecer la versión `0.1.0` en los manifests de Cargo (`[workspace.package] version = "0.1.0"`) y resolver `Cargo.lock` sin regresiones.
- Definir y documentar el contrato de versionamiento SemVer formal para el workspace Rust (`wasma-cli`, `wasma-ir`, `wasma-compiler`, `wasma-runtime`).
- Establecer la separación clara e independencia de políticas entre:
  - Versiones de paquetes Rust del workspace (`0.1.0` unificado).
  - Versión del portal web `wasma.org` (`0.1.0` en `package.json`, ciclo de release independiente).
  - Versión de la especificación Semantic IR (`v0.5` base estable, `v0.6-experimental` para extensiones de modelo declarativo y tipos).
  - Contratos de interfaces WIT (`wasi:cli`, `wasi:clocks`, `wasi:filesystem`, `wasi:io`, `wasi:random` anclados a WASI Preview 2 / 0.2.0).
- Desacoplar explícitamente las extensiones de Semantic IR v0.6 de los workers concurrentes del runtime (`experimental-workers`).
- Auditar las dependencias internas de path en los crates (`wasma-ir`, `wasma-compiler`, `wasma-runtime`) para que incluyan la versión explícita (`path + version = "0.1.0"`), garantizando compatibilidad con empaquetado para distribución futura en crates.io.
- Redactar el `CHANGELOG.md` base estructurado según Keep a Changelog y SemVer: documentar lo entregado a lo largo de INI-001 a INI-008 como desarrollo técnico histórico hacia v0.1.0, sin presentar tags anteriores de Git como releases oficiales distribuidas.
- Identificar y documentar limitaciones conocidas y compatibilidad declarada para la versión pública inicial v0.1.0.

## Criterios de aceptación verificados

- **AC-01 (Verificado):** Los paquetes Rust incluidos en la release (`wasma-cli`, `wasma-ir`, `wasma-compiler`, `wasma-runtime`) utilizan los nombres aprobados para Wasma, comparten la versión `0.1.0` según el contrato definido, mantienen sus dependencias coherentes (`path + version = "0.1.0"`) y generan un `Cargo.lock` válido y mínimo. La versión del sitio (`website/package.json`), la Semantic IR (`v0.5` / `v0.6-experimental`) y los contratos WIT conservan sus políticas de versionamiento independientes y documentadas en `docs/versioning.md`.
- **AC-02 (Verificado):** Se definió y documentó formalmente el contrato de versionamiento SemVer para el runtime, compiler, CLI y especificación Semantic IR en `docs/versioning.md`, estableciendo reglas pre-1.0 y post-1.0, lockstep versioning y el desacoplamiento explícito entre IR v0.6 y los workers del runtime.
- **AC-03 (Verificado):** Se generó y estructuró el `CHANGELOG.md` inicial consolidando lo entregado en INI-001 a INI-008 en `[0.1.0] - En preparación`, aclarando que las etiquetas y commits git previos corresponden a puntos de control técnicos internos y no a releases públicas distribuidas.
- **AC-04 (Verificado):** `cargo check --workspace --locked`, `cargo test --workspace --locked` (100% suites unitarias, integración y doc-tests con 0 fallos), `cargo fmt --all -- --check` y `cargo clippy --workspace --all-targets -- -D warnings` pasaron limpiamente.
- **AC-05 (Verificado):** Se documentaron las limitaciones conocidas en `docs/versioning.md` y `CHANGELOG.md`: workers concurrentes deshabilitados por defecto (`experimental-workers`), soporte exclusivo para WASI Preview 2 (0.2.0), binario oficial `wasma` con alias `agent-native` opcional deshabilitado por defecto, y requisitos topológicos para crates.io.

## Fuera de alcance

- Firma de binarios (corresponde a AN-069).
- Empaquetado multiplataforma en CI (corresponde a AN-070).
- Publicación efectiva de tags o release en GitHub (corresponde a AN-076).
- Publicación real de crates a `crates.io` (reservada para decisión posterior).

## Validación

1. Verificación de manifests: `cargo metadata --format-version 1 --no-deps` confirmó que `wasma-cli`, `wasma-ir`, `wasma-compiler` y `wasma-runtime` reportan versión `0.1.0`.
2. Verificación de dependencias internas: `wasma-ir`, `wasma-compiler` y `wasma-runtime` declaran `version = "0.1.0"` y `path`.
3. Verificación de compilación y pruebas: `cargo check --workspace --locked` (0.73s) y `cargo test --workspace --locked` (0 fallos).
4. Verificación de consistencia del lockfile: `git diff Cargo.lock` refleja únicamente el bump de versión de paquetes locales sin cambios en dependencias externas.
5. Verificación de versión de binario: `wasma --version` reporta `wasma 0.1.0`.
6. Verificación documental: `CHANGELOG.md` y `docs/versioning.md` estructurados y validados.

## Handoff y riesgos para AN-069 y AN-070

- **Riesgos pendientes para AN-069:**
  - Provisión y custodia segura de claves oficiales Minisign (`minisign.pub` / `minisign.key`).
  - Configuración de secrets en el repositorio GitHub (`MINISIGN_SECRET_KEY`, `MINISIGN_PASSPHRASE`).
  - Activación controlada del workflow de publicación: el job `publish` continúa formalmente deshabilitado y no fue modificado durante AN-068.
- **Continuidad para AN-070:**
  - Los artefactos ejecutables standalone de `wasma` v0.1.0 cuentan con contratos claros de versión para matrices de build multiplataforma (Linux x86_64, Windows x86_64, macOS aarch64/x86_64).
- **Estado de Git:**
  - Commit local previo `83518e7` (`docs(delivery): complete AN-079 and close INI-009 identity phase`) preservado en `main`.
  - `origin/main` permanece en `0cd2801`.
  - No se realiza push.

## Entregable

Contrato de versionamiento formal en `docs/versioning.md`, `CHANGELOG.md` estructurado según Keep a Changelog, manifests y lockfiles sincronizados en `0.1.0`, y evidencia de validación técnica completa.
