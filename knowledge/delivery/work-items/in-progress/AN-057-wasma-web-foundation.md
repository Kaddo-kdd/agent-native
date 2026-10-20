---
id: AN-057
type: feature
work_type: feature
title: Fundación web y landing oficial de Wasma
status: in-progress
created_at: '2026-10-20'
knowledge_level: K3
scope_confidence: medium
project_state: ai-assisted
refined_by: work-item-agent
domains:
  - Developer Experience, Documentation & Branding
code:
  - website/**
---

# AN-057 — Fundación web y landing oficial de Wasma

## Actor and outcome

Un desarrollador descubre qué es Wasma desde una landing con identidad oficial y
accede a documentación. El mantenedor dispone de una base estática extensible.
Derivado de [AN-056](AN-056-sitio-oficial-y-documentacion-wasma.md); sus restricciones,
fuentes, exclusiones y fases de aprobación son vinculantes.

## Current behavior / Target behavior

No existe `website/`; solo hay un banner de Agent Native. Crear Astro + Starlight +
TypeScript con pnpm y lockfile aislados de Cargo. Landing `/`, prefijo `/docs/`, tokens,
componentes comunes y estructura para el blog; no implementar aquí sus contenidos completos.

## Entry points / End-to-end flow

Entrada `/` → comprender “From spec to wasm.” → leer flujo y capacidades → CTA `/docs/`
→ introducción navegable. En móvil se puede abrir/cerrar navegación mediante teclado.

## Scope / Acceptance criteria

- AC-01: instalación reproducible y build estático; raíz personalizada y `/docs/`
  no colisionan; versiones compatibles y requisito Node/pnpm documentados al implementar.
- AC-02: logo e isotipo originales en `public/brand/`, variantes de tema, favicon y
  procedencia registrados; no usar el banner anterior como marca Wasma ni inventar el logo.
- AC-03: hero, descripción, flujo Specification → Semantic IR → Verification → Compilation
  → WebAssembly → Runtime, capacidades verificadas, ejemplo breve y footer MIT.
- AC-04: seis colores de AN-056 como variables CSS; tipografía y componentes compartidos;
  contraste comprobado. Claro/oscuro funcionan en landing y Starlight.
- AC-05: E2E `/` → `/docs/` funciona a 375/768/1440 px, con foco visible y sin pérdida de
  contenido al navegar con teclado. Los destinos pendientes de otros hijos no se declaran
  completos hasta integración; no añadir enlaces rotos al build final.
- AC-06: ningún cambio en crates, binario, variables de entorno, Cargo o workflows Rust.

## Impact analysis / Module coverage

UI, configuración, contenido y `website/`: affected. Crates Rust: reviewed-not-affected.
Backend, DB, auth, analytics y notificaciones: not-applicable. Hosting: deferred a AN-060.
No hay módulos multirepo o recursos externos registrados a inventar.

## Open Questions / Dependencies / Scope confidence

WEB-01 y WEB-02 de AN-056: faltan kit y fuente oficial; bloquean aceptación visual.
Se puede preparar estructura, pero no cerrar branding con placeholders. K3, confianza
medium por estos insumos y la integración nueva. No requiere completar RM-08.

## Out of scope

Contenidos de AN-058/059, CI/despliegue de AN-060 y cambios al producto Rust.

## How to test it / Definition of Done

Ejecutar install con lockfile, `pnpm --dir website check` y `pnpm --dir website build`.
Con `pnpm --dir website dev`, comprobar el recorrido, temas, teclado, reduced motion y
contraste en los tres tamaños; capturar landing y shell documental. Verificar procedencia
de assets y revisar diff de Cargo. DoD: AC satisfechos, evidencia local y revisión humana
de branding registrada. Sin commit hasta autorización explícita; mantener draft hasta
confirmación humana de readiness.


## Implementación autorizada — 2026-10-20

El usuario autorizó implementación, commit y push y suministró referencias de marca.
Sitio implementado en website/. Evidencia: website/validation/README.md.
Los PNG recibidos resuelven la referencia visual; fuente exacta y aceptación estética
siguen pendientes. Propuesta de hosting documentada, sin DNS ni despliegue.
Estado in-progress hasta revisión humana; no equivale a cierre ni merge autorizado.
