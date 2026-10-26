---
id: AN-059
type: feature
work_type: feature
title: Blog y roadmap público trazable de Wasma
status: completed
created_at: '2026-10-20'
completed_at: '2026-10-26'
knowledge_level: K3
scope_confidence: medium
project_state: ai-assisted
refined_by: work-item-agent
implemented_by: codex
closed_by: agentic
implementation_status: completed
validation_status: passed
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

## Evidencia de cierre

- **AC-01:** Colección de blog configurada en `website/src/content/docs/blog/` con índice cronológico y ruta `/blog/de-la-especificacion-al-componente/`.
- **AC-02:** Artículo inicial sobre origen y propósito verificado por `check-content.mjs`, con contexto histórico contrastado.
- **AC-03:** `/roadmap/` lee dinámicamente los 8 bloques (RM-01 a RM-08) directamente desde `knowledge/delivery/initiatives/` mediante `readRoadmap()`, enlazando a las fuentes canónicas.
- **AC-04:** Advertencias explícitas sobre alcances y estado experimental preservadas (Semantic IR v0.6 `--experimental`).
- **AC-05:** Pruebas unitarias de `website/scripts/checks.test.mjs` pasando (2/2 tests OK), verificando rechazo de estados no canónicos y links faltantes.
- **AC-06:** Enlaces externos reales y directos a GitHub (repositorio, issues, releases y licencia MIT).
- **AC-07:** Recorridos hacia el blog y roadmap comprobados en preview local con soporte de temas claro/oscuro y responsive en 375, 768 y 1440 px.
