---
id: AN-053
type: chore
status: completed
completed_at: 2026-10-20T00:00:00.000Z
created_at: 2026-10-20T00:00:00.000Z
title: Medición cross-OS de cold start en GitHub-hosted runners (engine, load, cold-run)
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: medium
code:
  - .github/workflows/bench.yml
  - crates/runtime/examples/bench_env.rs
  - scripts/bench_collect.py
  - scripts/bench_aggregate.py
source_work_item_candidate:
  - WI-CANDIDATE-08-02-01
depends_on:
  - AN-051
  - AN-052
source_roadmap_initiative: INIT-08-02
initiative: INI-008
domains:
  - Performance
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Obtener resultados repetidos y trazables de engine_construct, load_* y
  cold_run_* en ubuntu, windows y macos hospedados por GitHub, con una regla
  inequívoca para aceptar o descartar la variación entre corridas.
---

# AN-053 — Medición cross-OS de cold start

## Alcance de la medición (qué significa "cold")

Cold-run = `Engine::new()` + `load_component` + una invocación, según
`baseline.rs` y `extended.rs`. **No** equivale al cold start de AWS Lambda ni
al arranque de un proceso. Esto debe figurar en toda tabla publicada.

## Alcance

1. Ampliar `bench_env` con el **modelo de CPU** (sin identificadores únicos) para
   poder agrupar corridas por hardware. Sin modelo de CPU no hay comparación válida.
2. Cambiar `bench.yml` para producir **3 corridas completas** por sistema operativo
   **en un mismo job y sobre el mismo runner**, decisión humana aprobada el
   2026-10-20. Capturar cada corrida antes de iniciar la siguiente: Criterion
   sobrescribe `target/criterion`. No mezclar datos de distintas ejecuciones.
3. Script de agregación que lea `run-XX/criterion/**/new/estimates.json` de cada
   corrida y calcule la variación (ver Regla de variación). Preservar evidencia
   cruda, metadatos por corrida e identidad del workflow (run ID e intento).
4. Los resultados son una línea base de **GitHub-hosted runners**, no un benchmark
   universal de Linux, Windows o macOS.

## Conservación de evidencia

Cada artefacto corresponde a un sistema operativo y una ejecución del workflow;
el nombre incluye OS, run ID e intento. Su estructura es:

```text
bench-results/
  environment.json
  run-01/
    environment.json
    criterion/
    latency/
  run-02/
    environment.json
    criterion/
    latency/
  run-03/
    environment.json
    criterion/
    latency/
  summary.json
```

`latency/` se reserva para AN-054 y permanece vacío en AN-053. Tres repeticiones
en el mismo runner evalúan **repetibilidad local**. Una ejecución posterior es
verificación independiente de **reproducibilidad**: conserva otro artefacto y
resumen, incluso si usa el mismo modelo de CPU. No demuestra igualdad entre máquinas.

## Regla de variación (aprobada el 2026-10-20)

Por (sistema operativo, bench), con las 3 corridas `r1, r2, r3`:

- Valor de cada corrida: la **mediana** reportada por Criterion (`median.point_estimate`).
- `variacion = (max(r) - min(r)) / mediana(r1, r2, r3)`.
- Aceptable si `variacion < 10 %`. En caso contrario el bench se marca `inestable`
  y se conserva en los resultados técnicos, excluido de afirmaciones de línea
  base certificada. La estabilidad por sí sola no constituye certificación.
- Si las 3 corridas no comparten el mismo modelo de CPU, el grupo se marca
  `hardware-mixto` y no se evalúa la variación.
- Los benches con mediana menor a 10 µs se reportan además con su banda absoluta,
  porque un 10 % relativo sobre microsegundos puede ser ruido de reloj.
- Esta regla se aplica a la mediana agregada de Criterion; el p50 individual
  de AN-054 tiene evaluación separada y nunca se presenta como intercambiable.
- Corridas incompletas, estimaciones inválidas o metadatos ausentes impiden
  aceptar una fila; sus artefactos se conservan para diagnóstico.

## Criterios de aceptación

- AC-01: cada sistema operativo entrega 3 corridas completas con artefactos separados.
- AC-02: cada corrida incluye commit, fecha, ficha de entorno con modelo de CPU,
  versión de rustc/cargo/Wasmtime y fixture.
- AC-03: la variación se calcula con la regla anterior y el resultado (aceptable /
  inestable / hardware-mixto) queda registrado por bench.
- AC-04: `engine_construct`, `load_*` y `cold_run_*` quedan cubiertos en los 3 sistemas.
- AC-05: las cifras `--quick` no se usan en ningún resultado.
- AC-06: el workflow sigue siendo solo `workflow_dispatch` y no bloquea PR.
- AC-07: los resultados de las tres repeticiones no se sobrescriben, se conservan
  también si falla una corrida y se validan antes de aceptar la agregación.

## Autorización y revisión

Commit documental autorizado con AN-053/054/055 en `draft`. Implementación de
AN-053 autorizada el 2026-10-20. Antes de integrar código en `main`, revisión
humana del diff y de los resultados del workflow manual. INIT-08-02 permanece abierto.

## Fuera de alcance

Percentiles por invocación (AN-054) y publicación (AN-055).

## Evidencia de implementación y revisión

- [PR #6](https://github.com/Kaddo-kdd/agent-native/pull/6) integrado en `main`
  por autorización humana; merge `48f7518`. CI verde en los tres sistemas.
- [Workflow inicial](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333):
  tres repeticiones completas por OS; resúmenes regenerados idénticos. macOS:
  14 filas inestables; Linux y Windows: 9 aceptables y 5 inestables cada uno.
- [Verificación independiente desde main](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498):
  artefacto macOS completo, resumen regenerado idéntico; 13 filas inestables y
  1 aceptable. Mismo modelo Apple M1 (Virtual), imagen y hashes de fuentes.
  En ambas ejecuciones las 14 medianas de R3 son menores que las de R1.
  Hay deriva temporal reiterada, sin telemetría suficiente para atribuir su causa.
- El cierre acredita integridad de recolección y evaluación; no certifica una
  línea base ni modifica el umbral del 10 %. INIT-08-02 permanece abierto.
