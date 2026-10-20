---
id: AN-060
type: feature
work_type: feature
title: Validación integral y preparación de entrega del sitio Wasma
status: in-progress
created_at: '2026-10-20'
knowledge_level: K3
scope_confidence: medium
project_state: ai-assisted
refined_by: work-item-agent
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

## How to test it / Definition of Done

Ejecutar la secuencia de validación de AN-056 desde checkout limpio. Introducir un
enlace roto y un estado inválido en fixtures temporales para comprobar fallos del checker.
Revisar diff del workflow y ejecutar smoke documental; documentar comandos locales
equivalentes a CI. Levantar dev/preview, recorrer páginas y guardar capturas/resultados
en `website/validation/`. DoD: todos los AC verificables satisfechos, revisión humana
registrada e instrucciones de entrega comprobadas. El cierre global puede conservar
publicación operativa bloqueada según AN-056, pero no ocultar una CI aún pendiente.


## Implementación autorizada — 2026-10-20

El usuario autorizó implementación, commit y push y suministró referencias de marca.
Sitio implementado en website/. Evidencia: website/validation/README.md.
Los PNG recibidos resuelven la referencia visual; fuente exacta y aceptación estética
siguen pendientes. Propuesta de hosting documentada, sin DNS ni despliegue.
Estado in-progress hasta revisión humana; no equivale a cierre ni merge autorizado.
