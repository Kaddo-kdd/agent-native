---
id: AN-057
type: feature
work_type: feature
title: Fundación web y landing oficial de Wasma
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

## Evidencia de cierre

- **AC-01:** Build estático verificado con Astro 7.4.1 y pnpm 12.9.1 (`pnpm --dir website run build`), 13 páginas generadas sin colisión entre `/` y `/docs/`.
- **AC-02:** Identidad visual e isotipo en `public/brand/` con variantes clara y oscura. Revisión humana aprobada explícitamente en preview local.
- **AC-03:** Hero, descripción, flujo de 6 etapas, capacidades comprobadas, ejemplo y footer MIT validados en `website/src/pages/index.astro`.
- **AC-04:** Paleta de 6 colores en `website/src/styles/tokens.css` y `site.css`; contraste y alternancia claro/oscuro comprobados.
- **AC-05:** Recorrido E2E validado a 375, 768 y 1440 px según `website/validation/responsive.json` y capturas locales.
- **AC-06:** Los crates de Rust, `Cargo.toml`, `Cargo.lock` y workflows de Rust no sufrieron modificaciones.
