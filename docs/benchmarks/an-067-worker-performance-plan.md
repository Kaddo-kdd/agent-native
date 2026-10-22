# AN-067 — Plan de medición de workers host-side

## Propósito

Determinar si reutilizar hilos del host mejora la capacidad de ejecución sin
atribuir esa mejora a reutilización de instancias. Cada trabajo del pool sigue
creando un `Store` y una instancia nuevos.

## Comparación A/A

Cada pareja de mediciones usa el mismo commit, componente WASM, argumentos,
política, límites de fuel y memoria, perfil `release` y entorno. Cambia solo el
modo de despacho:

| Modo | Despacho |
| --- | --- |
| Secuencial | `Tenant::invoke` por solicitud, sin pool. |
| Workers | `WorkerPool` con `experimental-workers`, número de workers y cola registrados. |

La primera campaña se limita a Linux x86_64 GNU. Los resultados de otros
sistemas, si se capturan, se publican como sesiones independientes y no se
usan para fijar una conclusión de throughput.

## Matriz parametrizada

La herramienta recibe y guarda estos valores, sin asignar objetivos previos:

- `workers`: lista de capacidades positivas.
- `queue-capacity`: lista de capacidades acotadas.
- `concurrency`: lista de emisores simultáneos.
- `repetitions`, `warmup-requests` y `measured-requests` para campañas de
  tamaño fijo.
- `soak-duration` y `rss-sample-interval` para campañas prolongadas.
- `seed` para hacer reproducible el orden de casos.

La matriz debe incluir control secuencial, una fila de worker único y al menos
una fila con concurrencia ofrecida mayor que los workers. Las configuraciones
duplicadas se eliminan antes de ejecutar.

## Cargas

1. Core `identity`: operación breve para exponer coste de despacho y cola.
2. Component Model `point_sum`: operación de coste mayor ya presente en la
   línea base.
3. Saturación controlada: misma carga con concurrencia superior a la capacidad
   admitida para medir rechazos explícitos, sin reintentos ocultos.

WASI, entrada/salida externa y reutilización de instancias quedan fuera de esta
fase.

## Artefactos por sesión

Cada directorio de sesión debe contener:

- `manifest.json`: SHA, timestamp UTC, host, herramientas, perfil, features y
  parámetros completos.
- `requests.ndjson`: una fila por solicitud con latencia, resultado y causa de
  error o rechazo.
- `resources.ndjson`: muestras RSS disponibles, con fuente e intervalo.
- `summary.json`: métricas recalculables y referencias a las muestras.
- `SHA256SUMS`: hashes de los artefactos de la sesión.

El agregador calcula p50, p95 y p99 con un método documentado y estable, junto
con throughput completado, aceptaciones, rechazos, errores y ocupación máxima.
No elimina outliers ni agrupa sesiones distintas.

## Soak y decisión

Antes de iniciar un soak se registra duración, carga, workers, cola, criterio
de parada y método de muestreo de RSS. El informe describe variación de
throughput, percentiles, capacidad y RSS; una tendencia es una observación, no
una prueba de fuga.

La campaña termina con una recomendación basada en las repeticiones A/A. No
autoriza por sí misma habilitar workers por defecto ni adoptar una meta de
rendimiento.
