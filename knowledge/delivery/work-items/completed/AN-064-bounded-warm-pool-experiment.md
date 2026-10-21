---
id: AN-064
type: feature
status: completed
created_at: 2026-10-21T00:00:00.000Z
completed_at: 2026-10-21T00:00:00.000Z
title: Variante experimental de warm pool acotado
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: medium
code:
  - crates/runtime/src/
  - crates/runtime/tests/warm_pool_contract.rs
  - crates/runtime/tests/warm_pool_experiment.rs
  - crates/runtime/benches/
  - docs/benchmarks/
source_roadmap_initiative: INIT-08-03
initiative: INI-008
depends_on:
  - AN-063
domains:
  - Runtime & Memory
  - Performance
  - Security
related_capabilities:
  - Independent Runtime Execution
  - Resource Contention & Limits
expected_value: >-
  Probar si una variante de instancias preparadas puede superar a la baseline
  Linux sin relajar aislamiento, policy ni límites, permaneciendo desactivada
  para consumidores hasta la decisión de AN-065.
risks:
  - >-
    R-POOL-03: la limpieza parcial deja estado de guest, host context o límites
    consumidos. Mitigación: descarte obligatorio ante error, trap, límite,
    cancelación o fallo de limpieza; pruebas AN-063 ejecutadas contra el
    tratamiento.
  - >-
    R-POOL-04: el coste de checkout y wipe supera el beneficio. Mitigación:
    benchmark pareado y A/A definidos; AN-065 puede decidir no adoptar.
related_decisions:
  - ADR-005
  - ADR-007
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
---

# AN-064 — Variante experimental de warm pool acotado

## Estado: completado sin adopción

El experimento concluye que no es seguro implementar un warm pool genérico para
los componentes actuales. `warm_pool_feasibility` demuestra que una instancia
Wasmtime viva retiene un global mutable entre invocaciones; el runtime no dispone
de un reset genérico verificable de `Store`, memoria, tablas, globals, recursos o
host context. Por ello no se incorporó una variante funcional ni se modificó la
ruta productiva de Store e instancia frescos.

La evidencia se publicó en `1a25646` y aprobó CI `39573751113` (7/7 en Linux,
Windows y macOS) y Website `39573751072`.

## Objetivo

Implementar una variante experimental y reversible para medir un warm pool contra
la baseline de ADR-005. No sustituye el camino público actual, no se activa por
defecto y no habilita reutilización para consumidores.

## Contrato de la variante

La implementación deberá exponer una superficie interna o explícitamente
experimental con estas propiedades:

- capacidad máxima configurada y métrica de `checkout`, `return`, `discard` y
  `close` sin datos de tenant;
- identidad de pool ligada al artefacto, versión de runtime y configuración de
  compatibilidad que afecte la instancia; nunca compartida entre artefactos;
- policy, `ExecutionLimits`, `InstanceLimits` y host context se aplican para cada
  checkout, no se heredan de una llamada previa;
- toda finalización no satisfactoria —error de host, trap, fuel/memoria/I/O
  agotados, cancelación, panic o wipe fallido— descarta la instancia;
- `close` descarta toda instancia pendiente y hace que futuros checkouts fallen
  de forma explícita;
- la ruta baseline con `Store` fresco conserva su API y semántica sin cambios.

No se asumirá que el estado de una instancia puede resetearse correctamente. Si
Wasmtime o el contrato del guest no permiten una limpieza demostrable, AN-064
puede limitarse a un tratamiento que descarte la instancia y documentar que no
existe warm pool seguro para esos fixtures.

## Alcance de medición

Los primeros fixtures son `core_identity` y `component_model_point_sum`, sin
WASI. Cada medida incluye checkout, aplicación de controles, invocación y
wipe/descarte dentro de la frontera de solicitud. Engine, componente y API se
mantienen idénticos entre baseline y tratamiento.

Antes de medir beneficio se añadirá una comparación A/A de la baseline. El
umbral Z no se fija en este Work Item: AN-065 lo propondrá a partir de la
dispersión observada en Linux x86_64 GNU controlado.

## Hallazgo de viabilidad

AN-064 verificó que una instancia Wasmtime viva conserva su global mutable entre
llamadas. La evidencia ejecutable está en
`crates/runtime/tests/warm_pool_feasibility.rs`: dos llamadas a una instancia
reutilizada devuelven `1` y `2`; una instancia creada en un `Store` fresco vuelve
a devolver `1`. La API de Wasmtime usada por el runtime no aporta un reset
genérico y verificable de globals, memoria, tablas, recursos ni host context.

Por ello, un pool genérico para los componentes actuales no puede satisfacer
AC-03 y AC-04. No se implementará reutilización de instancias bajo AN-064. La
baseline de Store e instancia frescos se conserva. Una propuesta futura solo
sería posible con un contrato explícito de guest para reset verificable, con
revisión de seguridad y un Work Item nuevo; no se infiere de este hallazgo.

## Criterios de aceptación

- AC-01: la variante está aislada detrás de una API o flag experimental y no
  cambia la conducta por defecto.
- AC-02: capacidad, identidad, descarte y cierre están implementados y probados.
- AC-03: todos los casos de `warm_pool_contract` pasan contra el tratamiento, o
  se documenta que el tratamiento descarta de modo que no haya reutilización.
- AC-04: policy, fuel, memoria y host context se aplican por checkout.
- AC-05: no se publica una afirmación de mejora ni se certifica rendimiento.
- AC-06: el código y tests pasan CI en Linux, Windows y macOS antes de AN-065.

## Fuera de alcance

No incorpora workers, WASI pooling, una API estable, una meta Z ni una
certificación de WASI o percentiles. No modifica INIT-08-02 ni el umbral del
10 %.

## Seguimiento arquitectónico propuesto

El cierre de AN-064 no materializa AN-065. La propuesta
`docs/benchmarks/init-08-03-08-04-architecture-proposal.md` separa una futura
interfaz de guest reseteable, la evaluación independiente del pooling allocator
de Wasmtime y los workers host-side con instancia fresca. Todos requieren nuevas
decisiones humanas antes de iniciar implementación.