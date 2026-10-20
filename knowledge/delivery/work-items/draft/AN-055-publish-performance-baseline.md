---
id: AN-055
type: docs
status: draft
created_at: 2026-10-20T00:00:00.000Z
title: Publicación reproducible de la línea base de rendimiento en docs/benchmarks
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: medium
code:
  - docs/benchmarks/
source_work_item_candidate:
  - WI-CANDIDATE-08-02-03
depends_on:
  - AN-053
  - AN-054
source_roadmap_initiative: INIT-08-02
initiative: INI-008
domains:
  - Performance
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Línea base publicada, trazable y reproducible de GitHub-hosted runners, sin
  afirmar más de lo que los datos respaldan.
---

# AN-055 — Publicación de la línea base

## Alcance

1. Reporte en `docs/benchmarks/` generado a partir de los artefactos de AN-053 y
   AN-054, con un comando de regeneración documentado.
2. Cada fila incluye: commit, fecha de medición, sistema operativo, modelo de CPU,
   versión de toolchain y Wasmtime, fixture, unidad, número de muestras, resultado de
   la regla de variación y enlace a los artefactos originales.
3. Sección de **definiciones** (cold-run, warm-run, cache-hit) y de **limitaciones**:
   runners de GitHub con hardware variable, no es un benchmark universal por sistema
   operativo, no compara con implementaciones nativas ni con AWS Lambda.
4. Los benches `inestable` o `hardware-mixto` conservan sus cifras en los resultados
   técnicos con su motivo, excluidos de afirmaciones de línea base certificada.
5. Actualizar `runtime-baseline.md` (AN-017) indicando que su cifra de Wasmtime 27 y
   `--quick` queda reemplazada por esta línea base, conservando el historial.
6. Tabla de coste por invocación: WASM básico, WASI, fuel e IR v0.6. Identificar
   etapa, operación, entradas/salidas, API y método. Distinguir comparación de
   escenarios de estimación de sobrecarga: diferencias entre operaciones distintas
   no prueban el coste aislado de una característica. Para fuel, comparar la misma
   operación y explicitar cualquier diferencia de API y retorno.
7. Separar la estabilidad de medianas Criterion y p50 individual. Publicar p95/p99
   con rangos entre las tres corridas y advertencias, sin umbral inicial del 10 %.
   Documentar resolución y overhead calibrados, sin restarlos automáticamente.
8. Distinguir repetibilidad local (tres corridas en un job) de reproducibilidad
   (workflow posterior independiente). Mantener sus datos separados y enlazados.
   La tabla revisada será referencia de INIT-08-03 para decidir si caché o warm
   pools justifican su complejidad mediante mejoras medidas.

## Criterios de aceptación

- AC-01: todo valor certificado procede de corridas completas (no `--quick`) con
  estabilidad aceptable por método; los valores excluidos siguen visibles como
  resultados técnicos. No aplicar el umbral del 10 % a p95/p99 inicialmente.
- AC-02: percentiles etiquetados `percentil_por_invocacion`; Criterion etiquetado
  `estimacion_agregada`.
- AC-03: el reporte se regenera con un único comando a partir de los artefactos.
- AC-04: cada fila es trazable a commit, entorno, fixture, unidad, N, fecha y artefacto.
- AC-05: las limitaciones están al inicio del documento.
- AC-06: INIT-08-02 solo pasa a `completed` tras revisión humana de la publicación.

## Condición de cierre

Requiere aprobación explícita antes de declarar la línea base como certificada.
Mantener este work item en `draft` en el commit documental. Esperar mediciones
reales antes de publicar cifras; INIT-08-02 no pasa a `completed` con este commit.
