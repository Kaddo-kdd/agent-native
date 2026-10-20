---
id: AN-044
type: feature
status: draft
created_at: 2026-10-20T00:00:00.000Z
title: Adaptador WASI Preview 2 para I/O determinista en memoria
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/Cargo.toml
  - crates/runtime/src/wasi.rs
  - crates/runtime/src/engine.rs
  - crates/runtime/src/tenant.rs
  - crates/runtime/src/invoke.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/tests/wasi_adapter.rs
source_roadmap_initiative: INIT-07-01
source_work_item_candidate:
  - WI-CANDIDATE-07-01-01
domains:
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
  - Resource Contention & Limits
expected_value: >-
  Integrar el soporte de WASI Preview 2 (WASI 0.2 / Component Model) en el runtime independiente
  mediante `wasmtime-wasi = "29.0.1"`, permitiendo que los componentes WebAssembly interactúen
  con streams de entrada y salida deterministas en memoria (stdin inyectado, stdout/stderr capturados)
  sin dependencias de terminal interactiva (TTY), sin introducir dependencias de I/O en el compilador
  o IR (ADR-002), y manteniendo el ciclo de vida efímero por Store y las cuotas de fuel y memoria (ADR-005).
risks:
  - >-
    R-WASI-01: Contaminación de estado de I/O entre invocaciones o tenants sucesivos.
    Mitigación: Creación y descarte incondicional de `WasiCtx` y `ResourceTable` por invocación en el Store efímero.
  - >-
    R-WASI-02: Dependencias cruzadas prohibidas (runtime/compiler).
    Mitigación: Mantener `wasmtime-wasi` estrictamente encapsulado en `crates/runtime`; el compilador y Semantic IR
    permanecen 100% aislados de detalles de WASI.
  - >-
    R-WASI-03: Bloqueo de ejecución en lecturas de stdin no acotadas.
    Mitigación: Inyección de streams deterministas con EOF garantizado en memoria.
dependencies:
  - AN-038
  - AN-039
  - AN-040
source_signals:
  - 'Roadmap RM-07: INIT-07-01 Baseline WASI preview2 + stdio autorizado'
  - 'ADR-002: Runtime independiente de LLM y compilador'
  - 'ADR-004: Fail-explicit y contención estricta de errores'
  - 'ADR-005: Store fresco por invocación y prohibición de reutilización de instancias'
decision_candidates: []
related_decisions:
  - ADR-002
  - ADR-004
  - ADR-005
---

# AN-044 — Adaptador WASI Preview 2 para I/O determinista en memoria

## Actor y Resultado
Un desarrollador o sistema anfitrión invoca un componente WebAssembly compatible con WASI Preview 2 (WASI 0.2) suministrando datos de entrada estándar (`stdin`) y capturando de forma estructurada la salida estándar (`stdout`) y los diagnósticos de error (`stderr`) en búferes en memoria, bajo las garantías de aislamiento, límites de recursos (fuel y memoria) y ciclo de vida efímero establecidas en RM-06.

---

## Contexto y Diseño Arquitectónico

### 1. Desacoplamiento de Interfaces WIT Reales vs Concepto Stdio
WASI 0.2 no provee una interfaz WIT monolítica llamada `wasi:cli/stdio`. Define interfaces atómicas e independientes bajo el paquete `wasi:cli@0.2.0` y `wasi:io@0.2.0`:

- `wasi:cli/stdin@0.2.0`: obtención del handle de flujo de entrada (`get-stdin -> input-stream`).
- `wasi:cli/stdout@0.2.0`: obtención del handle de flujo de salida (`get-stdout -> output-stream`).
- `wasi:cli/stderr@0.2.0`: obtención del handle de flujo de errores (`get-stderr -> output-stream`).
- `wasi:io/streams@0.2.0`: operaciones sobre streams (lectura `read`, escritura `write`, `blocking-write-and-flush`, `check-write`).
- `wasi:io/error@0.2.0`: tipos y reporte de errores de E/S.

El adaptador en `crates/runtime` debe implementar el enlace con estas interfaces específicas mediante `wasmtime-wasi = "29.0.1"`, evitando asociar permisos globales a un concepto abstracto.

### 2. Integración sin Violación de Invariantes (ADR-002 y ADR-005)
- **Independencia (ADR-002):** `crates/compiler` y `crates/ir` NO dependen de `wasmtime-wasi` ni de tipos de I/O de bajo nivel. La I/O vive exclusivamente en la frontera del runtime.
- **Aislamiento por Invocación (ADR-005):** Cada invocación construye su propio `WasiCtx` con una tabla de recursos `ResourceTable` dedicada. Al finalizar la invocación, el `Store` se destruye junto con la tabla de recursos y los descriptores de streams abiertos, impidiendo cualquier retención de estado o descriptor fantasma entre tenants o llamadas consecutivas.
- **Límites Unificados:** El gasto computacional de las operaciones de streams y libcalls de WASI se debita del presupuesto de fuel del tenant (`TenantConfig::with_fuel_budget()`) y la memoria de búferes de streams respeta `InstanceLimits`.

### 3. Abstracción de Streams en Memoria para Determinismo
Para evitar la dependencia de TTYs reales o descriptores de archivo del sistema operativo del host:
- `stdin`: alimentado por pipes en memoria (`wasmtime_wasi::pipe::MemoryInputPipe`). Si no se suministra entrada, el flujo entrega inmediatamente `EOF`.
- `stdout` y `stderr`: capturados mediante pipes concurrentes en memoria (`wasmtime_wasi::pipe::MemoryOutputPipe`), exponiendo buffers extraíbles `Vec<u8>` e inspeccionables como UTF-8 al concluir la invocación.

---

## Contratos Públicos Propuestos (`crates/runtime`)

```rust
/// Configuración de I/O y streams para una invocación WASI.
#[derive(Debug, Clone, Default)]
pub struct WasiIoConfig {
    /// Bytes a suministrar como entrada estándar (stdin). Si es None, stdin retorna EOF inmediato.
    pub stdin: Option<Vec<u8>>,
    /// Si true, captura stdout en memoria; de lo contrario, se descarta.
    pub capture_stdout: bool,
    /// Si true, captura stderr en memoria; de lo contrario, se descarta.
    pub capture_stderr: bool,
}

/// Búferes capturados resultantes de la ejecución WASI.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WasiIoOutput {
    /// Bytes emitidos hacia stdout durante la invocación.
    pub stdout: Vec<u8>,
    /// Bytes emitidos hacia stderr durante la invocación.
    pub stderr: Vec<u8>,
}

impl WasiIoOutput {
    pub fn stdout_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.stdout)
    }
    pub fn stderr_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.stderr)
    }
}

/// Reporte de invocación extendido con captura de I/O WASI.
#[derive(Debug, Clone)]
pub struct WasiInvocationReport {
    pub values: Vec<Val>,
    pub fuel_consumed: Option<u64>,
    pub io: WasiIoOutput,
}
```

---

## Acceptance Criteria

- **AC-01 (Dependencia pinneada y compilación limpia):** `crates/runtime/Cargo.toml` incorpora `wasmtime-wasi = "29.0.1"`, compilando limpiamente en Linux, Windows MSVC y macOS sin introducir dependencias en `compiler` ni `ir`.
- **AC-02 (Captura y redirección en memoria determinista):** El runtime permite suministrar `stdin` desde bytes en memoria y capturar `stdout` y `stderr` sin acceder a la consola del host ni requerir descriptores interactivos.
- **AC-03 (Aislamiento estricto de recursos y streams):** Dos invocaciones consecutivas de un componente que escribe en `stdout` reciben buffers independientes; la segunda invocación nunca observa residuos de salida de la primera.
- **AC-04 (Compatibilidad con límites de recursos):** Las operaciones de streams se ejecutan bajo contención de fuel y memoria; un bucle de escritura masiva en `stdout` respeta el presupuesto de fuel configurado.
- **AC-05 (Suite de pruebas de integración):** Suite en `crates/runtime/tests/wasi_adapter.rs` validando ejecución con stdout capturado, lectura de stdin determinista, y descarte de contexto de recursos.
