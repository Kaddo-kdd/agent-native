//! `HostCtx`: per-case mutable state carried by `Store<T>` (AN-019).
//!
//! Lives inside Wasmtime's `Store` so that `Linker::func_wrap` closures
//! (which must be `Send + Sync`) can access it via
//! `caller.data_mut()` without `Rc`/`RefCell`. One `HostCtx` is built
//! afresh per case; this is how the executor guarantees isolation
//! between cases (ADR-005 baseline operativa + AN-019 isolation).

use std::collections::BTreeMap;

/// Mutable state for one case execution.
#[derive(Debug, Clone, Default)]
pub struct HostCtx {
    /// Key-value state read/written by `StateReader` / `StateWriter`
    /// stubs. Keys and values are `i32` only in MVP (OQ-06).
    pub kv: BTreeMap<i32, i32>,
    /// Append-only log of host calls, in invocation order.
    pub calls: Vec<RecordedCall>,
    /// AN-020 H3: when a `StateReader` is called with a key missing
    /// from `kv`, the stub records the miss here instead of silently
    /// returning 0. The executor converts a non-empty `misses` into a
    /// `CaseStatus::Error` with evidence that names each key, matching
    /// the fail-explicit contract of ADR-004.
    ///
    /// Why this and not an `Err` from the closure: on Windows + wasmtime
    /// 27 the setjmp/longjmp trap path triggers a `STATUS_STACK_BUFFER_OVERRUN`
    /// fast-fail in debug builds. Flagging the miss in `HostCtx` keeps
    /// the behaviour observable, deterministic and portable.
    pub misses: Vec<ReaderMiss>,
}

/// A single `StateReader` miss recorded by the stub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReaderMiss {
    pub import: String,
    pub key: i32,
}

/// A single host call recorded for later comparison against
/// `expected_host_calls`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCall {
    pub function: String,
    pub args: Vec<i32>,
}
