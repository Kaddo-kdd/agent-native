---
id: AN-063
type: chore
status: completed
created_at: 2026-10-21T00:00:00.000Z
completed_at: 2026-10-21T00:00:00.000Z
title: Contrato experimental y aislamiento para warm pool
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
code:
  - crates/runtime/tests/warm_pool_contract.rs
  - docs/benchmarks/init-08-03-experimental-design.md
source_roadmap_initiative: INIT-08-03
initiative: INI-008
depends_on:
  - AN-039
  - AN-040
  - AN-055
  - AN-062
domains:
  - Runtime & Memory
  - Performance
  - Security
related_capabilities:
  - Independent Runtime Execution
  - Resource Contention & Limits
expected_value: >-
  Convertir ADR-005 en un contrato ejecutable para evaluar un pool de instancias
  sin habilitar reutilización de producción: prueba el aislamiento de globals y
  memoria, así como el descarte tras trap o fuel agotado y la aplicación de
  políticas por tenant.
risks:
  - >-
    R-POOL-01: una implementación posterior reutiliza una instancia con estado de
    guest o límites agotados. Mitigación: fixture de contaminación y pruebas de
    éxito, trap y fuel como puerta obligatoria antes de AN-064.
  - >-
    R-POOL-02: una policy concedida a un tenant se filtra a otro. Mitigación:
    prueba de deny-by-default seguida de invocación autorizada con host context
    independiente.
related_decisions:
  - ADR-005
  - ADR-007
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
---

# AN-063 — Contrato experimental y aislamiento para warm pool

## Estado: completado

El contrato se integró en `main` mediante `b113b82` y su corrección `132b3c8`. La CI `39571561792` aprobó los siete jobs en Linux, Windows y macOS; Website `39571561818` también finalizó correctamente.

## Objetivo

Preparar una decisión segura sobre caché adicional o warm pool. AN-063 no añade
un pool, no retiene `Store` o instancias y no altera el comportamiento público.
Implementa la prueba de contrato que AN-064 deberá preservar o sustituir por una
limpieza verificable equivalente.

## Frontera experimental

Una solicitud se mide desde la recepción de componente e input hasta su resultado
o error. La baseline sigue creando un `Store` e instancia por llamada. Un futuro
tratamiento de pool incluirá checkout, aplicación de policy/límites, invocación y
wipe o descarte dentro de esa misma frontera.

## Implementación entregada

`crates/runtime/tests/warm_pool_contract.rs` construye un fixture Core WASM con
un global mutable y memoria lineal. Verifica que la baseline actual:

1. devuelve el global y memoria iniciales en cada invocación, incluso para el
   mismo tenant;
2. descarta estado después de un trap que mutó global o memoria;
3. descarta estado después de agotar fuel;
4. vuelve a aplicar `HostPolicy` por tenant antes de instanciar, y usa un host
   context independiente para la invocación autorizada.

## Criterios de aceptación

- AC-01: existe una frontera temporal explícita para baseline y futuro tratamiento.
- AC-02: un fixture de contaminación demuestra que globals y memoria no cruzan
  invocaciones ni tenants en la baseline.
- AC-03: trap y fuel agotado no dejan un recurso reutilizable implícito.
- AC-04: policy deny-by-default se valida de nuevo por tenant.
- AC-05: AN-063 no crea `Store`, instancia, API pública ni flag de pool
  persistentes; ADR-005 permanece como la única conducta de producción.

## Siguiente puerta

AN-064 queda materializado como el sucesor experimental. Solo puede crear una variante experimental acotada tras revisar este
contrato. Debe añadir capacidad máxima, ciclo de cierre, descarte en cualquier
error y pruebas que pasen con el tratamiento. Si no puede preservar AC-02--AC-04,
la decisión correcta es no implementar el pool.