---
id: AN-075
type: spike
status: draft
created_at: 2026-10-26T00:00:00.000Z
title: Validación con desarrolladores externos y reporte de fricciones
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: medium
initiative: INI-009
source_roadmap_initiative: INIT-09-08
priority: P1
depends_on:
  - AN-073
  - AN-074
domains:
  - Developer Experience, Documentation & Branding
expected_value: >-
  Ejecutar una prueba piloto con 5 a 10 desarrolladores externos evaluando facilidad de descubrimiento,
  instalación, uso del comando definitivo Wasma y comprensión de la documentación sin asistencia.
project_state: ai-assisted
template_version: 1
---

# AN-075 — Validación con desarrolladores externos y reporte de fricciones

## Contexto y problema

La hipótesis central de INI-009 es que desarrolladores ajenos al proyecto puedan descubrir, instalar, entender y usar Wasma de forma autónoma. Esta validación empírica revelará problemas de usabilidad, errores en los nuevos comandos o vacíos en la documentación antes de la release pública oficial.

## Criterios de aceptación

- AC-01: Se diseña el protocolo de prueba piloto con tareas concretas: localización del producto en el portal/repositorio, descarga e instalación, ejecución del tutorial con el nuevo comando `wasma`, modificación de un componente y evaluación de reglas.
- AC-02: Se reclutan entre 5 y 10 desarrolladores de perfiles diversos sin experiencia previa en Wasma ni en la arquitectura interna.
- AC-03: Se registran métricas observables y cualitativas: facilidad de descubrimiento del producto, tiempo de instalación, comprensión del comando definitivo y tasa de finalización autónoma.
- AC-04: Se evalúa explícitamente si persisten confusiones derivadas del cambio de identidad técnica respecto a Agent Native.
- AC-05: Se publica un informe consolidado de hallazgos y fricciones con propuestas de mejora priorizadas para iteraciones posteriores.

## Fuera de alcance

- Soporte personalizado o consultoría individual durante la prueba.
