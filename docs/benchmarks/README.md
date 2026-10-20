# Línea base medida candidata — pendiente de revisión humana

## Limitaciones y estado

**Ninguna fila está certificada.** Un workflow verde prueba recolección completa, no estabilidad ni certificación.
Las filas inestables permanecen visibles y quedan excluidas de afirmaciones de línea base certificada. Las aceptables con avisos requieren revisión del instrumento.

- Escenarios de runners hospedados, sin rendimiento universal por OS ni SLA.
- Repetibilidad en el mismo runner y reproducibilidad entre workflows son evaluaciones separadas.
- Las medianas Criterion no son p50 de invocaciones individuales.
- p95/p99 conservan rangos y advertencias de variabilidad, sin umbral inicial del 10 %.
- No se resta overhead; el salto de reloj observado no es su resolución intrínseca.
- Las salidas esperadas describen contratos; no son valores devueltos archivados.
- Operaciones distintas no aíslan la sobrecarga de WASI, IR v0.6 ni del runtime.
- Los intervalos de fuel son rangos observados, no de confianza; el orden fijo puede introducir deriva temporal.
- No hay comparaciones con implementaciones nativas ni con el arranque de AWS Lambda.

Umbral conservado: `(max - min) / mediana` de tres medianas Criterion < 10 %; por separado, la misma regla sobre tres p50 individuales.
Porcentajes y µs se redondean para mostrar; la clasificación usa los valores completos. p95/p99 no reciben ese umbral inicialmente.

## Regeneración offline

```text
python scripts/bench_publish.py
python scripts/bench_publish.py --check
```

inputs.json fija las nueve fuentes y la ejecución principal; no selecciona repeticiones favorables. evidence/ conserva JSON crudo comprimido y hashes, sin depender de la caducidad de artefactos GitHub.
Cada .json.gz contiene un objeto files con las rutas originales (run-XX/criterion/... y run-XX/latency/...). El compresor usa cabecera sin fecha ni nombre de máquina. Los hashes verifican integridad, no certificación ni firma de autoría.
El comando verifica hashes, procedencia, fuentes de fixtures y ambos resúmenes contra las muestras antes de generar esta publicación. No necesita red ni volver a medir.
Comprueba cada mediana Criterion como mediana de tiempos/iteraciones y recalcula los percentiles individuales con nearest-rank. Los intervalos bootstrap originales Criterion se conservan como evidencia; no se vuelven a simular ni se usan como intervalos de fuel.
[publication.json](publication.json) contiene todas las filas con procedencia completa; [cases.md](cases.md) detalla operación, entradas, salidas esperadas y API; [history.md](history.md) conserva grupos históricos independientes.

## Estado de las mediciones principales

| OS | Criterion (14 filas) | p50 individual (9 filas) | Filas con avisos |
|---|---|---|---|
| Linux | 11 aceptable; 3 inestable | 5 aceptable; 4 inestable | 0 |
| Windows | 8 aceptable; 6 inestable | 5 aceptable; 4 inestable | 9 |
| macOS | 1 aceptable; 13 inestable | 6 aceptable; 3 inestable | 0 |

## Linux — 39449438474-linux

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474) · [Artefacto original](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) · [Evidencia local](evidence/39449438474/linux.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `3cc3f9f846daa7d1201df6d5be13713a028c0a06` |
| Captura UTC | 2026-10-20T15:34:42.211104+00:00 (ficha de sesión, no timestamp de cada muestra) |
| OS / arquitectura | linux / x86_64 |
| CPU / lógicas disponibles | AMD EPYC 9V74 80-Core Processor / 4 |
| Rust | rustc 1.85.0 (4d91de4e4 2025-02-17) |
| Cargo | cargo 1.85.0 (d73d2caf9 2024-12-31) |
| Wasmtime / perfil | 29.0.1 / release (criterion) |
| Imagen | ubuntu24 / 20261012.334.1 |

### Criterion — estimacion_agregada

Medianas en µs; N cuenta muestras agrupadas Criterion, no invocaciones individuales.

| Benchmark / fixture | R1 µs | R2 µs | R3 µs | Variación | N por R | Estado editorial | Procedencia |
|---|---|---|---|---|---|---|---|
| `engine_construct` / engine | 1.595 | 1.640 | 1.634 | 2.76 % | 100/100/100 | Candidata a revisión | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `load_core_identity` / core_identity | 379.696 | 379.038 | 375.337 | 1.15 % | 100/100/100 | Candidata a revisión | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `load_component_model_point_sum` / cm_point_sum | 723.545 | 723.691 | 723.346 | 0.05 % | 100/100/100 | Candidata a revisión | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `cold_run_core_identity` / core_identity | 387.627 | 386.925 | 389.038 | 0.55 % | 100/100/100 | Candidata a revisión | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `cold_run_component_model_point_sum` / cm_point_sum | 831.431 | 823.000 | 827.812 | 1.02 % | 100/100/100 | Candidata a revisión | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `warm_run_core_identity` / core_identity | 1.095 | 1.160 | 1.096 | 5.90 % | 100/100/100 | Candidata a revisión | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `warm_run_component_model_point_sum` / cm_point_sum | 39.856 | 37.401 | 37.794 | 6.49 % | 100/100/100 | Candidata a revisión | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `cache_hit_run_core_identity` / core_identity | 1.324 | 1.389 | 1.323 | 4.99 % | 100/100/100 | Candidata a revisión | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `cache_hit_run_component_model_point_sum` / cm_point_sum | 42.210 | 37.498 | 38.638 | 12.20 % | 100/100/100 | Excluida: inestable | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `warm_run_component_model_wasi_echo` / cm_wasi_echo | 72.353 | 63.483 | 62.867 | 14.94 % | 100/100/100 | Excluida: inestable | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 2084.054 | 2021.163 | 2090.069 | 3.31 % | 100/100/100 | Candidata a revisión | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `warm_run_component_model_v06_greet` / cm_v06 | 42.450 | 40.466 | 37.171 | 13.05 % | 100/100/100 | Excluida: inestable | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `warm_run_component_model_v06_safe_div` / cm_v06 | 41.379 | 41.133 | 37.466 | 9.51 % | 100/100/100 | Candidata a revisión | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `warm_run_core_identity_fuel` / core_identity | 1.118 | 1.106 | 1.117 | 1.04 % | 100/100/100 | Candidata a revisión | [39449438474-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |

### Invocaciones — percentil_por_invocacion

p50 y rangos p95/p99 en µs. Nearest-rank; solo p50 recibe el umbral < 10 %.
Las colas muestran rangos observados entre repeticiones y pueden variar; no son intervalos de confianza.

| Benchmark | p50 R1 | p50 R2 | p50 R3 | Var. p50 | p95 min–max | p99 min–max | N / warmup | Estado editorial | Avisos / procedencia |
|---|---|---|---|---|---|---|---|---|---|
| `warm_run_core_identity` | 1.111 | 1.101 | 1.122 | 1.89 % | 1.162–1.202 | 1.222–1.352 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `warm_run_component_model_point_sum` | 32.329 | 32.659 | 35.574 | 9.94 % | 43.886–48.963 | 59.359–66.019 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `cache_hit_run_core_identity` | 1.332 | 1.312 | 1.352 | 3.00 % | 1.392–2.063 | 1.522–2.204 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `cache_hit_run_component_model_point_sum` | 33.741 | 33.771 | 37.216 | 10.29 % | 45.308–51.398 | 60.121–69.144 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `warm_run_component_model_wasi_echo` | 61.332 | 56.525 | 56.956 | 8.44 % | 86.139–94.843 | 116.375–221.333 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `warm_run_component_model_v06_greet` | 36.224 | 32.609 | 33.039 | 10.94 % | 44.056–48.693 | 58.507–66.119 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `warm_run_component_model_v06_safe_div` | 36.455 | 32.779 | 33.170 | 11.08 % | 43.686–50.095 | 57.707–67.041 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `warm_run_core_identity_fuel` | 1.122 | 1.242 | 1.152 | 10.42 % | 1.842–2.023 | 2.113–2.253 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |
| `warm_run_core_identity_limits_unmetered` | 1.122 | 1.121 | 1.152 | 2.76 % | 1.202–1.863 | 1.833–2.103 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277822994) |

### Comparación de escenarios por invocación

WASM básico, WASI, fuel e IR v0.6 realizan operaciones distintas. Los costes siguientes son p50 por escenario, no sobrecargas aisladas.

| Familia / operación | Entradas → contrato de salida | API | p50 R1/R2/R3 µs | Estado |
|---|---|---|---|---|
| WASM básico / identity | [42] → [42] | invoke | 1.111/1.101/1.122 | Candidata a revisión |
| Component Model / record / an:test/ops@0.1.0#make-point-sum | [3,4] → {"doubled":14,"sum":7} | invoke | 32.329/32.659/35.574 | Candidata a revisión |
| WASM básico / identity | [42] → [42] | Resolver::load + invoke | 1.332/1.312/1.352 | Candidata a revisión |
| Component Model / record / an:test/ops@0.1.0#make-point-sum | [3,4] → {"doubled":14,"sum":7} | Resolver::load + invoke | 33.741/33.771/37.216 | Excluida: inestable |
| WASI I/O / echo-and-log | {"args":[64,0],"stdin":"hello agent-native"} → {"return":18,"stderr":"","stdout":"hello agent-native"} | invoke_with_wasi | 61.332/56.525/56.956 | Candidata a revisión |
| IR v0.6 / string / greet | ["world"] → "hello, world" | invoke | 36.224/32.609/33.039 | Excluida: inestable |
| IR v0.6 / result / safe-div | [42,6] → {"ok":7} | invoke | 36.455/32.779/33.170 | Excluida: inestable |
| Fuel / identity / identity | [42] → {"fuel_consumed":"Some(unidades deterministas; cantidad no archivada)","values":[42]} | invoke_with_limits(fuel=1000000) | 1.122/1.242/1.152 | Excluida: inestable |
| Control fuel / identity sin límite / identity | [42] → {"fuel_consumed":null,"values":[42]} | invoke_with_limits(unmetered) | 1.122/1.121/1.152 | Candidata a revisión |

### Instrumento

Sin sustracción de overhead. Salto observado = cota empírica superior de resolución intrínseca; mediana vacía = aproximación inferior del coste del instrumento.

| Repetición | Salto observado ns | Mediana vacía ns |
|---|---|---|
| run-01 | 30 | 30 |
| run-02 | 30 | 30 |
| run-03 | 30 | 30 |

Una mediana vacía de 0 ns puede estar por debajo de la resolución: no demuestra coste cero. Las distribuciones y muestras completas antes/después están archivadas.

### Diferencias de fuel

ns por repetición; intervalo = rango de tres diferencias emparejadas de p50. **No es un intervalo de confianza**. Puede incluir deriva temporal; no hay atribución causal.

| Control | R1/R2/R3 ns | Rango ns | Ambos p50 estables | Interpretación |
|---|---|---|---|---|
| warm_run_core_identity | 11/141/30 | {"max":141,"min":11} | no | misma operación; cambian fuel, API y forma de retorno |
| warm_run_core_identity_limits_unmetered | 0/121/0 | {"max":121,"min":0} | no | misma operación y API; cambia fuel |

## Windows — 39449438474-windows

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474) · [Artefacto original](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) · [Evidencia local](evidence/39449438474/windows.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `3cc3f9f846daa7d1201df6d5be13713a028c0a06` |
| Captura UTC | 2026-10-20T15:34:39.071791+00:00 (ficha de sesión, no timestamp de cada muestra) |
| OS / arquitectura | windows / x86_64 |
| CPU / lógicas disponibles | AMD EPYC 7763 64-Core Processor / 4 |
| Rust | rustc 1.85.0 (4d91de4e4 2025-02-17) |
| Cargo | cargo 1.85.0 (d73d2caf9 2024-12-31) |
| Wasmtime / perfil | 29.0.1 / release (criterion) |
| Imagen | win25-vs2026 / 20261007.264.1 |

### Criterion — estimacion_agregada

Medianas en µs; N cuenta muestras agrupadas Criterion, no invocaciones individuales.

| Benchmark / fixture | R1 µs | R2 µs | R3 µs | Variación | N por R | Estado editorial | Procedencia |
|---|---|---|---|---|---|---|---|
| `engine_construct` / engine | 2.863 | 2.620 | 2.646 | 9.18 % | 100/100/100 | Candidata a revisión | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `load_core_identity` / core_identity | 455.428 | 512.367 | 471.065 | 12.09 % | 100/100/100 | Excluida: inestable | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `load_component_model_point_sum` / cm_point_sum | 877.828 | 919.603 | 904.451 | 4.62 % | 100/100/100 | Candidata a revisión | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `cold_run_core_identity` / core_identity | 439.148 | 476.271 | 467.015 | 7.95 % | 100/100/100 | Candidata a revisión | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `cold_run_component_model_point_sum` / cm_point_sum | 916.995 | 917.475 | 969.793 | 5.75 % | 100/100/100 | Candidata a revisión | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `warm_run_core_identity` / core_identity | 1.957 | 1.955 | 1.947 | 0.49 % | 100/100/100 | Candidata a revisión | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `warm_run_component_model_point_sum` / cm_point_sum | 28.385 | 42.721 | 32.272 | 44.42 % | 100/100/100 | Excluida: inestable | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `cache_hit_run_core_identity` / core_identity | 2.138 | 2.395 | 2.410 | 11.39 % | 100/100/100 | Excluida: inestable | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `cache_hit_run_component_model_point_sum` / cm_point_sum | 30.013 | 43.942 | 33.884 | 41.11 % | 100/100/100 | Excluida: inestable | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `warm_run_component_model_wasi_echo` / cm_wasi_echo | 68.625 | 75.378 | 64.823 | 15.38 % | 100/100/100 | Excluida: inestable | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 2448.780 | 2624.889 | 2403.273 | 9.05 % | 100/100/100 | Candidata a revisión | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `warm_run_component_model_v06_greet` / cm_v06 | 25.550 | 27.063 | 24.540 | 9.88 % | 100/100/100 | Candidata a revisión | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `warm_run_component_model_v06_safe_div` / cm_v06 | 25.256 | 26.787 | 24.618 | 8.59 % | 100/100/100 | Candidata a revisión | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `warm_run_core_identity_fuel` / core_identity | 2.134 | 1.962 | 1.931 | 10.37 % | 100/100/100 | Excluida: inestable | [39449438474-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |

### Invocaciones — percentil_por_invocacion

p50 y rangos p95/p99 en µs. Nearest-rank; solo p50 recibe el umbral < 10 %.
Las colas muestran rangos observados entre repeticiones y pueden variar; no son intervalos de confianza.

| Benchmark | p50 R1 | p50 R2 | p50 R3 | Var. p50 | p95 min–max | p99 min–max | N / warmup | Estado editorial | Avisos / procedencia |
|---|---|---|---|---|---|---|---|---|---|
| `warm_run_core_identity` | 2.100 | 2.000 | 2.100 | 4.76 % | 2.500–3.700 | 3.800–4.300 | 10000 / 1000 | Revisar instrumento | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `warm_run_component_model_point_sum` | 26.000 | 26.400 | 27.400 | 5.30 % | 28.500–31.900 | 52.800–65.300 | 10000 / 1000 | Revisar instrumento | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `cache_hit_run_core_identity` | 2.300 | 2.100 | 2.300 | 8.70 % | 2.200–2.600 | 2.300–3.700 | 10000 / 1000 | Revisar instrumento | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `cache_hit_run_component_model_point_sum` | 27.300 | 27.400 | 28.600 | 4.74 % | 46.900–50.900 | 65.400–74.300 | 10000 / 1000 | Revisar instrumento | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `warm_run_component_model_wasi_echo` | 57.300 | 58.200 | 60.300 | 5.15 % | 80.300–96.300 | 108.600–115.300 | 10000 / 1000 | Revisar instrumento | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `warm_run_component_model_v06_greet` | 26.100 | 26.400 | 39.700 | 51.52 % | 28.100–47.900 | 64.600–75.100 | 10000 / 1000 | Excluida: inestable | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `warm_run_component_model_v06_safe_div` | 26.200 | 26.500 | 39.700 | 50.94 % | 31.200–61.500 | 56.400–77.000 | 10000 / 1000 | Excluida: inestable | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `warm_run_core_identity_fuel` | 2.200 | 1.900 | 2.100 | 14.29 % | 2.000–2.500 | 2.100–2.900 | 10000 / 1000 | Excluida: inestable | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |
| `warm_run_core_identity_limits_unmetered` | 2.200 | 1.900 | 2.100 | 14.29 % | 2.000–2.500 | 2.500–4.000 | 10000 / 1000 | Excluida: inestable | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12277678333) |

### Comparación de escenarios por invocación

WASM básico, WASI, fuel e IR v0.6 realizan operaciones distintas. Los costes siguientes son p50 por escenario, no sobrecargas aisladas.

| Familia / operación | Entradas → contrato de salida | API | p50 R1/R2/R3 µs | Estado |
|---|---|---|---|---|
| WASM básico / identity | [42] → [42] | invoke | 2.100/2.000/2.100 | Revisar instrumento |
| Component Model / record / an:test/ops@0.1.0#make-point-sum | [3,4] → {"doubled":14,"sum":7} | invoke | 26.000/26.400/27.400 | Revisar instrumento |
| WASM básico / identity | [42] → [42] | Resolver::load + invoke | 2.300/2.100/2.300 | Revisar instrumento |
| Component Model / record / an:test/ops@0.1.0#make-point-sum | [3,4] → {"doubled":14,"sum":7} | Resolver::load + invoke | 27.300/27.400/28.600 | Revisar instrumento |
| WASI I/O / echo-and-log | {"args":[64,0],"stdin":"hello agent-native"} → {"return":18,"stderr":"","stdout":"hello agent-native"} | invoke_with_wasi | 57.300/58.200/60.300 | Revisar instrumento |
| IR v0.6 / string / greet | ["world"] → "hello, world" | invoke | 26.100/26.400/39.700 | Excluida: inestable |
| IR v0.6 / result / safe-div | [42,6] → {"ok":7} | invoke | 26.200/26.500/39.700 | Excluida: inestable |
| Fuel / identity / identity | [42] → {"fuel_consumed":"Some(unidades deterministas; cantidad no archivada)","values":[42]} | invoke_with_limits(fuel=1000000) | 2.200/1.900/2.100 | Excluida: inestable |
| Control fuel / identity sin límite / identity | [42] → {"fuel_consumed":null,"values":[42]} | invoke_with_limits(unmetered) | 2.200/1.900/2.100 | Excluida: inestable |

### Instrumento

Sin sustracción de overhead. Salto observado = cota empírica superior de resolución intrínseca; mediana vacía = aproximación inferior del coste del instrumento.

| Repetición | Salto observado ns | Mediana vacía ns |
|---|---|---|
| run-01 | 100 | 0 |
| run-02 | 100 | 0 |
| run-03 | 100 | 0 |

Una mediana vacía de 0 ns puede estar por debajo de la resolución: no demuestra coste cero. Las distribuciones y muestras completas antes/después están archivadas.

### Diferencias de fuel

ns por repetición; intervalo = rango de tres diferencias emparejadas de p50. **No es un intervalo de confianza**. Puede incluir deriva temporal; no hay atribución causal.

| Control | R1/R2/R3 ns | Rango ns | Ambos p50 estables | Interpretación |
|---|---|---|---|---|
| warm_run_core_identity | 100/-100/0 | {"max":100,"min":-100} | no | misma operación; cambian fuel, API y forma de retorno |
| warm_run_core_identity_limits_unmetered | 0/0/0 | {"max":0,"min":0} | no | misma operación y API; cambia fuel |

## macOS — 39449438474-macos

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474) · [Artefacto original](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) · [Evidencia local](evidence/39449438474/macos.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `3cc3f9f846daa7d1201df6d5be13713a028c0a06` |
| Captura UTC | 2026-10-20T15:33:57.943222+00:00 (ficha de sesión, no timestamp de cada muestra) |
| OS / arquitectura | macos / aarch64 |
| CPU / lógicas disponibles | Apple M1 (Virtual) / 3 |
| Rust | rustc 1.85.0 (4d91de4e4 2025-02-17) |
| Cargo | cargo 1.85.0 (d73d2caf9 2024-12-31) |
| Wasmtime / perfil | 29.0.1 / release (criterion) |
| Imagen | macos26 / 20261013.0424.1 |

### Criterion — estimacion_agregada

Medianas en µs; N cuenta muestras agrupadas Criterion, no invocaciones individuales.

| Benchmark / fixture | R1 µs | R2 µs | R3 µs | Variación | N por R | Estado editorial | Procedencia |
|---|---|---|---|---|---|---|---|
| `engine_construct` / engine | 1.613 | 1.915 | 1.486 | 26.57 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `load_core_identity` / core_identity | 262.429 | 229.690 | 286.002 | 21.46 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `load_component_model_point_sum` / cm_point_sum | 405.031 | 456.438 | 345.425 | 27.41 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `cold_run_core_identity` / core_identity | 215.269 | 215.229 | 152.683 | 29.08 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `cold_run_component_model_point_sum` / cm_point_sum | 476.543 | 469.408 | 416.965 | 12.69 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `warm_run_core_identity` / core_identity | 1.209 | 1.152 | 1.341 | 15.68 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `warm_run_component_model_point_sum` / cm_point_sum | 13.429 | 12.897 | 15.904 | 22.39 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `cache_hit_run_core_identity` / core_identity | 1.385 | 1.461 | 1.333 | 9.25 % | 100/100/100 | Candidata a revisión | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `cache_hit_run_component_model_point_sum` / cm_point_sum | 16.357 | 18.667 | 15.796 | 17.56 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `warm_run_component_model_wasi_echo` / cm_wasi_echo | 34.276 | 27.127 | 27.667 | 25.84 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 1558.083 | 1612.065 | 1087.773 | 33.65 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `warm_run_component_model_v06_greet` / cm_v06 | 16.712 | 14.779 | 10.920 | 39.19 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `warm_run_component_model_v06_safe_div` / cm_v06 | 16.445 | 11.595 | 12.955 | 37.43 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `warm_run_core_identity_fuel` / core_identity | 1.120 | 1.130 | 1.256 | 12.05 % | 100/100/100 | Excluida: inestable | [39449438474-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |

### Invocaciones — percentil_por_invocacion

p50 y rangos p95/p99 en µs. Nearest-rank; solo p50 recibe el umbral < 10 %.
Las colas muestran rangos observados entre repeticiones y pueden variar; no son intervalos de confianza.

| Benchmark | p50 R1 | p50 R2 | p50 R3 | Var. p50 | p95 min–max | p99 min–max | N / warmup | Estado editorial | Avisos / procedencia |
|---|---|---|---|---|---|---|---|---|---|
| `warm_run_core_identity` | 1.167 | 1.167 | 1.209 | 3.60 % | 1.542–1.584 | 1.791–3.208 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `warm_run_component_model_point_sum` | 14.000 | 14.208 | 14.375 | 2.64 % | 28.084–50.208 | 59.000–154.208 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `cache_hit_run_core_identity` | 1.459 | 1.333 | 1.417 | 8.89 % | 1.625–3.875 | 1.916–7.625 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `cache_hit_run_component_model_point_sum` | 14.791 | 19.708 | 16.042 | 30.65 % | 31.833–42.417 | 62.542–73.958 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `warm_run_component_model_wasi_echo` | 27.333 | 32.375 | 30.875 | 16.33 % | 46.125–92.666 | 104.042–158.417 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `warm_run_component_model_v06_greet` | 12.333 | 11.958 | 13.375 | 11.49 % | 35.541–42.125 | 64.708–75.667 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `warm_run_component_model_v06_safe_div` | 11.542 | 12.417 | 11.542 | 7.58 % | 22.959–34.334 | 51.750–64.750 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `warm_run_core_identity_fuel` | 1.041 | 1.042 | 1.084 | 4.13 % | 1.292–1.875 | 1.459–3.166 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |
| `warm_run_core_identity_limits_unmetered` | 1.042 | 1.083 | 1.041 | 4.03 % | 1.375–3.042 | 1.500–10.375 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39449438474/artifacts/12278836646) |

### Comparación de escenarios por invocación

WASM básico, WASI, fuel e IR v0.6 realizan operaciones distintas. Los costes siguientes son p50 por escenario, no sobrecargas aisladas.

| Familia / operación | Entradas → contrato de salida | API | p50 R1/R2/R3 µs | Estado |
|---|---|---|---|---|
| WASM básico / identity | [42] → [42] | invoke | 1.167/1.167/1.209 | Candidata a revisión |
| Component Model / record / an:test/ops@0.1.0#make-point-sum | [3,4] → {"doubled":14,"sum":7} | invoke | 14.000/14.208/14.375 | Candidata a revisión |
| WASM básico / identity | [42] → [42] | Resolver::load + invoke | 1.459/1.333/1.417 | Candidata a revisión |
| Component Model / record / an:test/ops@0.1.0#make-point-sum | [3,4] → {"doubled":14,"sum":7} | Resolver::load + invoke | 14.791/19.708/16.042 | Excluida: inestable |
| WASI I/O / echo-and-log | {"args":[64,0],"stdin":"hello agent-native"} → {"return":18,"stderr":"","stdout":"hello agent-native"} | invoke_with_wasi | 27.333/32.375/30.875 | Excluida: inestable |
| IR v0.6 / string / greet | ["world"] → "hello, world" | invoke | 12.333/11.958/13.375 | Excluida: inestable |
| IR v0.6 / result / safe-div | [42,6] → {"ok":7} | invoke | 11.542/12.417/11.542 | Candidata a revisión |
| Fuel / identity / identity | [42] → {"fuel_consumed":"Some(unidades deterministas; cantidad no archivada)","values":[42]} | invoke_with_limits(fuel=1000000) | 1.041/1.042/1.084 | Candidata a revisión |
| Control fuel / identity sin límite / identity | [42] → {"fuel_consumed":null,"values":[42]} | invoke_with_limits(unmetered) | 1.042/1.083/1.041 | Candidata a revisión |

### Instrumento

Sin sustracción de overhead. Salto observado = cota empírica superior de resolución intrínseca; mediana vacía = aproximación inferior del coste del instrumento.

| Repetición | Salto observado ns | Mediana vacía ns |
|---|---|---|
| run-01 | 41 | 42 |
| run-02 | 41 | 41 |
| run-03 | 41 | 41 |

Una mediana vacía de 0 ns puede estar por debajo de la resolución: no demuestra coste cero. Las distribuciones y muestras completas antes/después están archivadas.

### Diferencias de fuel

ns por repetición; intervalo = rango de tres diferencias emparejadas de p50. **No es un intervalo de confianza**. Puede incluir deriva temporal; no hay atribución causal.

| Control | R1/R2/R3 ns | Rango ns | Ambos p50 estables | Interpretación |
|---|---|---|---|---|
| warm_run_core_identity | -126/-125/-125 | {"max":-125,"min":-126} | sí | misma operación; cambian fuel, API y forma de retorno |
| warm_run_core_identity_limits_unmetered | -1/-41/43 | {"max":43,"min":-41} | sí | misma operación y API; cambia fuel |

## Referencia para INIT-08-03

Esta publicación permite localizar etapas y escenarios candidatos, pero no decide todavía caché o warm pools ni fija una mejora Z%.
Antes de optimizar: revisar filas concretas aceptables y su instrumento, estabilizar mediciones relevantes (especialmente macOS), contrastar controles de igual operación/API y medir memoria/aislamiento donde corresponda.
Cold-run vs warm-run cambia Engine/carga; no demuestra por sí solo el beneficio marginal de un warm pool. El Resolver existente ya cachea artefactos, no instancias.
La certificación y el cierre de AN-055 / INIT-08-02 requieren revisión humana explícita. Las cifras Wasmtime 27/--quick de AN-017 se conservan como historial y no son la referencia para nuevas decisiones.
