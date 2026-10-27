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
  Crear una ruta de inicio rápido para desarrolladores utilizando únicamente la CLI y rutas definitivas de Wasma,
  probando el tutorial desde la release candidata y documentando cualquier alias transitorio.
project_state: ai-assisted
template_version: 1
---

# AN-073 — Onboarding, quickstart y reconciliación de documentación

## Contexto y problema

La documentación actual incluye referencias a compilación local desde fuentes y nombres heredados. Se requiere una ruta clara para usuarios que descargan el binario oficial precompilado de Wasma, separando los requisitos de desarrollo de los de uso, empleando exclusivamente los comandos y rutas definitivos.

## Criterios de aceptación

- AC-01: Se publica una guía «Quickstart / Inicio rápido» de 5 minutos en el portal orientada exclusivamente al uso del binario precompilado y la CLI definitiva (`wasma`).
- AC-02: Se actualiza la página `/docs/installation/` con los enlaces a los paquetes oficiales de GitHub Releases v0.1.0 y verificación de sumas criptográficas.
- AC-03: Se eliminan menciones a nombres o rutas heredadas no aprobadas en la documentación pública, documentando de forma clara cualquier alias transitorio aprobado en AN-077.
- AC-04: El tutorial de onboarding es probado y validado de punta a punta a partir de la release candidata producida en AN-070.
- AC-05: Todos los ejemplos del quickstart son ejecutables sin requerir claves de API ni dependencias adicionales no estándar.

## Fuera de alcance

- Documentación avanzada de contribución interna al runtime.
- Nuevos vídeos o material multimedia fuera del portal web.
