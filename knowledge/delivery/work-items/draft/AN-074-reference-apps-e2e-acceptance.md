---
id: AN-074
type: feature
status: draft
created_at: 2026-10-26T00:00:00.000Z
title: Aplicaciones de referencia y pruebas E2E
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-07
priority: P1
depends_on:
  - AN-070
domains:
  - Developer Experience, Documentation & Branding
expected_value: >-
  Construir dos aplicaciones de referencia con manifests, código y ejemplos bajo los nombres finales de Wasma,
  verificando que la migración de identidad técnica no afecte los contratos WASM.
project_state: ai-assisted
template_version: 1
---

# AN-074 — Aplicaciones de referencia y pruebas E2E

## Contexto y problema

Los desarrolladores necesitan evaluar casos de uso realistas más allá de ejemplos elementales. Dos aplicaciones de referencia completas bajo la identidad definitiva de Wasma permitirán validar la versatilidad de la plataforma y comprobar de forma empírica que la migración técnica no alteró los contratos ni la ejecución de los componentes WebAssembly.

## Criterios de aceptación

- AC-01: Ejemplo 1 — Motor de reglas de negocio: componente que evalúa reglas deterministas de descuento o puntuación, con especificación, IR, verificación y pruebas YAML bajo la identidad definitiva.
- AC-02: Ejemplo 2 — Integración host: aplicación que embebe el runtime de Wasma (`wasma-runtime`) y ejecuta componentes demostrando cuotas de fuel, memoria y aislamiento estricto de fallos.
- AC-03: Ambas aplicaciones se incorporan en `examples/` con manifests de Cargo actualizados a los nuevos nombres, README reproducible y suite de tests automatizada.
- AC-04: Se verifica que la migración técnica de identidad no altere los contratos WebAssembly ni los artefactos generados.
- AC-05: Se publican guías y walkthroughs en el portal de documentación enlazando al código de ambos ejemplos.

## Fuera de alcance

- SDKs para lenguajes distintos de Rust o TypeScript (considerables en iniciativas posteriores).
