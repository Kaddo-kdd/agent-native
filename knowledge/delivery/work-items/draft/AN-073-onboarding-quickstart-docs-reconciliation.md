---
id: AN-073
type: docs
status: draft
created_at: 2026-10-26T00:00:00.000Z
title: Onboarding, quickstart y reconciliación de documentación
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-06
priority: P0
depends_on:
  - AN-068
  - AN-070
domains:
  - Developer Experience, Documentation & Branding
expected_value: >-
  Crear una ruta de inicio rápido para desarrolladores sin conocimientos previos de Rust o Wasmtime,
  actualizando la documentación del portal con los paquetes reales v0.1.0.
project_state: ai-assisted
template_version: 1
---

# AN-073 — Onboarding, quickstart y reconciliación de documentación

## Contexto y problema

La documentación actual incluye referencias a compilación local desde fuentes. Se requiere una ruta clara para usuarios que solo descargan la CLI distribuida, separando los requisitos de desarrollo de los de uso.

## Criterios de aceptación

- AC-01: Se publica una guía «Quickstart / Inicio rápido» de 5 minutos en el portal orientada al consumo del binario precompilado.
- AC-02: Se actualiza la página `/docs/installation/` con los enlaces a los paquetes oficiales de GitHub Releases v0.1.0.
- AC-03: Se eliminan menciones a estados técnicos previos en la documentación pública, manteniendo explícitas las advertencias experimentales vigentes.
- AC-04: Todos los ejemplos del quickstart son ejecutables sin claves de API ni dependencias adicionales.
