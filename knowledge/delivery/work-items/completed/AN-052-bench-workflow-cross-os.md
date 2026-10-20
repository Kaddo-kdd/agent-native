---
id: AN-052
type: chore
status: completed
completed_at: 2026-10-20T00:00:00.000Z
created_at: 2026-10-20T00:00:00.000Z
title: Job manual de benchmarks cross-OS que publica resultados como artefacto
knowledge_level: K2
suggested_knowledge_level: K2
scope_confidence: high
code:
  - .github/workflows/bench.yml
source_roadmap_initiative: INIT-08-01
initiative: INI-008
depends_on:
  - AN-051
domains:
  - Runtime & Memory
expected_value: >-
  Obtener mediciones comparables en Linux, Windows y macOS sin bloquear los PR
  (mismo criterio que el modo LLM real, R-06) y alimentar INIT-08-02.
---

# AN-052 — Job manual de benchmarks cross-OS

## Alcance

- Workflow `bench.yml` solo `workflow_dispatch`, matriz ubuntu/windows/macos.
- Cada job ejecuta `bench_env` y ambos benches (`baseline`, `extended`), y sube
  como artefacto la ficha de entorno y `target/criterion`.
- Permisos mínimos (`contents: read`), acciones fijadas a las mismas versiones que `ci.yml`.

## Criterios de aceptación

- AC-01: el workflow no se dispara en push ni en PR.
- AC-02: publica un artefacto por sistema operativo con ficha de entorno y resultados.
- AC-03: no usa secretos ni publica releases.
