---
id: AN-077
type: spike
status: draft
created_at: 2026-10-27T00:00:00.000Z
title: Evaluación de identidad técnica, disponibilidad de nombres y estrategia de migración
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-10
priority: P0
depends_on: []
domains:
  - Developer Experience, Documentation & Branding
  - Distribution & Release
expected_value: >-
  Inventario exhaustivo de referencias técnicas a agent-native, matriz de disponibilidad
  y colisiones de nombres para Wasma, y estrategia aprobada de migración y compatibilidad.
project_state: ai-assisted
template_version: 1
---

# AN-077 — Evaluación de identidad técnica, disponibilidad de nombres y estrategia de migración

## Contexto y problema

Aunque Wasma tiene una identidad pública definida en `wasma.org`, los nombres técnicos de sus componentes todavía corresponden a Agent Native (ejecutable `agent-native`, crates `agent-native-ir`, `agent-native-compiler`, `agent-native-runtime`, `agent-native`, y repositorio `Kaddo-kdd/agent-native`). Renombrarlos sin revisar su disponibilidad o las dependencias existentes puede introducir incompatibilidades y afectar la futura distribución de la release v0.1.0.

## Alcance

- Inventariar referencias actuales a `agent-native` en código, Cargo, CLI, scripts, tests, CI/CD, documentación, URLs, Kaddo, ejemplos y artefactos.
- Distinguir nombres comerciales, binarios ejecutables, nombres de paquetes Cargo, identificadores de librerías Rust, repositorios e identificadores históricos.
- Comprobar disponibilidad y posibles colisiones de `wasma`, `wasma-ir`, `wasma-compiler`, `wasma-runtime` y otras variantes necesarias en los registros pertinentes.
- Evaluar conflictos con proyectos existentes y riesgos de identificación o marca. La revisión técnica no equivale a una conclusión jurídica.
- Definir los nombres públicos finales, la estrategia de compatibilidad y el orden de migración.
- Determinar si se conservará temporalmente `agent-native` como alias, y por cuánto tiempo.
- Identificar dependencias externas, referencias históricas y operaciones que necesitan aprobación separada.
- Definir una estrategia de rollback para la migración técnica y una recuperación operativa para el cambio de repositorio.

## Criterios de aceptación

- AC-01: Existe un inventario completo de nombres, ubicaciones, consumidores y riesgos asociados.
- AC-02: Existe una matriz de disponibilidad, colisiones y alternativas con fuentes verificables.
- AC-03: Se aprueba una identidad técnica objetivo para CLI, crates, repositorio y distribución.
- AC-04: Se define explícitamente si habrá alias de compatibilidad para la CLI anterior.
- AC-05: Se clasifican las referencias que deben migrarse frente a las que deben conservarse por trazabilidad histórica.
- AC-06: Se documentan riesgos, dependencias, rollback y plan de validación.
- AC-07: La decisión humana queda registrada antes de iniciar cambios que afecten interfaces públicas.

## Fuera de alcance

- Modificar nombres en el código o Cargo.
- Versionar a `0.1.0`.
- Renombrar el repositorio en GitHub.
- Publicar paquetes en registros externos.
