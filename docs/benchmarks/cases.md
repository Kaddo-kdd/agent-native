# Escenarios y fronteras temporales

Las salidas son **esperadas por contrato del fixture**. Los recolectores consumen los resultados, pero no archivaron esos valores devueltos.
Comparar estos escenarios muestra costes de operaciones distintas; no aísla la sobrecarga de WASI o IR v0.6.

| Benchmark | Familia / etapa | Operación | Entradas | Salida esperada (no observada) | API Criterion / individual |
|---|---|---|---|---|---|
| `cache_hit_run_component_model_point_sum` | Component Model / record / cache_hit_run | an:test/ops@0.1.0#make-point-sum | [3,4] | {"doubled":14,"sum":7} | Resolver::load + invoke / Resolver::load + invoke |
| `cache_hit_run_core_identity` | WASM básico / cache_hit_run | identity | [42] | [42] | Resolver::load + invoke / Resolver::load + invoke |
| `cold_run_component_model_point_sum` | Component Model / record / cold_run | an:test/ops@0.1.0#make-point-sum | [3,4] | {"doubled":14,"sum":7} | Engine::new + load_component + invoke / None |
| `cold_run_component_model_wasi_echo` | WASI I/O / cold_run | echo-and-log | {"args":[64,0],"stdin":"hello agent-native"} | {"return":18,"stderr":"","stdout":"hello agent-native"} | Engine::new + load_component_with_policy + invoke_with_wasi / None |
| `cold_run_core_identity` | WASM básico / cold_run | identity | [42] | [42] | Engine::new + load_component + invoke / None |
| `engine_construct` | Engine / engine | construct | null | "Engine construido (sin invocación)" | Engine::new / None |
| `load_component_model_point_sum` | Component Model / record / load | an:test/ops@0.1.0#make-point-sum | null | "Artefacto cargado; función no invocada" | Engine::new + load_component / None |
| `load_core_identity` | WASM básico / load | identity | null | "Artefacto cargado; función no invocada" | Engine::new + load_component / None |
| `warm_run_component_model_point_sum` | Component Model / record / warm_run | an:test/ops@0.1.0#make-point-sum | [3,4] | {"doubled":14,"sum":7} | invoke / invoke |
| `warm_run_component_model_v06_greet` | IR v0.6 / string / warm_run | greet | ["world"] | "hello, world" | invoke / invoke |
| `warm_run_component_model_v06_safe_div` | IR v0.6 / result / warm_run | safe-div | [42,6] | {"ok":7} | invoke / invoke |
| `warm_run_component_model_wasi_echo` | WASI I/O / warm_run | echo-and-log | {"args":[64,0],"stdin":"hello agent-native"} | {"return":18,"stderr":"","stdout":"hello agent-native"} | invoke_with_wasi / invoke_with_wasi |
| `warm_run_core_identity` | WASM básico / warm_run | identity | [42] | [42] | invoke / invoke |
| `warm_run_core_identity_fuel` | Fuel / identity / warm_run | identity | [42] | {"fuel_consumed":"Some(unidades deterministas; cantidad no archivada)","values":[42]} | invoke_with_limits / invoke_with_limits(fuel=1000000) |
| `warm_run_core_identity_limits_unmetered` | Control fuel / identity sin límite / warm_run | identity | [42] | {"fuel_consumed":null,"values":[42]} | invoke_with_limits / invoke_with_limits(unmetered) |

## Qué incluye cada duración

Cold-run: Engine nuevo + carga del artefacto + una invocación. No es arranque de proceso ni cold start de Lambda.
Warm-run: Engine y artefacto precargados; Store nuevo por invocación. Cache-hit: lookup Resolver + invocación, también con Store nuevo.
Criterion mide el closure completo, incluida preparación dentro de él y destrucción del resultado. La compilación IR→WASM queda fuera.
El recolector individual prepara argumentos y buffers WASI antes del reloj y destruye resultados después. Esos límites no son intercambiables con Criterion.
El control identity vía invoke devuelve valores; los controles invoke_with_limits devuelven InvocationReport. El segundo control sin fuel conserva la misma API y forma de retorno que el caso fuel.
Los escenarios se ejecutan en orden fijo. Las diferencias de fuel pueden incluir deriva temporal; un intervalo observado no demuestra causalidad.

Las definiciones están fijadas a hashes de fuentes por OS en cases.json. Diferencias de finales de línea entre checkouts no se confunden con equivalencia de fuentes.
