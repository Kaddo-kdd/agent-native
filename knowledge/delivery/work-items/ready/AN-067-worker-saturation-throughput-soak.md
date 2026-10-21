---
id: AN-067
type: chore
status: ready
created_at: 2026-10-21T00:00:00.000Z
title: Saturación, throughput y soak de workers host-side
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: medium
code:
  - crates/runtime/benches/
  - scripts/
  - docs/benchmarks/
  - .github/workflows/
source_roadmap_initiative: INIT-08-04
initiative: INI-008
depends_on:
  - AN-065
  - AN-066
domains:
  - Performance
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
  - Resource Contention & Limits
expected_value: >-
  Caracterizar con evidencia controlada la saturación y comportamiento sostenido
  de workers acotados, sin prometer objetivos de throughput antes de medirlos.
related_decisions:
  - ADR-007
project_state: ai-assisted
template_version: 1
---

# AN-067 — Saturación, throughput y soak de workers host-side

## Criterios de aceptación

- AC-01: protocolo A/A y campañas con N workers/cola definidos por configuración
  reproducible en Linux x86_64 GNU.
- AC-02: cada sesión conserva SHA, entorno, capacidad, carga, resultados por
  repetición y señales de memoria/RSS disponibles.
- AC-03: reporta throughput, latencias por solicitud, rechazos de backpressure,
  errores y saturación sin combinar sesiones independientes.
- AC-04: el soak define duración y carga antes de ejecutarse, y publica señales
  de degradación o fugas observadas sin afirmar su ausencia sin evidencia.
- AC-05: cualquier objetivo cuantitativo posterior se propone a partir de A/A y
  requiere revisión humana.

## Fuera de alcance

No certifica rendimiento cross-OS, WASI ni una mejora de instanciación.