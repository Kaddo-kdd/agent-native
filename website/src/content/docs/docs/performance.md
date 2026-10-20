---
title: Rendimiento
description: Cómo interpretar la línea base candidata de Wasma y consultar su evidencia canónica.
---

## Evidencia candidata, no certificación

La [publicación canónica de benchmarks](https://github.com/Kaddo-kdd/agent-native/blob/main/docs/benchmarks/README.md)
conserva metodología, procedencia, advertencias y resultados. **Ninguna fila se presenta
aquí como certificada.** Un workflow verde confirma recolección, no estabilidad ni SLA.

Esta página no duplica cifras para evitar divergencia con la evidencia mantenida.

## Qué comparar con cuidado

- Las medianas Criterion no son p50 de invocaciones individuales.
- Cold start, warm invocation y cache hit son operaciones distintas.
- Los runners hospedados no describen rendimiento universal de un sistema operativo.
- Las filas inestables siguen visibles; no deben seleccionarse solo repeticiones favorables.
- p95/p99 conservan sus advertencias; no tienen automáticamente el mismo umbral que p50.
- No hay aquí una comparación certificada con ejecución nativa o arranque de AWS Lambda.

## Fuentes reproducibles

Consulta [publication.json](https://github.com/Kaddo-kdd/agent-native/blob/main/docs/benchmarks/publication.json)
para filas y procedencia, [cases.md](https://github.com/Kaddo-kdd/agent-native/blob/main/docs/benchmarks/cases.md)
para contratos y [history.md](https://github.com/Kaddo-kdd/agent-native/blob/main/docs/benchmarks/history.md)
para grupos históricos. La regeneración offline usa `scripts/bench_publish.py`; no es
necesario ejecutarla para construir este sitio.

## Hito y capacidades

RM-08 continúa en curso. Tener una medición de cache hit no demuestra entrega de un
warm pool multi-tenant ni concurrencia host-side. Consulta el [roadmap](/roadmap/) y
sus iniciativas para el estado detallado.
