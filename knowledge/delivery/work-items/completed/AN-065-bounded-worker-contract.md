---
id: AN-065
type: feature
status: completed
created_at: 2026-10-21T00:00:00.000Z
completed_at: 2026-10-21
title: Contrato y prototipo experimental de workers acotados
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: medium
code:
  - crates/runtime/src/
  - crates/runtime/tests/
source_roadmap_initiative: INIT-08-04
initiative: INI-008
depends_on:
  - AN-039
  - AN-040
  - AN-055
domains:
  - Runtime & Memory
  - Performance
  - Security
related_capabilities:
  - Independent Runtime Execution
  - Resource Contention & Limits
expected_value: >-
  Establecer una superficie experimental de concurrencia host-side con capacidad
  y backpressure explícitos, preservando Store e instancia frescos en cada
  solicitud y sin activar comportamiento productivo por defecto.
risks:
  - >-
    R-WORK-01: una cola o worker sin límite consume recursos sin control.
    Mitigación: capacidad fija, rechazo tipado y cierre explícito.
  - >-
    R-WORK-02: una solicitud transmite policy, límites o estado al siguiente
    trabajo. Mitigación: Store/instancia y controles creados por solicitud;
    AN-066 prueba contaminación y fallos concurrentes.
  - >-
    R-WORK-03: una cancelación, panic o trap bloquea capacidad de forma
    permanente. Mitigación: recuperación por worker y liberación verificable de
    cada slot.
related_decisions:
  - ADR-005
  - ADR-007
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
implemented_by: codex
closed_by: codex
implementation_status: completed
validation_status: passed
template_version: 1
---

# AN-065 — Contrato y prototipo experimental de workers acotados

## Propósito

Diseñar y, tras revisión humana, implementar un prototipo interno de workers
host-side. Una solicitud conserva el modelo ADR-005: el worker crea Store e
instancia frescos y los descarta al concluir, aun cuando el worker permanezca
vivo para atender otra solicitud.

## Alcance previsto

- capacidad máxima de workers y cola;
- backpressure con rechazo tipado al alcanzar capacidad;
- ciclo de vida `submit`, resultado/error y `close`;
- recuperación de capacidad ante éxito, error, trap, límites agotados,
  cancelación o panic;
- observabilidad agregada de cola y slots, sin datos de tenant;
- API interna o experimental, sin cambiar la ruta pública existente.

## Criterios de aceptación

- AC-01: cada solicitud crea Store e instancia frescos y aplica su propia
  policy, fuel, límites y host context.
- AC-02: workers y cola tienen límites configurables; el exceso se rechaza de
  forma explícita y no bloquea indefinidamente.
- AC-03: un fallo de solicitud libera o reemplaza su capacidad sin afectar las
  demás solicitudes.
- AC-04: `close` no acepta nuevas solicitudes y finaliza con un resultado
  documentado para las pendientes.
- AC-05: no se comparte un `Resolver` mutable ni una instancia entre tenants.
- AC-06: el prototipo no se habilita por defecto ni cambia la API productiva.
- AC-07: AN-066 y AN-067 quedan como puertas antes de cualquier adopción.

## Trazabilidad de identificador

AN-065 se reservó originalmente, sin materializarse, para una comparación A/A
de warm pool. Tras el cierre de AN-064 por no adopción, la decisión humana de
2026-10-21 reasigna el identificador a workers acotados. No se ejecutó ni se
infiere una comparación de warm pool con ese identificador.

## Puerta previa a código

Antes de implementar, se presentará un refinamiento técnico con API propuesta,
modelo de concurrencia, estrategia de aislamiento, taxonomía de errores y plan
de pruebas para revisión humana. El diseño propuesto está en
[`docs/benchmarks/an-065-bounded-workers-technical-design.md`](../../../../docs/benchmarks/an-065-bounded-workers-technical-design.md).

## Evidencia de implementación

- `bde8a8a`, `f05565d` y `9ce2fa8` implementaron y endurecieron el prototipo,
  sus pruebas y la validación de CI para la configuración predeterminada y
  `--all-features`.
- La CI [#39719357129](https://github.com/Kaddo-kdd/agent-native/actions/runs/39719357129)
  aprobó 7 de 7 jobs en Linux, Windows y macOS: formato, Clippy, tests y build
  en ambas configuraciones, pruebas offline de AWS Lambda e integridad de
  benchmarks.
- `experimental-workers` permanece deshabilitada por defecto. Cada solicitud
  usa `Tenant::invoke`, que crea un `Store` y una instancia nuevos; no existe un
  pool de instancias ni se modificó la ruta productiva.

## Learning

Reutilizar hilos del host puede mejorar la capacidad sin reutilizar instancias
WebAssembly. El prototipo debe mantenerse experimental hasta que AN-066 pruebe
aislamiento concurrente y recuperación ante fallos, y AN-067 aporte mediciones
de saturación, latencia y estabilidad.
