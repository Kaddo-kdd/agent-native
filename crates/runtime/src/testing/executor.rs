//! YAML executor driver (AN-019 + AN-020).
//!
//! Runs every case in a `Suite` against a loaded `Component`, builds a
//! `wasmtime::Linker<HostCtx>` from the suite's `host_stubs:` block, and
//! compares outputs + recorded host-call log against the case's
//! `expected` / `expected_host_calls` fields.
//!
//! Isolation: a fresh `HostCtx` is built for every case — initial `kv` is
//! cloned from `case.host_state`, `calls` starts empty. Combined with
//! Wasmtime's fresh `Store` per invocation, two cases cannot share state.
//!
//! AN-020 changes vs AN-019:
//!
//! - `Expected::Record` compared by name + type + value, detecting both
//!   missing and extra fields (H2).
//! - `StateReader` miss (`host_state.kv` key absent) traps with a typed
//!   `wasmtime::Error` surfaced as `CaseStatus::Error` (H3).
//! - Component-Model artifacts without functional imports (type-only or
//!   none) delegate to `Component::invoke`; artifacts with functional
//!   imports keep returning a deferred-to-RM-07 error (H1).
//! - `expected_host_calls[*].import` is the canonical field; the
//!   recorded call name (`interface.operation`) is matched short or
//!   qualified per the H4 rule.

use std::collections::BTreeMap;

use wasmtime::component::{Type, Val};

use crate::engine::{Component, Payload};
use crate::limits::ExecutionLimits;
use crate::policy::{
    WASI_CLI_STDERR_0_2_0, WASI_CLI_STDIN_0_2_0, WASI_CLI_STDOUT_0_2_0, WASI_IO_ERROR_0_2_0,
    WASI_IO_POLL_0_2_0, WASI_IO_STREAMS_0_2_0,
};
use crate::testing::errors::ExecutorError;
use crate::testing::host_ctx::{HostCtx, ReaderMiss, RecordedCall};
use crate::testing::report::{CaseResult, CaseStatus, SuiteReport};
use crate::testing::stub::{HostStubRegistry, StubKind};
use crate::testing::suite::{
    ArgValue, Case, Expected, ExpectedHostCall, ExpectedResult, ExpectedScalar, Suite,
};
use crate::wasi::WasiIoConfig;

const SUPPORTED_WASI_INTERFACES: &[&str] = &[
    WASI_CLI_STDIN_0_2_0,
    WASI_CLI_STDOUT_0_2_0,
    WASI_CLI_STDERR_0_2_0,
    WASI_IO_ERROR_0_2_0,
    WASI_IO_POLL_0_2_0,
    WASI_IO_STREAMS_0_2_0,
];

/// Executes a full suite and returns a report. Never panics on case
/// failures: failing cases become `CaseStatus::Fail`, structural errors
/// become `CaseStatus::Error` with the message in `evidence`.
///
/// # Errors
///
/// Returns an [`ExecutorError`] only for suite-wide problems: an
/// unauthorised stub declaration or a signature mismatch detected before
/// any case runs. Per-case failures are embedded in the returned
/// [`SuiteReport`] instead of surfacing as `Err`.
pub fn run_suite(component: &Component, suite: &Suite) -> Result<SuiteReport, ExecutorError> {
    let registry = build_registry(suite)?;
    authorise_stubs(component, suite, &registry)?;

    let mut report = SuiteReport::new(suite.suite.clone());
    for case in &suite.cases {
        report.push(run_case(component, &registry, case));
    }
    Ok(report)
}

fn build_registry(suite: &Suite) -> Result<HostStubRegistry, ExecutorError> {
    let mut registry = HostStubRegistry::new();
    for decl in &suite.host_stubs {
        let kind: StubKind = decl.kind.clone().into();
        validate_stub_indices(&decl.interface, &decl.operation, &kind)?;
        registry.register(&decl.interface, &decl.operation, kind);
    }
    Ok(registry)
}

fn validate_stub_indices(
    interface: &str,
    operation: &str,
    kind: &StubKind,
) -> Result<(), ExecutorError> {
    match *kind {
        StubKind::StateReader { key_arg } => {
            if key_arg >= 1 {
                return Err(ExecutorError::SignatureMismatch {
                    interface: interface.to_string(),
                    operation: operation.to_string(),
                    declared_kind: "state_reader".to_string(),
                });
            }
        }
        StubKind::StateWriter { key_arg, value_arg } => {
            if key_arg >= 2 || value_arg >= 2 || key_arg == value_arg {
                return Err(ExecutorError::SignatureMismatch {
                    interface: interface.to_string(),
                    operation: operation.to_string(),
                    declared_kind: "state_writer".to_string(),
                });
            }
        }
        StubKind::Noop { .. } => {}
    }
    Ok(())
}

/// Enforces the "stubs do not authorise" rule: every stub must target an
/// import the policy already authorises. The `HostPolicy` carried by the
/// `Component` is consulted using both the Core WASM shape
/// `(module, name)` and the Component Model shape `(interface, operation)`.
fn authorise_stubs(
    component: &Component,
    suite: &Suite,
    _registry: &HostStubRegistry,
) -> Result<(), ExecutorError> {
    let policy = &component.policy;
    for decl in &suite.host_stubs {
        let allowed = policy.is_allowed_core(&decl.interface, &decl.operation)
            || policy.is_wit_interface_allowed(&decl.interface)
            || policy.is_wit_operation_allowed(&decl.interface, &decl.operation);
        if !allowed {
            return Err(ExecutorError::UnauthorizedByPolicy {
                interface: decl.interface.clone(),
                operation: decl.operation.clone(),
            });
        }
    }
    Ok(())
}

struct CaseOutcome {
    results: Vec<Val>,
    host_ctx: HostCtx,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn check_console_stream(
    stream_name: &str,
    expected: Option<&String>,
    captured: &[u8],
    evidence: &mut Vec<String>,
) -> bool {
    let Some(expected_str) = expected else {
        return true;
    };
    let got_str = String::from_utf8_lossy(captured);
    if got_str.as_ref() == expected_str.as_str() {
        true
    } else {
        evidence.push(format!(
            "expected {stream_name} {expected_str:?}, got {:?}",
            got_str.as_ref()
        ));
        false
    }
}

fn run_case(component: &Component, registry: &HostStubRegistry, case: &Case) -> CaseResult {
    let mut evidence = Vec::new();
    let host_ctx = HostCtx {
        kv: case.host_state.kv.clone(),
        calls: Vec::new(),
        poisoned: false,
        misses: Vec::new(),
    };
    let args: Vec<Val> = case.args.iter().map(arg_to_val).collect();

    let outcome = if component.is_core_module() {
        run_core_case(component, registry, &case.function, &args, host_ctx).map(|(results, ctx)| {
            CaseOutcome {
                results,
                host_ctx: ctx,
                stdout: Vec::new(),
                stderr: Vec::new(),
            }
        })
    } else {
        run_component_case(
            component,
            &case.function,
            &args,
            host_ctx,
            case.stdin.as_deref(),
            case.expected_stdout.is_some() || case.expected_stderr.is_some(),
        )
    };

    let CaseOutcome {
        results,
        host_ctx: final_ctx,
        stdout: captured_stdout,
        stderr: captured_stderr,
    } = match outcome {
        Ok(out) => out,
        Err(e) => {
            evidence.push(format!("execution error: {e}"));
            return CaseResult {
                id: case.id.clone(),
                status: CaseStatus::Error,
                evidence,
            };
        }
    };

    if final_ctx.poisoned {
        for miss in &final_ctx.misses {
            evidence.push(format!(
                "state_reader '{}' miss: key={}",
                miss.import, miss.key
            ));
        }
        for call in final_ctx.calls.iter().filter(|c| c.suppressed) {
            evidence.push(format!(
                "suppressed host call: {}({:?})",
                call.function, call.args
            ));
        }
        return CaseResult {
            id: case.id.clone(),
            status: CaseStatus::Error,
            evidence,
        };
    }

    let return_matches = match &case.expected {
        Expected::Bool(expected_bool) => check_bool(&results, *expected_bool, &mut evidence),
        Expected::Primitive(expected_i32) => {
            check_primitive(&results, *expected_i32, &mut evidence)
        }
        Expected::String(expected_str) => check_string(&results, expected_str, &mut evidence),
        Expected::Result(expected_res) => check_result(&results, expected_res, &mut evidence),
        Expected::Record(expected_fields) => check_record(&results, expected_fields, &mut evidence),
    };

    let calls_match = match &case.expected_host_calls {
        Some(expected) => check_calls(&final_ctx.calls, expected, &mut evidence),
        None => true,
    };

    let stdout_matches = check_console_stream(
        "stdout",
        case.expected_stdout.as_ref(),
        &captured_stdout,
        &mut evidence,
    );
    let stderr_matches = check_console_stream(
        "stderr",
        case.expected_stderr.as_ref(),
        &captured_stderr,
        &mut evidence,
    );

    let status = if return_matches && calls_match && stdout_matches && stderr_matches {
        CaseStatus::Pass
    } else {
        CaseStatus::Fail
    };
    CaseResult {
        id: case.id.clone(),
        status,
        evidence,
    }
}

fn arg_to_val(a: &ArgValue) -> Val {
    match a {
        ArgValue::I32(i) => Val::S32(*i),
        ArgValue::Bool(b) => Val::Bool(*b),
        ArgValue::String(s) => Val::String(s.clone()),
    }
}

fn run_core_case(
    component: &Component,
    registry: &HostStubRegistry,
    func: &str,
    args: &[Val],
    host_ctx: HostCtx,
) -> Result<(Vec<Val>, HostCtx), ExecutorError> {
    let stubs: Vec<(String, String, StubKind)> = registry.snapshot_for_core().into_iter().collect();

    let (results, ctx) = component
        .invoke_core_with_host_ctx(func, args, host_ctx, move |_engine, linker| {
            for (interface, operation, kind) in &stubs {
                register_core_stub(linker, interface, operation, *kind)?;
            }
            Ok(())
        })
        .map_err(ExecutorError::from)?;
    Ok((results, ctx))
}

/// Component Model path.
///
/// AN-020 H1 + AN-049 (INIT-07-03):
/// - If a CM artifact imports WASI 0.2 console interfaces (`SUPPORTED_WASI_INTERFACES`)
///   or the case specifies deterministic console I/O (`stdin` / `expected_stdout` /
///   `expected_stderr`), executes via `Component::invoke_with_wasi` with in-memory pipes.
/// - If a CM artifact has no functional imports and no console I/O expectations, delegates
///   to `Component::invoke`.
/// - Non-WASI functional imports without a stub continue to return a typed error.
fn run_component_case(
    component: &Component,
    func: &str,
    args: &[Val],
    host_ctx: HostCtx,
    stdin_input: Option<&str>,
    needs_console_capture: bool,
) -> Result<CaseOutcome, ExecutorError> {
    if let Some(blocker) = first_non_wasi_functional_import(component) {
        return Err(ExecutorError::HostError(format!(
            "functional import '{blocker}' requires a stub (RM-07)"
        )));
    }
    if stdin_input.is_some() || needs_console_capture || has_supported_wasi_import(component) {
        let mut wasi_cfg = WasiIoConfig::capturing();
        if let Some(s) = stdin_input {
            wasi_cfg = wasi_cfg.with_stdin(s.as_bytes().to_vec());
        }
        let report = component
            .invoke_with_wasi(
                &component.policy,
                &ExecutionLimits::unmetered(),
                func,
                args,
                wasi_cfg,
            )
            .map_err(ExecutorError::from)?;
        return Ok(CaseOutcome {
            results: report.report.values,
            host_ctx,
            stdout: report.io.stdout,
            stderr: report.io.stderr,
        });
    }
    let results = component.invoke(func, args).map_err(ExecutorError::from)?;
    Ok(CaseOutcome {
        results,
        host_ctx,
        stdout: Vec::new(),
        stderr: Vec::new(),
    })
}

fn has_supported_wasi_import(component: &Component) -> bool {
    let Payload::Component(ref wt_component) = component.payload else {
        return false;
    };
    wt_component
        .component_type()
        .imports(&component.engine)
        .any(|(name, _)| SUPPORTED_WASI_INTERFACES.contains(&name))
}

/// Returns the first non-WASI functional import (if any) found in a Component
/// Model payload.
fn first_non_wasi_functional_import(component: &Component) -> Option<String> {
    use wasmtime::component::types::ComponentItem;
    let Payload::Component(ref wt_component) = component.payload else {
        return None;
    };
    for (name, item) in wt_component.component_type().imports(&component.engine) {
        if SUPPORTED_WASI_INTERFACES.contains(&name) {
            continue;
        }
        match item {
            ComponentItem::ComponentFunc(_) => return Some(name.to_string()),
            ComponentItem::ComponentInstance(inst_ty) => {
                for (op_name, op_item) in inst_ty.exports(&component.engine) {
                    if matches!(op_item, ComponentItem::ComponentFunc(_)) {
                        return Some(format!("{name}#{op_name}"));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn register_core_stub(
    linker: &mut wasmtime::Linker<HostCtx>,
    interface: &str,
    operation: &str,
    kind: StubKind,
) -> Result<(), wasmtime::Error> {
    let interface_owned = interface.to_string();
    let operation_owned = operation.to_string();
    match kind {
        StubKind::StateReader { key_arg: _ } => {
            let name_for_log = format!("{interface}.{operation}");
            linker.func_wrap(
                interface,
                operation,
                move |mut caller: wasmtime::Caller<'_, HostCtx>, arg0: i32| -> i32 {
                    let data = caller.data_mut();
                    let suppressed = data.poisoned;
                    data.calls.push(RecordedCall {
                        function: name_for_log.clone(),
                        args: vec![arg0],
                        suppressed,
                    });
                    if suppressed {
                        return 0;
                    }
                    if let Some(v) = data.kv.get(&arg0).copied() {
                        v
                    } else {
                        data.misses.push(ReaderMiss {
                            import: name_for_log.clone(),
                            key: arg0,
                        });
                        data.poisoned = true;
                        0
                    }
                },
            )?;
        }
        StubKind::StateWriter { key_arg, value_arg } => {
            let name_for_log = format!("{interface_owned}.{operation_owned}");
            linker.func_wrap(
                interface,
                operation,
                move |mut caller: wasmtime::Caller<'_, HostCtx>, a0: i32, a1: i32| -> i32 {
                    let data = caller.data_mut();
                    let suppressed = data.poisoned;
                    let args_vec = vec![a0, a1];
                    let key = args_vec[key_arg];
                    let value = args_vec[value_arg];
                    data.calls.push(RecordedCall {
                        function: name_for_log.clone(),
                        args: args_vec,
                        suppressed,
                    });
                    if suppressed {
                        return 0;
                    }
                    data.kv.insert(key, value);
                    value
                },
            )?;
        }
        StubKind::Noop { returns } => {
            let name_for_log = format!("{interface_owned}.{operation_owned}");
            linker.func_wrap(
                interface,
                operation,
                move |mut caller: wasmtime::Caller<'_, HostCtx>| -> i32 {
                    let data = caller.data_mut();
                    let suppressed = data.poisoned;
                    data.calls.push(RecordedCall {
                        function: name_for_log.clone(),
                        args: Vec::new(),
                        suppressed,
                    });
                    if suppressed { 0 } else { returns }
                },
            )?;
        }
    }
    Ok(())
}

fn check_bool(results: &[Val], expected: bool, evidence: &mut Vec<String>) -> bool {
    match results.first() {
        Some(Val::Bool(got)) if *got == expected => true,
        Some(Val::S32(got)) if (*got != 0) == expected => true,
        Some(other) => {
            evidence.push(format!("expected Bool({expected}), got {other:?}"));
            false
        }
        None => {
            evidence.push(format!("expected Bool({expected}), got no results"));
            false
        }
    }
}

fn check_primitive(results: &[Val], expected: i32, evidence: &mut Vec<String>) -> bool {
    match results.first() {
        Some(Val::S32(got)) if *got == expected => true,
        Some(Val::S32(got)) => {
            evidence.push(format!("expected S32({expected}), got S32({got})"));
            false
        }
        Some(other) => {
            evidence.push(format!(
                "expected S32({expected}), got non-S32 result: {other:?}"
            ));
            false
        }
        None => {
            evidence.push(format!("expected S32({expected}), got no results"));
            false
        }
    }
}

fn check_string(results: &[Val], expected: &str, evidence: &mut Vec<String>) -> bool {
    match results.first() {
        Some(Val::String(got)) if got == expected => true,
        Some(Val::String(got)) => {
            evidence.push(format!(
                "expected String({expected:?}), got String({got:?})"
            ));
            false
        }
        Some(other) => {
            evidence.push(format!(
                "expected String({expected:?}), got non-String result: {other:?}"
            ));
            false
        }
        None => {
            evidence.push(format!("expected String({expected:?}), got no results"));
            false
        }
    }
}

fn check_scalar(got: Option<&Val>, expected: &ExpectedScalar) -> bool {
    match (got, expected) {
        (Some(Val::Bool(b)), ExpectedScalar::Bool(eb)) => b == eb,
        (Some(Val::S32(i)), ExpectedScalar::I32(ei)) => i == ei,
        (Some(Val::String(s)), ExpectedScalar::String(es)) => s == es,
        _ => false,
    }
}

fn check_result(results: &[Val], expected: &ExpectedResult, evidence: &mut Vec<String>) -> bool {
    match (results.first(), expected) {
        (Some(Val::Result(Ok(inner))), ExpectedResult::Ok(exp_ok)) => {
            if check_scalar(inner.as_deref(), exp_ok) {
                true
            } else {
                evidence.push(format!("expected Ok({exp_ok:?}), got Ok({inner:?})"));
                false
            }
        }
        (Some(Val::Result(Err(inner))), ExpectedResult::Err(exp_err)) => {
            if check_scalar(inner.as_deref(), exp_err) {
                true
            } else {
                evidence.push(format!("expected Err({exp_err:?}), got Err({inner:?})"));
                false
            }
        }
        (Some(other), _) => {
            evidence.push(format!("expected Result({expected:?}), got {other:?}"));
            false
        }
        (None, _) => {
            evidence.push(format!("expected Result({expected:?}), got no results"));
            false
        }
    }
}

/// AN-020 H2: compare the first result's `Val::Record` fields by name,
/// type and value. Detect both missing (expected but not in result) and
/// extra (in result but not expected) fields.
fn check_record(
    results: &[Val],
    expected: &BTreeMap<String, i32>,
    evidence: &mut Vec<String>,
) -> bool {
    let Some(first) = results.first() else {
        evidence.push("expected record result, got no results".to_string());
        return false;
    };
    let Val::Record(got_fields) = first else {
        evidence.push(format!(
            "expected record result, got {}",
            crate::testing::executor::val_kind(first)
        ));
        return false;
    };

    let mut got_map: BTreeMap<String, Val> = BTreeMap::new();
    for (name, value) in got_fields {
        got_map.insert(name.clone(), value.clone());
    }

    let mut ok = true;
    for (name, expected_value) in expected {
        match got_map.remove(name) {
            Some(Val::S32(got_value)) => {
                if got_value != *expected_value {
                    evidence.push(format!(
                        "record field '{name}': expected S32({expected_value}), got S32({got_value})"
                    ));
                    ok = false;
                }
            }
            Some(other) => {
                evidence.push(format!(
                    "record field '{name}': expected S32({expected_value}), got {}",
                    val_kind(&other)
                ));
                ok = false;
            }
            None => {
                evidence.push(format!("expected field '{name}' missing in result record"));
                ok = false;
            }
        }
    }
    for (name, _) in got_map {
        evidence.push(format!("unexpected field '{name}' in result record"));
        ok = false;
    }
    ok
}

/// Short kebab-style tag for a `Val`. Mirrors the private helper in
/// `invoke.rs`; duplicated here because that one is `pub(crate)` and the
/// executor only needs the names.
fn val_kind(v: &Val) -> &'static str {
    match v {
        Val::Bool(_) => "bool",
        Val::S8(_) => "s8",
        Val::U8(_) => "u8",
        Val::S16(_) => "s16",
        Val::U16(_) => "u16",
        Val::S32(_) => "s32",
        Val::U32(_) => "u32",
        Val::S64(_) => "s64",
        Val::U64(_) => "u64",
        Val::Float32(_) => "float32",
        Val::Float64(_) => "float64",
        Val::Char(_) => "char",
        Val::String(_) => "string",
        Val::List(_) => "list",
        Val::Record(_) => "record",
        Val::Tuple(_) => "tuple",
        Val::Variant(_, _) => "variant",
        Val::Enum(_) => "enum",
        Val::Option(_) => "option",
        Val::Result(_) => "result",
        Val::Flags(_) => "flags",
        Val::Resource(_) => "resource",
    }
}

fn check_calls(
    recorded: &[RecordedCall],
    expected: &[ExpectedHostCall],
    evidence: &mut Vec<String>,
) -> bool {
    if recorded.len() != expected.len() {
        evidence.push(format!(
            "host-call count mismatch: expected {}, got {}",
            expected.len(),
            recorded.len()
        ));
        return false;
    }
    for (idx, (rec, exp)) in recorded.iter().zip(expected.iter()).enumerate() {
        if !import_matches(&exp.import, &rec.function) {
            evidence.push(format!(
                "host-call #{idx} import mismatch: expected '{}', got '{}'",
                exp.import, rec.function
            ));
            return false;
        }
        if rec.args != exp.args {
            evidence.push(format!(
                "host-call #{idx} args mismatch: expected {:?}, got {:?}",
                exp.args, rec.args
            ));
            return false;
        }
    }
    true
}

/// AN-020 H4 normalisation rule for `expected_host_calls[*].import`:
///
/// - If `expected` contains a `.`, match literally against `recorded`
///   (which is always `interface.operation`).
/// - Otherwise match against the operation part of `recorded` only
///   (strip prefix up to the first `.`).
fn import_matches(expected: &str, recorded: &str) -> bool {
    if expected.contains('.') {
        return expected == recorded;
    }
    match recorded.split_once('.') {
        Some((_iface, op)) => op == expected,
        None => recorded == expected,
    }
}

impl HostStubRegistry {
    /// Snapshot for the Core executor path: `(interface, operation, kind)`
    /// triples that the linker builder iterates to register `func_wrap`
    /// closures.
    #[must_use]
    pub fn snapshot_for_core(&self) -> Vec<(String, String, StubKind)> {
        self.iter_entries()
            .map(|(k, v)| (k.0.clone(), k.1.clone(), *v))
            .collect()
    }
}

// Silence unused-import if `Type` isn't needed in this build.
#[allow(dead_code)]
fn _type_import_sanity(_: Type) {}
