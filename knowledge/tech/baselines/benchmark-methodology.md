# Metodología de benchmarks (INIT-08-01 / AN-051)

Complementa [`runtime-baseline.md`](runtime-baseline.md) (AN-017). Las cifras de
esta metodología **no son un SLA**; sirven para comparar antes y después de una
optimización sobre la misma máquina.

## Benches

| Archivo | Contenido |
|---|---|
| `crates/runtime/benches/baseline.rs` | 9 benches de AN-017 (engine, load, cold, warm, cache hit). Sin cambios. |
| `crates/runtime/benches/extended.rs` | WASI I/O, IR v0.6 `string`/`result`, invocación con fuel. |

Convención de nombres: `<stage>_<path>_<fixture>`.

| Bench | Qué aísla |
|---|---|
| `warm_run_component_model_wasi_echo` | `invoke_with_wasi` con `stdin` en memoria, artefacto precargado |
| `cold_run_component_model_wasi_echo` | Engine + carga con política + invocación WASI |
| `warm_run_component_model_v06_greet` | `string.concat` con parámetro y retorno `string` |
| `warm_run_component_model_v06_safe_div` | `result<i32, string>` |
| `warm_run_core_identity_fuel` | Coste del medido de fuel frente a `warm_run_core_identity` |

## Procedimiento

1. Cerrar aplicaciones pesadas y conectar el equipo a corriente.
2. `python scripts/bench_collect.py` (Python 3.9+, sin dependencias adicionales).
3. El recolector compila ambos benches y ejecuta tres repeticiones completas
   de `baseline` y `extended` en el mismo proceso de job/runner. Cada repetición
   usa `CRITERION_HOME=bench-results/run-XX/criterion` (Criterion 0.5) y no toca
   resultados previos en `target/criterion`. Se rechaza reutilizar la carpeta de
   salida; usar `--output <carpeta-nueva>` para otra ejecución local.
4. Conservar el artefacto completo: entorno raíz y por repetición (OS, CPU,
   toolchain, Wasmtime, commit, fecha UTC, run ID/intento y hashes de fuentes),
   logs, `collection.json`, datos crudos `criterion/**/new/` y `summary.json`.
   `run-XX/latency/` queda reservado para AN-054.
5. Regenerar el resumen con `python scripts/bench_aggregate.py bench-results`.
   No combinar directorios de workflows diferentes ni editar evidencia cruda.

El workflow manual produce un artefacto por OS, identificado por run ID e intento.
Sube evidencia incluso tras fallos y la retiene 30 días; descargarla para archivo
duradero antes de publicar una referencia. Una repetición fallida o con datos
incompletos no puede aceptarse, aunque haya archivos de resultados parciales.

Las estimaciones agregadas de Criterion no son percentiles por invocación.
El recolector individual, calibración de reloj/overhead y p50/p95/p99 pertenecen
a AN-054; `summary.json` de AN-053 solo evalúa medianas Criterion.

No usar `--quick` para publicar cifras: solo sirve para comprobar que compila y corre.

## Criterio de varianza

- Por bench y OS: `variacion = (max(r) - min(r)) / mediana(r1, r2, r3)`, donde
  cada `r` es `median.point_estimate` (ns) de una corrida Criterion completa.
  `aceptable` si es estrictamente < 0.10; en otro caso `inestable`. Conservar
  ambas en el resumen técnico. Ser aceptable no certifica una línea base.
- Registrar rango absoluto min/max/span y aviso por debajo de 10 µs. Rechazar
  datos ausentes, no finitos, no positivos, de comandos fallidos o con menos
  de 100 muestras Criterion (los datos `--quick` no son evidencia completa).
- Verificar mismo CPU, commit, toolchain, Wasmtime, fuentes e identidad de
  workflow. Hardware mixto impide evaluar variación; otros metadatos inválidos
  producen `incompleto`. Ninguna fila se certifica automáticamente.
- Tres corridas en un runner miden repetibilidad local. Otro workflow comprueba
  reproducibilidad de forma independiente, conservando sus propios datos.
- Una mejora solo se declara si supera esa banda y se cita la fila exacta.
- Los resultados cross-OS no se comparan entre sistemas distintos como si fueran
  la misma máquina; se comparan tendencias por sistema (INIT-08-02).

## Comparaciones para INIT-08-03

La tabla final (AN-055) distinguirá etapas, operaciones y métodos para WASM básico,
WASI, fuel e IR v0.6. Comparar identity, echo y greet mide escenarios distintos,
no la sobrecarga aislada de WASI o v0.6. Para atribuir sobrecarga hacen falta
controles de operación, entrada/salida, API y etapa comparables. Tampoco restar
automáticamente el overhead del instrumento a muestras individuales (AN-054).
Los resultados medidos y revisados serán la referencia para decidir caché o
warm pools. La publicación y certificación esperan AN-054/055 y revisión humana.

## Fuera de alcance

Memoria por instancia, concurrencia (INIT-08-04) y comparación contra
implementaciones nativas (INIT-08-02).
