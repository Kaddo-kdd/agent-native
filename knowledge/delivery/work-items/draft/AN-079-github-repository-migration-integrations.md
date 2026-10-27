---
id: AN-079
type: chore
status: draft
created_at: 2026-10-27T00:00:00.000Z
title: Migración de repositorio GitHub y continuidad de integraciones
knowledge_level: K4
suggest_knowledge_level: K4
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-12
priority: P0
depends_on:
  - AN-078
domains:
  - Distribution & Release
  - Developer Experience, Documentation & Branding
expected_value: >-
  Migración auditada del repositorio GitHub de agent-native a Wasma, garantizando continuidad en CI/CD,
  redirecciones, despliegue del portal web wasma.org y trazabilidad histórica.
project_state: ai-assisted
template_version: 1
---

# AN-079 — Migración de repositorio GitHub y continuidad de integraciones

## Contexto y problema

El repositorio remoto de GitHub continúa identificado como `Kaddo-kdd/agent-native`. Su renombramiento a la identidad definitiva de Wasma puede afectar URLs públicas, workflows de CI/CD, automatizaciones, badges, documentación, despliegues automáticos del portal web `wasma.org` y consumidores externos que clonan o referencian el repositorio.

## Alcance

- Auditar exhaustivamente las dependencias de la URL actual `github.com/Kaddo-kdd/agent-native`.
- Evaluar y, únicamente con autorización humana explícita, renombrar `Kaddo-kdd/agent-native` al nombre aprobado en AN-077.
- Revisar GitHub Actions, badges de estado, enlaces públicos, releases y enlaces de evidencia experimental.
- Actualizar referencias de repositorio en Cargo (`repository` metadata), documentación, Kaddo, ejemplos y scripts de automatización.
- Revisar y verificar la conexión del hosting del portal (`wasma.org`) con GitHub (Netlify / hosting integration).
- Verificar webhooks, integraciones externas, remotos Git (`origin`) y rutas utilizadas por automatizaciones.
- Validar las redirecciones automáticas de GitHub desde la ubicación anterior.
- Documentar el tratamiento de enlaces históricos y artefactos que deban seguir disponibles.

## Criterios de aceptación

- AC-01: Existe un inventario completo de integraciones, workflows, enlaces externos y dependencias del repositorio.
- AC-02: El nombre de repositorio destino fue aprobado en AN-077 y se encuentra disponible en GitHub.
- AC-03: La operación de renombramiento en GitHub se ejecuta únicamente tras contar con autorización humana específica.
- AC-04: El repositorio conserva íntegramente historial de Git, ramas, tags, PRs, issues y evidencia histórica.
- AC-05: `git clone`, `fetch`, `push` y los enlaces canónicos funcionan correctamente desde la nueva ubicación.
- AC-06: Los workflows principales de GitHub Actions pasan exitosamente bajo el nuevo nombre de repositorio.
- AC-07: El portal público `wasma.org` conserva su acceso y su mecanismo de despliegue continuo funciona sin incidencias.
- AC-08: Los enlaces históricos se comprueban; los que no puedan redirigirse automáticamente reciben tratamiento y redirección documentada.
- AC-09: La documentación de instalación, clonado y contribución apunta a la URL canónica definitiva.

## Fuera de alcance

- Crear un repositorio nuevo sin migrar el historial de Git.
- Trasladar el proyecto a otra organización de GitHub.
- Cambiar el dominio público `wasma.org`.
- Publicar la release oficial v0.1.0 (cubierto en AN-076).

## Entregable

Repositorio migrado formalmente, integraciones verificadas y reporte de evidencia de continuidad operativa.
