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
2. `cargo run -p agent-native-runtime --example bench_env > bench-env.json`
3. `cargo bench -p agent-native-runtime --bench baseline`
4. `cargo bench -p agent-native-runtime --bench extended`
5. Repetir los pasos 3 y 4 al menos tres veces y conservar `bench-env.json`.

No usar `--quick` para publicar cifras: solo sirve para comprobar que compila y corre.

## Criterio de varianza

- Una cifra es publicable si la mediana de tres corridas completas difiere menos
  del 10 % entre sí. Si no, repetir en una máquina más estable.
- Una mejora solo se declara si supera esa banda y se cita la fila exacta.
- Los resultados cross-OS no se comparan entre sistemas distintos como si fueran
  la misma máquina; se comparan tendencias por sistema (INIT-08-02).

## Fuera de alcance

Memoria por instancia, concurrencia (INIT-08-04) y comparación contra
implementaciones nativas (INIT-08-02).
