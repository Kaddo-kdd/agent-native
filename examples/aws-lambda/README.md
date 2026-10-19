# AWS Lambda Runtime Example for Agent-Native

Ejemplo de integración que demuestra la ejecución de componentes WebAssembly generados por Agent-Native dentro del servicio serverless administrado **AWS Lambda** (`provided.al2023`).

> **Baseline técnica:** Certificado sobre [`v0.0.4-m3`](https://github.com/Kaddo-kdd/agent-native/releases/tag/v0.0.4-m3). Este ejemplo demuestra la portabilidad del runtime autónomo en un cloud provider sin alterar el núcleo del producto ni anticipar el alcance completo de RM-04 / RM-05.

---

## 1. Arquitectura y Principios de Diseño

El ejemplo separa estrictamente el tiempo de construcción del tiempo de ejecución (ADR-001 / ADR-002 / ADR-005):

```text
              BUILD TIME (Local / CI)
              -----------------------
crates/compiler/tests/fixtures/ir/program-005.ir.json
                      │
                      ▼
        agent-native-compiler (CompileContext)
                      │
                      ▼
         OUT_DIR/program-005.wasm (Component Model + WIT)
                      │
                      ▼
          bootstrap (Binario Rust nativo)
                      │
                      ▼
               bootstrap.zip (~3.2 MB)


              RUN TIME (AWS Lambda)
              ---------------------
               JSON Event
                   │
                   ▼
       bootstrap (lambda_runtime)
                   │
                   ▼
          agent-native-runtime
                   │
         [Engine + Component cached]
         [Fresh Store per invocation]
         [HostPolicy::deny_all()]
                   │
                   ▼
       Component Model Execution
                   │
                   ▼
             JSON Response
```

### Invariantes Verificadas
- **Runtime Autónomo (ADR-002):** El ejecutable desplegado en Lambda **no** incluye el compilador ni depende de LLMs o servicios externos.
- **Aislamiento por Invocación (ADR-005):** El motor `Engine` y el bytecode del componente se cargan una vez por entorno de ejecución (reutilizados en warm starts). Cada invocación se ejecuta en un `wasmtime::Store` nuevo y efímero, impidiendo fuga de memoria o estado entre eventos.
- **Seguridad deny-by-default (ADR-004):** Se ejecuta con `HostPolicy::deny_all()`. Cualquier import host no autorizado falla explícitamente pre-instantiate.

---

## 2. Artefactos y Contratos Utilizados

El ejemplo reutiliza los fixtures certificados en M3:

- **Semantic IR v0.5:** [`crates/compiler/tests/fixtures/ir/program-005.ir.json`](../../crates/compiler/tests/fixtures/ir/program-005.ir.json)
- **Contrato WIT:** [`crates/compiler/tests/fixtures/wit/program-005.wit`](../../crates/compiler/tests/fixtures/wit/program-005.wit)
- **Interfaz exportada:** `an:test/ops@0.1.0`

### Operaciones Soportadas

| Operación | Invocación interna | Entrada JSON | Salida JSON |
|---|---|---|---|
| `double-value` | `an:test/ops@0.1.0#double-value` | `{"x": 21}` | `{"result": 42}` |
| `make-point-sum` | `an:test/ops@0.1.0#make-point-sum` | `{"x": 3, "y": 4}` | `{"result": {"sum": 7, "doubled": 14}}` |

---

## 3. Requisitos de Construcción

- **Rust:** 1.85+ (edición 2024).
- **Cargo Lambda:** 1.9+ (`pip install cargo-lambda` o `winget install cargo-lambda`).
- **Zig:** Requerido por Cargo Lambda para compilación cruzada hacia Linux (`pip install ziglang`).
- **AWS CLI:** Opcional (solo para despliegue en cuenta cloud).

---

## 4. Pruebas Locales Automatizadas (Offline)

Las pruebas ejercitan el mismo handler y el mismo componente WASM compilado por `build.rs`, sin requerir credenciales AWS ni red:

```bash
cd examples/aws-lambda
cargo test
```

La suite cubre 11 pruebas:
1. `test_double_value_success`: invocación de `double-value(21) -> 42`.
2. `test_double_value_zero_and_negative`: verificación de límites (`0` y números negativos).
3. `test_make_point_sum_success`: serialización de record `{sum: 7, doubled: 14}`.
4. `test_make_point_sum_negative_values`: cálculo con operandos negativos.
5. `test_warm_invocation_reuse`: múltiples invocaciones secuenciales sobre el mismo `AppState` (aislamiento).
6. `test_invalid_operation`: fallo explícito ante operaciones no permitidas.
7. `test_missing_argument`: validación de campos obligatorios.
8. `test_invalid_argument_type`: rechazo de tipos no compatibles (strings, booleanos).
9. `test_integer_out_of_range`: validación de rango de `i32`.
10. `test_invalid_component_bytes`: fallo explícito de carga.
11. `test_event_fixtures`: validación de los tres archivos JSON en `events/`.

---

## 5. Empaquetado para AWS Lambda

Para compilar el binario nativo `bootstrap` optimizado para `provided.al2023` en arquitectura `x86_64`:

```bash
cargo lambda build --release --x86-64
```

Para generar el archivo ZIP listo para desplegar:

```bash
# Windows PowerShell
Compress-Archive -Path target/lambda/agent-native-aws-lambda/bootstrap -DestinationPath target/lambda/agent-native-aws-lambda/bootstrap.zip -Force

# Linux / macOS
cd target/lambda/agent-native-aws-lambda && zip bootstrap.zip bootstrap
```

### Métricas de Tamaño del Artefacto
- **Binario descomprimido (`bootstrap`):** ~8.1 MB (incluye runtime Wasmtime y componente WASM embebido).
- **Paquete comprimido (`bootstrap.zip`):** ~3.2 MB (límite de Lambda: 50 MB en subida directa).

---

## 6. Despliegue Manual en AWS Lambda

### 6.1. Crear el Rol IAM (Privilegios Mínimos)

Crear una política de confianza para Lambda:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": { "Service": "lambda.amazonaws.com" },
      "Action": "sts:AssumeRole"
    }
  ]
}
```

Crear el rol y adjuntar la política básica de CloudWatch Logs:
```bash
aws iam create-role \
  --role-name agent-native-lambda-role \
  --assume-role-policy-document file://trust-policy.json

aws iam attach-role-policy \
  --role-name agent-native-lambda-role \
  --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole
```

### 6.2. Crear la Función Lambda

```bash
aws lambda create-function \
  --function-name agent-native-wasm-poc \
  --runtime provided.al2023 \
  --role arn:aws:iam::<ACCOUNT_ID>:role/agent-native-lambda-role \
  --handler bootstrap \
  --architectures x86_64 \
  --memory-size 256 \
  --timeout 10 \
  --zip-file fileb://target/lambda/agent-native-aws-lambda/bootstrap.zip
```

---

## 7. Invocación y Verificación

### 7.1. Invocación de `double-value`

```bash
aws lambda invoke \
  --function-name agent-native-wasm-poc \
  --cli-binary-format raw-in-base64-out \
  --payload file://events/double-value.json \
  response-double.json
```

Contenido de `response-double.json`:
```json
{
  "result": 42
}
```

### 7.2. Invocación de `make-point-sum`

```bash
aws lambda invoke \
  --function-name agent-native-wasm-poc \
  --cli-binary-format raw-in-base64-out \
  --payload file://events/make-point-sum.json \
  response-point.json
```

Contenido de `response-point.json`:
```json
{
  "result": {
    "doubled": 14,
    "sum": 7
  }
}
```

### 7.3. Invocación con Error (Operación Inválida)

```bash
aws lambda invoke \
  --function-name agent-native-wasm-poc \
  --cli-binary-format raw-in-base64-out \
  --payload file://events/invalid-operation.json \
  response-invalid.json
```

Salida esperada (error estructurado de Lambda):
```json
{
  "errorMessage": "Invalid operation 'unknown-op'. Allowed operations are 'double-value' and 'make-point-sum'",
  "errorType": "agent_native_aws_lambda::handler::HandlerError"
}
```

---

## 8. Logs y Limpieza de Recursos

### Consultar Logs
```bash
aws logs tail /aws/lambda/agent-native-wasm-poc --follow
```

### Eliminar Recursos (Evitar Costos Remanentes)
```bash
# Eliminar la función
aws lambda delete-function --function-name agent-native-wasm-poc

# Eliminar el grupo de CloudWatch Logs (AWS no lo elimina automáticamente al borrar la función)
aws logs delete-log-group --log-group-name /aws/lambda/agent-native-wasm-poc

# Desvincular política y eliminar rol IAM
aws iam detach-role-policy \
  --role-name agent-native-lambda-role \
  --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole

aws iam delete-role --role-name agent-native-lambda-role
```

---

## 9. Limitaciones y Precisiones Técnicas

- **Interpretación de Cold Start (Init Duration):** Los 92.21 ms observados en `Init Duration` corresponden al tiempo agregado de aprovisionamiento del microVM/entorno Lambda, arranque del proceso nativo `bootstrap`, inicialización del motor Wasmtime y carga del Component Model; no deben atribuirse exclusivamente a la inicialización de Wasmtime.
- **Estabilidad de memoria vs. fugas bajo carga:** La observación de 25–26 MB durante las ejecuciones de prueba demuestra estabilidad durante esa secuencia puntual, pero no constituye evidencia concluyente de ausencia de fugas bajo carga sostenida o concurrencia prolongada (sujeto a los benchmarks formales de RM-06 y RM-08).
- **Sin imports funcionales:** El componente WASM utilizado no declara imports de host externos (alcance certificado de M3). Los stubs y capacidades WASI están planificados para RM-07.
- **Instrumentación de memoria guest:** La memoria reportada en CloudWatch refleja el proceso completo de la función Lambda (Rust runtime + Wasmtime engine + guest). La medición granular de memoria por instancia WASM está formalmente diferida a INIT-06-02.
