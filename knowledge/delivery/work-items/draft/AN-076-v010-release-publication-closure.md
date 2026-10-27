---
id: AN-076
type: chore
status: draft
created_at: 2026-10-26T00:00:00.000Z
title: Publicación oficial v0.1.0 y cierre de INI-009
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-09
priority: P0
depends_on:
  - AN-069
  - AN-070
  - AN-071
  - AN-072
  - AN-073
  - AN-075
domains:
  - Distribution & Release
expected_value: >-
  Formalizar la publicación oficial de la versión v0.1.0 en GitHub Releases tras validar
  migración de identidad completa, firma, empaquetado, portal, documentación y piloto externo.
project_state: ai-assisted
template_version: 1
---

# AN-076 — Publicación oficial v0.1.0 y cierre de INI-009

## Contexto y problema

Work Item de coordinación final para la primera entrega pública v0.1.0 de Wasma. Exige que todas las fases precedentes estén completas y verificadas: consolidación de identidad técnica, migración del repositorio, contrato de versión 0.1.0, pipeline de firma, paquetes multiplataforma, documentación de onboarding, aplicaciones de referencia, transparencia en el portal y validación empírica con desarrolladores externos.

## Criterios de aceptación

- AC-01: La migración de identidad técnica y del repositorio se encuentran formalmente validadas, comprobando que los contratos Semantic IR, WIT y las garantías de seguridad no sufrieron regresiones.
- AC-02: Los paquetes binarios firmados y el catálogo de hashes SHA-256 están publicados en GitHub Releases bajo el tag oficial `v0.1.0`.
- AC-03: Las notas de versión, changelog, limitaciones conocidas y compatibilidad declarada están publicados en el portal y en GitHub Releases bajo la nomenclatura definitiva.
- AC-04: El portal web refleja de forma transparente la sección del autor y los metadatos de build/commit en el footer sin discrepancias.
- AC-05: Los resultados del piloto de adopción con desarrolladores externos están documentados y todos los defectos bloqueantes identificados han sido resueltos.
- AC-06: Se cuenta con la autorización humana explícita y documentada para la liberación pública oficial.
- AC-07: Se evalúa y formaliza el cierre de INI-009 con `kaddo_complete_initiative`.

## Fuera de alcance

- Certificación para entornos de producción de misión crítica (Wasma v0.1.0 es una release de adopción e inicio).
