---
type: initiative
id: INI-008
title: RM-08 — Performance & Concurrency
status: completed
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
    materialized_as: 'AN-051 (completed), AN-052 (completed)'
  - id: INIT-08-02
    title: Línea base pública cross-OS
    status: candidate
    materialized_as: >-
      AN-053 (completed), AN-054 (completed), AN-055 (completed), AN-061
      (completed), AN-062 (completed)
  - id: INIT-08-03
    title: Caché de componentes y warm pool
    status: completed
    materialized_as: 'AN-063 (completed); AN-064 (completed, sin adopción)'
  - id: INIT-08-04
    title: Concurrencia host-side (workers)
    status: completed
    materialized_as: 'AN-065, AN-066 y AN-067 (completed)'
completed_at: '2026-10-24'
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
3. Existe una decisión humana trazable sobre optimización de instanciación:
   adopción respaldada por una comparación controlada o no adopción sustentada
   por seguridad y evidencia.
4. Workers acotados preservan aislamiento y backpressure; AN-067 caracteriza
   throughput y saturación sin objetivo inventado.
5. El soak define duración, carga y señales de degradación tras la baseline
   controlada, y publica su evidencia.

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

### Cierre condicionado de AN-061 y AN-062 (2026-10-21)

Se aprobó cerrar AN-061 y AN-062 como investigación diagnóstica. La excepción
aceptada registra que el A/A se ejecutó en runners hospedados, no en un host
controlado: Linux y Windows cambiaron de CPU registrada entre sesiones y macOS
no prueba identidad física. La decisión cierra los work items, no certifica
WASI ni modifica AN-055, INIT-08-02, el umbral del 10 % o las siete referencias
parciales de Windows. Ver `docs/benchmarks/an061-an062-closure-proposal.md`.

### Cierre limitado de AN-055 (2026-10-21)

Se aprobó cerrar AN-055 como publicación reproducible de evidencia técnica.
Solo las siete referencias Criterion no WASI de Windows permanecen certificadas
parcialmente; Linux, macOS, WASI y percentiles individuales siguen sin
certificación. INIT-08-02 continúa abierta bajo su alcance cross-OS original.
Ver `docs/benchmarks/an055-init0802-closure-proposal.md`.


### Refinamiento de INIT-08-03 (2026-10-21)

INIT-08-03 se refina como una decisión experimental y de aislamiento, no como
una optimización ya aprobada. La referencia será Linux x86_64 GNU en una
comparación controlada A/A y baseline/tratamiento con idéntico SHA, componente,
entrada, API y frontera temporal. El Resolver ya proporciona caché de artefactos;
cualquier caché adicional debe justificar qué coste no cubre. Un warm pool solo
puede ser experimental tras demostrar aislamiento, wipe o descarte seguro y
aplicación por checkout de policy, fuel y límites. El valor Z se propondrá a
partir de la dispersión A/A, no de cifras cross-OS o históricas. Ver
`docs/benchmarks/init-08-03-experimental-design.md`.


### Propuesta pendiente — trazabilidad AN-065 y criterio M8 (2026-10-21)

AN-065 no se materializó para warm pool. La propuesta solicita reutilizar el
identificador para el contrato de workers y registra la sustitución para evitar
ambigüedad histórica. También propone reemplazar el criterio M8 de mejora Z% de
caché por una decisión trazable de adopción o no adopción de optimización de
instanciación. Ninguno de estos cambios aplica sin aprobación humana. Ver
`docs/benchmarks/init-08-03-08-04-architecture-proposal.md`.

### Propuesta pendiente — ajuste INIT-08-03/08-04 (2026-10-21)

Pendiente de decisión humana: cerrar INIT-08-03 por no adopción de warm pool
genérico; tratar guests reseteables y pooling allocator de Wasmtime como
alternativas futuras independientes; y permitir INIT-08-04 con workers host-side
de capacidad acotada y Store/instancia fresca, dependiente de INIT-06-03,
INIT-08-01 y AN-055. Ver
`docs/benchmarks/init-08-03-08-04-architecture-proposal.md`.

### Resultado de AN-064 (2026-10-21)

AN-064 concluyó sin adoptar un warm pool genérico. La prueba ejecutable muestra
que una instancia Wasmtime reutilizada conserva estado mutable y el runtime no
dispone de una limpieza genérica verificable. Se mantiene Store e instancia
frescos por invocación; AN-065 no se materializa sin un tratamiento seguro que
comparar. INIT-08-03 continúa abierta para una propuesta futura con contrato de
reset explícito y revisión de seguridad.


### Decisión humana aprobada — transición a workers frescos (2026-10-21)

Se cierra INIT-08-03 por no adopción de warm pool genérico, preservando AN-063,
AN-064 y ADR-005. RM-08/M8 reemplaza el requisito de Z% obligatorio por una
decisión trazable de optimización de instanciación. AN-065 se reasigna, con
trazabilidad explícita, al contrato de workers acotados. INIT-08-04 avanza con
Store e instancia frescos por solicitud, capacidad y backpressure explícitos.
Guests reseteables y pooling allocator de Wasmtime permanecen como investigaciones
futuras independientes sin implementación autorizada. AN-065, AN-066 y AN-067
quedan materializados y listos; no se inicia código hasta revisar AN-065.
