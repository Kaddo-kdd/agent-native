//! AN-019 end-to-end: compile IR → load component → run YAML suites.
//!
//! Covers acceptance criteria 4 (≥10 cases across ≥3 suites + ≥1 with
//! `host_stubs`), 5 (stub kinds), 6 (negative taxonomy), 7 (isolation),
//! 8 (`expected_host_calls` order+args), 9 (`SuiteReport` serialisable).

use agent_native_compiler::compile_ir_to_wasm;
use agent_native_runtime::testing::{
    CaseStatus, ExecutorError, HostStubRegistry, StubKind, parse_suite, run_suite,
};
use agent_native_runtime::{Engine, HostPolicy, RuntimeError, Val};
use serde_json::json;

// ---------- IR factories ---------------------------------------------------

fn identity_ir() -> serde_json::Value {
    json!({
        "ir_version": "0.1",
        "spec_version": 1,
        "functions": [{
            "name": "identity",
            "export": true,
            "params": [{"name": "value", "type": "i32"}],
            "returns": "i32",
            "requirements": ["SMOKE-001"],
            "body": {"op": "param", "name": "value"}
        }]
    })
}

fn kv_read_ir() -> serde_json::Value {
    json!({
        "ir_version": "0.3",
        "spec_version": 1,
        "capabilities": ["kv_store"],
        "imports": [{
            "module": "host",
            "name": "kv_get",
            "capability": "kv_store",
            "params": [{"name": "key", "type": "i32"}],
            "returns": "i32"
        }],
        "functions": [{
            "name": "read_key",
            "export": true,
            "params": [{"name": "key", "type": "i32"}],
            "returns": "i32",
            "requirements": ["KV-READ-001"],
            "body": {
                "op": "call_import",
                "import": "kv_get",
                "args": [{"op": "param", "name": "key"}]
            }
        }]
    })
}

fn kv_write_ir() -> serde_json::Value {
    json!({
        "ir_version": "0.3",
        "spec_version": 1,
        "capabilities": ["kv_store"],
        "imports": [{
            "module": "host",
            "name": "kv_set",
            "capability": "kv_store",
            "params": [
                {"name": "key", "type": "i32"},
                {"name": "value", "type": "i32"}
            ],
            "returns": "i32"
        }],
        "functions": [{
            "name": "write_key",
            "export": true,
            "params": [
                {"name": "k", "type": "i32"},
                {"name": "v", "type": "i32"}
            ],
            "returns": "i32",
            "requirements": ["KV-WRITE-001"],
            "body": {
                "op": "call_import",
                "import": "kv_set",
                "args": [
                    {"op": "param", "name": "k"},
                    {"op": "param", "name": "v"}
                ]
            }
        }]
    })
}

fn noop_ir() -> serde_json::Value {
    json!({
        "ir_version": "0.3",
        "spec_version": 1,
        "capabilities": ["ambient"],
        "imports": [{
            "module": "host",
            "name": "ping",
            "capability": "ambient",
            "params": [],
            "returns": "i32"
        }],
        "functions": [{
            "name": "trigger",
            "export": true,
            "params": [],
            "returns": "i32",
            "requirements": ["NOOP-001"],
            "body": {
                "op": "call_import",
                "import": "ping",
                "args": []
            }
        }]
    })
}

fn load_core(ir: &serde_json::Value, policy: &HostPolicy) -> agent_native_runtime::Component {
    let wasm = compile_ir_to_wasm(ir).expect("compile");
    let engine = Engine::new().expect("engine");
    engine
        .load_component_with_policy(&wasm, policy)
        .expect("load")
}

fn allow_core_host(name: &str) -> HostPolicy {
    let mut p = HostPolicy::deny_all();
    p.allow_core("host", name);
    p
}

// ---------- Happy path (AC #4, #5, #8) ------------------------------------

#[test]
fn smoke_suite_passes_three_identity_cases() {
    let component = load_core(&identity_ir(), &HostPolicy::deny_all());
    let yaml = include_str!("suites/smoke.yaml");
    let suite = parse_suite(yaml).expect("parse smoke suite");
    let report = run_suite(&component, &suite).expect("run");
    assert!(report.ok(), "suite not ok: {report:?}");
    assert_eq!(report.total, 3);
    assert_eq!(report.passed, 3);
}

#[test]
fn kv_read_suite_passes_three_state_reader_cases() {
    let component = load_core(&kv_read_ir(), &allow_core_host("kv_get"));
    let yaml = include_str!("suites/kv-read.yaml");
    let suite = parse_suite(yaml).expect("parse kv-read suite");
    let report = run_suite(&component, &suite).expect("run");
    assert!(report.ok(), "suite not ok: {report:?}");
    assert_eq!(report.total, 3);
    assert_eq!(report.passed, 3);
}

#[test]
fn kv_write_suite_passes_two_state_writer_cases() {
    let component = load_core(&kv_write_ir(), &allow_core_host("kv_set"));
    let yaml = include_str!("suites/kv-write.yaml");
    let suite = parse_suite(yaml).expect("parse kv-write suite");
    let report = run_suite(&component, &suite).expect("run");
    assert!(report.ok(), "suite not ok: {report:?}");
    assert_eq!(report.total, 2);
}

#[test]
fn noop_suite_passes_zero_arg_stub() {
    let component = load_core(&noop_ir(), &allow_core_host("ping"));
    let yaml = include_str!("suites/noop.yaml");
    let suite = parse_suite(yaml).expect("parse noop suite");
    let report = run_suite(&component, &suite).expect("run");
    assert!(report.ok(), "suite not ok: {report:?}");
    assert_eq!(report.total, 1);
}

// AC #7 — isolation: a case must not leak state into the next.
#[test]
fn isolation_suite_rebuilds_hostctx_per_case() {
    let component = load_core(&kv_read_ir(), &allow_core_host("kv_get"));
    let yaml = include_str!("suites/isolation.yaml");
    let suite = parse_suite(yaml).expect("parse isolation suite");
    let report = run_suite(&component, &suite).expect("run");
    assert!(report.ok(), "isolation suite should pass: {report:?}");
    assert_eq!(report.passed, 2);
}

// AC #9 — SuiteReport is serialisable.
#[test]
fn suite_report_serialises_to_json() {
    let component = load_core(&identity_ir(), &HostPolicy::deny_all());
    let suite = parse_suite(include_str!("suites/smoke.yaml")).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    let json = serde_json::to_string(&report).expect("serialise");
    assert!(json.contains("\"suite\":\"an-019-smoke\""));
    assert!(json.contains("\"status\":\"pass\""));
}

// AC #8 — expected_host_calls order + args enforced.
#[test]
fn expected_host_calls_mismatch_is_reported_as_fail() {
    let component = load_core(&kv_read_ir(), &allow_core_host("kv_get"));
    let yaml = r"
suite: calls-mismatch
spec_version: 1
host_stubs:
  - interface: host
    operation: kv_get
    kind: state_reader
    key_arg: 0
cases:
  - id: wrong-args-recorded
    function: read_key
    args: [5]
    expected: 0
    expected_host_calls:
      - function: host.kv_get
        args: [6]
";
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert_eq!(report.failed, 1, "expected a failure: {report:?}");
    assert_eq!(report.cases[0].status, CaseStatus::Fail);
    assert!(
        report.cases[0]
            .evidence
            .iter()
            .any(|e| e.contains("args mismatch"))
    );
}

// ---------- Negative taxonomy (AC #6) --------------------------------------

#[test]
fn negative_invalid_yaml_syntax_returns_invalid_yaml() {
    let bad = ": not valid yaml :::";
    let err = parse_suite(bad).unwrap_err();
    assert!(matches!(err, ExecutorError::InvalidYaml(_)), "got {err:?}");
}

#[test]
fn negative_operation_not_found_surfaces_as_case_error() {
    let component = load_core(&identity_ir(), &HostPolicy::deny_all());
    let yaml = r"
suite: op-not-found
spec_version: 1
cases:
  - id: missing-op
    function: no_such_op
    args: [1]
    expected: 1
";
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert_eq!(report.errored, 1);
    assert_eq!(report.cases[0].status, CaseStatus::Error);
    assert!(
        report.cases[0]
            .evidence
            .iter()
            .any(|e| e.contains("FunctionNotFound") || e.contains("no_such_op"))
    );
}

#[test]
fn negative_import_not_authorized_fails_pre_instantiate() {
    // Load with deny-all policy, but the IR declares host.kv_get.
    let component = load_core(&kv_read_ir(), &HostPolicy::deny_all());
    let yaml = r"
suite: unauth-import
spec_version: 1
cases:
  - id: policy-denies
    function: read_key
    args: [1]
    expected: 0
";
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert_eq!(report.errored, 1, "{report:?}");
    assert!(
        report.cases[0]
            .evidence
            .iter()
            .any(|e| e.contains("UnauthorizedImport") || e.contains("kv_get"))
    );
}

#[test]
fn negative_stub_declared_for_unauthorised_import_rejects_suite() {
    // Policy denies host.kv_get, yet a stub declares it. The executor must
    // refuse the whole suite before running any case.
    let component = load_core(&identity_ir(), &HostPolicy::deny_all());
    let yaml = r"
suite: stub-without-policy
spec_version: 1
host_stubs:
  - interface: host
    operation: kv_get
    kind: state_reader
    key_arg: 0
cases:
  - id: unused
    function: identity
    args: [1]
    expected: 1
";
    let suite = parse_suite(yaml).expect("parse");
    let err = run_suite(&component, &suite).unwrap_err();
    match err {
        ExecutorError::UnauthorizedByPolicy {
            interface,
            operation,
        } => {
            assert_eq!(interface, "host");
            assert_eq!(operation, "kv_get");
        }
        other => panic!("expected UnauthorizedByPolicy, got {other:?}"),
    }
}

#[test]
fn negative_stub_missing_for_declared_import_surfaces_at_runtime() {
    // Policy authorises host.kv_get but no stub is registered: wasmtime
    // fails to link the import, which the executor surfaces as a case
    // error (not as a crate-wide refusal).
    let component = load_core(&kv_read_ir(), &allow_core_host("kv_get"));
    let yaml = r"
suite: no-stub
spec_version: 1
cases:
  - id: no-stub
    function: read_key
    args: [1]
    expected: 0
";
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert_eq!(report.errored, 1, "{report:?}");
}

#[test]
fn negative_stub_signature_mismatch_rejects_declaration() {
    // Declaring state_writer with key_arg == value_arg is a signature
    // error caught at suite-build time (validation).
    let component = load_core(&kv_write_ir(), &allow_core_host("kv_set"));
    let yaml = r"
suite: sig-mismatch
spec_version: 1
host_stubs:
  - interface: host
    operation: kv_set
    kind: state_writer
    key_arg: 0
    value_arg: 0
cases:
  - id: unused
    function: write_key
    args: [1, 2]
    expected: 2
";
    let suite = parse_suite(yaml).expect("parse");
    let err = run_suite(&component, &suite).unwrap_err();
    match err {
        ExecutorError::SignatureMismatch { declared_kind, .. } => {
            assert_eq!(declared_kind, "state_writer");
        }
        other => panic!("expected SignatureMismatch, got {other:?}"),
    }
}

#[test]
fn negative_wrong_expected_result_is_fail_not_panic() {
    let component = load_core(&identity_ir(), &HostPolicy::deny_all());
    let yaml = r"
suite: wrong-result
spec_version: 1
cases:
  - id: expected-wrong
    function: identity
    args: [42]
    expected: 7
";
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert_eq!(report.failed, 1);
    assert_eq!(report.cases[0].status, CaseStatus::Fail);
    assert!(
        report.cases[0]
            .evidence
            .iter()
            .any(|e| e.contains("expected S32(7)"))
    );
}

#[test]
fn negative_invoke_component_with_host_ctx_on_core_payload_errors() {
    // Symmetric guard: calling the Component-Model variant on a Core
    // module returns a typed error without panicking (AC #6 extended).
    let component = load_core(&identity_ir(), &HostPolicy::deny_all());
    let err = component
        .invoke_component_with_host_ctx("identity", &[Val::S32(1)], (), |_e, _l| Ok(()))
        .unwrap_err();
    match err {
        RuntimeError::ExecutionFailed(msg) => {
            assert!(msg.contains("Core-WASM payload"), "msg: {msg}");
        }
        other => panic!("expected ExecutionFailed, got {other:?}"),
    }
}

// ---------- Registry sanity ------------------------------------------------

#[test]
fn registry_rejects_arity_against_observed_import() {
    let mut reg = HostStubRegistry::new();
    reg.register("host", "ping", StubKind::Noop { returns: 1 });
    // Observed arity 2 against a noop (arity 0) must be rejected.
    let err = reg.validate_arity("host", "ping", 2).unwrap_err();
    assert!(matches!(err, ExecutorError::SignatureMismatch { .. }));
}

// ---------- Invoke extension points unchanged core path --------------------

/// Regression guard: the extracted precheck helpers must preserve the
/// behaviour of `Component::invoke` (AC #3).
#[test]
fn invoke_signature_preserved_after_refactor() {
    let component = load_core(&identity_ir(), &HostPolicy::deny_all());
    let results = component.invoke("identity", &[Val::S32(99)]).unwrap();
    assert_eq!(results, vec![Val::S32(99)]);

    let err = component.invoke("nope", &[]).unwrap_err();
    assert!(matches!(err, RuntimeError::FunctionNotFound(_)));
}
