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
depends_on:
  - AN-078
  - AN-079
domains:
  - Distribution & Release
expected_value: >-
  Auditar la coherencia de versiones entre crates Rust con identidad definitiva Wasma,
  tags de Git, changelog y notas de versión para la release v0.1.0.
project_state: ai-assisted
template_version: 1
---

# AN-068 — Auditoría de preparación de release y contrato de versionamiento

## Contexto y problema

Wasma completó su núcleo técnico con 67 Work Items y consolidó su identidad técnica y repositorio en AN-078 y AN-079. Para publicar la primera versión oficial v0.1.0, se requiere establecer un contrato de versionamiento estricto sobre los paquetes definitivos, realizar una auditoría de dependencias internas, y estructurar la política de changelog y notas de versión. La versión `0.1.0` existente del sitio web no constituye evidencia de que la CLI tenga esa misma versión.

## Alcance

- Auditar todos los crates con sus nombres definitivos de Wasma post-migración.
- Establecer la versión `0.1.0` en los manifests de Cargo y resolver `Cargo.lock`.
- Definir políticas independientes de versionamiento para Semantic IR y contratos WIT.
- Redactar el changelog base y notas de versión iniciales.
- Identificar y documentar limitaciones conocidas y compatibilidad declarada para v0.1.0.

## Criterios de aceptación

- AC-01: Los paquetes Rust incluidos en la release utilizan los nombres aprobados para Wasma, comparten la versión `0.1.0` según el contrato definido, mantienen sus dependencias coherentes y generan un `Cargo.lock` válido. La versión del sitio, la Semantic IR y los contratos WIT conservan sus políticas de versionamiento independientes.
- AC-02: Se define y documenta el contrato de versionamiento SemVer para el runtime, CLI y especificación Semantic IR.
- AC-03: Se genera y estructura el `CHANGELOG.md` inicial consolidando lo entregado en INI-001 a INI-008.
- AC-04: Se valida que `cargo check --workspace` y `cargo test --workspace` compilen y pasen limpiamente con las versiones actualizadas.
- AC-05: Se documentan las limitaciones conocidas y exclusiones explícitas de la versión v0.1.0.

## Fuera de alcance

- Firma de binarios (corresponde a AN-069).
- Empaquetado multiplataforma en CI (corresponde a AN-070).
- Publicación efectiva de tags o release en GitHub (corresponde a AN-076).
