---
id: AN-067
type: chore
status: completed
created_at: 2026-10-21T00:00:00.000Z
completed_at: 2026-10-23
title: Saturación, throughput y soak de workers host-side
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
code:
  - crates/runtime/benches/
  - crates/runtime/examples/
  - crates/runtime/src/workers.rs
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
refined_by: codex
implemented_by: codex
closed_by: agentic
implementation_status: completed
validation_status: passed
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

## Protocolo de campaña

- El entorno de referencia es Linux x86_64 GNU. Los runners hospedados pueden
  aportar evidencia comparativa, pero no certifican rendimiento universal.
- Cada sesión compara A/A el camino secuencial de `Tenant::invoke` con el pool
  `experimental-workers`, usando el mismo commit, componente, límites de fuel,
  política, carga y fase de calentamiento.
- La matriz de workers, capacidad de cola, concurrencia ofrecida, repeticiones,
  warmup y duración de soak se recibe como configuración obligatoria de la
  campaña y se persiste junto con el resultado. No se fija una meta de mejora,
  un tamaño N ni una duración sin una primera ejecución controlada.
- Los workloads iniciales deben incluir una invocación Core corta y una
  invocación Component Model de coste mayor. Cualquier workload adicional se
  documenta con su fixture, argumentos, límites y contrato de salida.

## Datos y métricas

- Cada repetición publica un manifiesto con SHA, fecha UTC, versión de Rust y
  Wasmtime, perfil de compilación, CPU, núcleos lógicos, sistema operativo,
  feature flags y todos los parámetros de carga.
- Las muestras crudas por solicitud conservan inicio, fin, resultado y causa de
  rechazo o error. La agregación calcula throughput completado, p50/p95/p99,
  aceptaciones, rechazos por backpressure, errores y ocupación máxima
  observada; nunca combina sesiones distintas.
- El recolector registra RSS o declara explícitamente que la señal no estuvo
  disponible. En soak se publican máximos, muestras temporales y cualquier
  tendencia observada, sin afirmar fuga o ausencia de fuga sin evidencia.

## Entregables y puertas

- Un ejecutable o script reproducible genera directorios de sesión inmutables
  con manifiesto, muestras crudas, resumen y hashes de integridad.
- La automatización valida esquema, recalcula agregados desde las muestras y
  conserva resultados separados de cada campaña.
- El informe propone objetivos cuantitativos solo después de comparar las
  repeticiones A/A. La adopción del pool sigue requiriendo revisión humana y
  mantiene `experimental-workers` deshabilitado por defecto.

## Evidencia de cierre

- **AC-01:** Implementado en `scripts/worker_bench_collect.py`, `crates/runtime/examples/worker_bench.rs` y el workflow `.github/workflows/bench.yml` (scopes `workers-aa`, `workers-smoke`, `workers-soak`) para Linux x86_64 GNU con configuración explícita de workers, colas y concurrencia.
- **AC-02:** Cada corrida genera artefactos inmutables por sesión: `manifest.json`, `requests.ndjson`, `resources.ndjson` (con muestreo periódico de RSS en Linux), `summary.json` y `SHA256SUMS`.
- **AC-03:** `worker_bench_collect.py` calcula throughput completado, latencias (p50/p95/p99), errores y rechazos de backpressure por sesión sin fusionar sesiones independientes.
- **AC-04:** Campaña de soak delimitada explícitamente con `--soak-seconds` (15 minutos en workflow `workers-soak`, w2/q4/c4), conservando muestras temporales de RSS y tendencias observadas sin asumir ausencia de fugas.
- **AC-05:** `experimental-workers` se mantiene deshabilitado por defecto bajo feature flag. Cualquier adopción o umbral cuantitativo posterior queda sujeto a revisión humana basada en la evidencia A/A.
- Validaciones unitarias locales superadas con `python -m unittest scripts/test_worker_bench_collect.py` (8/8 tests OK).
