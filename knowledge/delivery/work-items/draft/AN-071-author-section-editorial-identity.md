---
id: AN-071
type: feature
status: draft
created_at: 2026-10-26T00:00:00.000Z
title: Sección de autor e identidad editorial del proyecto
knowledge_level: K2
suggested_knowledge_level: K2
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-04
priority: P1
depends_on:
  - AN-077
domains:
  - Developer Experience, Documentation & Branding
expected_value: >-
  Incorporar una sección de autor sobria y contextualizada en la landing page principal
  aportando transparencia sobre el creador, origen de Wasma y su evolución desde Agent Native.
project_state: ai-assisted
template_version: 1
---

# AN-071 — Sección de autor e identidad editorial del proyecto

## Contexto y problema

El portal de Wasma presenta la arquitectura, las capacidades y la documentación, pero carece de contexto sobre quién respalda el proyecto, su origen y su trayectoria de evolución desde el prototipo y núcleo técnico Agent Native. Una sección de autor profesional fortalece la confianza de desarrolladores externos sin convertir la landing en una página personal.

## Criterios de aceptación

- AC-01: Se incorpora un componente accesible y responsive «Sobre el autor» en `website/src/pages/index.astro`, ubicado tras la sección de capacidades y antes del cierre/CTA final.
- AC-02: Se presenta la información aprobada: nombre del creador (Julián Darío Luna Patiño), perfil profesional, motivación de Wasma, enfoque en herramientas open source y relación histórica con Agent Native sin atribuciones incorrectas.
- AC-03: Se integran enlaces externos verificables a perfiles profesionales y repositorios de comunidad.
- AC-04: El diseño respeta los tokens de color (`tokens.css`), contraste y soporte para temas claro y oscuro.
- AC-05: Se mantiene la navegación accesible por teclado y la visualización correcta a 375, 768 y 1440 px sin scroll horizontal.

## Fuera de alcance

- Crear una página `/autor/` separada (evaluable para fases posteriores).
- Modificar el banner principal o la identidad de marca del producto.
