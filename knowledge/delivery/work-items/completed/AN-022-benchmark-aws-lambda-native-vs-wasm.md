---
id: AN-022
type: spike
status: completed
completed_at: 2026-10-19
title: "AN-022 — Benchmark AWS Lambda: Rust Native vs Agent-Native WASM"
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - examples/aws-lambda-benchmark/**
source_roadmap_initiative: INIT-05-02
cross_initiatives:
  - INIT-08-01
  - INIT-08-02
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Performance
  - Distribution & DX
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Cuantificar de forma empírica, reproducible y verificable el overhead que introduce
  la abstracción de Agent-Native (Wasmtime 27 + Component Model + fresh Store por invocación)
  frente a una implementación nativa pura en Rust sobre AWS Lambda (provided.al2023, x86_64).
  Mide latencia (cold/warm), memoria, tamaño de binario/paquete y costo operativo estimado.
dependencies:
  - AN-021
source_signals:
  - "Baseline técnica: v0.0.4-m3"
  - "Validación empírica para roadmap RM-08 (Performance & Concurrency)"
  - "Insumo técnico para INIT-05-02 (API pública del runtime)"
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-19T00:00:00.000Z
ready_at: '2026-10-19'
---

# AN-022 — Benchmark AWS Lambda: Rust Native vs Agent-Native WASM

**Actor y resultado:**
Un ingeniero de rendimiento o evaluador técnico dispone de un arnés de benchmarking automatizado y reproducible que compara dos implementaciones idénticas en interfaz y comportamiento desplegadas sobre AWS Lambda (`provided.al2023`, `x86_64`):
1. **Lambda A (Rust Native):** Ejecuta la lógica directamente en código Rust nativo compilado, sin Wasmtime ni componentes.
2. **Lambda B (Agent-Native WASM):** Ejecuta el componente `program-005.wasm` mediante `agent-native-runtime` bajo las invariantes de ADR-005 (nuevo `Store` por invocación).

El benchmark cuantifica el costo real de la abstracción y portabilidad de Agent-Native sin alterar el runtime para optimizar artificialmente los resultados.

**Current behavior:**
- AN-021 demostró la viabilidad funcional y portabilidad de `agent-native-runtime` en AWS Lambda con una inicialización fría agregada de 92.21 ms y ejecuciones cálidas de ~1.2 ms (25 MB RAM).
- No existía una medición comparativa contra una línea base nativa en Rust que aislara el overhead específico introducido por Wasmtime y el Component Model frente al costo intrínseco del entorno serverless de AWS Lambda.

**Target behavior:**
- Subdirectorio `examples/aws-lambda-benchmark/` con la implementación Rust Native (`native/`), suite de pruebas de paridad funcional, arnés de automatización (`scripts/`) y almacenamiento de resultados procesables (`results/`).
- Verificación mecánica de paridad de entradas/salidas y de aislamiento de dependencias (Rust Native sin Wasmtime).
- Comparativa empírica de métricas:
  - Tamaño de binario y paquete ZIP.
  - Cold Start (`Init Duration` agregado del entorno).
  - Warm Starts (`Duration` p50, p95, p99, media y desviación estándar).
  - Memoria máxima (`Max Memory Used`).
  - Costo económico modelado (por 100k y 1M invocaciones).
- Documentación reproducible (`README.md`, `BENCHMARK_REPORT.md` y datasets CSV/JSON).

## Acceptance criteria

### AC-01 — Paridad funcional verificable
- Ambas funciones aceptan el mismo esquema de evento JSON y producen exactamente el mismo payload JSON de salida para `double-value` y `make-point-sum`.
- Pruebas negativas y de validación idénticas (11 tests offline pasando; aserción de paridad activa en cada invocación en vivo).

### AC-02 — Aislamiento de dependencias en Rust Native
- El binario de Lambda A (Rust Native) no depende de `agent-native-runtime`, `wasmtime` ni `agent-native-compiler`.
- Verificado mecánicamente mediante `cargo tree` (0 coincidencias).

### AC-03 — Fidelidad de Lambda B (Agent-Native WASM)
- Reutiliza el componente WASM `program-005.wasm` compilado determinísticamente desde Semantic IR v0.5 con contratos WIT.
- Cumple ADR-005: Store nuevo por invocación. Sin pools de instancias ni modificaciones al núcleo.

### AC-04 — Equivalencia de configuración cloud
- Ambas funciones se configuran de manera idéntica en AWS Lambda: `provided.al2023`, `x86_64`, 256 MB de RAM, 10s timeout, misma región (`us-west-2`), sin VPC.

### AC-05 — Metodología rigurosa de Cold Starts
- Procedimiento reproducible para provocar inicializaciones frías verificadas mediante rotación de variable de entorno (`BENCHMARK_SALT=<uuid>`), registrando `Init Duration` desde el log `REPORT`.
- Distinción explícita entre tiempo base del microVM/bootstrap (~36 ms) y tiempo del runtime Wasmtime (~49 ms).

### AC-06 — Muestreo estadístico de Warm Starts
- Batería de 50 invocaciones cálidas por operación y por implementación (200 en total), intercaladas secuencialmente.
- Cálculo de métricas agregadas: p50, p95, p99, media, mín, máx y desviación estándar.

### AC-07 — Cuantificación de recursos y tamaño
- Medición de tamaño de ejecutable (`bootstrap`) y comprimido (`bootstrap.zip`).
- Registro de memoria máxima reportada por Lambda (`Max Memory Used`).

### AC-08 — Modelado económico
- Cálculo de costos estimados para 100.000 y 1.000.000 de invocaciones, explicitando los supuestos de tarificación de AWS en `us-west-2` a 256 MB.

### AC-09 — Seguridad y limpieza
- Rol IAM con privilegios mínimos. Sin secretos en el repositorio.
- Eliminación comprobada de funciones Lambda, roles IAM y grupos de CloudWatch Logs tras el experimento.

### AC-10 — Reporte y reproducibilidad
- Reporte Markdown exhaustivo (`BENCHMARK_REPORT.md`), README y datasets estructurados (`results/benchmark_summary.json`, `cold_starts.csv`, `warm_invocations.csv`).

## Validation

```bash
cargo test --manifest-path examples/aws-lambda-benchmark/native/Cargo.toml
cargo test --manifest-path examples/aws-lambda/Cargo.toml
cargo test --workspace
py -3.9 examples/aws-lambda-benchmark/scripts/run_benchmark.py --dry-run
```

## Definition of Done

- [x] Paridad funcional verificada entre Lambda A y Lambda B.
- [x] Artefactos compilados y medidos (overhead de tamaño cuantificado).
- [x] Datos estadísticos recolectados en AWS Lambda y procesados (latencia, memoria, cold start).
- [x] Overhead de Agent-Native calculado en valor absoluto y porcentual.
- [x] Modelado de costos documentado.
- [x] Recursos cloud eliminados completamente.
- [x] CI en verde y Learning consolidado en Kaddo.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap iniciativa | INIT-05-02, INIT-08-01, INIT-08-02 |
| Baseline | `v0.0.4-m3` |
| Work Items previos | AN-021 |
| ADRs | ADR-001, ADR-002, ADR-004, ADR-005 |

## Learning

**Lo que salió como se esperaba:**
- **Paridad funcional perfecta:** La misma entrada JSON generó exactamente la misma salida sin discrepancias en ninguna de las 220 invocaciones en AWS (20 cold starts y 200 warm starts).
- **Estabilidad de memoria (ADR-005):** El patrón de instanciación con un nuevo `wasmtime::Store` por invocación mantuvo la memoria de Lambda B absolutamente plana en 25.0 MB (desviación estándar: 0.0 MB en `double-value` y 0.4 MB en `make-point-sum`), demostrando ausencia total de fugas de memoria o retención de estado entre ejecuciones cálidas.
- **Sobrecarga de ejecución cálida casi imperceptible:** La mediana de ejecución (p50) en la operación estructurada `make-point-sum` fue de **0.91 ms para Rust Native frente a 0.99 ms para Agent-Native WASM**, lo que sitúa la sobrecarga neta del Component Model en apenas **~80 microsegundos (+0.08 ms)**.

**Lo inesperado:**
- **Desglose del Cold Start:** Medir la línea base nativa permitió descomponer empíricamente el `Init Duration` de AWS Lambda:
  - Inicialización base del microVM y bootstrap de Linux (`provided.al2023`): **36.27 ms** promedio.
  - Inicialización de Wasmtime 27, compilación interna JIT y carga del componente: **48.95 ms** promedio.
  - Cold start total agregado: **85.22 ms**. Esto demuestra que Wasmtime añade menos de 50 ms a la inicialización fría, disipando cualquier preocupación sobre latencias de segundo orden en serverless.
- **Predecibilidad de latencia en WASM:** En la operación `double-value`, la implementación WASM mostró menor desviación estándar (σ = 0.22 ms vs 0.53 ms en Native) y una latencia máxima menor (1.76 ms vs 3.73 ms), posiblemente debido a la gestión interna de memoria preasignada en Wasmtime.
- **Impacto en facturación nulo (<0.5%):** Dado que AWS Lambda factura con granularidad de 1 ms, ambas funciones se redondean a 1–2 ms de cómputo. El costo estimado por 1 millón de peticiones es de **\$0.2054 USD (Nativo)** vs **\$0.2063 USD (WASM)**, una diferencia real de apenas **+\$0.0009 USD**.

### Evidencias de ejecución

1. **Compilación y Aislamiento de Artefactos:**
   - Lambda A (Rust Native): binario de 2,018,016 bytes (1.92 MB), zip de 896,145 bytes (0.85 MB).
   - Lambda B (Agent-Native WASM): binario de 8,194,512 bytes (7.81 MB), zip de 3,267,419 bytes (3.12 MB).
   - `cargo tree` certificó 0 dependencias sobre `agent-native-runtime` o `wasmtime` en el crate nativo.

2. **Resultados Empíricos en AWS Lambda (`us-west-2`, 256 MB):**
   - **Cold Starts (10 muestras verificadas por función):**
     - Native: media 36.27 ms, p50 32.52 ms, máx 57.54 ms, memoria 19.2 MB.
     - WASM: media 85.22 ms, p50 82.28 ms, máx 105.93 ms, memoria 25.1 MB.
     - Overhead de Agent-Native: **+48.95 ms** en media, **+5.9 MB** en memoria.
   - **Warm Starts (50 iteraciones intercaladas por operación):**
     - `double-value`: Native p50 0.99 ms vs WASM p50 1.06 ms (+0.07 ms).
     - `make-point-sum`: Native p50 0.91 ms vs WASM p50 0.99 ms (+0.08 ms).
     - Memoria cálida: 20.0 MB constante en Native vs 25.0 MB constante en WASM.

3. **Limpieza Cloud Verificada (100% destruido):**
   - Funciones `agent-native-benchmark-native` y `agent-native-benchmark-wasm` eliminadas (`ResourceNotFoundException`).
   - Rol IAM `agent-native-benchmark-role` eliminado (`NoSuchEntity`).
   - Grupos de logs CloudWatch `/aws/lambda/agent-native-benchmark-native` y `/aws/lambda/agent-native-benchmark-wasm` eliminados (`logGroups: []`).
