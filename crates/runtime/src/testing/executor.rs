//! YAML executor driver (AN-019).
//!
//! Runs every case in a `Suite` against a loaded `Component`, builds a
//! `wasmtime::Linker<HostCtx>` from the suite's `host_stubs:` block, and
//! compares outputs + recorded host-call log against the case's
//! `expected` / `expected_host_calls` fields.
//!
//! Isolation: a fresh `HostCtx` is built for every case — initial `kv` is
//! cloned from `case.host_state`, `calls` starts empty. Combined with
//! Wasmtime's fresh `Store` per invocation, two cases cannot share state.

use wasmtime::component::Val;

use crate::engine::Component;
use crate::testing::errors::ExecutorError;
use crate::testing::host_ctx::{HostCtx, RecordedCall};
use crate::testing::report::{CaseResult, CaseStatus, SuiteReport};
use crate::testing::stub::{HostStubRegistry, StubKind};
use crate::testing::suite::{Case, Expected, ExpectedHostCall, Suite};

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

fn run_case(component: &Component, registry: &HostStubRegistry, case: &Case) -> CaseResult {
    let mut evidence = Vec::new();
    let host_ctx = HostCtx {
        kv: case.host_state.kv.clone(),
        calls: Vec::new(),
    };
    let args: Vec<Val> = case.args.iter().map(|v| Val::S32(*v)).collect();

    let outcome = if component.is_core_module() {
        run_core_case(component, registry, &case.function, &args, host_ctx)
    } else {
        run_component_case(component, registry, &case.function, &args, host_ctx)
    };

    let (results, final_ctx) = match outcome {
        Ok(pair) => pair,
        Err(e) => {
            evidence.push(format!("execution error: {e}"));
            return CaseResult {
                id: case.id.clone(),
                status: CaseStatus::Error,
                evidence,
            };
        }
    };

    let return_matches = match &case.expected {
        Expected::Primitive(expected_i32) => {
            check_primitive(&results, *expected_i32, &mut evidence)
        }
        Expected::Record(_) => {
            evidence.push(
                "record expectations are not supported in AN-019 MVP; use the primitive form"
                    .to_string(),
            );
            false
        }
    };

    let calls_match = match &case.expected_host_calls {
        Some(expected) => check_calls(&final_ctx.calls, expected, &mut evidence),
        None => true,
    };

    let status = if return_matches && calls_match {
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

fn run_component_case(
    _component: &Component,
    _registry: &HostStubRegistry,
    _func: &str,
    _args: &[Val],
    _host_ctx: HostCtx,
) -> Result<(Vec<Val>, HostCtx), ExecutorError> {
    // Component Model stub wiring arrives in AN-020 (per RM-07 split).
    // The pre-check still runs through `authorise_stubs`, so a suite that
    // targets a Component-Model artifact today is rejected cleanly.
    Err(ExecutorError::HostError(
        "Component-Model host stubs are not yet wired (see RM-07)".to_string(),
    ))
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
                    data.calls.push(RecordedCall {
                        function: name_for_log.clone(),
                        args: vec![arg0],
                    });
                    data.kv.get(&arg0).copied().unwrap_or(0)
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
                    let args_vec = vec![a0, a1];
                    let key = args_vec[key_arg];
                    let value = args_vec[value_arg];
                    data.calls.push(RecordedCall {
                        function: name_for_log.clone(),
                        args: args_vec,
                    });
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
                    caller.data_mut().calls.push(RecordedCall {
                        function: name_for_log.clone(),
                        args: Vec::new(),
                    });
                    returns
                },
            )?;
        }
    }
    Ok(())
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
        if rec.function != exp.function {
            evidence.push(format!(
                "host-call #{idx} function mismatch: expected '{}', got '{}'",
                exp.function, rec.function
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

impl HostStubRegistry {
    /// Snapshot for the Core executor path: `(interface, operation, kind)`
    /// triples that the linker builder iterates to register `func_wrap`
    /// closures.
    #[must_use]
    pub fn snapshot_for_core(&self) -> Vec<(String, String, StubKind)> {
        // Intentional allocation: the executor passes this vector into a
        // `move` closure (`build_linker`) so the registry itself can be
        // held by reference elsewhere.
        self.iter_entries()
            .map(|(k, v)| (k.0.clone(), k.1.clone(), *v))
            .collect()
    }
}
