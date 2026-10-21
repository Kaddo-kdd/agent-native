---
id: AN-066
type: chore
status: ready
created_at: 2026-10-21T00:00:00.000Z
title: Aislamiento concurrente, backpressure y fallos de workers
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: medium
code:
  - crates/runtime/tests/
source_roadmap_initiative: INIT-08-04
initiative: INI-008
depends_on:
  - AN-065
domains:
  - Runtime & Memory
  - Security
  - Performance
related_capabilities:
  - Independent Runtime Execution
  - Resource Contention & Limits
expected_value: >-
  Verificar que la concurrencia host-side no comparte estado entre solicitudes y
  que saturación, cancelación y fallos preservan capacidad y aislamiento.
related_decisions:
  - ADR-005
project_state: ai-assisted
template_version: 1
---

# AN-066 — Aislamiento concurrente, backpressure y fallos de workers

## Criterios de aceptación

- AC-01: solicitudes alternadas y simultáneas de tenants distintos no comparten
  globals, memoria, recursos, policy, fuel ni resultados.
- AC-02: cola llena produce el error de backpressure acordado; no hay cola
  ilimitada ni starvation silenciosa.
- AC-03: trap, error de host, fuel/memoria agotados, cancelación y panic de una
  solicitud no bloquean otro tenant ni pierden capacidad.
- AC-04: el cierre concurrente tiene semántica determinista para solicitudes en
  cola y en ejecución.
- AC-05: la suite corre en CI de Linux, Windows y macOS.

## Fuera de alcance

No mide throughput de producto ni habilita workers por defecto; AN-067 realizará
la caracterización cuantitativa.