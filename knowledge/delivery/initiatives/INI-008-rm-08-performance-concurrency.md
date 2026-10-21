---
type: initiative
id: INI-008
title: RM-08 — Performance & Concurrency
status: in-progress
knowledge_level: K4
domains:
  - Performance
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
roadmap_block: RM-08
roadmap_initiatives:
  - INIT-08-01
  - INIT-08-02
  - INIT-08-03
  - INIT-08-04
created_at: '2026-10-17'
external_links:
  - knowledge/delivery/roadmap.md
candidates:
  - id: INIT-08-01
    title: Harness de benchmarks reproducible
    status: completed
    materialized_as: AN-051 (completed), AN-052 (completed)
  - id: INIT-08-02
    title: Línea base pública cross-OS
    status: candidate
    materialized_as: AN-053 (completed), AN-054 (completed), AN-055 (in-progress)
  - id: INIT-08-03
    title: Caché de componentes y warm pool
    status: candidate
  - id: INIT-08-04
    title: Concurrencia host-side (workers)
    status: candidate
---

# RM-08 — Performance & Concurrency

## Goal

Caracterizar el runtime con benchmarks reproducibles cross-OS, publicar una línea base objetiva, y optimizar (caché, warm pool, workers) **sobre** mediciones previas — nunca a ciegas.

## Expected Value

Materializa el business goal "decisiones basadas en evidencia" y permite publicar afirmaciones de rendimiento defendibles.

## Scope

- Harness `criterion` con hardware baseline documentado y varianza acotada.
- Línea base pública (cold start, warm invocation, memoria por instancia, throughput, p50/p95/p99) en `docs/benchmarks/`.
- Module cache y warm pool con wipe entre tenants.
- Worker pool host-side y soak test.

## Out of Scope

- Primitivas asíncronas dentro del guest (Not Now).
- Comparaciones directas con Rust/Go/TypeScript nativos — se documentan solo si la metodología lo permite.
- Afirmaciones de superioridad frente a otros runtimes.

## Success Criteria

1. Dos corridas consecutivas del harness producen varianza < Y% (Y documentado).
2. Línea base publicada con metodología, hardware y resultados por operación/OS.
3. Caché mejora warm start en Z% medible vs línea base (Z fijado por INIT-08-02).
4. Throughput escala con N workers hasta saturar; sin estado compartido entre tenants.
5. Soak test de M horas sin degradación perceptible ni fuga de memoria.

## Dependencies

- INIT-03-06 (línea base técnica del runtime) completada — es la entrada mínima.
- INIT-05-01 (CLI) completado para orquestar las corridas.
- INIT-06-03 (aislamiento multi-tenant) para INIT-08-03.

## Work Item Candidates

Ver `candidates:`.

## Open Questions

### Decisiones humanas para INIT-08-02 (2026-10-20)

- Aprobadas tres repeticiones completas por job en el mismo runner, con evidencia
  independiente por OS, ejecución, repetición y benchmark (AN-053).
- Umbral estricto < 10 % sobre `(max - min) / mediana` de las tres medianas
  Criterion y, por separado, de los tres p50 individuales (AN-054).
  p95/p99 se publican con rangos y advertencias, sin este umbral inicialmente.
- Recolector independiente con nearest-rank; calibrar resolución y overhead,
  sin sustracción automática. Conservar resultados inestables, sin certificarlos.
- Separar repetibilidad local de reproducibilidad entre workflows independientes.
- Commit documental de AN-053/054/055 autorizado en `draft`. Implementar AN-053;
  AN-054 puede avanzar tras estabilizar esa recolección. AN-055 espera mediciones.
  Integrar código de AN-053 en `main` requiere revisión del diff y workflow manual.
- La certificación requiere revisión humana. INIT-08-02 permanece abierto.
- Entregar tabla WASM básico / WASI / fuel / IR v0.6 con operación y sobrecarga
  diferenciadas, sin atribuir a una característica diferencias de operaciones.
  Usarla para decidir con evidencia las optimizaciones de INIT-08-03.

- OQ L153 (umbrales de rendimiento) — se cierra al publicar INIT-08-02; los números concretos salen de la medición, no de supuestos.
- Riesgo R-10 (afirmaciones de rendimiento sin medición) — mitigado por este bloque entero.

### Dictamen humano sobre AN-055 (2026-10-20)

Se acepta la publicación como candidata técnica, incluidos la regeneración,
trazabilidad y control offline en CI. No hay certificación automática de las 31
filas aceptables sin avisos, de las filas con avisos de instrumento ni de una
línea base cross-OS completa.

Antes de cerrar INIT-08-02 se requiere una nueva ejecución independiente de
AN-054 desde `main`, análisis entre sesiones —especialmente en macOS—,
caracterización de los avisos del reloj en Windows y una matriz aprobada de
referencias parciales por operación y entorno. La evidencia inestable permanece
publicada con sus limitaciones; no se exige estabilidad universal para una
posible certificación parcial.

### Preparación de INIT-08-03 (2026-10-20)

La publicación candidata permite identificar la carga de componentes en Linux y
Windows como candidata para comparar caché y warm pool. No fija aún un objetivo
Z%: las mediciones disponibles no bastan para distinguir una mejora objetivo del
ruido observado.

Antes de implementar una optimización, el work item de INIT-08-03 debe definir
una comparación controlada entre la implementación actual y la optimizada, con
el mismo componente, entrada, API, hardware y frontera temporal. La mejora debe
superar de forma clara la variación medida y justificar el coste de complejidad.

La certificación sigue abierta. La decisión pendiente es aprobar referencias
parciales, delimitadas por operación y entorno, o exigir primero la comprobación
independiente para cualquier cifra. La preferencia técnica registrada es exigir
esa comprobación para percentiles individuales y evaluar una certificación
parcial de Criterion solamente tras formalizar la revisión.

### Dictamen técnico de referencias parciales (2026-10-21)

Siete referencias Criterion no WASI de Windows son favorables para aprobación
humana limitada al entorno registrado. La referencia WASI
`cold_run_component_model_wasi_echo` requiere una nueva confirmación con runtime
idéntico o una equivalencia WASI documentada. No se modifica el umbral del 10 %:
las variaciones exactas y condiciones de medición, incluidas las filas cercanas
al límite, permanecen publicadas. La futura huella de fuentes efectivas del
runtime es un requisito metodológico para evitar incertidumbre entre sesiones.

## Learning

_Pendiente._

### Aprobación parcial de Criterion Windows (2026-10-21)

Se aprobó una referencia humana parcial para siete operaciones Criterion no WASI
en Windows, delimitada por los datasets `39449438474-windows` y
`39534970668-windows`, el runner, CPU y versiones registrados. La decisión queda
reproducible en `docs/benchmarks/inputs.json` y su matriz de alcance; no altera el
estado abierto de INIT-08-02 ni establece una línea base universal o cross-OS.

WASI, percentiles individuales, Linux y macOS siguen como evidencia técnica. El
siguiente trabajo de INIT-08-02 es registrar la huella efectiva de fuentes del
runtime y repetir la confirmación WASI con versiones idénticas en dos workflows
independientes. INIT-08-03 mantiene su puerta de evidencia: su comparación
controlada definirá cualquier umbral de mejora a partir de mediciones nuevas.
