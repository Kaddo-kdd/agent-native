---
id: AN-044
type: feature
status: completed
created_at: 2026-10-20T00:00:00.000Z
completed_at: 2026-10-20T02:30:00.000Z
title: Adaptador WASI Preview 2 para I/O determinista en memoria
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/Cargo.toml
  - crates/runtime/src/wasi.rs
  - crates/runtime/src/invoke.rs
  - crates/runtime/src/engine.rs
  - crates/runtime/src/tenant.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/src/error.rs
  - crates/runtime/tests/wasi_adapter.rs
source_roadmap_initiative: INIT-07-01
initiative: INI-007
source_work_item_candidate:
  - WI-CANDIDATE-07-01-01
domains:
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
  - Resource Contention & Limits
expected_value: >-
  Integrar el soporte de WASI Preview 2 (WASI 0.2 / Component Model) en el
  runtime independiente mediante `wasmtime-wasi = "29.0.1"`, permitiendo que los
  componentes WebAssembly interactúen con streams deterministas en memoria
  (stdin inyectado, stdout/stderr capturados y acotados) sin TTY, sin
  dependencias de I/O en el compilador o IR (ADR-002), con linker poblado
  únicamente con las interfaces concedidas por la política (ADR-004) y
  manteniendo el Store efímero y los límites de fuel y memoria (ADR-005).
risks:
  - >-
    R-WASI-01: Contaminación de estado de I/O entre invocaciones o tenants.
    Mitigación: `WasiCtx` y `ResourceTable` se crean y descartan por invocación
    dentro del Store efímero.
  - >-
    R-WASI-02: Dependencias cruzadas prohibidas (runtime/compiler). Mitigación:
    `wasmtime-wasi` queda encapsulado en `crates/runtime`; se verifica con
    `cargo tree`.
  - >-
    R-WASI-03: Bloqueo en lecturas de stdin no acotadas. Mitigación: stdin en
    memoria con EOF garantizado.
  - >-
    R-WASI-04: Auto-concesión por linker. `wasmtime_wasi::add_to_linker_sync`
    registra TODAS las interfaces (filesystem, sockets, clocks, random...).
    Usarlo expondría capacidades no concedidas. Mitigación: registrar interfaz
    por interfaz (módulos `bindings::sync::*`) solo para las concedidas por
    `HostPolicy`; prohibido `add_to_linker_sync` global. Test negativo
    obligatorio.
  - >-
    R-WASI-05: Agotamiento de memoria del host por salida masiva. El fuel solo
    mide instrucciones guest y `InstanceLimits` solo limita memoria
    lineal/tablas del guest; ninguno acota los búferes del host. Mitigación:
    capacidades máximas explícitas para stdout, stderr y stdin con error tipado
    al exceder.
  - >-
    R-WASI-06: Transitivas de `wasmtime-wasi` que exijan un rustc superior a
    1.85 (ocurrió con `ureq`). Mitigación: verificar compilación con la
    toolchain pinneada y fijar versiones en `Cargo.lock`.
dependencies:
  - AN-038
  - AN-039
  - AN-040
source_signals:
  - 'Roadmap RM-07: INIT-07-01 Baseline WASI preview2 + stdio autorizado'
  - 'ADR-002: Runtime independiente de LLM y compilador'
  - 'ADR-004: Fail-explicit y contención estricta de errores'
  - >-
    ADR-005: Store fresco por invocación y prohibición de reutilización de
    instancias
decision_candidates: []
related_decisions:
  - ADR-002
  - ADR-004
  - ADR-005
ready_at: '2026-10-20'
---

# AN-044 — Adaptador WASI Preview 2 para I/O determinista en memoria

## Actor y Resultado
Un sistema anfitrión invoca un componente WASI Preview 2 suministrando `stdin` desde memoria y capturando `stdout`/`stderr` en búferes acotados, bajo el aislamiento, los límites (fuel y memoria) y el ciclo de vida efímero de RM-06.

## Alcance y frontera con AN-045
- **AN-044:** mecánica de I/O (contexto WASI, streams en memoria, límites de búfer) y **linker restringido a lo concedido**. Reutiliza el `HostPolicy` existente: conceder `allow_wit_interface("wasi:cli/stdout@0.2.0")` ya basta para habilitar la interfaz.
- **AN-045:** ergonomía y semántica de permisos (`allow_wasi_stdout()`, expansión `stdio`, manifiestos, suite negativa).
- Sin AN-045, AN-044 no es "abierto por defecto": cualquier interfaz no concedida sigue rechazada por `precheck_component_imports`.
- Fuera de alcance: generación de programas echo/cat desde Semantic IR (INIT-07-02).

## Contexto y Diseño

### 1. Interfaces WIT reales (no existe `wasi:cli/stdio`)
`stdio` es un nombre conceptual. Interfaces concretas (paquetes `wasi:cli` y `wasi:io`, 0.2.x):
- `wasi:cli/stdin`, `wasi:cli/stdout`, `wasi:cli/stderr`
- `wasi:io/streams`, `wasi:io/error`, `wasi:io/poll` (dependencias de tipo/recurso de los streams)

Componentes compilados con toolchains habituales (p. ej. Rust `wasm32-wasip2`) importan además `wasi:cli/environment`, `wasi:cli/exit` y `wasi:cli/terminal-*`. Esta WI **no** los concede ni implementa (diferidos por decisión); el baseline se valida con fixtures WIT mínimos que importan solo lo necesario (ver Estrategia de pruebas). El soporte de binarios reales queda como ampliación posterior con criterios propios.

### 2. Integración sin violar invariantes
- **ADR-002:** `compiler` e `ir` no dependen de `wasmtime-wasi`.
- **ADR-005:** un `WasiCtx` + `ResourceTable` nuevos por invocación; se destruyen con el Store.
- **Linker gobernado por política:** el linker se construye a partir del conjunto concedido; una interfaz no concedida no se registra. Es una segunda barrera tras el precheck (defensa en profundidad).
- **API síncrona:** el runtime es síncrono; usar `wasmtime_wasi::bindings::sync` y evitar tokio.
- Pinnear `wasmtime-wasi = "=29.0.1"` con `default-features = false`.

### 3. Streams en memoria
- `stdin`: `MemoryInputPipe`; sin entrada, EOF inmediato.
- `stdout`/`stderr`: `BoundedOutput` (`HostOutputStream` + `StdoutStream`) con **capacidad máxima** configurable. Al exceder, la escritura falla atómicamente y la invocación devuelve error tipado `RuntimeError::IoLimitExceeded` (sin truncado silencioso, ADR-004).
- Si `capture_*` es `false`, la salida se descarta en el sumidero pero sigue contabilizando contra `max_*_bytes`, nunca se hereda del host.

### 4. Qué miden (y qué no) los límites
- **Fuel:** solo instrucciones WebAssembly del guest. Las llamadas host de streams **no** se debitan; un bucle de escritura queda acotado porque cada iteración consume fuel guest.
- **Memoria guest** (`InstanceLimits`): memoria lineal y tablas del guest.
- **Memoria de búferes host:** acotada exclusivamente por los límites de I/O de esta WI.

## Contratos Públicos Entregados (`crates/runtime`)

```rust
#[derive(Debug, Clone, Default)]
pub struct WasiIoConfig {
    /// Bytes de stdin. `None` => EOF inmediato.
    pub stdin: Option<Vec<u8>>,
    pub capture_stdout: bool,
    pub capture_stderr: bool,
    /// Máximo de bytes retenidos por stream de salida (por defecto finito, 1 MiB).
    pub max_stdout_bytes: usize,
    pub max_stderr_bytes: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WasiIoOutput { pub stdout: Vec<u8>, pub stderr: Vec<u8> }

impl WasiIoOutput {
    pub fn stdout_utf8(&self) -> Result<&str, std::str::Utf8Error>;
    pub fn stderr_utf8(&self) -> Result<&str, std::str::Utf8Error>;
}

/// Compone el reporte existente para no duplicar campos.
#[derive(Debug, Clone)]
pub struct WasiInvocationReport {
    pub report: InvocationReport,
    pub io: WasiIoOutput,
}

impl Component {
    pub fn invoke_with_wasi(
        &self, policy: &HostPolicy, limits: &ExecutionLimits,
        func_name: &str, args: &[Val], io: WasiIoConfig,
    ) -> Result<WasiInvocationReport, RuntimeError>;
}
```

### Error tipado aprobado (ADR-004)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoStream { Stdout, Stderr }

RuntimeError::IoLimitExceeded {
    stream: IoStream,
    /// Límite configurado (`max_stdout_bytes` / `max_stderr_bytes`).
    limit: usize,
    /// Tamaño ACUMULADO que habría quedado retenido si la escritura rechazada se hubiera aceptado
    /// (= bytes ya retenidos + longitud de la escritura rechazada). Siempre > `limit`.
    attempted_bytes: usize,
}
```

Semántica inequívoca:
- Un total retenido igual a `limit` es válido; `limit + 1` se rechaza.
- La escritura rechazada es **atómica**: no retiene ningún byte de esa escritura (sin truncado parcial).
- Los bytes ya retenidos antes del rechazo no se devuelven en el reporte: la invocación falla con el error tipado.

### Propagación sin comparar texto
El stream de salida es una implementación propia y acotada de `HostOutputStream` (`BoundedOutput` en `crates/runtime/src/wasi.rs`). Al exceder el límite registra un `IoOverflow { stream, limit, attempted_bytes }` en estado compartido por invocación (`Arc<Mutex<Option<IoOverflow>>>`) y devuelve `StreamError::Trap(anyhow::Error::new(IoLimitTrap { ... }))`. Tras la ejecución, el runtime consulta primero el slot `overflow` y de forma secundaria `err.downcast_ref::<IoLimitTrap>()` para producir `RuntimeError::IoLimitExceeded`. Jamás se inspecciona `to_string()`.

## Resultados del Spike y Verificación (AC-S1..AC-S5 y AC-01..AC-07)

- **AC-S1 (Versiones y precheck antes de instanciar):** Verificado en `test_ac_s1_exact_wit_version_matching_and_patch_rejection`. `wasmtime-wasi 29.0.1` embebe WITs en versión `0.2.3` y el `Linker` de Wasmtime resuelve automáticamente importaciones `@0.2.x` compatibles (`@0.2.0` y `@0.2.3`), pero `precheck_component_imports` exige **igualdad exacta de cadenas** frente a `HostPolicy` antes de instanciar. Conceder `@0.2.0` e importar `@0.2.3` (o `@0.3.0`) produce `RuntimeError::UnauthorizedImport` sin ejecutar código guest.
- **AC-S2 (Registro individual en `wasmtime-wasi = "29.0.1"`):** Confirmado y cableado mediante `wasmtime_wasi::bindings::cli::{stdin,stdout,stderr}::add_to_linker_get_host` y `wasmtime_wasi::bindings::{io::error, sync::io::{streams,poll}}::add_to_linker_get_host`, condicionado exclusivamente a las interfaces presentes en el componente (que ya superaron `precheck_component_imports`). Prohibido `wasmtime_wasi::add_to_linker_sync`.
- **AC-S3 (Propagación tipada sin texto):** Verificado en `test_ac_05_and_s3_s4_io_limits_exact_overflow_accumulated_and_typed_propagation`. El texto de `IoLimitTrap::Display` es deliberadamente opaco (`"opaque-io-trap-marker"`) sin números ni nombres de stream, demostrando que `RuntimeError::IoLimitExceeded` proviene exclusivamente del estado tipado.
- **AC-S4 (`check-write` sin esperas infinitas):** `BoundedOutput::check_write` devuelve un presupuesto constante por llamada (`64 * 1024`) mientras el stream esté abierto, evitando que `blocking-write-and-flush` entre en espera infinita cuando el presupuesto restante es menor que el chunk del guest, y aplicando la verificación exacta acumulada en `write`.
- **AC-S5 / AC-01 (Rust 1.85 y ADR-002):** `wasmtime-wasi = { version = "=29.0.1", default-features = false }` compila limpiamente en Rust 1.85 Windows MSVC; `wat = "=1.221.3"` y `wasm-encoder = "=0.221.3"` fijados en `[dev-dependencies]`. `cargo tree -p agent-native-ir --edges normal` y `cargo tree -p agent-native-compiler --edges normal` confirman cero dependencias de `wasmtime-wasi`.
- **AC-02..AC-07:** Verificados en `crates/runtime/tests/wasi_adapter.rs` (7 tests en verde).
