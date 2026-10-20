---
id: AN-059
type: feature
work_type: feature
title: Blog y roadmap público trazable de Wasma
status: draft
created_at: '2026-10-20'
knowledge_level: K3
scope_confidence: medium
project_state: ai-assisted
refined_by: work-item-agent
domains:
  - Developer Experience, Documentation & Branding
depends_on:
  - AN-057
code:
  - website/src/**
  - website/scripts/**
---

# AN-059 — Blog y roadmap público trazable de Wasma

## Actor and outcome / Problem

Un visitante entiende el origen y avance real de Wasma y encuentra cómo contribuir.
Hoy solo hay información dispersa y estados históricos en el roadmap. Derivado de
[AN-056](AN-056-sitio-oficial-y-documentacion-wasma.md), sin crear otro roadmap manual.

## Current behavior / Target behavior / End-to-end flow

Sin blog ni roadmap web → `/` → `/blog/` → artículo con contexto verificable →
`/roadmap/` → hito y fuente canónica; enlaces de comunidad llevan a issues/releases
y a la sección de contribución existente del README.

## Scope / Acceptance criteria

- AC-01: colección Markdown/MDX con título, resumen, fecha, etiquetas y slug;
  índice cronológico descendente y ruta `/blog/<slug>/` con SEO y código/imágenes.
- AC-02: al menos un artículo sobre origen, propósito y estado observado, con fecha
  editorial real y fuentes; no anunciar un lanzamiento o certificación inexistente.
- AC-03: `/roadmap/` muestra RM-01…RM-08 con enlaces a iniciativas/roadmap canónicos.
  Derivar estados del frontmatter de INI-001…INI-008 durante build, sin archivo paralelo
  de estados mantenido a mano. No renderizar cuerpos internos completos.
- AC-04: separar estado del hito y disponibilidad de capacidades: preservar límites
  del baseline WASI y AN-050 pendiente, IR v0.6 experimental y RM-08 en curso.
  No convertir una discrepancia en una afirmación de disponibilidad.
- AC-05: validación detecta fuente faltante/estado desconocido; una modificación de
  estado en fixture temporal se refleja en la vista generada. Los archivos canónicos
  reales no se alteran como parte de esa prueba.
- AC-06: enlaces reales a repositorio, issues, releases, licencia y contribuciones;
  no inventar `CONTRIBUTING.md` ni funcionalidad social propia.
- AC-07: E2E landing → blog → artículo y landing → roadmap → evidencia funciona
  en ambos temas y en móvil; todas las páginas tienen title/description/canonical/OG.

## Impact analysis / Module coverage

`website/`, contenido y generación estática: affected. `knowledge/delivery/`: fuente
canónica de lectura. Rust, workflows y benchmarks: reviewed-not-affected.
Backend/DB/CMS/auth/notificaciones: not-applicable. No recursos externos nuevos.

## Open Questions / Dependencies / Scope confidence

AN-057 para estructura; AN-058 para enlaces de documentación en aceptación integrada.
Supuesto explícito de AN-056: estados estructurados de iniciativas y limitaciones de WIs
prevalecen sobre prosa histórica. Si la revisión revela conflicto material, registrar
“pendiente de verificación” y resolver antes de aprobar la afirmación. Confianza medium,
K3 por selección editorial y consistencia de fuentes. No hay pregunta bloqueante nueva.

## Out of scope

CMS, cuentas, calendarios de lanzamiento prometidos, traducción completa y cambiar
estados de iniciativas para satisfacer la web.

## How to test it / Definition of Done

Build + check:content/check:links definidos en AN-060. Comprobar orden de artículos
con fixtures, ausencia de fuentes y propagación de cambios de estado. Revisar
manualmente las afirmaciones contra fuentes y capturar blog/roadmap en ambos temas.
DoD: AC satisfechos, fuentes identificables y revisión editorial/visual integrada.
Aplican todas las aprobaciones de AN-056; no publicar artículos durante refinamiento.
