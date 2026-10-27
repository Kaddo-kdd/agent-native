---
id: AN-069
type: feature
status: draft
created_at: 2026-10-26T00:00:00.000Z
title: Firma oficial, integridad y controles del pipeline de publicación
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-02
priority: P0
depends_on:
  - AN-068
domains:
  - Distribution & Release
  - Security
expected_value: >-
  Configurar la generación de SHA-256 y firma Minisign en el pipeline de GitHub Actions
  con verificación de nombres definitivos de artefactos, repositorio migrado y custodia segura de claves.
project_state: ai-assisted
template_version: 1
---

# AN-069 — Firma oficial, integridad y controles del pipeline de publicación

## Contexto y problema

El workflow `.github/workflows/release.yml` actualmente cuenta con un bloqueo explícito para publicación desatendida. Para una release pública oficial v0.1.0 bajo la identidad definitiva de Wasma, se requiere un mecanismo estricto de firma Minisign y catálogo de hashes SHA-256 verificables de forma independiente, asegurando la concordancia entre los nombres definitivos de artefactos, los metadatos de release y la nueva ubicación del repositorio.

## Criterios de aceptación

- AC-01: El workflow de release genera un archivo `SHA256SUMS` con los hashes criptográficos de todos los paquetes binarios producidos bajo la nomenclatura definitiva de Wasma.
- AC-02: Se verifica que los nombres de artefactos, metadatos del paquete y la URL del repositorio coincidan estrictamente con la identidad técnica y repositorio migrado en AN-078 y AN-079.
- AC-03: Se integra firma Minisign de `SHA256SUMS` utilizando GitHub Secrets para la clave privada (sin claves en el repositorio).
- AC-04: Se documenta la clave pública oficial y el procedimiento de verificación para los usuarios (`minisign -Vm ... -p ...`).
- AC-05: La publicación requiere un gate de aprobación humana explícita antes de liberar el tag o release en GitHub Releases.

## Fuera de alcance

- Distribución en gestores de paquetes externos (Homebrew, Cargo crates.io, winget).
- Modificación de infraestructura DNS o hosting.
