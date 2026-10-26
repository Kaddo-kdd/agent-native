---
id: AN-068
type: chore
status: draft
created_at: 2026-10-26T00:00:00.000Z
title: Auditoría de preparación de release y contrato de versionamiento
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-01
priority: P0
depends_on: []
domains:
  - Distribution & Release
expected_value: >-
  Auditar la coherencia de versiones entre crates Rust (ir, compiler, runtime, cli),
  tags de Git, changelog y notas de versión para la release v0.1.0.
project_state: ai-assisted
template_version: 1
---

# AN-068 — Auditoría de preparación de release y contrato de versionamiento

## Contexto y problema

Wasma completó su núcleo técnico con 67 Work Items. Actualmente los crates en `crates/*` tienen versiones `0.0.0` o dispares en `Cargo.toml`. Para publicar la primera versión oficial v0.1.0, se requiere un contrato de versionamiento estricto, auditoría de dependencias internas, alineación de nombres y política de changelog.

## Criterios de aceptación

- AC-01: Todos los crates del workspace (`agent-native-ir`, `agent-native-compiler`, `agent-native-runtime`, `agent-native-cli`) unifican su versión a `0.1.0` de forma coherente con `Cargo.lock`.
- AC-02: Se define y documenta el contrato de versionamiento SemVer para el runtime, CLI y especificación Semantic IR.
- AC-03: Se genera y estructura el `CHANGELOG.md` inicial consolidando lo entregado en INI-001 a INI-008.
- AC-04: Se valida que `cargo check --workspace` y `cargo test --workspace` compilen y pasen limpiamente con las versiones actualizadas.
- AC-05: Se documentan las limitaciones conocidas y exclusiones explícitas de la versión v0.1.0.

## Fuera de alcance

- Firma de binarios (corresponde a AN-069).
- Empaquetado multiplataforma en CI (corresponde a AN-070).
- Publicación efectiva de tags o release en GitHub (corresponde a AN-076).
