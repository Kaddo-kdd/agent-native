---
id: AN-021
type: feature
status: completed
completed_at: 2026-10-19
title: AN-021 — AWS Lambda Runtime Example
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - examples/aws-lambda/**
source_roadmap_initiative: INIT-05-02
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Distribution & DX
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Demostrar la portabilidad y reutilización del runtime existente dentro de un
  servicio cloud administrado (AWS Lambda), procesando eventos JSON contra un
  componente WASM compilado con contratos WIT, sin depender del compilador ni
  LLM en runtime.
dependencies:
  - AN-014
  - AN-020
source_signals:
  - 'Baseline post-M3: v0.0.4-m3'
  - Validación de portabilidad cloud para INIT-05-02
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-19T00:00:00.000Z
ready_at: '2026-10-19'
---

# AN-021 — AWS Lambda Runtime Example

**Actor y resultado:**
Un desarrollador puede construir un componente WASM localmente con `agent-native-compiler`, empaquetarlo con un adaptador Rust independiente basado en `agent-native-runtime` y `lambda_runtime`, desplegarlo en AWS Lambda (`provided.al2023`) e invocarlo mediante eventos JSON sin requerir compilador, LLM ni herramientas de desarrollo en runtime.

**Current behavior:**
- Agent-Native alcanzó M3 el 2026-10-19 (`v0.0.4-m3`), con compilador IR v0.5 determinista y runtime autónomo Wasmtime 27 probado en CI local y GitHub Actions.
- No existe ninguna demostración ni fixture que ejecute el runtime dentro de un servicio cloud administrado (AWS Lambda / serverless).
- La reutilización del runtime como crate consumidor fuera del workspace no está ejercitada en un ejemplo ejecutable.

**Target behavior:**
- Se crea el directorio `examples/aws-lambda/` con su propio `Cargo.toml`, `build.rs`, `src/main.rs`, `src/handler.rs`, `tests/integration.rs`, `events/` y `README.md`.
- `build.rs` compila `crates/compiler/tests/fixtures/ir/program-005.ir.json` con `agent-native-compiler` y `CompileContext`, generando el artefacto WASM determinista.
- El adaptador (`src/handler.rs`) consume exclusivamente `agent-native-runtime` (`Engine`, `Component`, `Val`, `HostPolicy`), recibiendo eventos JSON (`double-value` y `make-point-sum`) y retornando respuestas JSON estructuradas.
- Nuevo `Store` por invocación según ADR-005 (aislamiento entre invocaciones calientes).
- Política `HostPolicy::deny_all()` activa.
- Tests locales automatizados (`tests/integration.rs`) verifican el flujo sin requerir credenciales AWS.
- Validación real documentada en AWS Lambda con recolección de métricas y procedimiento de limpieza.

**Problem:**
Validar que las abstracciones del runtime son portables a un entorno serverless real y que el aislamiento de dependencias (ADR-002) se sostiene en un entorno de producción restringido (Linux x86_64, sandbox Lambda).

**Scope confidence:** high. El fixture `program-005` y la invocación Component Model ya están certificados en M3 (`e2e_v05.rs`). El adaptador Lambda es un consumidor directo de la API pública existente.

## Acceptance criteria

### AC-01 — Compilación verificable del componente
- Se genera un artefacto WASM válido a partir de `program-005.ir.json`.
- Se reutiliza `agent-native-compiler` con `CompileContext`.
- El artefacto es reproducible a partir de los mismos archivos fuente y contexto.
- No se utiliza un binario WASM preparado manualmente sin trazabilidad.

### AC-02 — Runtime independiente
- El adaptador consume las APIs públicas `Engine`, `Component` y `Val` de `agent-native-runtime`.
- El componente se carga con `Engine::load_component` y se invoca con `Component::invoke`.
- No se invoca al compilador durante la ejecución de Lambda.
- No existe dependencia de LLM, Kaddo o servicios externos para procesar el evento.

### AC-03 — Ciclo de vida del runtime
- Inicializar `Engine` y cargar el componente una vez por entorno Lambda (reutilización en warm starts).
- No reutilizar instancias WASM ni `wasmtime::Store` entre invocaciones (ADR-005).
- No introducir pools ni optimizaciones de instancias persistentes.

### AC-04 — Compatibilidad funcional
- `double-value(21)` devuelve `{ "result": 42 }`.
- `make-point-sum(3, 4)` devuelve `{ "result": { "sum": 7, "doubled": 14 } }`.
- Ambas se ejecutan a través de WASM (sin cálculos equivalentes en Rust).
- Serialización JSON fiel a los tipos y nombres del contrato WIT.

### AC-05 — Validación y manejo de errores
- Pruebas negativas para: operación no permitida, argumentos faltantes, tipos incompatibles, enteros fuera de rango `i32`, error de runtime.
- Errores explícitos y estructurados, sin fallbacks silenciosos.
- `HostPolicy::deny_all()` como política por defecto.

### AC-06 — Empaquetado y despliegue
- Construcción para AWS Lambda `provided.al2023` en `x86_64`.
- Binario `bootstrap` nativo que incorpora o acompaña el componente WASM.
- Medición de tamaño de paquete comprimido y descomprimido.

### AC-07 — Pruebas locales automatizadas
- Suite de pruebas de integración offline (`tests/integration.rs`) ejercitando el mismo handler.
- CI principal del workspace permanece verde en Linux, Windows y macOS.

### AC-08 — Validación real en AWS
- Despliegue manual en cuenta AWS y recolección de evidencia:
  - Región y arquitectura.
  - SHA del commit.
  - Respuestas JSON e invocaciones exitosas / negativas.
  - Duración de ejecución, latencia de inicialización (cold start) y memoria total reportada por Lambda.
- Si no hay credenciales AWS disponibles, mantener pendiente AC-08 sin cerrar el WI como validado en la nube.

### AC-09 — Seguridad del despliegue
- Rol IAM con privilegios mínimos (logs básicos).
- Sin secretos ni credenciales en el repositorio.
- Documentación del procedimiento de limpieza (`cleanup`).

### AC-10 — Documentación reproducible
- `README.md` completo en `examples/aws-lambda/` con arquitectura, requisitos, build local, pruebas, empaquetado, despliegue, invocación y métricas.
- Referencia a la baseline `v0.0.4-m3`.

## Out of scope

- CLI `agent-native deploy` o comando `an-run`.
- Generación de IR con LLM o agentes.
- Modificaciones a Semantic IR v0.5 o al núcleo del runtime/compilador.
- Composición multi-componente o host providers WASI/AWS.
- Despliegue continuo automático o integración con API Gateway / DynamoDB / S3.

## Technical risks

- **R1 — Tamaño del ejecutable:** Wasmtime incrementa el tamaño del binario. Medir tamaño ZIP y verificar compatibilidad con límites de Lambda (50 MB comprimido directo / 250 MB descomprimido).
- **R2 — Compatibilidad Linux:** Binario `bootstrap` debe compilarse para Linux `x86_64` (glibc compatible con AL2023 o musl).
- **R3 — Latencia en Cold Start:** Inicialización de `Engine` y carga de componente en cold start.
- **R4 — Empaquetado del WASM:** El componente WASM debe incluirse de forma independiente o vía `include_bytes!` reproducible.
- **R5 — Alcance del runtime:** Solo componentes sin imports funcionales (soporte M3).
- **R6 — Cambios arquitectónicos accidentales:** No incorporar dependencias AWS dentro de `crates/runtime`.

## Implementation plan

- **Fase 1 — Construcción y verificación local:**
  - Scaffolding de `examples/aws-lambda/` (`Cargo.toml`, `build.rs`, `src/handler.rs`, `src/main.rs`).
  - Suite de integración offline (`tests/integration.rs`) y eventos JSON (`events/`).
  - Verificación local y empaquetado Linux (`bootstrap`).
- **Fase 2 — Validación en AWS Lambda:**
  - Creación de rol IAM y función Lambda en `provided.al2023`.
  - Invocación de eventos (`double-value`, `make-point-sum`, evento inválido).
  - Recolección de métricas y evidencia.
- **Fase 3 — Documentación y cierre:**
  - `examples/aws-lambda/README.md`.
  - Verificación de CI verde en workspace.
  - Learning del Work Item.

## Validation

```bash
cargo check -p agent-native-aws-lambda
cargo test -p agent-native-aws-lambda
cargo test --workspace
```

## Definition of Done

- Ejemplo funcional y reproducible en `examples/aws-lambda/`.
- Ambas funciones se ejecutan a través del runtime WASM de Agent-Native.
- Tests locales y escenarios negativos en verde.
- Invariantes ADR-001, ADR-002, ADR-004 y ADR-005 respetadas.
- Documentación completa de construcción, despliegue y destrucción de recursos.
- Evidencia de despliegue real en AWS recopilada (o formalmente pendiente de credenciales).

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap iniciativa | INIT-05-02 (API pública reutilizable del runtime) |
| Baseline | `v0.0.4-m3` |
| ADRs | ADR-001, ADR-002, ADR-004, ADR-005 |
| Prerrequisitos | AN-014, AN-020 |

## Learning

**Lo que salió como se esperaba:**
- La separación estricta de responsabilidades (ADR-001 / ADR-002) funcionó a la perfección: el adaptador Lambda depende exclusivamente de `agent-native-runtime` y compila en un ejecutable nativo Linux independiente sin requerir dependencias del compilador ni LLM en tiempo de ejecución.
- La estrategia de memoria de ADR-005 (reutilización de `Engine` y bytecode del componente en el estado de Lambda + creación de un `wasmtime::Store` fresco y efímero en cada llamada a `Component::invoke`) demostró un comportamiento serverless ideal:
  - Sin fugas de memoria entre invocaciones calientes (memoria constante de 25 MB en AWS Lambda).
  - Aislamiento total sin retención de estado entre ejecuciones sucesivas.
- El empaquetado mediante `cargo-lambda` y Zig generó un binario `bootstrap` nativo de 7.8 MB y un archivo comprimido `bootstrap.zip` de tan solo 3.2 MB, disipando completamente el riesgo R1 sobre el tamaño de Wasmtime.
- Las dos operaciones WIT certificadas (`double-value` y `make-point-sum`) retornaron los resultados exactos esperados tanto en pruebas locales como en las invocaciones reales sobre AWS Lambda.

**Lo inesperado:**
- La latencia observada en AWS Lambda (`provided.al2023`, x86_64, 256 MB) superó las expectativas iniciales:
  - Cold start: **Init Duration de 92.21 ms** (inicialización del motor Wasmtime y parseo del Component Model), seguido de una primera ejecución de 1.29 ms.
  - Invocaciones calientes (warm starts): **1.12 ms a 2.34 ms** de tiempo de cómputo (con facturación mínima de 2 ms a 3 ms).
- El manejo de errores bajo el tipo estructurado `HandlerError` se propagó nítidamente a la respuesta de Lambda sin fallbacks silenciosos ante operaciones no permitidas o tipos inválidos.

**Riesgos evaluados:**
- **R1 (Tamaño del ejecutable):** Mitigado. Binario de 7.8 MB descomprimido / 3.2 MB comprimido (límite Lambda: 50 MB en zip directo).
- **R2 (Compatibilidad Linux):** Mitigado. Binario compilado para `x86_64-unknown-linux-gnu` ejecutado con éxito en `provided.al2023`.
- **R3 (Latencia de Cold Start):** Mitigado. Init Duration < 100 ms (92 ms) en runtime serverless.
- **R4 (Empaquetado WASM):** Mitigado. `build.rs` compila `program-005.ir.json` con `CompileContext` determinista e `include_bytes!` incorpora el componente en el binario sin depender de rutas locales.
- **R5 (Alcance del runtime):** Mitigado. Respetado el alcance de M3 sin host imports funcionales.
- **R6 (Cambios arquitectónicos):** Mitigado. Workspace raíz intacto; `examples/aws-lambda` tiene su propio manifest y no contamina el producto.

### Evidencias de ejecución

1. **Pruebas Locales (Offline):**
   - 11/11 tests de integración en `examples/aws-lambda/tests/integration.rs` pasaron exitosamente.
   - 75/75 tests del workspace principal permanecen en verde.
   - Clippy pedantic y rustfmt limpios con 0 advertencias.

2. **Validación Real en AWS Lambda (2026-10-19):**
   - **Cuenta:** `570743710918`
   - **Región:** `us-west-2`
   - **Función:** `agent-native-wasm-poc` (`provided.al2023`, `x86_64`, 256 MB)
   - **Invocación 1 (Cold Start):**
     - Evento: `{"operation": "double-value", "args": {"x": 21}}`
     - Respuesta: `{"result": 42}`
     - Init Duration: 92.21 ms | Duration: 1.29 ms | Max Memory: 25 MB
   - **Invocación 2 (Warm Start - Record return):**
     - Evento: `{"operation": "make-point-sum", "args": {"x": 3, "y": 4}}`
     - Respuesta: `{"result": {"doubled": 14, "sum": 7}}`
     - Duration: 1.24 ms | Max Memory: 25 MB
   - **Invocación 3 (Error negativo):**
     - Evento: `{"operation": "unknown-op", "args": {"x": 10}}`
     - Error: `Unhandled` con mensaje explícito `"Invalid operation 'unknown-op'..."`
     - Duration: 1.12 ms | Max Memory: 25 MB
   - **Invocaciones 4, 5, 6 (Warm Reuse en bucle):**
     - Respuestas deterministas: `{"result": 2}`, `{"result": 4}`, `{"result": 6}`
     - Duraciones: 2.34 ms, 1.91 ms, 1.17 ms | Max Memory: 25-26 MB
   - **Limpieza de Recursos (Cleanup):**
     - Función `agent-native-wasm-poc` eliminada.
     - Rol IAM `agent-native-lambda-role` desvinculado y eliminado.
     - Sin recursos facturables remanentes.

