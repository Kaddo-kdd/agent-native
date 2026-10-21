---
id: AN-062
type: chore
status: in-progress
created_at: 2026-10-21T00:00:00.000Z
title: Diagnóstico de variabilidad Criterion en cold-run WASI
knowledge_level: K2
suggested_knowledge_level: K3
scope_confidence: medium
code:
  - crates/runtime/benches/
  - crates/runtime/examples/bench_env.rs
  - scripts/bench_collect.py
  - scripts/bench_aggregate.py
  - scripts/bench_evidence.py
  - .github/workflows/bench.yml
source_roadmap_initiative: INIT-08-02
initiative: INI-008
depends_on:
  - AN-061
domains:
  - Performance
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Explicar o acotar la variación de cold-run WASI mediante señales de entorno y
  mediciones de Engine, carga e invocación conservadas separadamente.
---

# AN-062 — Diagnóstico de variabilidad Criterion en cold-run WASI

## Problema observado

AN-061 conservó dos sesiones con el mismo commit, huellas efectivas, CPU e imagen
por OS. Windows fue inestable en una sesión y aceptable en la otra; macOS mostró
el patrón inverso. Esto descarta una diferencia registrada de fuentes como
explicación suficiente, pero no atribuye la variación a WASI, Criterion ni al
runner.

## Alcance

1. Añadir un instrumento diagnóstico separado del benchmark histórico
   `cold_run_component_model_wasi_echo`. Medirá tres fronteras no solapadas:
   construcción de `Engine`, carga del componente con política y llamada
   `invoke_with_wasi` sobre un componente ya cargado.
2. Registrar antes y después de cada repetición las señales disponibles y no
   identificadoras del host: carga de CPU, frecuencia publicada, CPUs disponibles,
   afinidad/scheduling cuando la plataforma lo exponga, y calibración del reloj.
   La ausencia de una señal se registra como no disponible; no se infiere un valor.
3. Ejecutar una comparación A/A del diagnóstico en un entorno controlado o runner
   dedicado, con mismo SHA, API, fixture, entrada, límites y fronteras temporales.
4. Conservar cada sesión y cada etapa por separado. El diagnóstico no reemplaza ni
   recalcula las seis evidencias AN-061.

## Criterios de aceptación

- AC-01: el benchmark histórico conserva nombre, closure y semántica de medición.
- AC-02: cada etapa diagnóstica declara su frontera temporal y evita atribuir la
  suma de etapas a una causalidad WASI.
- AC-03: la ficha de entorno registra señales disponibles y distingue datos de
  configuración de observaciones dinámicas.
- AC-04: un A/A nuevo archiva dos sesiones separadas; resultados estables o
  inestables se conservan por igual.
- AC-05: ningún resultado del diagnóstico certifica WASI automáticamente ni
  modifica el umbral estricto del 10 %.

## Fuera de alcance

No implementa caché, warm pools ni cambios del runtime de producción. Tampoco
cierra AN-055, INIT-08-02 o AN-061 sin una decisión humana explícita.

## Estado

Instrumento integrado en `main` y pendiente de CI. No se ha ejecutado todav�a
el diagn�stico ni el A/A. El banco `wasi_diagnostic` registra tres etapas
separadas, conserva `extended.rs` sin cambios y guarda
`host_telemetry_before` y `host_telemetry_after` en `collection.json`.
El archivo `criterion-wasi-diagnostic` conserva solamente esas tres etapas.
