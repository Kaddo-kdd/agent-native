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
  Construir dos ejemplos prácticos reproducibles (motor de reglas de negocio e integración host)
  con pruebas de aceptación y documentación de casos de uso.
project_state: ai-assisted
template_version: 1
---

# AN-074 — Aplicaciones de referencia y pruebas E2E

## Contexto y problema

Los desarrolladores necesitan ver casos de uso realistas más allá de un `identity` matemático. Dos aplicaciones de referencia completas permitirán validar la versatilidad de Wasma.

## Criterios de aceptación

- AC-01: Ejemplo 1 — Motor de reglas de negocio: componente que evalúa reglas deterministas de descuento o puntuación, con especificación, IR, verificación y pruebas YAML.
- AC-02: Ejemplo 2 — Integración host: aplicación que embebe el runtime de Wasma y ejecuta componentes demostrando cuotas de fuel, memoria y aislamiento de fallos.
- AC-03: Ambas aplicaciones se incorporan en `examples/` con README reproducible y suite de tests automatizada.
- AC-04: Se publican guías y walkthroughs en el portal de documentación enlazando al código de ambos ejemplos.
