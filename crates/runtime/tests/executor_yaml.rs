//! AN-019 + AN-020 end-to-end: compile IR → load component → run YAML
//! suites.
//!
//! Covers AN-020 AC-01..AC-16 and preserves AN-019 regression tests
//! migrated to the new schema (`import` canonical, bool args,
//! `StateReader` fail-explicit, Component-Model type-only delegation).

use std::path::PathBuf;

use agent_native_compiler::{CompileContext, compile_ir_to_wasm, compile_ir_to_wasm_with_context};
use agent_native_runtime::testing::{
    CaseStatus, ExecutorError, HostStubRegistry, StubKind, parse_suite, run_suite,
};
use agent_native_runtime::{Component, Engine, HostPolicy, RuntimeError, Val};
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

fn identity_bool_ir() -> serde_json::Value {
    json!({
        "ir_version": "0.1",
        "spec_version": 1,
        "functions": [{
            "name": "identity_bool",
            "export": true,
            "params": [{"name": "value", "type": "bool"}],
            "returns": "bool",
            "requirements": ["BOOL-001"],
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

fn load_core(ir: &serde_json::Value, policy: &HostPolicy) -> Component {
    let wasm = compile_ir_to_wasm(ir).expect("compile");
    let engine = Engine::new().expect("engine");
    engine
        .load_component_with_policy(&wasm, policy)
        .expect("load")
}

fn compiler_crate_ctx() -> CompileContext {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.push("compiler");
    CompileContext::new(root)
}

fn load_component_probe_use_types(policy: &HostPolicy) -> Component {
    let ir_text = include_str!("../../compiler/tests/fixtures/ir/probe-use-types.ir.json");
    let ir: serde_json::Value = serde_json::from_str(ir_text).expect("fixture json");
    let wasm = compile_ir_to_wasm_with_context(&ir, &compiler_crate_ctx()).expect("compile");
    let engine = Engine::new().expect("engine");
    engine
        .load_component_with_policy(&wasm, policy)
        .expect("load")
}

/// Policy that authorises the type import of the probe-use-types
/// fixture. The import is `an:probe/types@0.1.0` carrying only the
/// record type `pt`; no functional symbols cross it, so authorising it
/// just lets the runtime proceed. Separate from stub registration
/// (ADR-004).
fn probe_use_types_policy() -> HostPolicy {
    let mut p = HostPolicy::deny_all();
    p.allow_wit_interface("an:probe/types@0.1.0");
    p
}

fn allow_core_host(name: &str) -> HostPolicy {
    let mut p = HostPolicy::deny_all();
    p.allow_core("host", name);
    p
}

// ---------- AC-01: Component Model without functional imports --------------

#[test]
fn ac01_component_model_without_functional_imports_runs_via_invoke() {
    let component = load_component_probe_use_types(&probe_use_types_policy());
    let yaml = include_str!("suites/records.yaml");
    let suite = parse_suite(yaml).expect("parse records suite");
    let report = run_suite(&component, &suite).expect("run");
    assert!(report.ok(), "AC-01 records suite should pass: {report:?}");
    assert_eq!(
        report.passed, 2,
        "AC-01 expected 2 records cases, got {report:?}"
    );
}

// ---------- AC-02: CM with functional import but no stub -> HostError -----

#[test]
fn ac02_component_with_functional_import_without_stub_errors() {
    let ir_text = include_str!("../../compiler/tests/fixtures/ir/smoke-v4.ir.json");
    let ir: serde_json::Value = serde_json::from_str(ir_text).expect("json");
    let wasm = compile_ir_to_wasm_with_context(&ir, &compiler_crate_ctx()).expect("compile");
    let engine = Engine::new().expect("engine");
    let mut policy = HostPolicy::deny_all();
    policy.allow_wit_interface("kaddo:commercial/host-services@0.1.0");
    let component = engine
        .load_component_with_policy(&wasm, &policy)
        .expect("load");

    let yaml = r#"
suite: ac02-cm-no-stub
spec_version: 1
cases:
  - id: cm-no-stub-blocked
    function: "kaddo:commercial/order-calculator@0.1.0#calculate-order"
    args: [1, 1, 1, 1, 1, 1]
    expected: 0
"#;
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert_eq!(report.errored, 1, "{report:?}");
    assert!(
        report.cases[0]
            .evidence
            .iter()
            .any(|e| e.contains("functional import") && e.contains("RM-07")),
        "evidence: {:?}",
        report.cases[0].evidence
    );
}

// ---------- AC-03: Record comparison -------------------------------------

#[test]
fn ac03_record_missing_field_fails_with_evidence() {
    let component = load_component_probe_use_types(&probe_use_types_policy());
    let yaml = r#"
suite: ac03-missing-field
spec_version: 1
cases:
  - id: missing-y
    function: "an:probe/ops@0.1.0#make-pt"
    args: [1, 2]
    expected:
      x: 1
      z: 999
"#;
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert_eq!(report.failed, 1);
    assert!(
        report.cases[0]
            .evidence
            .iter()
            .any(|e| e.contains("expected field 'z' missing")),
        "evidence: {:?}",
        report.cases[0].evidence
    );
}

#[test]
fn ac03_record_extra_field_fails_with_evidence() {
    let component = load_component_probe_use_types(&probe_use_types_policy());
    let yaml = r#"
suite: ac03-extra-field
spec_version: 1
cases:
  - id: only-x
    function: "an:probe/ops@0.1.0#make-pt"
    args: [1, 2]
    expected:
      x: 1
"#;
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert_eq!(report.failed, 1);
    assert!(
        report.cases[0]
            .evidence
            .iter()
            .any(|e| e.contains("unexpected field 'y'")),
        "evidence: {:?}",
        report.cases[0].evidence
    );
}

// ---------- AC-04: StateReader miss fails explicit ------------------------

#[test]
fn ac04_state_reader_miss_fails_explicit() {
    let component = load_core(&kv_read_ir(), &allow_core_host("kv_get"));
    let yaml = r"
suite: ac04-reader-miss
spec_version: 1
host_stubs:
  - interface: host
    operation: kv_get
    kind: state_reader
    key_arg: 0
cases:
  - id: reader-miss-key-99
    function: read_key
    args: [99]
    expected: 0
";
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert_eq!(report.errored, 1, "{report:?}");
    assert!(
        report.cases[0]
            .evidence
            .iter()
            .any(|e| e.contains("state_reader") && e.contains("miss") && e.contains("key=99")),
        "evidence: {:?}",
        report.cases[0].evidence
    );
}

// ---------- AC-05: import canonical, function alias, both -> reject --------

#[test]
fn ac05_import_canonical_parses() {
    let yaml = r"
suite: ac05-import
spec_version: 1
cases:
  - id: dummy
    function: identity
    args: [0]
    expected: 0
    expected_host_calls:
      - import: host.kv_get
        args: [1]
";
    parse_suite(yaml).expect("import canonical parses");
}

#[test]
fn ac05_function_alias_parses() {
    let yaml = r"
suite: ac05-alias
spec_version: 1
cases:
  - id: dummy
    function: identity
    args: [0]
    expected: 0
    expected_host_calls:
      - function: host.kv_get
        args: [1]
";
    parse_suite(yaml).expect("function alias parses");
}

#[test]
fn ac05_both_import_and_function_rejected() {
    let yaml = r"
suite: ac05-both
spec_version: 1
cases:
  - id: dummy
    function: identity
    args: [0]
    expected: 0
    expected_host_calls:
      - import: host.kv_get
        function: host.kv_get
        args: [1]
";
    let err = parse_suite(yaml).unwrap_err();
    match err {
        ExecutorError::InvalidYaml(msg) => {
            assert!(
                msg.contains("both") && msg.contains("import") && msg.contains("function"),
                "msg: {msg}"
            );
        }
        other => panic!("expected InvalidYaml, got {other:?}"),
    }
}

// ---------- AC-06: short vs qualified normalisation ------------------------

#[test]
fn ac06_short_name_matches_qualified_recorded_call() {
    let component = load_core(&kv_read_ir(), &allow_core_host("kv_get"));
    let yaml = include_str!("suites/kv-read.yaml");
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert!(report.ok(), "{report:?}");
    assert_eq!(report.passed, 3);
}

#[test]
fn ac06_ambiguous_dotted_name_rejected() {
    let yaml = r"
suite: ac06-ambig
spec_version: 1
cases:
  - id: ambig
    function: identity
    args: [0]
    expected: 0
    expected_host_calls:
      - import: 'host.'
        args: [1]
";
    let err = parse_suite(yaml).unwrap_err();
    assert!(matches!(err, ExecutorError::InvalidYaml(ref m) if m.contains("ambiguous")));
}

// ---------- AC-07: args booleans -------------------------------------------

#[test]
fn ac07_bool_args_execute_against_core_module() {
    let component = load_core(&identity_bool_ir(), &HostPolicy::deny_all());
    let yaml = include_str!("suites/bool-args.yaml");
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert!(report.ok(), "{report:?}");
    assert_eq!(report.passed, 2);
}

// ---------- AC-08: host_state sub-maps extra rejected ---------------------

#[test]
fn ac08_unsupported_host_state_submap_rejected() {
    let yaml = r"
suite: ac08-sub-map
spec_version: 1
cases:
  - id: with-credit-limits
    function: identity
    args: [0]
    expected: 0
    host_state:
      kv:
        1: 1
      credit_limits:
        1: 100
";
    let err = parse_suite(yaml).unwrap_err();
    match err {
        ExecutorError::InvalidYaml(msg) => {
            assert!(msg.contains("credit_limits"), "msg: {msg}");
        }
        other => panic!("expected InvalidYaml, got {other:?}"),
    }
}

// ---------- AC-09: root without suite/spec_version rejected ---------------

#[test]
fn ac09_root_without_suite_rejected() {
    let yaml = r"
cases:
  - id: no-root
    function: identity
    args: [0]
    expected: 0
";
    let err = parse_suite(yaml).unwrap_err();
    assert!(matches!(err, ExecutorError::InvalidYaml(_)));
}

#[test]
fn ac09_root_without_spec_version_rejected() {
    let yaml = r"
suite: ac09
cases:
  - id: no-sv
    function: identity
    args: [0]
    expected: 0
";
    let err = parse_suite(yaml).unwrap_err();
    assert!(matches!(err, ExecutorError::InvalidYaml(_)));
}

// ---------- AC-10: poc-yaml-contract.md published --------------------------

#[test]
fn ac10_poc_yaml_contract_published() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("knowledge")
        .join("tech")
        .join("poc-yaml-contract.md");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("expected {} to exist: {e}", path.display()));
    for heading in [
        "## Campos idénticos",
        "## Campos renombrados o normalizados",
        "## Campos deliberadamente omitidos",
        "## Estructuras-raíz no soportadas",
    ] {
        assert!(
            text.contains(heading),
            "poc-yaml-contract.md missing heading '{heading}'"
        );
    }
}

// ---------- AN-019 regression suites migrated to AN-020 schema ------------

#[test]
fn regression_smoke_suite_passes() {
    let component = load_core(&identity_ir(), &HostPolicy::deny_all());
    let suite = parse_suite(include_str!("suites/smoke.yaml")).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert!(report.ok(), "{report:?}");
    assert_eq!(report.passed, 3);
}

#[test]
fn regression_kv_write_suite_passes() {
    let component = load_core(&kv_write_ir(), &allow_core_host("kv_set"));
    let suite = parse_suite(include_str!("suites/kv-write.yaml")).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert!(report.ok(), "{report:?}");
    assert_eq!(report.passed, 2);
}

#[test]
fn regression_noop_suite_passes() {
    let component = load_core(&noop_ir(), &allow_core_host("ping"));
    let suite = parse_suite(include_str!("suites/noop.yaml")).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert!(report.ok(), "{report:?}");
}

#[test]
fn regression_isolation_suite_passes() {
    let component = load_core(&kv_read_ir(), &allow_core_host("kv_get"));
    let suite = parse_suite(include_str!("suites/isolation.yaml")).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert!(report.ok(), "{report:?}");
    assert_eq!(report.passed, 2);
}

#[test]
fn regression_suite_report_serialises_to_json() {
    let component = load_core(&identity_ir(), &HostPolicy::deny_all());
    let suite = parse_suite(include_str!("suites/smoke.yaml")).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    let json = serde_json::to_string(&report).expect("serialise");
    assert!(json.contains("\"suite\":\"an-020-smoke\""));
    assert!(json.contains("\"status\":\"pass\""));
}

#[test]
fn regression_expected_host_calls_mismatch_is_reported_as_fail() {
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
    host_state:
      kv:
        5: 500
    expected: 500
    expected_host_calls:
      - import: host.kv_get
        args: [6]
";
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert_eq!(report.failed, 1, "{report:?}");
    assert_eq!(report.cases[0].status, CaseStatus::Fail);
    assert!(
        report.cases[0]
            .evidence
            .iter()
            .any(|e| e.contains("args mismatch"))
    );
}

// ---------- AN-019 negatives retained --------------------------------------

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
}

#[test]
fn negative_import_not_authorized_fails_pre_instantiate() {
    let component = load_core(&kv_read_ir(), &HostPolicy::deny_all());
    let yaml = r"
suite: unauth-import
spec_version: 1
cases:
  - id: policy-denies
    function: read_key
    args: [1]
    host_state:
      kv:
        1: 1
    expected: 1
";
    let suite = parse_suite(yaml).expect("parse");
    let report = run_suite(&component, &suite).expect("run");
    assert_eq!(report.errored, 1);
}

#[test]
fn negative_stub_declared_for_unauthorised_import_rejects_suite() {
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
    assert!(matches!(err, ExecutorError::UnauthorizedByPolicy { .. }));
}

#[test]
fn negative_stub_signature_mismatch_rejects_declaration() {
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
    assert!(matches!(err, ExecutorError::SignatureMismatch { .. }));
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
}

#[test]
fn registry_rejects_arity_against_observed_import() {
    let mut reg = HostStubRegistry::new();
    reg.register("host", "ping", StubKind::Noop { returns: 1 });
    let err = reg.validate_arity("host", "ping", 2).unwrap_err();
    assert!(matches!(err, ExecutorError::SignatureMismatch { .. }));
}

#[test]
fn invoke_signature_preserved_after_refactor() {
    let component = load_core(&identity_ir(), &HostPolicy::deny_all());
    let results = component.invoke("identity", &[Val::S32(99)]).unwrap();
    assert_eq!(results, vec![Val::S32(99)]);

    let err = component.invoke("nope", &[]).unwrap_err();
    assert!(matches!(err, RuntimeError::FunctionNotFound(_)));
}

#[test]
fn negative_invoke_component_with_host_ctx_on_core_payload_errors() {
    let component = load_core(&identity_ir(), &HostPolicy::deny_all());
    let err = component
        .invoke_component_with_host_ctx("identity", &[Val::S32(1)], (), |_e, _l| Ok(()))
        .unwrap_err();
    assert!(matches!(err, RuntimeError::ExecutionFailed(_)));
}
