# Ejecuciones históricas independientes

Datos técnicos sin certificar. Cada bloque conserva tres repeticiones propias; no se agrupan seis o nueve medianas.
Mismo modelo de CPU no demuestra misma máquina física. 15 workflows conservan invocaciones individuales sin agruparlas.

## 39441524333-linux

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) · [Evidencia archivada](evidence/39441524333/linux.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `8661b7da48a47204497b149809f43b44dc7e9042` |
| Captura UTC | 2026-10-20T14:43:26.287686+00:00 (ficha de sesión, no timestamp de cada muestra) |
| OS / arquitectura | linux / x86_64 |
| CPU / lógicas disponibles | AMD EPYC 7763 64-Core Processor / 4 |
| Rust | rustc 1.85.0 (4d91de4e4 2025-02-17) |
| Cargo | cargo 1.85.0 (d73d2caf9 2024-12-31) |
| Wasmtime / perfil | 29.0.1 / release (criterion) |
| Imagen | ubuntu24 / 20261012.334.1 |

### Criterion — estimacion_agregada

Medianas en µs; N cuenta muestras agrupadas Criterion, no invocaciones individuales.

| Benchmark / fixture | R1 µs | R2 µs | R3 µs | Variación | N por R | Estado editorial | Procedencia |
|---|---|---|---|---|---|---|---|
| `engine_construct` / engine | 1.553 | 1.562 | 1.566 | 0.83 % | 100/100/100 | Candidata a revisión | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `load_core_identity` / core_identity | 389.970 | 396.685 | 393.402 | 1.71 % | 100/100/100 | Candidata a revisión | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `load_component_model_point_sum` / cm_point_sum | 761.575 | 763.635 | 763.958 | 0.31 % | 100/100/100 | Candidata a revisión | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `cold_run_core_identity` / core_identity | 398.264 | 404.477 | 405.337 | 1.75 % | 100/100/100 | Candidata a revisión | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `cold_run_component_model_point_sum` / cm_point_sum | 845.170 | 881.714 | 866.345 | 4.22 % | 100/100/100 | Candidata a revisión | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `warm_run_core_identity` / core_identity | 1.076 | 1.075 | 1.107 | 2.95 % | 100/100/100 | Candidata a revisión | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `warm_run_component_model_point_sum` / cm_point_sum | 38.549 | 34.263 | 22.952 | 45.52 % | 100/100/100 | Excluida: inestable | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `cache_hit_run_core_identity` / core_identity | 1.256 | 1.246 | 1.260 | 1.15 % | 100/100/100 | Candidata a revisión | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `cache_hit_run_component_model_point_sum` / cm_point_sum | 41.291 | 35.187 | 24.268 | 48.38 % | 100/100/100 | Excluida: inestable | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `warm_run_component_model_wasi_echo` / cm_wasi_echo | 58.792 | 67.832 | 68.976 | 15.01 % | 100/100/100 | Excluida: inestable | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 2203.997 | 2119.259 | 2117.563 | 4.08 % | 100/100/100 | Candidata a revisión | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `warm_run_component_model_v06_greet` / cm_v06 | 28.225 | 33.795 | 34.136 | 17.49 % | 100/100/100 | Excluida: inestable | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `warm_run_component_model_v06_safe_div` / cm_v06 | 27.827 | 33.751 | 34.285 | 19.13 % | 100/100/100 | Excluida: inestable | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |
| `warm_run_core_identity_fuel` / core_identity | 1.095 | 1.089 | 1.087 | 0.65 % | 100/100/100 | Candidata a revisión | [39441524333-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12274333600) |

Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.

## 39441524333-macos

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) · [Evidencia archivada](evidence/39441524333/macos.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `8661b7da48a47204497b149809f43b44dc7e9042` |
| Captura UTC | 2026-10-20T14:43:56.716066+00:00 (ficha de sesión, no timestamp de cada muestra) |
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
| `engine_construct` / engine | 1.937 | 1.593 | 1.340 | 37.48 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `load_core_identity` / core_identity | 244.468 | 187.037 | 159.323 | 45.52 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `load_component_model_point_sum` / cm_point_sum | 584.127 | 401.715 | 322.773 | 65.06 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `cold_run_core_identity` / core_identity | 265.067 | 194.031 | 162.387 | 52.92 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `cold_run_component_model_point_sum` / cm_point_sum | 495.975 | 368.013 | 398.992 | 32.07 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `warm_run_core_identity` / core_identity | 1.282 | 1.158 | 1.076 | 17.78 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `warm_run_component_model_point_sum` / cm_point_sum | 15.996 | 19.890 | 11.909 | 49.90 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `cache_hit_run_core_identity` / core_identity | 1.825 | 1.738 | 1.507 | 18.28 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `cache_hit_run_component_model_point_sum` / cm_point_sum | 21.610 | 15.453 | 16.321 | 37.73 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `warm_run_component_model_wasi_echo` / cm_wasi_echo | 36.994 | 30.763 | 27.563 | 30.66 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 1753.925 | 1095.412 | 1195.256 | 55.09 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `warm_run_component_model_v06_greet` / cm_v06 | 14.308 | 11.876 | 12.473 | 19.50 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `warm_run_component_model_v06_safe_div` / cm_v06 | 16.662 | 12.400 | 12.959 | 32.89 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |
| `warm_run_core_identity_fuel` / core_identity | 1.182 | 0.995 | 1.085 | 17.23 % | 100/100/100 | Excluida: inestable | [39441524333-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275481805) |

Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.

## 39441524333-windows

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) · [Evidencia archivada](evidence/39441524333/windows.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `8661b7da48a47204497b149809f43b44dc7e9042` |
| Captura UTC | 2026-10-20T14:44:09.979613+00:00 (ficha de sesión, no timestamp de cada muestra) |
| OS / arquitectura | windows / x86_64 |
| CPU / lógicas disponibles | AMD EPYC 9V45 96-Core Processor / 4 |
| Rust | rustc 1.85.0 (4d91de4e4 2025-02-17) |
| Cargo | cargo 1.85.0 (d73d2caf9 2024-12-31) |
| Wasmtime / perfil | 29.0.1 / release (criterion) |
| Imagen | win25-vs2026 / 20261018.275.1 |

### Criterion — estimacion_agregada

Medianas en µs; N cuenta muestras agrupadas Criterion, no invocaciones individuales.

| Benchmark / fixture | R1 µs | R2 µs | R3 µs | Variación | N por R | Estado editorial | Procedencia |
|---|---|---|---|---|---|---|---|
| `engine_construct` / engine | 1.610 | 1.621 | 1.570 | 3.19 % | 100/100/100 | Candidata a revisión | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `load_core_identity` / core_identity | 219.791 | 246.758 | 223.753 | 12.05 % | 100/100/100 | Excluida: inestable | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `load_component_model_point_sum` / cm_point_sum | 479.168 | 504.036 | 481.058 | 5.17 % | 100/100/100 | Candidata a revisión | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `cold_run_core_identity` / core_identity | 213.880 | 232.974 | 234.355 | 8.79 % | 100/100/100 | Candidata a revisión | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `cold_run_component_model_point_sum` / cm_point_sum | 518.051 | 533.306 | 539.890 | 4.09 % | 100/100/100 | Candidata a revisión | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `warm_run_core_identity` / core_identity | 2.085 | 2.080 | 2.116 | 1.76 % | 100/100/100 | Candidata a revisión | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `warm_run_component_model_point_sum` / cm_point_sum | 16.728 | 21.179 | 19.313 | 23.05 % | 100/100/100 | Excluida: inestable | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `cache_hit_run_core_identity` / core_identity | 2.228 | 2.208 | 2.220 | 0.89 % | 100/100/100 | Candidata a revisión | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `cache_hit_run_component_model_point_sum` / cm_point_sum | 17.288 | 21.707 | 20.094 | 21.99 % | 100/100/100 | Excluida: inestable | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `warm_run_component_model_wasi_echo` / cm_wasi_echo | 34.127 | 36.657 | 32.995 | 10.73 % | 100/100/100 | Excluida: inestable | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 1333.924 | 1314.646 | 1295.305 | 2.94 % | 100/100/100 | Candidata a revisión | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `warm_run_component_model_v06_greet` / cm_v06 | 16.449 | 15.914 | 16.661 | 4.54 % | 100/100/100 | Candidata a revisión | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `warm_run_component_model_v06_safe_div` / cm_v06 | 16.931 | 15.557 | 16.284 | 8.44 % | 100/100/100 | Candidata a revisión | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |
| `warm_run_core_identity_fuel` / core_identity | 2.330 | 2.068 | 2.313 | 11.32 % | 100/100/100 | Excluida: inestable | [39441524333-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39441524333/artifacts/12275123853) |

Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.

## 39446048498-linux

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) · [Evidencia archivada](evidence/39446048498/linux.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `48f7518a24a877410195479532e864acd86d4259` |
| Captura UTC | 2026-10-20T15:12:33.891697+00:00 (ficha de sesión, no timestamp de cada muestra) |
| OS / arquitectura | linux / x86_64 |
| CPU / lógicas disponibles | AMD EPYC 9V45 96-Core Processor / 4 |
| Rust | rustc 1.85.0 (4d91de4e4 2025-02-17) |
| Cargo | cargo 1.85.0 (d73d2caf9 2024-12-31) |
| Wasmtime / perfil | 29.0.1 / release (criterion) |
| Imagen | ubuntu24 / 20261018.339.1 |

### Criterion — estimacion_agregada

Medianas en µs; N cuenta muestras agrupadas Criterion, no invocaciones individuales.

| Benchmark / fixture | R1 µs | R2 µs | R3 µs | Variación | N por R | Estado editorial | Procedencia |
|---|---|---|---|---|---|---|---|
| `engine_construct` / engine | 0.997 | 0.946 | 0.970 | 5.28 % | 100/100/100 | Candidata a revisión | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `load_core_identity` / core_identity | 216.322 | 211.247 | 211.036 | 2.50 % | 100/100/100 | Candidata a revisión | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `load_component_model_point_sum` / cm_point_sum | 458.655 | 458.859 | 469.085 | 2.27 % | 100/100/100 | Candidata a revisión | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `cold_run_core_identity` / core_identity | 213.886 | 213.832 | 223.869 | 4.69 % | 100/100/100 | Candidata a revisión | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `cold_run_component_model_point_sum` / cm_point_sum | 530.296 | 523.467 | 553.172 | 5.60 % | 100/100/100 | Candidata a revisión | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `warm_run_core_identity` / core_identity | 0.759 | 0.747 | 0.776 | 3.81 % | 100/100/100 | Candidata a revisión | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `warm_run_component_model_point_sum` / cm_point_sum | 17.238 | 19.908 | 20.246 | 15.11 % | 100/100/100 | Excluida: inestable | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `cache_hit_run_core_identity` / core_identity | 0.868 | 0.883 | 0.912 | 4.93 % | 100/100/100 | Candidata a revisión | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `cache_hit_run_component_model_point_sum` / cm_point_sum | 17.732 | 20.481 | 20.970 | 15.81 % | 100/100/100 | Excluida: inestable | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `warm_run_component_model_wasi_echo` / cm_wasi_echo | 33.369 | 33.569 | 35.095 | 5.14 % | 100/100/100 | Candidata a revisión | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 1255.829 | 1229.847 | 1293.926 | 5.10 % | 100/100/100 | Candidata a revisión | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `warm_run_component_model_v06_greet` / cm_v06 | 19.295 | 17.200 | 19.896 | 13.98 % | 100/100/100 | Excluida: inestable | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `warm_run_component_model_v06_safe_div` / cm_v06 | 19.782 | 17.640 | 20.160 | 12.74 % | 100/100/100 | Excluida: inestable | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |
| `warm_run_core_identity_fuel` / core_identity | 0.786 | 0.781 | 0.781 | 0.65 % | 100/100/100 | Candidata a revisión | [39446048498-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275828777) |

Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.

## 39446048498-macos

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) · [Evidencia archivada](evidence/39446048498/macos.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `48f7518a24a877410195479532e864acd86d4259` |
| Captura UTC | 2026-10-20T15:12:31.451410+00:00 (ficha de sesión, no timestamp de cada muestra) |
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
| `engine_construct` / engine | 1.657 | 1.394 | 1.296 | 25.94 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `load_core_identity` / core_identity | 207.540 | 164.430 | 144.706 | 38.21 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `load_component_model_point_sum` / cm_point_sum | 381.992 | 334.525 | 300.110 | 24.48 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `cold_run_core_identity` / core_identity | 186.769 | 165.932 | 147.305 | 23.78 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `cold_run_component_model_point_sum` / cm_point_sum | 387.789 | 338.925 | 318.459 | 20.46 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `warm_run_core_identity` / core_identity | 1.122 | 1.098 | 1.025 | 8.83 % | 100/100/100 | Candidata a revisión | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `warm_run_component_model_point_sum` / cm_point_sum | 12.533 | 11.899 | 11.085 | 12.17 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `cache_hit_run_core_identity` / core_identity | 1.494 | 1.426 | 1.324 | 11.91 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `cache_hit_run_component_model_point_sum` / cm_point_sum | 15.791 | 16.517 | 14.123 | 15.16 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `warm_run_component_model_wasi_echo` / cm_wasi_echo | 28.938 | 26.214 | 26.026 | 11.11 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 1270.781 | 1003.118 | 1012.331 | 26.44 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `warm_run_component_model_v06_greet` / cm_v06 | 12.048 | 11.016 | 10.909 | 10.34 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `warm_run_component_model_v06_safe_div` / cm_v06 | 12.182 | 11.103 | 10.881 | 11.71 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |
| `warm_run_core_identity_fuel` / core_identity | 1.076 | 0.942 | 0.965 | 13.90 % | 100/100/100 | Excluida: inestable | [39446048498-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12276563200) |

Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.

## 39446048498-windows

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) · [Evidencia archivada](evidence/39446048498/windows.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `48f7518a24a877410195479532e864acd86d4259` |
| Captura UTC | 2026-10-20T15:14:55.463529+00:00 (ficha de sesión, no timestamp de cada muestra) |
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
| `engine_construct` / engine | 2.799 | 2.596 | 2.585 | 8.24 % | 100/100/100 | Candidata a revisión | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `load_core_identity` / core_identity | 477.443 | 443.855 | 459.268 | 7.31 % | 100/100/100 | Candidata a revisión | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `load_component_model_point_sum` / cm_point_sum | 921.311 | 867.728 | 875.503 | 6.12 % | 100/100/100 | Candidata a revisión | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `cold_run_core_identity` / core_identity | 446.549 | 450.703 | 434.909 | 3.54 % | 100/100/100 | Candidata a revisión | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `cold_run_component_model_point_sum` / cm_point_sum | 936.281 | 931.152 | 920.054 | 1.74 % | 100/100/100 | Candidata a revisión | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `warm_run_core_identity` / core_identity | 2.103 | 1.968 | 2.006 | 6.74 % | 100/100/100 | Candidata a revisión | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `warm_run_component_model_point_sum` / cm_point_sum | 36.142 | 32.972 | 31.853 | 13.01 % | 100/100/100 | Excluida: inestable | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `cache_hit_run_core_identity` / core_identity | 2.538 | 2.141 | 2.149 | 18.46 % | 100/100/100 | Excluida: inestable | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `cache_hit_run_component_model_point_sum` / cm_point_sum | 36.951 | 33.861 | 32.901 | 11.96 % | 100/100/100 | Excluida: inestable | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `warm_run_component_model_wasi_echo` / cm_wasi_echo | 70.802 | 72.290 | 68.276 | 5.67 % | 100/100/100 | Candidata a revisión | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 2409.450 | 2421.262 | 2402.670 | 0.77 % | 100/100/100 | Candidata a revisión | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `warm_run_component_model_v06_greet` / cm_v06 | 24.744 | 27.121 | 30.498 | 21.21 % | 100/100/100 | Excluida: inestable | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `warm_run_component_model_v06_safe_div` / cm_v06 | 24.611 | 27.362 | 26.641 | 10.33 % | 100/100/100 | Excluida: inestable | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |
| `warm_run_core_identity_fuel` / core_identity | 1.974 | 2.014 | 1.989 | 2.00 % | 100/100/100 | Candidata a revisión | [39446048498-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39446048498/artifacts/12275874311) |

Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.

## 39534970668-linux

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) · [Evidencia archivada](evidence/39534970668/linux.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `679490606a398e807c19dd6dfa2dec794de6311d` |
| Captura UTC | 2026-10-21T02:49:07.694755+00:00 (ficha de sesión, no timestamp de cada muestra) |
| OS / arquitectura | linux / x86_64 |
| CPU / lógicas disponibles | Intel(R) Xeon(R) 6973P-C / 4 |
| Rust | rustc 1.85.0 (4d91de4e4 2025-02-17) |
| Cargo | cargo 1.85.0 (d73d2caf9 2024-12-31) |
| Wasmtime / perfil | 29.0.1 / release (criterion) |
| Imagen | ubuntu24 / 20261018.339.1 |

### Criterion — estimacion_agregada

Medianas en µs; N cuenta muestras agrupadas Criterion, no invocaciones individuales.

| Benchmark / fixture | R1 µs | R2 µs | R3 µs | Variación | N por R | Estado editorial | Procedencia |
|---|---|---|---|---|---|---|---|
| `engine_construct` / engine | 0.990 | 1.103 | 0.995 | 11.32 % | 100/100/100 | Excluida: inestable | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `load_core_identity` / core_identity | 242.580 | 276.090 | 243.700 | 13.75 % | 100/100/100 | Excluida: inestable | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `load_component_model_point_sum` / cm_point_sum | 537.116 | 539.248 | 541.873 | 0.88 % | 100/100/100 | Candidata a revisión | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `cold_run_core_identity` / core_identity | 250.718 | 252.323 | 256.477 | 2.28 % | 100/100/100 | Candidata a revisión | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `cold_run_component_model_point_sum` / cm_point_sum | 690.243 | 612.149 | 625.265 | 12.49 % | 100/100/100 | Excluida: inestable | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `warm_run_core_identity` / core_identity | 0.751 | 0.761 | 0.759 | 1.38 % | 100/100/100 | Candidata a revisión | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `warm_run_component_model_point_sum` / cm_point_sum | 14.392 | 14.184 | 13.224 | 8.24 % | 100/100/100 | Candidata a revisión | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `cache_hit_run_core_identity` / core_identity | 0.923 | 0.918 | 0.913 | 1.15 % | 100/100/100 | Candidata a revisión | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `cache_hit_run_component_model_point_sum` / cm_point_sum | 14.566 | 14.694 | 14.102 | 4.06 % | 100/100/100 | Candidata a revisión | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `warm_run_component_model_wasi_echo` / cm_wasi_echo | 31.290 | 30.999 | 32.250 | 4.00 % | 100/100/100 | Candidata a revisión | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 1503.982 | 1698.586 | 1620.958 | 12.01 % | 100/100/100 | Excluida: inestable | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `warm_run_component_model_v06_greet` / cm_v06 | 13.875 | 13.085 | 13.825 | 5.71 % | 100/100/100 | Candidata a revisión | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `warm_run_component_model_v06_safe_div` / cm_v06 | 13.426 | 12.894 | 13.918 | 7.63 % | 100/100/100 | Candidata a revisión | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `warm_run_core_identity_fuel` / core_identity | 0.788 | 0.784 | 0.774 | 1.81 % | 100/100/100 | Candidata a revisión | [39534970668-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |

### Invocaciones — percentil_por_invocacion

p50 y rangos p95/p99 en µs. Nearest-rank; solo p50 recibe el umbral < 10 %.
Las colas muestran rangos observados entre repeticiones y pueden variar; no son intervalos de confianza.

| Benchmark | p50 R1 | p50 R2 | p50 R3 | Var. p50 | p95 min–max | p99 min–max | N / warmup | Estado editorial | Avisos / procedencia |
|---|---|---|---|---|---|---|---|---|---|
| `warm_run_core_identity` | 0.727 | 0.717 | 0.832 | 15.82 % | 0.812–0.931 | 0.876–0.985 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `warm_run_component_model_point_sum` | 13.078 | 14.267 | 9.826 | 33.96 % | 15.613–20.007 | 34.064–36.981 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `cache_hit_run_core_identity` | 0.869 | 0.877 | 0.852 | 2.88 % | 0.939–1.250 | 1.011–1.347 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `cache_hit_run_component_model_point_sum` | 15.451 | 15.529 | 12.042 | 22.57 % | 17.923–20.481 | 26.882–31.774 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `warm_run_component_model_wasi_echo` | 30.404 | 30.339 | 29.680 | 2.39 % | 40.214–43.155 | 55.791–59.108 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `warm_run_component_model_v06_greet` | 13.433 | 12.745 | 13.502 | 5.64 % | 16.685–17.496 | 26.399–27.085 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `warm_run_component_model_v06_safe_div` | 13.642 | 12.786 | 12.834 | 6.67 % | 16.795–17.707 | 25.812–28.172 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `warm_run_core_identity_fuel` | 0.789 | 0.780 | 0.776 | 1.67 % | 0.849–1.156 | 0.885–1.267 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |
| `warm_run_core_identity_limits_unmetered` | 0.770 | 0.783 | 0.771 | 1.69 % | 0.840–1.152 | 0.880–1.248 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317850238) |

## 39534970668-macos

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) · [Evidencia archivada](evidence/39534970668/macos.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `679490606a398e807c19dd6dfa2dec794de6311d` |
| Captura UTC | 2026-10-21T02:48:32.986271+00:00 (ficha de sesión, no timestamp de cada muestra) |
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
| `engine_construct` / engine | 2.930 | 1.355 | 1.353 | 116.32 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `load_core_identity` / core_identity | 327.672 | 160.216 | 218.497 | 76.64 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `load_component_model_point_sum` / cm_point_sum | 608.682 | 314.677 | 557.028 | 52.78 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `cold_run_core_identity` / core_identity | 207.341 | 163.399 | 307.471 | 69.49 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `cold_run_component_model_point_sum` / cm_point_sum | 399.738 | 314.775 | 446.545 | 32.96 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `warm_run_core_identity` / core_identity | 1.493 | 0.960 | 1.142 | 46.68 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `warm_run_component_model_point_sum` / cm_point_sum | 11.615 | 11.933 | 12.151 | 4.49 % | 100/100/100 | Candidata a revisión | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `cache_hit_run_core_identity` / core_identity | 1.489 | 1.265 | 1.407 | 15.94 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `cache_hit_run_component_model_point_sum` / cm_point_sum | 16.360 | 14.494 | 14.708 | 12.69 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `warm_run_component_model_wasi_echo` / cm_wasi_echo | 43.178 | 26.781 | 28.239 | 58.06 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 1383.832 | 1035.316 | 1173.236 | 29.71 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `warm_run_component_model_v06_greet` / cm_v06 | 12.914 | 11.342 | 14.683 | 25.87 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `warm_run_component_model_v06_safe_div` / cm_v06 | 14.772 | 11.356 | 17.300 | 40.23 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `warm_run_core_identity_fuel` / core_identity | 1.049 | 1.115 | 1.687 | 57.21 % | 100/100/100 | Excluida: inestable | [39534970668-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |

### Invocaciones — percentil_por_invocacion

p50 y rangos p95/p99 en µs. Nearest-rank; solo p50 recibe el umbral < 10 %.
Las colas muestran rangos observados entre repeticiones y pueden variar; no son intervalos de confianza.

| Benchmark | p50 R1 | p50 R2 | p50 R3 | Var. p50 | p95 min–max | p99 min–max | N / warmup | Estado editorial | Avisos / procedencia |
|---|---|---|---|---|---|---|---|---|---|
| `warm_run_core_identity` | 1.334 | 1.459 | 1.542 | 14.26 % | 1.875–3.792 | 3.542–6.750 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `warm_run_component_model_point_sum` | 25.459 | 17.208 | 15.542 | 57.63 % | 50.458–79.542 | 108.625–176.000 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `cache_hit_run_core_identity` | 1.417 | 1.375 | 1.833 | 32.32 % | 1.750–4.000 | 1.917–8.458 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `cache_hit_run_component_model_point_sum` | 24.416 | 23.375 | 18.125 | 26.91 % | 57.750–84.875 | 124.042–200.875 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `warm_run_component_model_wasi_echo` | 33.541 | 32.292 | 37.500 | 15.53 % | 86.292–122.333 | 145.792–243.292 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `warm_run_component_model_v06_greet` | 13.750 | 11.959 | 16.666 | 34.23 % | 19.542–44.291 | 39.625–86.125 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `warm_run_component_model_v06_safe_div` | 13.292 | 11.792 | 16.791 | 37.61 % | 19.625–53.167 | 28.792–144.792 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `warm_run_core_identity_fuel` | 1.041 | 1.042 | 1.125 | 8.06 % | 1.375–3.333 | 1.500–9.209 | 10000 / 1000 | Candidata a revisión |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |
| `warm_run_core_identity_limits_unmetered` | 1.083 | 1.000 | 1.375 | 34.63 % | 1.291–3.083 | 1.375–6.583 | 10000 / 1000 | Excluida: inestable |  [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12317715346) |

## 39534970668-windows

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) · [Evidencia archivada](evidence/39534970668/windows.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `679490606a398e807c19dd6dfa2dec794de6311d` |
| Captura UTC | 2026-10-21T02:49:10.784343+00:00 (ficha de sesión, no timestamp de cada muestra) |
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
| `engine_construct` / engine | 2.664 | 2.619 | 2.829 | 7.90 % | 100/100/100 | Candidata a revisión | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `load_core_identity` / core_identity | 463.651 | 464.127 | 458.109 | 1.30 % | 100/100/100 | Candidata a revisión | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `load_component_model_point_sum` / cm_point_sum | 875.684 | 865.287 | 877.993 | 1.45 % | 100/100/100 | Candidata a revisión | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `cold_run_core_identity` / core_identity | 459.354 | 448.309 | 454.364 | 2.43 % | 100/100/100 | Candidata a revisión | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `cold_run_component_model_point_sum` / cm_point_sum | 916.032 | 928.809 | 945.714 | 3.20 % | 100/100/100 | Candidata a revisión | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `warm_run_core_identity` / core_identity | 2.015 | 1.963 | 2.161 | 9.80 % | 100/100/100 | Candidata a revisión | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `warm_run_component_model_point_sum` / cm_point_sum | 28.155 | 34.174 | 37.163 | 26.36 % | 100/100/100 | Excluida: inestable | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `cache_hit_run_core_identity` / core_identity | 2.241 | 2.202 | 2.401 | 8.92 % | 100/100/100 | Candidata a revisión | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `cache_hit_run_component_model_point_sum` / cm_point_sum | 29.010 | 35.117 | 37.941 | 25.43 % | 100/100/100 | Excluida: inestable | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `warm_run_component_model_wasi_echo` / cm_wasi_echo | 69.669 | 68.080 | 67.368 | 3.38 % | 100/100/100 | Candidata a revisión | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 2348.750 | 2414.210 | 2430.830 | 3.40 % | 100/100/100 | Candidata a revisión | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `warm_run_component_model_v06_greet` / cm_v06 | 26.285 | 24.709 | 25.759 | 6.12 % | 100/100/100 | Candidata a revisión | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `warm_run_component_model_v06_safe_div` / cm_v06 | 26.748 | 25.350 | 26.183 | 5.34 % | 100/100/100 | Candidata a revisión | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `warm_run_core_identity_fuel` / core_identity | 2.159 | 2.153 | 1.941 | 10.14 % | 100/100/100 | Excluida: inestable | [39534970668-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |

### Invocaciones — percentil_por_invocacion

p50 y rangos p95/p99 en µs. Nearest-rank; solo p50 recibe el umbral < 10 %.
Las colas muestran rangos observados entre repeticiones y pueden variar; no son intervalos de confianza.

| Benchmark | p50 R1 | p50 R2 | p50 R3 | Var. p50 | p95 min–max | p99 min–max | N / warmup | Estado editorial | Avisos / procedencia |
|---|---|---|---|---|---|---|---|---|---|
| `warm_run_core_identity` | 1.900 | 2.100 | 2.100 | 9.52 % | 2.100–2.500 | 2.200–4.200 | 10000 / 1000 | Revisar instrumento | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `warm_run_component_model_point_sum` | 36.800 | 26.100 | 33.000 | 32.42 % | 38.200–56.900 | 59.400–75.900 | 10000 / 1000 | Excluida: inestable | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `cache_hit_run_core_identity` | 2.100 | 2.300 | 2.300 | 8.70 % | 2.200–2.600 | 2.800–3.300 | 10000 / 1000 | Revisar instrumento | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `cache_hit_run_component_model_point_sum` | 27.900 | 26.800 | 26.000 | 7.09 % | 32.500–48.100 | 51.400–62.500 | 10000 / 1000 | Revisar instrumento | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `warm_run_component_model_wasi_echo` | 60.500 | 59.900 | 57.800 | 4.51 % | 90.200–95.800 | 107.400–112.200 | 10000 / 1000 | Revisar instrumento | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `warm_run_component_model_v06_greet` | 26.800 | 25.800 | 32.700 | 25.75 % | 31.000–46.500 | 51.000–61.000 | 10000 / 1000 | Excluida: inestable | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `warm_run_component_model_v06_safe_div` | 37.100 | 25.900 | 33.200 | 33.73 % | 30.800–42.800 | 56.000–74.300 | 10000 / 1000 | Excluida: inestable | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `warm_run_core_identity_fuel` | 1.900 | 2.100 | 2.100 | 9.52 % | 2.000–2.500 | 2.100–3.000 | 10000 / 1000 | Revisar instrumento | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |
| `warm_run_core_identity_limits_unmetered` | 1.900 | 2.100 | 2.200 | 14.29 % | 2.000–4.000 | 2.300–4.700 | 10000 / 1000 | Excluida: inestable | mediana de overhead no resuelta por reloj [artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39534970668/artifacts/12318315159) |

## 39549485766-linux

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39549485766) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39549485766/artifacts/12322284282) · [Evidencia archivada](evidence/39549485766/linux-wasi-confirmation.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `96c9179319ee374c84708a1f5c8ecd819a7f19e1` |
| Captura UTC | 2026-10-21T05:27:22.038535+00:00 (ficha de sesión, no timestamp de cada muestra) |
| OS / arquitectura | linux / x86_64 |
| CPU / lógicas disponibles | AMD EPYC 7763 64-Core Processor / 4 |
| Rust | rustc 1.85.0 (4d91de4e4 2025-02-17) |
| Cargo | cargo 1.85.0 (d73d2caf9 2024-12-31) |
| Wasmtime / perfil | 29.0.1 / release (criterion) |
| Imagen | ubuntu24 / 20261012.334.1 |

### Criterion — estimacion_agregada

Medianas en µs; N cuenta muestras agrupadas Criterion, no invocaciones individuales.

| Benchmark / fixture | R1 µs | R2 µs | R3 µs | Variación | N por R | Estado editorial | Procedencia |
|---|---|---|---|---|---|---|---|
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 2212.118 | 2211.003 | 2186.810 | 1.14 % | 100/100/100 | Candidata a revisión | [39549485766-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39549485766/artifacts/12322284282) |

Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.

## 39549485766-macos

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39549485766) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39549485766/artifacts/12323003714) · [Evidencia archivada](evidence/39549485766/macos-wasi-confirmation.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `96c9179319ee374c84708a1f5c8ecd819a7f19e1` |
| Captura UTC | 2026-10-21T05:26:22.355996+00:00 (ficha de sesión, no timestamp de cada muestra) |
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
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 987.910 | 1026.037 | 984.006 | 4.25 % | 100/100/100 | Candidata a revisión | [39549485766-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39549485766/artifacts/12323003714) |

Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.

## 39549485766-windows

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39549485766) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39549485766/artifacts/12323063560) · [Evidencia archivada](evidence/39549485766/windows-wasi-confirmation.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `96c9179319ee374c84708a1f5c8ecd819a7f19e1` |
| Captura UTC | 2026-10-21T05:27:19.959513+00:00 (ficha de sesión, no timestamp de cada muestra) |
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
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 2432.161 | 2658.245 | 2360.538 | 12.24 % | 100/100/100 | Excluida: inestable | [39549485766-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39549485766/artifacts/12323063560) |

Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.

## 39550537942-linux

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39550537942) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39550537942/artifacts/12323940364) · [Evidencia archivada](evidence/39550537942/linux-wasi-confirmation.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `96c9179319ee374c84708a1f5c8ecd819a7f19e1` |
| Captura UTC | 2026-10-21T05:37:26.411940+00:00 (ficha de sesión, no timestamp de cada muestra) |
| OS / arquitectura | linux / x86_64 |
| CPU / lógicas disponibles | AMD EPYC 7763 64-Core Processor / 4 |
| Rust | rustc 1.85.0 (4d91de4e4 2025-02-17) |
| Cargo | cargo 1.85.0 (d73d2caf9 2024-12-31) |
| Wasmtime / perfil | 29.0.1 / release (criterion) |
| Imagen | ubuntu24 / 20261012.334.1 |

### Criterion — estimacion_agregada

Medianas en µs; N cuenta muestras agrupadas Criterion, no invocaciones individuales.

| Benchmark / fixture | R1 µs | R2 µs | R3 µs | Variación | N por R | Estado editorial | Procedencia |
|---|---|---|---|---|---|---|---|
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 2244.833 | 2239.765 | 2279.267 | 1.76 % | 100/100/100 | Candidata a revisión | [39550537942-linux](https://github.com/Kaddo-kdd/agent-native/actions/runs/39550537942/artifacts/12323940364) |

Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.

## 39550537942-macos

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39550537942) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39550537942/artifacts/12324270310) · [Evidencia archivada](evidence/39550537942/macos-wasi-confirmation.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `96c9179319ee374c84708a1f5c8ecd819a7f19e1` |
| Captura UTC | 2026-10-21T05:37:38.701960+00:00 (ficha de sesión, no timestamp de cada muestra) |
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
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 1258.578 | 1082.326 | 1829.547 | 59.37 % | 100/100/100 | Excluida: inestable | [39550537942-macos](https://github.com/Kaddo-kdd/agent-native/actions/runs/39550537942/artifacts/12324270310) |

Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.

## 39550537942-windows

[Workflow](https://github.com/Kaddo-kdd/agent-native/actions/runs/39550537942) · [Artefacto](https://github.com/Kaddo-kdd/agent-native/actions/runs/39550537942/artifacts/12323208551) · [Evidencia archivada](evidence/39550537942/windows-wasi-confirmation.json.gz)

| Campo | Evidencia |
|---|---|
| Commit medido | `96c9179319ee374c84708a1f5c8ecd819a7f19e1` |
| Captura UTC | 2026-10-21T05:38:42.089000+00:00 (ficha de sesión, no timestamp de cada muestra) |
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
| `cold_run_component_model_wasi_echo` / cm_wasi_echo | 2306.445 | 2365.680 | 2389.963 | 3.53 % | 100/100/100 | Candidata a revisión | [39550537942-windows](https://github.com/Kaddo-kdd/agent-native/actions/runs/39550537942/artifacts/12323208551) |

Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.

## Diagnóstico macOS

- 39441524333-macos: 14 inestable; R3 menor que R1 en 14/14 medianas.
- 39446048498-macos: 1 aceptable; 13 inestable; R3 menor que R1 en 14/14 medianas.
- 39449438474-macos: 1 aceptable; 13 inestable; R3 menor que R1 en 10/14 medianas.
- 39534970668-macos: 1 aceptable; 13 inestable; R3 menor que R1 en 8/14 medianas.
- 39549485766-macos: 1 aceptable; R3 menor que R1 en 1/1 medianas.
- 39550537942-macos: 1 inestable; R3 menor que R1 en 0/1 medianas.

La deriva se repite. Los metadatos no contienen telemetría de carga, energía/frecuencia o scheduling del host: no permiten atribuir la causa a la virtualización.
Mantener el 10 %; medir condiciones del host y contrastar en un runner controlado antes de relajar metodología o certificar macOS.
AN-054 añadió otro instrumento y cambió la secuencia de recolección: no atribuir cambios numéricos a una mejora del runtime.

## Comparabilidad entre sesiones independientes

Coincidir en modelo de CPU o imagen no identifica una misma máquina física; no se agrupan muestras.

| OS | Anterior | Verificación | Fuentes registradas idénticas | Mismo modelo CPU | Misma imagen |
|---|---|---|---|---|---|
| linux | 39441524333-linux | 39446048498-linux | sí | no | no |
| linux | 39441524333-linux | 39534970668-linux | no | no | no |
| windows | 39441524333-windows | 39446048498-windows | sí | no | no |
| windows | 39441524333-windows | 39534970668-windows | no | no | no |
| macos | 39441524333-macos | 39446048498-macos | sí | sí | sí |
| macos | 39441524333-macos | 39534970668-macos | no | sí | sí |
