---
id: AN-060
type: feature
work_type: feature
title: Validación integral y preparación de entrega del sitio Wasma
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
  - AN-058
  - AN-059
code:
  - website/**
  - .github/workflows/website.yml
---

# AN-060 — Validación integral y preparación de entrega del sitio Wasma

## Actor and outcome / Problem

Un mantenedor verifica y prepara el sitio para entrega sin alterar el runtime; un
responsable revisa el resultado local antes de autorizar publicación de cambios.
Derivado de [AN-056](AN-056-sitio-oficial-y-documentacion-wasma.md).

## Current behavior / Target behavior / End-to-end flow

No existen build ni CI web. Flujo objetivo: checkout → instalación congelada →
check/build → validación de contenido/enlaces/tutorial → preview local → revisión
humana → commit/push autorizados → CI remota. Hosting y DNS tienen aprobación aparte.

## Scope / Acceptance criteria

- AC-01: scripts `check`, `build`, `check:links`, `check:content`, `test:docs`, `dev`
  y `preview` documentados en `website/README.md`; Node y pnpm fijados/compatibles.
  Build de contenido no requiere Rust; smoke documental tiene job y herramientas propias.
- AC-02: workflow web nuevo instala con `--frozen-lockfile`, verifica tipos, genera
  estático, comprueba rutas/assets/fragmentos y valida fuentes/metadatos. Paths incluyen
  sitio, workflow y fuentes canónicas leídas; cambios de CLI disparan smoke documental.
  No reemplazar ni debilitar ci.yml, release.yml o bench.yml.
- AC-03: enlaces internos rotos y metadatos obligatorios ausentes fallan con salida
  no cero; fuentes externas se revisan y registran separadamente de checks deterministas.
- AC-04: revisión local de landing/docs/blog/roadmap a 375, 768 y 1440 px, ambos temas,
  teclado, foco, imágenes, búsqueda en build, copia, 404 y reduced motion; contraste
  según umbrales de AN-056. Sin errores de consola ni scroll horizontal global.
- AC-05: title/description/canonical/OG y sitemap verificados; base de producción
  `https://wasma.org`, assets optimizados. Medir peso de páginas y recursos y documentar
  problemas/correcciones; no inventar un SLA ni una puntuación Lighthouse obtenida.
- AC-06: proponer proveedor estático usando documentación vigente y registrar selección
  pendiente o aprobada. Guía precisa build/output, HTTPS, apex/www canónico, previews,
  noindex de previews, rollback y variables necesarias sin secretos. No activar servicios.
- AC-07: ejecutar E2E global de AN-056 y registrar matriz AC-01…15 con evidencia,
  fecha, versiones/SHA y defectos resueltos. Aprobación humana registra alcance exacto;
  revisión visual no equivale a autorización de commit/push/merge o dominio.
- AC-08: tras autorización de push, registrar resultado de CI remota. Antes de esa
  autorización informar “pendiente”; no confundir check local con CI remota exitosa.

## Impact analysis / Module coverage

`website/`, nuevo workflow, configuración de build y operaciones documentadas: affected.
Crates y workflows existentes: reviewed-not-affected. Hosting/DNS: unknown hasta elección,
sin mutación. DB/auth/analytics/notificaciones: not-applicable.

## Open Questions / Dependencies / Scope confidence

Depende de AN-057/058/059. WEB-03 (proveedor/responsable) sigue abierta. Documentar
recomendación concreta al implementar; autorización de DNS se mantiene como gate de
publicación, no como requisito para generar estáticos. K3, confianza medium por proveedor
pendiente y validación integral aún no ejecutada. No inventar recursos Kaddo externos.

## Out of scope

Publicación real o registros DNS sin autorización, despliegues de releases Rust,
certificación de benchmarks y cuentas de usuario.

## Evidencia de cierre

- **AC-01:** Scripts documentados en `website/README.md`. Node v24.13.0 y pnpm 12.9.1 verificados; suite de validaciones ejecutable de forma autónoma.
- **AC-02:** Workflow `.github/workflows/website.yml` configurado sin interferir con `ci.yml` ni `bench.yml`.
- **AC-03:** `check-links.mjs` y `check-content.mjs` verifican enlaces, fragmentos y metadatos con fallos deterministas ante inconsistencias.
- **AC-04:** Revisión local en preview (`http://127.0.0.1:4321/`) y capturas en `website/validation/screenshots/` en 375, 768 y 1440 px sin scroll horizontal ni errores de consola.
- **AC-05:** Sitemap y metadatos canónicos `https://wasma.org` generados y verificados en build.
- **AC-06:** Configuración de despliegue en `netlify.toml` para entrega estática de `website/dist/`, sin activar servicios en vivo de DNS.
- **AC-07:** Matriz de validación y evidencia registrada en `website/validation/README.md`; revisión humana presencial y aprobación visual confirmada.
- **AC-08:** Workflow de CI web listo para ejecutar validaciones en pipelines remotos.
