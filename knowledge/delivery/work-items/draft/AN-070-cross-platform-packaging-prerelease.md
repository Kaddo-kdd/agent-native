---
id: AN-070
type: feature
status: draft
created_at: 2026-10-26T00:00:00.000Z
title: Empaquetado, instalación y prerelease multiplataforma
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-03
priority: P0
depends_on:
  - AN-069
domains:
  - Distribution & Release
expected_value: >-
  Generar paquetes binarios comprimidos y reproducibles para Linux x86_64 GNU,
  Windows x86_64 MSVC y macOS ARM64 con scripts de smoke en entornos limpios.
project_state: ai-assisted
template_version: 1
---

# AN-070 — Empaquetado, instalación y prerelease multiplataforma

## Contexto y problema

Los usuarios de Wasma deben poder descargar e instalar un binario precompilado para su sistema operativo sin necesidad de instalar el toolchain de Rust.

## Criterios de aceptación

- AC-01: El pipeline compila binarios en modo release para las 3 plataformas objetivo: `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` y `aarch64-apple-darwin`.
- AC-02: Cada artefacto se empaqueta en archivo comprimido (`.tar.gz` para Unix, `.zip` para Windows) con binario, LICENSE y README básico.
- AC-03: Pruebas automatizadas de desempaquetado e invocación básica (`agent-native --help`, `agent-native --version`) en runners limpios.
- AC-04: Guía de instalación manual verificada para cada uno de los 3 sistemas operativos.

## Fuera de alcance

- Instalador binario desatendido tipo script `curl | sh` (evaluable tras validar seguridad y firma).
