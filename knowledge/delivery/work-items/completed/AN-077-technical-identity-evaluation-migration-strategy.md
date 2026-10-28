---
id: AN-077
type: spike
status: completed
created_at: 2026-10-27T00:00:00.000Z
ready_at: '2026-10-28'
completed_at: '2026-10-28'
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
code:
  - docs/reports/an-077-technical-identity-evaluation.md
  - knowledge/tech/decisions/ADR-008-identidad-tecnica-migracion-wasma.md
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

- Modificar código funcional, nombres en Cargo o crates.io.
- Renombrar el repositorio en GitHub o cambiar URLs de hosting.
- Crear releases o publicar artefactos oficiales.

## Validación

1. Cobertura del inventario sobre todo el repositorio (crates, binario, tests, CI/CD, scripts, docs, website, Kaddo).
2. Verificación de fuentes externas consultadas para disponibilidad de nombres en crates.io y GitHub.
3. Evaluación y comparación justificada de las 3 alternativas estratégicas de migración.
4. Definición de riesgos, dependencias, rollback y plan secuencial para AN-078 y AN-079.
5. Decisión arquitectónica aceptada por decisión humana (ADR-008).

## Entregables

- Documento de evaluación técnica y reporte de investigación en `docs/reports/an-077-technical-identity-evaluation.md`.
- Decisión arquitectónica formalizada y aceptada en `knowledge/tech/decisions/ADR-008-identidad-tecnica-migracion-wasma.md`.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Iniciativa | INI-009 — Public Release, Technical Identity & Developer Adoption |
| Candidato de roadmap | INIT-09-10 |
| Decisión arquitectónica | ADR-008 (accepted) |
| Reporte de investigación | `docs/reports/an-077-technical-identity-evaluation.md` |
| Siguiente Work Item | AN-078 (Migración técnica de Agent Native a Wasma) |

## Learning

**Lo que salió como se esperaba:**
- El inventario automatizado permitió clasificar con precisión 1551 ocurrencias, distinguiendo las ~280 superficies activas de código y configuración frente a los registros históricos inmutables (WIs AN-001..AN-067 y ADR-001..ADR-007).
- La consulta en registros externos confirmó la disponibilidad completa para `wasma-ir`, `wasma-compiler`, `wasma-runtime` y el repositorio `Kaddo-kdd/wasma`.

**Lo inesperado y resolución decisiva:**
- El nombre de paquete `wasma` ya se encuentra registrado en `crates.io` por un proyecto de gestión de ventanas en Windows (*Azencorporation*).
- Se resolvió adoptando el estándar de la comunidad Rust: el crate se denomina `wasma-cli` (completamente libre en crates.io) y produce el binario ejecutable `wasma` (`[[bin]] name = "wasma"`).
- Se clarificó la estructura real del workspace (dependencias internas relativas por ruta, sin `[workspace.dependencies]`).
- Se acordó que la instalación estándar solo provee `wasma`; el alias `agent-native` queda relegado a una feature flag deshabilitada por defecto (`--features legacy-alias`) con política de deprecación para v0.2.0.
- Se fijó la precedencia de `WASMA_LLM_*` con sanitización estricta de credenciales en advertencias de fallback.
- Se descartó `git reset --hard` como estrategia de recuperación, optando por reversiones auditables vía `git revert`.
