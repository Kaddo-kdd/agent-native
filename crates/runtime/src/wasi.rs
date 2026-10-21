//! WASI Preview 2 adapter (INIT-07-01 / AN-044).
//!
//! Lets a Component Model artifact use deterministic, in-memory console I/O:
//! `stdin` is injected from memory, `stdout`/`stderr` are captured into bounded buffers. No
//! terminal, file descriptor or host resource is inherited.
//!
//! # Security model (ADR-004)
//!
//! - `wasi:cli/stdio` does **not** exist as a WIT interface; the concrete interfaces are
//!   `wasi:cli/stdin`, `wasi:cli/stdout`, `wasi:cli/stderr` and the supporting `wasi:io/*`.
//! - Authorisation is decided **before** instantiation by the existing pre-instantiate check
//!   ([`HostPolicy`]), with **exact** interface identifiers (version included). Wasmtime's
//!   linker may resolve compatible `@0.2.x` versions, but that never widens authorisation.
//! - The linker is populated **per interface**, only with supported interfaces that the
//!   component actually imports (and that therefore already passed the policy check).
//!   `wasmtime_wasi::add_to_linker_sync` — which would register filesystem, sockets, clocks and
//!   randomness — is never used.
//!
//! # Isolation (ADR-005) and limits
//!
//! Every invocation builds a fresh `Store`, `WasiCtx` and `ResourceTable`; nothing survives it.
//! Fuel and `InstanceLimits` keep applying. Fuel counts guest instructions only: host stream
//! calls are **not** debited, so host buffers are bounded explicitly by
//! [`WasiIoConfig::max_stdout_bytes`] / [`WasiIoConfig::max_stderr_bytes`].

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use wasmtime::Store;
use wasmtime::component::{Linker, ResourceTable, Val};
use wasmtime_wasi::pipe::{ClosedInputStream, MemoryInputPipe};
use wasmtime_wasi::{
    HostOutputStream, HostWallClock, StdoutStream, StreamError, StreamResult, Subscribe, WasiCtx,
    WasiCtxBuilder, WasiImpl, WasiView, async_trait,
};

use crate::engine::{Component, Payload};
use crate::error::{IoStream, RuntimeError};
use crate::invoke::{
    UNMETERED_FUEL_BUDGET, component_val_kind, component_val_matches_type, map_execution_error,
    map_instantiation_error, precheck_component_imports, resolve_component_func,
};
use crate::limits::{ExecutionLimits, InvocationReport, StoreResourceLimiter};
use crate::policy::HostPolicy;

/// Default per-stream output capacity: 1 MiB.
pub const DEFAULT_MAX_OUTPUT_BYTES: usize = 1024 * 1024;

/// Per-call write permit reported to the guest by the bounded output streams.
///
/// It is never zero, so a well-behaved guest never waits for capacity that will not come; the
/// accumulated limit is enforced in `write` and produces [`RuntimeError::IoLimitExceeded`].
const WRITE_PERMIT: usize = 4096;

/// I/O configuration for a single WASI invocation.
#[derive(Debug, Clone)]
pub struct WasiIoConfig {
    /// Bytes served as standard input. `None` means immediate end-of-file.
    pub stdin: Option<Vec<u8>>,
    /// Capture `stdout` in memory. When `false`, writes are accepted and discarded.
    pub capture_stdout: bool,
    /// Capture `stderr` in memory. When `false`, writes are accepted and discarded.
    pub capture_stderr: bool,
    /// Maximum bytes retained for `stdout` (only meaningful with `capture_stdout`).
    pub max_stdout_bytes: usize,
    /// Maximum bytes retained for `stderr` (only meaningful with `capture_stderr`).
    pub max_stderr_bytes: usize,
}

impl Default for WasiIoConfig {
    fn default() -> Self {
        Self {
            stdin: None,
            capture_stdout: false,
            capture_stderr: false,
            max_stdout_bytes: DEFAULT_MAX_OUTPUT_BYTES,
            max_stderr_bytes: DEFAULT_MAX_OUTPUT_BYTES,
        }
    }
}

impl WasiIoConfig {
    /// Configuration capturing both output streams with default limits and no stdin.
    #[must_use]
    pub fn capturing() -> Self {
        Self {
            capture_stdout: true,
            capture_stderr: true,
            ..Self::default()
        }
    }

    /// Sets the bytes served as standard input.
    #[must_use]
    pub fn with_stdin(mut self, stdin: impl Into<Vec<u8>>) -> Self {
        self.stdin = Some(stdin.into());
        self
    }

    /// Sets both output limits.
    #[must_use]
    pub fn with_max_output_bytes(mut self, max_stdout: usize, max_stderr: usize) -> Self {
        self.max_stdout_bytes = max_stdout;
        self.max_stderr_bytes = max_stderr;
        self
    }
}

/// Bytes captured from the guest during one invocation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WasiIoOutput {
    /// Bytes written to `stdout`.
    pub stdout: Vec<u8>,
    /// Bytes written to `stderr`.
    pub stderr: Vec<u8>,
}

impl WasiIoOutput {
    /// `stdout` as UTF-8.
    ///
    /// # Errors
    ///
    /// Returns the UTF-8 error if the bytes are not valid UTF-8.
    pub fn stdout_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.stdout)
    }

    /// `stderr` as UTF-8.
    ///
    /// # Errors
    ///
    /// Returns the UTF-8 error if the bytes are not valid UTF-8.
    pub fn stderr_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.stderr)
    }
}

/// Invocation report extended with the captured I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasiInvocationReport {
    /// Values and fuel accounting, identical to a regular invocation.
    pub report: InvocationReport,
    /// Captured console output.
    pub io: WasiIoOutput,
}

/// Record of the first output overflow observed during an invocation.
#[derive(Debug, Clone, Copy)]
struct IoOverflow {
    stream: IoStream,
    limit: usize,
    attempted_bytes: usize,
}

type OverflowSlot = Arc<Mutex<Option<IoOverflow>>>;

/// Marker carried inside the Wasmtime trap so the failure is typed, never string-matched.
#[derive(Debug, thiserror::Error)]
#[error("captured output limit exceeded")]
struct IoLimitTrap;

/// Bounded in-memory output stream (replaces `MemoryOutputPipe`, whose overflow is textual).
#[derive(Clone)]
struct BoundedOutput {
    stream: IoStream,
    capture: bool,
    limit: usize,
    buffer: Arc<Mutex<Vec<u8>>>,
    overflow: OverflowSlot,
}

impl BoundedOutput {
    fn new(stream: IoStream, capture: bool, limit: usize, overflow: OverflowSlot) -> Self {
        Self {
            stream,
            capture,
            limit,
            buffer: Arc::new(Mutex::new(Vec::new())),
            overflow,
        }
    }

    fn contents(&self) -> Vec<u8> {
        self.buffer
            .lock()
            .map_or_else(|poisoned| poisoned.into_inner().clone(), |b| b.clone())
    }

    fn overflowed(&self) -> bool {
        self.overflow.lock().is_ok_and(|slot| slot.is_some())
    }

    fn trap() -> StreamError {
        StreamError::Trap(anyhow_from(IoLimitTrap))
    }
}

fn anyhow_from(marker: IoLimitTrap) -> wasmtime::Error {
    wasmtime::Error::new(marker)
}

#[async_trait]
impl HostOutputStream for BoundedOutput {
    fn write(&mut self, bytes: Bytes) -> StreamResult<()> {
        if self.overflowed() {
            return Err(Self::trap());
        }
        if !self.capture {
            return Ok(());
        }
        let Ok(mut buffer) = self.buffer.lock() else {
            return Err(StreamError::trap("output buffer poisoned"));
        };
        let attempted = buffer.len().saturating_add(bytes.len());
        if attempted > self.limit {
            if let Ok(mut slot) = self.overflow.lock() {
                slot.get_or_insert(IoOverflow {
                    stream: self.stream,
                    limit: self.limit,
                    attempted_bytes: attempted,
                });
            }
            return Err(Self::trap());
        }
        buffer.extend_from_slice(&bytes);
        Ok(())
    }

    fn flush(&mut self) -> StreamResult<()> {
        if self.overflowed() {
            return Err(Self::trap());
        }
        Ok(())
    }

    fn check_write(&mut self) -> StreamResult<usize> {
        if self.overflowed() {
            return Err(Self::trap());
        }
        Ok(WRITE_PERMIT)
    }
}

#[async_trait]
impl Subscribe for BoundedOutput {
    async fn ready(&mut self) {}
}

impl StdoutStream for BoundedOutput {
    fn stream(&self) -> Box<dyn HostOutputStream> {
        Box::new(self.clone())
    }

    fn isatty(&self) -> bool {
        false
    }
}

/// Per-invocation store data: WASI context, resource table and the RM-06 limiter.
struct WasiStoreData {
    ctx: WasiCtx,
    table: ResourceTable,
    limiter: StoreResourceLimiter,
}

/// Fixed Unix epoch clock used by the command profile.
///
/// It prevents a guest from observing host time while satisfying the Rust 1.85
/// command component's required `wasi:clocks/wall-clock` import.
struct DeterministicWallClock;

impl HostWallClock for DeterministicWallClock {
    fn resolution(&self) -> Duration {
        Duration::from_nanos(1)
    }

    fn now(&self) -> Duration {
        Duration::ZERO
    }
}

impl WasiView for WasiStoreData {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

/// Strips the `@version` suffix. Only used to pick a host implementation; authorisation is
/// always decided on the exact identifier by [`HostPolicy`].
fn interface_base(name: &str) -> &str {
    name.split('@').next().unwrap_or(name)
}

// NB: workaround for rustc inference, mirrors `wasmtime_wasi::add_to_linker_sync`.
fn type_annotate<T: WasiView, F>(val: F) -> F
where
    F: Fn(&mut T) -> WasiImpl<&mut T>,
{
    val
}

/// Registers, one by one, the supported WASI interfaces that the component imports.
///
/// Called only after the pre-instantiate policy check succeeded, so every imported interface
/// has been authorised by exact identifier. Unsupported `wasi:*` interfaces are not provided and
/// fail at instantiation (explicit failure, no silent stub).
fn add_imported_interfaces(
    linker: &mut Linker<WasiStoreData>,
    imported: &[String],
) -> wasmtime::Result<()> {
    let closure = type_annotate::<WasiStoreData, _>(|t| WasiImpl(t));
    let link_options = wasmtime_wasi::bindings::sync::LinkOptions::default();
    let mut seen = HashSet::new();
    for name in imported {
        let base = interface_base(name);
        if !seen.insert(base.to_string()) {
            continue;
        }
        match base {
            "wasi:cli/stdin" => {
                wasmtime_wasi::bindings::cli::stdin::add_to_linker_get_host(linker, closure)?;
            }
            "wasi:cli/stdout" => {
                wasmtime_wasi::bindings::cli::stdout::add_to_linker_get_host(linker, closure)?;
            }
            "wasi:cli/stderr" => {
                wasmtime_wasi::bindings::cli::stderr::add_to_linker_get_host(linker, closure)?;
            }
            "wasi:cli/environment" => {
                wasmtime_wasi::bindings::cli::environment::add_to_linker_get_host(linker, closure)?;
            }
            "wasi:cli/exit" => {
                wasmtime_wasi::bindings::cli::exit::add_to_linker_get_host(
                    linker,
                    &link_options.clone().into(),
                    closure,
                )?;
            }
            "wasi:clocks/wall-clock" => {
                wasmtime_wasi::bindings::clocks::wall_clock::add_to_linker_get_host(
                    linker, closure,
                )?;
            }
            "wasi:filesystem/types" => {
                wasmtime_wasi::bindings::sync::filesystem::types::add_to_linker_get_host(
                    linker, closure,
                )?;
            }
            "wasi:filesystem/preopens" => {
                wasmtime_wasi::bindings::filesystem::preopens::add_to_linker_get_host(
                    linker, closure,
                )?;
            }
            "wasi:io/error" => {
                wasmtime_wasi::bindings::io::error::add_to_linker_get_host(linker, closure)?;
            }
            "wasi:io/streams" => {
                wasmtime_wasi::bindings::sync::io::streams::add_to_linker_get_host(
                    linker, closure,
                )?;
            }
            "wasi:io/poll" => {
                wasmtime_wasi::bindings::sync::io::poll::add_to_linker_get_host(linker, closure)?;
            }
            _ => {}
        }
    }
    Ok(())
}

impl Component {
    /// Invokes an exported function of a Component Model artifact with in-memory WASI console
    /// I/O under an explicit policy and execution limits.
    ///
    /// Every WASI interface the component imports must be authorised by `policy` using its
    /// exact identifier (version included); otherwise the call fails with
    /// [`RuntimeError::UnauthorizedImport`] **before** instantiation, so no guest code runs.
    ///
    /// # Errors
    ///
    /// - [`RuntimeError::UnauthorizedImport`] for any import not granted by `policy`.
    /// - [`RuntimeError::IoLimitExceeded`] when a captured stream would exceed its limit.
    /// - [`RuntimeError::FuelExhausted`] / [`RuntimeError::MemoryLimitExceeded`] as usual.
    /// - [`RuntimeError::ExecutionFailed`] if the artifact is a Core WASM module.
    pub fn invoke_with_wasi(
        &self,
        policy: &HostPolicy,
        limits: &ExecutionLimits,
        func_name: &str,
        args: &[Val],
        io: WasiIoConfig,
    ) -> Result<WasiInvocationReport, RuntimeError> {
        let Payload::Component(component) = &self.payload else {
            return Err(RuntimeError::ExecutionFailed(
                "invoke_with_wasi requires a Component Model artifact".to_string(),
            ));
        };
        invoke_component_wasi(&self.engine, component, policy, limits, func_name, args, io)
    }
}

#[allow(clippy::too_many_lines)]
fn invoke_component_wasi(
    engine: &wasmtime::Engine,
    component: &wasmtime::component::Component,
    policy: &HostPolicy,
    limits: &ExecutionLimits,
    func_name: &str,
    args: &[Val],
    io: WasiIoConfig,
) -> Result<WasiInvocationReport, RuntimeError> {
    // 1. Authorisation first (exact identifiers). Nothing below runs on denial.
    precheck_component_imports(engine, component, policy)?;

    // 2. Per-interface linker.
    let imported: Vec<String> = component
        .component_type()
        .imports(engine)
        .filter(|(_, item)| {
            matches!(
                item,
                wasmtime::component::types::ComponentItem::ComponentInstance(_)
            )
        })
        .map(|(name, _)| name.to_string())
        .collect();
    let mut linker: Linker<WasiStoreData> = Linker::new(engine);
    add_imported_interfaces(&mut linker, &imported)
        .map_err(|e| RuntimeError::InstantiationFailed(e.to_string()))?;

    // 3. Fresh, isolated I/O state for this invocation.
    // Only attach stream handles for interfaces the component actually imports (and that
    // already passed the `HostPolicy` precheck). Auxiliary `wasi:io/*` interfaces alone
    // never populate stdin/stdout/stderr handles in `WasiCtx` (AN-045 AC-06).
    let has_stdin = imported
        .iter()
        .any(|name| interface_base(name) == "wasi:cli/stdin");
    let has_stdout = imported
        .iter()
        .any(|name| interface_base(name) == "wasi:cli/stdout");
    let has_stderr = imported
        .iter()
        .any(|name| interface_base(name) == "wasi:cli/stderr");

    let overflow: OverflowSlot = Arc::default();
    let stdout = BoundedOutput::new(
        IoStream::Stdout,
        io.capture_stdout && has_stdout,
        io.max_stdout_bytes,
        Arc::clone(&overflow),
    );
    let stderr = BoundedOutput::new(
        IoStream::Stderr,
        io.capture_stderr && has_stderr,
        io.max_stderr_bytes,
        Arc::clone(&overflow),
    );
    let mut builder = WasiCtxBuilder::new();
    builder.wall_clock(DeterministicWallClock);
    match io.stdin.filter(|_| has_stdin) {
        Some(bytes) => builder.stdin(MemoryInputPipe::new(Bytes::from(bytes))),
        None => builder.stdin(ClosedInputStream),
    };
    if has_stdout {
        builder.stdout(stdout.clone());
    }
    if has_stderr {
        builder.stderr(stderr.clone());
    }

    let data = WasiStoreData {
        ctx: builder.build(),
        table: ResourceTable::new(),
        limiter: StoreResourceLimiter {
            limits: limits.instance_limits,
            last_growth_denial: None,
        },
    };
    let mut store = Store::new(engine, data);
    store.limiter(|d| &mut d.limiter);
    let _ = store.set_fuel(limits.fuel_budget.unwrap_or(UNMETERED_FUEL_BUDGET));

    let typed_overflow = |fallback: RuntimeError| -> RuntimeError {
        match overflow.lock().ok().and_then(|slot| *slot) {
            Some(o) => RuntimeError::IoLimitExceeded {
                stream: o.stream,
                limit: o.limit,
                attempted_bytes: o.attempted_bytes,
            },
            None => fallback,
        }
    };

    let instance = linker.instantiate(&mut store, component).map_err(|e| {
        typed_overflow(map_instantiation_error(
            &e,
            store.data().limiter.last_growth_denial,
        ))
    })?;

    let func = resolve_component_func(&instance, &mut store, func_name)
        .ok_or_else(|| RuntimeError::FunctionNotFound(func_name.to_string()))?;

    let params = func.params(&store);
    if params.len() != args.len() {
        return Err(RuntimeError::ArityMismatch {
            name: func_name.to_string(),
            expected: params.len(),
            got: args.len(),
        });
    }
    for (index, ((_name, expected_ty), provided)) in params.iter().zip(args.iter()).enumerate() {
        if !component_val_matches_type(provided, expected_ty) {
            return Err(RuntimeError::ArgTypeMismatch {
                function: func_name.to_string(),
                index,
                expected: format!("{expected_ty:?}"),
                found: component_val_kind(provided).to_string(),
            });
        }
    }

    let mut results = vec![Val::Bool(false); func.results(&store).len()];
    func.call(&mut store, args, &mut results).map_err(|e| {
        typed_overflow(map_execution_error(
            &e,
            limits.fuel_budget,
            store.data().limiter.last_growth_denial,
        ))
    })?;
    func.post_return(&mut store).map_err(|e| {
        typed_overflow(map_execution_error(
            &e,
            limits.fuel_budget,
            store.data().limiter.last_growth_denial,
        ))
    })?;

    let remaining = store.get_fuel().unwrap_or(0);
    let fuel_consumed = limits.fuel_budget.map(|b| b.saturating_sub(remaining));

    Ok(WasiInvocationReport {
        report: InvocationReport {
            values: results,
            fuel_consumed,
        },
        io: WasiIoOutput {
            stdout: stdout.contents(),
            stderr: stderr.contents(),
        },
    })
}
