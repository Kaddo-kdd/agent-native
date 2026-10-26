---
id: AN-072
type: feature
status: draft
created_at: 2026-10-26T00:00:00.000Z
title: Versión, commit y procedencia del despliegue en el footer
knowledge_level: K2
suggested_knowledge_level: K2
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-05
priority: P1
depends_on: []
domains:
  - Developer Experience, Documentation & Branding
expected_value: >-
  Agregar al footer global del sitio web metadatos automáticos de versión del portal,
  SHA corto del commit Git y procedencia del build enlazada al repositorio.
project_state: ai-assisted
template_version: 1
---

# AN-072 — Versión, commit y procedencia del despliegue en el footer

## Contexto y problema

Actualmente el footer del sitio web solo muestra la licencia MIT y enlaces genéricos. Para ofrecer total trazabilidad a auditores y usuarios, el footer debe indicar exactamente qué versión del sitio y qué commit específico de Git está desplegado.

## Criterios de aceptación

- AC-01: El footer de `website/src/layouts/Site.astro` y componentes asociados incluye metadatos generados en tiempo de build: versión web (de `package.json`) y SHA corto del commit Git (`git rev-parse --short HEAD` o variable de entorno de CI/Netlify `COMMIT_REF`).
- AC-02: El SHA corto enlaza directamente al commit específico en GitHub (`https://github.com/Kaddo-kdd/agent-native/commit/<SHA>`).
- AC-03: En entornos locales o de desarrollo sin git/CI disponible, se muestra un indicador explícito (ej. `dev`).
- AC-04: La solución distingue la versión del portal web de la versión de la CLI de Wasma (no infiere la CLI a partir de la web).
- AC-05: El diseño es discreto, accesible, responsive y compatible con temas claro y oscuro.

## Fuera de alcance

- Añadir telemetría o analíticas en el footer.
- Cambiar la URL canónica o estructura general del footer.
