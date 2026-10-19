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
    /// Append-only log of host calls, in invocation order. Each entry
    /// carries `suppressed = true` when the stub recorded the attempted
    /// call but did not execute its effect because the context was
    /// already poisoned (AN-020 patch 1 condition 3).
    pub calls: Vec<RecordedCall>,
    /// AN-020 patch 1 (D1 option A): the first `StateReader` miss sets
    /// `poisoned = true`. Subsequent stubs consult this flag before
    /// executing: `StateWriter` must not mutate `kv`; `Noop` must not
    /// contribute its nominal return effect. The executor enforces
    /// `CaseStatus::Error` on any case that ends with
    /// `poisoned == true`, even if the guest produces an apparently
    /// correct result. Observed effect: later host-side effects are
    /// suppressed; the guest itself may continue computing with a
    /// provisional `0` until the invocation returns.
    ///
    /// We **do not** claim this is a Wasmtime trap. Returning
    /// `Result::Err` from a `Linker::func_wrap` closure was observed to
    /// crash the test process on Windows with wasmtime 27
    /// (`STATUS_STACK_BUFFER_OVERRUN` from the `setjmp`/`longjmp` trap
    /// path interacting with the `extern "C"` trampoline). That is a
    /// recorded experimental finding, not a universally proven root
    /// cause; safe propagation of host-function errors remains a
    /// post-M3 investigation.
    pub poisoned: bool,
    /// List of recorded misses (first entry triggers `poisoned`).
    pub misses: Vec<ReaderMiss>,
}

/// A single `StateReader` miss recorded by the stub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReaderMiss {
    pub import: String,
    pub key: i32,
}

/// A single host call recorded for later comparison against
/// `expected_host_calls`. `suppressed` is `true` when the stub
/// executed its intake side (log the call) but skipped its effect
/// side because `HostCtx::poisoned` was already `true`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCall {
    pub function: String,
    pub args: Vec<i32>,
    pub suppressed: bool,
}
