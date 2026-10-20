---
id: AN-054
type: chore
status: draft
created_at: 2026-10-20T00:00:00.000Z
title: Medición de warm invocation y distribución de latencias por invocación
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: medium
code:
  - crates/runtime/examples/bench_latency.rs
  - .github/workflows/bench.yml
source_work_item_candidate:
  - WI-CANDIDATE-08-02-02
depends_on:
  - AN-053
source_roadmap_initiative: INIT-08-02
initiative: INI-008
domains:
  - Performance
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Publicar p50/p95/p99 solo cuando correspondan a invocaciones individuales
  realmente medidas, y distinguir explícitamente las estimaciones agregadas de
  Criterion.
---

# AN-054 — Warm invocation y distribución de latencias

## Definiciones (obligatorias en todo reporte)

- **Warm-run:** componente precargado, **Store nuevo por invocación** (ADR-005).
- **Cache-hit:** reutiliza el artefacto vía `Resolver`; no reutiliza estado del guest.
- Percentil: calculado sobre latencias de **invocaciones individuales**.

## Problema de método

Criterion agrupa muchas iteraciones por muestra. Un p95 o p99 calculado sobre esos
promedios **no** es el p95 o p99 de invocaciones individuales. Por eso:

1. Criterion se usa para **estimaciones agregadas** (media, mediana, intervalo de
   confianza) y así se etiquetan: `estimacion_agregada`.
2. Los percentiles salen de un recolector propio, `bench_latency` (example), que
   mide cada invocación con `std::time::Instant` y guarda cada latencia.

## Cálculo de percentiles (diseño aprobado el 2026-10-20)

- Por fixture y operación: **1 000 invocaciones de calentamiento descartadas** y
  **10 000 invocaciones medidas** (parametrizable; el número real se reporta).
- Cada invocación se mide por separado: `t0 = Instant::now()` justo antes de
  `invoke` y `elapsed` justo después, con `black_box` sobre el resultado.
- Orden ascendente de las N latencias en nanosegundos. **Percentil por rango más
  cercano** (nearest-rank): `p_k = muestra[ceil(k/100 * N) - 1]` para
  k = 50, 95, 99. No se interpola.
- Se reportan también `min`, `max` y `N`.
- **Resolución del reloj:** se mide la resolución efectiva de `Instant` en cada
  sistema. Si `p50 < 10 x resolucion`, la fila se marca `cerca-del-limite-de-reloj`
  y no se publica un percentil sin esa advertencia (el overhead de `Instant::now`
  pesa en operaciones de ~2 µs).
- Calibrar y guardar también el overhead del instrumento con mediciones vacías
  repetidas (N, min, p50, p95, p99 y max), describiendo el procedimiento y sus
  límites. No restarlo automáticamente de las muestras ni de los percentiles.
  Advertir si es significativo frente a las latencias medidas; conservar todas
  las muestras crudas por corrida en `run-XX/latency/`.
- `p99` solo se publica si `N >= 1 000` medidas; con menos se omite.
- Las 3 corridas se evalúan con la misma regla de variación de AN-053, aplicada a `p50`.
  Esta evaluación es independiente de la mediana agregada de Criterion: no se
  combinan sus muestras ni se tratan ambos indicadores como equivalentes.
  `p95` y `p99` se publican con su rango entre corridas, sin exigir el 10 %
  (las colas son más ruidosas en runners compartidos).
  Incluir advertencias de variabilidad para las colas; las filas `inestable`
  permanecen en resultados técnicos y quedan fuera de la línea base certificada.

## Alcance de fixtures

`warm_run` y `cache_hit_run` (core `identity` y CM `point_sum`), WASI echo,
IR v0.6 `greet` y `safe_div`, e `identity` con fuel frente a sin fuel.

## Criterios de aceptación

- AC-01: `bench_latency` emite JSON con `N`, `warmup`, `min`, `p50`, `p95`,
  `p99`, `max` en ns, resolución del reloj y metadatos de entorno.
- AC-02: una prueba unitaria verifica nearest-rank con datos conocidos (p. ej. 1..100
  da p50=50, p95=95, p99=99; N=10 da p99 = máximo).
- AC-03: los reportes distinguen `estimacion_agregada` (Criterion) de
  `percentil_por_invocacion` (`bench_latency`).
- AC-04: las filas cerca del límite de reloj están marcadas.
- AC-05: 3 corridas por sistema operativo con artefactos separados y trazabilidad por
  commit, fecha, entorno y fixture.
- AC-06: el coste de fuel se expresa como diferencia frente a `warm_run_core_identity`
  con intervalo, no como porcentaje aislado.
- AC-07: resolución y overhead calibrados, muestras sin corrección automática,
  e indicadores de repetibilidad separados por método y ejecución del workflow.

## Secuencia autorizada

Conservar en `draft` para el commit documental. La implementación puede avanzar
después de estabilizar la recolección de AN-053. La tabla final distinguirá
costes de operación de sobrecarga de ejecución; cambiar de fixture (identity,
echo, greet, safe_div) no aísla por sí solo la sobrecarga de WASI o IR v0.6.
Solo atribuir diferencias a una característica con controles de operación,
entrada, salida, API y etapa comparables; en otro caso describir escenarios.

## Fuera de alcance

Concurrencia (INIT-08-04) y caché de componentes (INIT-08-03).
