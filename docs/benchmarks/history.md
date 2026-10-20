# Ejecuciones históricas independientes

Datos técnicos sin certificar. Cada bloque conserva tres repeticiones propias; no se agrupan seis o nueve medianas.
Mismo modelo de CPU no demuestra misma máquina física. Solo el último workflow tiene invocaciones individuales.

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

## Diagnóstico macOS

- 39441524333-macos: 14 inestable; R3 menor que R1 en 14/14 medianas.
- 39446048498-macos: 1 aceptable; 13 inestable; R3 menor que R1 en 14/14 medianas.
- 39449438474-macos: 1 aceptable; 13 inestable; R3 menor que R1 en 10/14 medianas.

La deriva se repite. Los metadatos no contienen telemetría de carga, energía/frecuencia o scheduling del host: no permiten atribuir la causa a la virtualización.
Mantener el 10 %; medir condiciones del host y contrastar en un runner controlado antes de relajar metodología o certificar macOS.
AN-054 añadió otro instrumento y cambió la secuencia de recolección: no atribuir cambios numéricos a una mejora del runtime.

## Comparabilidad de las dos ejecuciones AN-053

Coincidir en modelo de CPU o imagen no identifica una misma máquina física; no se agrupan muestras.

| OS | Anterior | Verificación | Fuentes registradas idénticas | Mismo modelo CPU | Misma imagen |
|---|---|---|---|---|---|
| linux | 39441524333-linux | 39446048498-linux | sí | no | no |
| windows | 39441524333-windows | 39446048498-windows | sí | no | no |
| macos | 39441524333-macos | 39446048498-macos | sí | sí | sí |
