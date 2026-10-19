# Agent-Native — Benchmark AWS Lambda: Rust Native vs Agent-Native WASM

Este subdirectorio contiene el arnés de benchmark comparativo, reproducible y verificable entre dos funciones desplegadas en AWS Lambda:

- **Lambda A (Rust Native):** Ejecución directa en Rust nativo de la lógica de negocio, sin `agent-native-runtime`, Wasmtime ni compilador.
- **Lambda B (Agent-Native WASM):** Ejecución mediante `agent-native-runtime`, Wasmtime 27 y el componente `program-005.wasm` bajo las invariantes de [ADR-005](../../docs/architecture/adr/ADR-005-canonical-abi-memory-management.md) (nuevo `Store` por invocación).

El propósito es cuantificar el costo real de abstracción y portabilidad de Agent-Native en latencia (cold/warm), memoria, tamaño de artefacto y costo económico.

> **Reporte Completo:** Consulte el informe detallado con datasets y percentiles en [BENCHMARK_REPORT.md](./BENCHMARK_REPORT.md).

### Resumen de Resultados Empíricos (AWS `us-west-2`, 256 MB)

| Métrica | Lambda A (Rust Native) | Lambda B (Agent-Native WASM) | Overhead de Agent-Native |
|---|---|---|---|
| **Binario (`bootstrap`)** | 1.92 MB | 7.81 MB | +5.89 MB (+306%) |
| **Paquete ZIP (`bootstrap.zip`)** | 0.85 MB | 3.12 MB | +2.26 MB (+265%) |
| **Cold Start (`Init Duration` medio)** | 36.27 ms | 85.22 ms | +48.95 ms (+135%) |
| **Warm Start (p50 mediana)** | 0.91 ms | 0.99 ms | **+0.08 ms (+80 µs)** |
| **Memoria Pico (`Max Memory Used`)** | 20.0 MB | 25.0 MB | +5.0 MB (+25%) |
| **Costo por 1M invocaciones** | \$0.2054 USD | \$0.2063 USD | **+\$0.0009 USD (< 0.5%)** |

---

## 1. Arquitectura y Paridad Funcional

Ambas funciones exponen exactamente la misma interfaz JSON para dos operaciones:

| Operación | Evento Entrada | Respuesta Esperada |
|---|---|---|
| `double-value` | `{"operation": "double-value", "args": {"x": 21}}` | `{"result": 42}` |
| `make-point-sum` | `{"operation": "make-point-sum", "args": {"x": 3, "y": 4}}` | `{"result": {"sum": 7, "doubled": 14}}` |

### Aislamiento de dependencias
- **Lambda A:** implementada en [`native/`](./native/), no incluye `wasmtime`, `agent-native-runtime` ni `agent-native-compiler` en su árbol de dependencias (`cargo tree` verificado).
- **Lambda B:** reutiliza [`examples/aws-lambda/`](../aws-lambda/), cargando el componente `program-005.wasm` generado desde Semantic IR v0.5 con contratos WIT (`an:test/ops@0.1.0`).

---

## 2. Configuración Cloud Idéntica

Ambas funciones se configuran con los mismos parámetros en AWS Lambda:

- **Arquitectura:** `x86_64`
- **Runtime:** `provided.al2023`
- **Memoria:** 256 MB
- **Timeout:** 10 segundos
- **Región:** `us-west-2`
- **Red:** Sin VPC
- **Permisos IAM:** Rol de ejecución básico con `AWSLambdaBasicExecutionRole`

---

## 3. Estructura del Directorio

```
examples/aws-lambda-benchmark/
├── README.md               # Este documento
├── native/                 # Crate Rust nativo (Lambda A)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── handler.rs      # Lógica equivalente pura
│   │   └── main.rs         # Bootstrap con lambda_runtime
│   └── tests/
│       └── integration.rs  # 11 pruebas de paridad funcional offline
├── scripts/
│   ├── run_benchmark.py    # Orquestador del benchmark (Python stdlib)
│   └── run_benchmark.ps1   # Wrapper de PowerShell
└── results/                # Datasets y resúmenes generados
    ├── raw_invocations.csv
    ├── cold_starts.csv
    └── benchmark_summary.json
```

---

## 4. Reproducción Local (Offline)

### Ejecutar pruebas de paridad en ambos crates
```powershell
# Pruebas de Lambda A (Rust Native)
cargo test --manifest-path examples/aws-lambda-benchmark/native/Cargo.toml

# Pruebas de Lambda B (Agent-Native WASM)
cargo test --manifest-path examples/aws-lambda/Cargo.toml

# Pruebas del workspace principal del repositorio
cargo test --workspace
```

### Compilar binarios de producción para AWS Lambda
```powershell
# Lambda A (Rust Native)
cd examples/aws-lambda-benchmark/native
cargo lambda build --release --x86-64
Compress-Archive -Path target/lambda/agent-native-aws-lambda-native/bootstrap -DestinationPath target/lambda/agent-native-aws-lambda-native/bootstrap.zip -Force

# Lambda B (Agent-Native WASM)
cd ../../aws-lambda
cargo lambda build --release --x86-64
Compress-Archive -Path target/lambda/agent-native-aws-lambda/bootstrap -DestinationPath target/lambda/agent-native-aws-lambda/bootstrap.zip -Force
```

### Validar artefactos sin conexión (Dry Run)
```powershell
py -3.9 examples/aws-lambda-benchmark/scripts/run_benchmark.py --dry-run
```

---

## 5. Ejecución en AWS (Fase 2)

> **Nota:** La ejecución en la nube requiere credenciales configuradas de AWS (`aws sts get-caller-identity`).

### 1. Desplegar ambas funciones
```powershell
py -3.9 examples/aws-lambda-benchmark/scripts/run_benchmark.py --deploy
```

### 2. Ejecutar la batería de benchmark
```powershell
# 10 cold starts verificados y 50 invocaciones cálidas alternadas por operación
py -3.9 examples/aws-lambda-benchmark/scripts/run_benchmark.py --run --cold-runs 10 --warm-runs 50
```

### 3. Limpiar recursos completamente
```powershell
py -3.9 examples/aws-lambda-benchmark/scripts/run_benchmark.py --teardown
```
Elimina:
- Ambas funciones Lambda (`agent-native-benchmark-native`, `agent-native-benchmark-wasm`).
- El rol IAM (`agent-native-benchmark-role`).
- Los dos grupos de CloudWatch Logs (`/aws/lambda/agent-native-benchmark-native`, `/aws/lambda/agent-native-benchmark-wasm`).

---

## 6. Métricas y Metodología

### Cold Start
Se provoca mediante la actualización de una variable de entorno en la configuración de la función (`BENCHMARK_SALT=<uuid>`), lo que fuerza a AWS Lambda a descartar el contenedor previo y crear un microVM/sandbox nuevo. Se mide exclusivamente la línea `REPORT` del log de Lambda, registrando `Init Duration` (tiempo total de inicialización del runtime y el proceso).

### Warm Starts
Invocaciones consecutivas alternando entre la función nativa y WASM para neutralizar posibles efectos de variabilidad temporal o "noisy neighbors". Se verifica la paridad de la respuesta en cada invocación y se registran `Duration`, `Billed Duration`, `Max Memory Used` y la latencia cliente observada.

### Modelo de Costos
Calculado conforme a las tarifas vigentes de AWS Lambda en `us-west-2` (256 MB):
- Solicitudes: \$0.20 por millón de invocaciones (\$0.0000002 / req).
- Cómputo: \$0.0000166667 por GB-segundo, equivalente a \$0.000000004166675 por ms facturado a 256 MB.
