# INIT-08-03 — Diseño experimental para caché de componentes y warm pool

## Propósito

Decidir con evidencia si Wasma debe incorporar una optimización adicional al
`Resolver` content-addressed existente o un warm pool de instancias. La iniciativa
no presupone que ninguna de las dos reduzca la latencia útil ni autoriza a
reutilizar estado de guest por defecto.

El `Resolver` ya evita recompilaciones y recargas redundantes de artefactos. Por
tanto, los benchmarks históricos `cache_hit_run_*` miden **cache hit de
artefacto + invocación**, no un pool de instancias. La diferencia histórica entre
cold y warm tampoco mide por sí sola el beneficio marginal de un warm pool:
cambian varias etapas a la vez.

## Alcance de la primera iteración

La primera entrega es un experimento controlado y reversible en Linux x86_64 GNU,
plataforma operativa recomendada por ADR-007. Windows y macOS siguen cubiertos por
CI funcional, pero sus cifras no se usarán para decidir una optimización de
rendimiento.

El experimento compara, con el mismo SHA, componente, entrada, API, política,
límites, proceso y fronteras temporales:

1. **Baseline:** `Resolver` actual; artefacto ya cargado cuando corresponda y
   `Store`/instancia fresca por invocación.
2. **Tratamiento de caché:** únicamente si se identifica un coste adicional que
   el `Resolver` actual no cubra. Debe declarar qué dato se retiene, su clave,
   invalidación, límite de capacidad y por qué no duplica la caché existente.
3. **Tratamiento de pool:** checkout de una instancia preparada, invocación,
   limpieza verificable y devolución o descarte. El pool no se habilita para
   tráfico de producto en esta entrega.

Cada variante debe mantener la misma frontera de solicitud: desde que el runtime
recibe el componente/entrada hasta que devuelve el resultado o error. Las etapas
internas pueden medirse por separado para diagnóstico, pero no se suman ni se
atribuyen causalidad fuera de la frontera declarada.

Los fixtures iniciales serán los caminos no WASI ya disponibles (`core_identity`
y `component_model_point_sum`). WASI queda excluido de la comparación de decisión
mientras su referencia de rendimiento siga sin certificación. Los resultados de
fixtures distintos no se comparan entre sí.

## Invariantes no negociables

ADR-005 conserva `Store` fresco por invocación como comportamiento de producción.
Una variante de pool solo es admisible como experimento aislado si demuestra:

- no se comparte memoria, globals, recursos, handles, fuel ni resultados entre
  solicitudes o tenants;
- `HostPolicy`, `ExecutionLimits` e `InstanceLimits` se aplican en cada checkout,
  igual que en el camino baseline;
- una limpieza fallida, incompleta o no verificable descarta la instancia; nunca
  la devuelve al pool;
- los errores, traps, cancelaciones y límites agotados también descartan la
  instancia;
- la capacidad es acotada, el cierre libera todos los recursos y las métricas no
  exponen datos de tenant;
- el comportamiento público conserva el camino seguro existente hasta una
  aprobación humana posterior.

Antes de ejecutar el pool se añadirá un fixture de contaminación controlada y
pruebas negativas que prueben que el estado de una invocación no es observable en
la siguiente. Si el runtime no puede garantizar esa propiedad, el resultado
válido es **no implementar pool** y conservar la baseline de ADR-005.

## Protocolo de medición

- Ejecutar las variantes intercaladas en una misma sesión Linux controlada o
  dedicada; conservar cada repetición por separado y registrar SHA, dependencias,
  CPU, imagen, frecuencia/afinidad disponible y resolución del reloj.
- Hacer al menos una segunda sesión independiente sin cambio de código ni
  instrumento. No mezclar muestras entre sesiones.
- Mantener Criterion y el recolector por invocación separados. Reportar mediana
  Criterion y p50/p95/p99 individuales con su rango, sin intercambiarlos.
- Incluir una comparación A/A de baseline contra sí misma. Su dispersión es la
  referencia para establecer la diferencia mínima detectable de esa campaña.
- Calcular la mejora observada por cada fixture y etapa contra controles pareados;
  no reutilizar porcentajes cross-OS ni cifras históricas como meta Z%.

El umbral estricto actual de 10 % continúa clasificando estabilidad local de las
medianas Criterion y p50. No certifica percentiles ni convierte el experimento
en una certificación WASI.

## Puertas de decisión

### Puerta 1 — preparación

Se puede materializar la instrumentación experimental si el diseño declara API,
clave y ciclo de vida de cada recurso retenido, y las pruebas de aislamiento
cubren éxito, error, trap, agotamiento de fuel/memoria, cambio de política y
cierre del pool.

### Puerta 2 — evidencia

Tras las dos sesiones, el informe debe presentar la A/A, la dispersión por
fixture, las variantes comparadas, memoria/capacidad y cualquier descarte. Una
mejora solo es candidata si supera claramente la diferencia mínima detectable
que arroje el control A/A y no introduce una regresión de seguridad, límites o
semántica. El valor Z se propondrá entonces, con datos, para revisión humana.

### Puerta 3 — producto

Solo una aprobación humana expresa puede convertir una variante de pool en
comportamiento de producción. Si los datos no muestran un beneficio claro o la
limpieza no se demuestra, se documenta la decisión de no implementar y se cierra
la iniciativa sin añadir complejidad.

## Secuencia de Work Items propuesta

1. **AN-063 — Contrato de experimento y aislamiento para warm pool** (`chore`,
   K4): **completado**. Define la frontera temporal y su fixture de
   contaminación valida globals, memoria, trap, fuel y policy sin cambiar el
   camino de producción.
2. **AN-064 — Implementación experimental de caché/pool acotado** (`feature`,
   K4): **en curso**, detrás de una API o flag
   experimental, con capacidad acotada, descarte seguro y pruebas negativas.
3. **AN-065 — Comparación controlada y decisión de producto** (`chore`, K4):
   ejecutar A/A y baseline/tratamiento en Linux controlado, publicar evidencia y
   proponer Z o la decisión de no adoptar la optimización.

INIT-08-04 depende de una decisión explícita de AN-065; un worker pool no debe
ampliar un mecanismo de reutilización cuya seguridad o beneficio no esté probado.

## Fuera de alcance

- Certificar WASI, p50/p95/p99 o rendimiento cross-OS.
- Cambiar el umbral de estabilidad del 10 %.
- Presentar Linux como más rápido que Windows o macOS.
- Optimizar allocator, guest async o workers host-side.
- Habilitar reutilización de instancias para consumidores sin una aprobación
  posterior basada en el experimento.

## Evidencia de partida

- `docs/benchmarks/README.md` y `docs/benchmarks/certification-matrix.md`:
  publicación técnica y certificaciones parciales existentes.
- `docs/benchmarks/an055-init0802-closure-proposal.md`: INIT-08-02 sigue abierta;
  su estado no bloquea un experimento controlado, pero tampoco aporta una meta Z.
- `knowledge/tech/decisions/ADR-005-modelo-memoria-canonical-abi.md`: baseline
  segura de Store fresco por invocación.
- `knowledge/tech/decisions/ADR-007-plataforma-operativa-inicial-wasma.md`:
  Linux x86_64 GNU es la plataforma operativa inicial recomendada.
- `docs/benchmarks/wasi-diagnostic-aa.md`: la variabilidad observada impide usar
  WASI o runners hospedados no controlados para afirmar una mejora de producto.