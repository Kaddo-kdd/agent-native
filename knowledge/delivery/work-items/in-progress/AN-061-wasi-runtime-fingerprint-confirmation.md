---
id: AN-061
type: chore
status: in-progress
created_at: 2026-10-21T00:00:00.000Z
title: Confirmación WASI independiente y huella efectiva del runtime
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - scripts/bench_collect.py
  - scripts/bench_aggregate.py
  - scripts/bench_evidence.py
  - scripts/bench_summary.py
  - .github/workflows/bench.yml
  - scripts/test_bench_collect.py
  - scripts/test_bench_aggregate.py
  - scripts/test_bench_publish.py
source_roadmap_initiative: INIT-08-02
initiative: INI-008
depends_on:
  - AN-055
domains:
  - Performance
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Confirmar de forma acotada el escenario WASI con dos workflows independientes
  sobre el mismo runtime identificado criptográficamente, sin repetir ni mezclar
  la batería cross-OS completa.
---

# AN-061 — Confirmación WASI y huella del runtime

## Alcance

1. Cada ficha de entorno nueva registra `runtime_source_sha256`: hashes por ruta de
   las fuentes Rust de `crates/runtime/src` y `crates/ir/src`, los manifiestos,
   `Cargo.lock` y la toolchain; además registra
   `runtime_source_fingerprint_sha256`, una huella determinista de ese inventario.
2. El agregador exige que esas huellas, cuando están presentes, coincidan entre las
   tres repeticiones del mismo job. La evidencia histórica sin esta instrumentación
   conserva su validez técnica y no se reinterpreta.
3. `bench.yml` acepta el alcance manual `wasi-confirmation`. Ejecuta tres
   repeticiones Criterion solo de `cold_run_component_model_wasi_echo`, sin
   percentiles individuales ni pooling de muestras.
4. La evidencia `criterion-wasi-confirmation` conserva el entorno, las muestras
   Criterion y los resúmenes necesarios para una regeneración offline.

## Criterios de aceptación

- AC-01: una alteración en cualquier huella de fuente de runtime entre repeticiones
  invalida el resumen del job.
- AC-02: dos ejecuciones manuales independientes, despachadas desde el mismo SHA de
  `main`, producen artefactos WASI con la misma huella efectiva del runtime y tres
  repeticiones completas cada una.
- AC-03: la evidencia de alcance WASI se valida y archiva offline sin requerir
  percentiles individuales.
- AC-04: la comparación conserva datasets separados y no certifica la fila por
  código; una aprobación humana posterior decide cualquier ampliación.

## Fuera de alcance

No modifica el umbral del 10 %, no caracteriza el reloj de Windows, no resuelve la
variabilidad de macOS y no fija Z% para INIT-08-03.

## Estado

La instrumentación y el workflow acotado se implementan en este work item. Quedan
pendientes los dos workflows manuales y su revisión humana antes de modificar la
matriz de certificación o cerrar AN-055 / INIT-08-02.

## Evidencia de dos sesiones independientes (2026-10-21)

Los workflows `39549485766` y `39550537942` terminaron correctamente con el
mismo SHA `96c9179319ee374c84708a1f5c8ecd819a7f19e1`. Sus seis artefactos se
archivaron como datasets `criterion-wasi-confirmation`, sin pooling, y el análisis
está en `docs/benchmarks/wasi-confirmation.md`.

Las huellas del runtime, las fuentes de escenarios, CPU e imagen coinciden dentro
de cada par de OS. Windows fue `inestable` (12,24 %) en la primera sesión y
`aceptable` (3,53 %) en la segunda; macOS invierte ese patrón. Por ello WASI no
se certifica y AN-061 permanece abierto para diagnóstico de variabilidad, sin
nuevas repeticiones por ahora.
