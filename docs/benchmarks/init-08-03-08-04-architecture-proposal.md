# Propuesta arquitectónica — Ajuste de INIT-08-03 e INIT-08-04

## Estado y decisión que se conserva

AN-064 queda correctamente cerrado como **no adopción de warm pool genérico**.
La prueba `warm_pool_feasibility` mostró que una instancia Wasmtime viva conserva
un global mutable; no hay en el runtime actual un reset genérico verificable de
`Store`, memoria, tablas, globals, recursos ni host context. ADR-005 permanece
vigente: toda solicitud productiva crea y descarta su Store e instancia.

La propuesta no reabre AN-064, no certifica rendimiento, no modifica INIT-08-02,
ni cambia el umbral de estabilidad del 10 %.

## Ajuste propuesto para INIT-08-03

Se propone redefinir su resultado como una **decisión de arquitectura sobre
reutilización de instancias**, no como la obligación de entregar un warm pool.
Sus criterios de cierre propuestos son:

1. AN-063 valida la baseline de aislamiento frente a globals, memoria, trap,
   fuel y policy; AN-064 documenta la no viabilidad de la reutilización genérica.
2. ADR-005 conserva Store e instancia frescos como comportamiento de producto.
3. El roadmap separa los dos caminos futuros que no forman parte de AN-064:
   contrato de guest reseteable y pooling allocator.
4. No se promulga una API de pool ni se afirma una mejora de rendimiento.

Con esos criterios, INIT-08-03 **podría cerrarse por no adopción**, previa
aprobación humana. El trabajo de reset y el allocator no deben mantenerla abierta:
son alternativas con hipótesis, riesgos y aceptación propios.

### Camino futuro A — Guests reseteables

Es una capacidad experimental futura, no una implementación autorizada. Requiere
un Work Item independiente que defina:

- interfaz de reset con nombre, versión, precondiciones y errores tipados;
- cobertura explícita de memoria, globals, tablas, recursos Canonical ABI,
  host context, fuel, límites e I/O, o exclusión verificable de cada categoría;
- verificación posterior al reset frente a un fixture de contaminación y pruebas
  negativas de éxito, trap, cancelación, error de host y límites agotados;
- identidad de compatibilidad que impida mezclar artefactos, versiones, policy o
  tenant en una reutilización;
- descarte incondicional si falta la interfaz, el reset falla o la verificación
  es incompleta;
- revisión de seguridad antes de exponer una API experimental.

El criterio de éxito no es una reducción de latencia: es demostrar que la
reutilización no comparte estado observable. Solo después se consideraría una
comparación controlada en Linux.

### Camino futuro B — Pooling allocator de Wasmtime

Wasmtime 29 expone `PoolingAllocationConfig` bajo la característica
`pooling-allocator`. Según la documentación local de esa versión, recicla slots
de asignación **después de que la instancia se desasigna** y puede conservar
recursos de memoria para acelerar instanciaciones posteriores. No equivale a
retener una instancia viva y, en principio, podría conservar la semántica de
Store/instancia fresca de ADR-005.

No se presupone que sea aplicable o beneficioso aquí:

- el crate actual no activa la característica `pooling-allocator`;
- exige capacidades fijas de instancias, memorias y tablas al construir el
  `Engine`;
- puede reservar memoria virtual considerable y retener RSS; Wasmtime advierte
  que su rendimiento no está igualmente afinado en Windows;
- la compatibilidad con Component Model, límites por invocación, las fixtures y
  los tres sistemas requiere pruebas propias;
- su beneficio se mide contra una A/A y baseline Linux controladas, sin inferir
  resultados desde AN-064.

Se propone un futuro Work Item independiente de evaluación, no dependiente de un
contrato de guest reset. Debe compilar con la feature, fijar límites conservadores,
probar destrucción de instancia entre tenants, medir memoria/RSS y rechazar la
alternativa si no supera el ruido o cambia garantías de aislamiento.

## Ajuste propuesto para INIT-08-04 — Concurrencia host-side

INIT-08-04 no depende de la adopción de warm pool. Se propone sustituir su
dependencia por **INIT-06-03, INIT-08-01 y la publicación de AN-055**: los workers
host-side pueden despachar solicitudes concurrentes mientras cada una conserva
Store e instancia fresca. INIT-08-03 solo será una dependencia informativa de la
decisión de no reutilizar instancias.

### Contrato de workers propuesto

- capacidad de workers y cola explícitamente acotadas;
- backpressure explícito: rechazo tipado cuando la cola está llena, sin cola
  ilimitada ni espera silenciosa;
- cada solicitud aplica su propia `HostPolicy`, `ExecutionLimits`,
  `InstanceLimits` y host context, y crea Store/instancia fresca;
- ningún `Resolver` mutable se comparte sin sincronización y sin una política
  de aislamiento explícita; se preferirá componente inmutable pre-cargado o
  estado local de worker con límites documentados;
- una cancelación, panic, trap, error del host o agotamiento de recursos finaliza
  solo la solicitud afectada y libera su capacidad;
- `close` deja de aceptar trabajo, drena o rechaza según contrato y libera los
  workers sin perder errores.

### Trazabilidad de AN-065 y Work Items propuestos

El identificador **AN-065 no fue materializado**. En el diseño inicial de
INIT-08-03 se reservaba para una comparación A/A de warm pool; AN-064 concluyó
que no hay tratamiento genérico seguro que comparar. Esta propuesta reutiliza
AN-065 para workers acotados y registra expresamente la sustitución para que no
parezca que la comparación de warm pool fue ejecutada o descartada por datos de
rendimiento. La reasignación solo entra en vigor con aprobación humana.

| ID propuesto | Entrega | Criterios principales |
|---|---|---|
| AN-065 | Contrato y prototipo de workers acotados | API interna/experimental; capacidad y backpressure; Store/instancia fresca; sin activación por defecto. |
| AN-066 | Suite concurrente de aislamiento y fallos | Tenants alternados y simultáneos; policy, fuel, memoria, trap, panic, cancelación, cola llena y cierre; ningún estado cruza solicitudes. |
| AN-067 | Medición de saturación, throughput y soak | A/A, N workers, cola y latencias por solicitud; memoria/RSS, errores y degradación; sesiones separadas Linux. |

No se fija N, duración de soak, SLA ni porcentaje de throughput antes de la
primera campaña controlada. AN-067 propondrá criterios cuantitativos con la
variación observada y requerirá revisión humana para cualquier adopción.

## Cambios propuestos en el roadmap

1. Marcar INIT-08-03 como candidata a cierre por decisión de no adopción, sin
   cambiar todavía su estado.
2. Añadir “Guests reseteables” y “Pooling allocator de Wasmtime” como trabajo
   futuro independiente, ambos sin iniciativa activa.
3. Cambiar las dependencias de INIT-08-04 a INIT-06-03, INIT-08-01 y AN-055;
   conservar INIT-08-03 como antecedente de seguridad, no como bloqueo.
4. Sustituir los Work Items genéricos de worker/soak por AN-065, AN-066 y AN-067.
5. Reformular el criterio 3 de RM-08 y M8: sustituir “caché mejora warm start
   en Z%” por una **decisión trazable sobre optimización de instanciación**.
   Puede consistir en adopción con mejora medida en campaña controlada o en no
   adopción documentada por un límite de seguridad; ningún Z% se presume.
6. Ajustar el hito RM-08 para separar evidencia de concurrencia con instancias
   frescas de cualquier optimización posterior de instanciación.


## Reformulación propuesta de RM-08 y M8

| Elemento actual | Ajuste propuesto pendiente de aprobación |
|---|---|
| RM-08, criterio 3: “Caché mejora warm start en Z%” | “Existe una decisión humana trazable sobre optimización de instanciación: adopción respaldada por una comparación controlada o no adopción sustentada por seguridad y evidencia.” |
| RM-08, criterio 4: throughput con N workers | “Workers acotados preservan aislamiento y backpressure; la campaña AN-067 caracteriza throughput y saturación sin objetivo inventado.” |
| RM-08, criterio 5: soak M horas sin degradación | “AN-067 define duración, carga y señales de degradación tras una baseline controlada; los datos y límites quedan publicados.” |
| M8: línea base, caché cuantificado, concurrencia | “Línea base técnica publicada, decisión de instanciación trazable y concurrencia host-side evaluada con aislamiento, saturación y soak.” |

## Decisiones humanas requeridas

- Aprobar o rechazar el cierre de INIT-08-03 por no adopción.
- Aprobar que INIT-08-04 avance con workers de instancia fresca.
- Decidir si alguna de las dos alternativas futuras merece materialización.

Hasta esas decisiones no se implementa código, no se crean Work Items activos y
no se cambia el estado de las iniciativas.