//! `HostPolicy` tests — deny-by-default and selective allow-listing.

use wasma_compiler::compile_ir_to_wasm;
use wasma_runtime::{Engine, HostPolicy, RuntimeError, Val};

fn program_003_wasm() -> Vec<u8> {
    let ir_text = include_str!("../../compiler/tests/fixtures/ir/program-003.ir.json");
    let ir: serde_json::Value = serde_json::from_str(ir_text).expect("valid JSON");
    compile_ir_to_wasm(&ir).expect("compile must succeed")
}

#[test]
fn deny_all_policy_rejects_declared_imports() {
    let wasm = program_003_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component_with_policy(&wasm, &HostPolicy::deny_all())
        .expect("load");

    let err = component
        .invoke("process_order", &[Val::S32(1)])
        .unwrap_err();
    assert!(
        matches!(err, RuntimeError::UnauthorizedImport { ref module, .. } if module == "host"),
        "expected UnauthorizedImport, got {err:?}"
    );
}

#[test]
fn allow_pair_authorises_specific_import_only() {
    let mut policy = HostPolicy::deny_all();
    policy.allow("host", "kv_get");

    let wasm = program_003_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component_with_policy(&wasm, &policy)
        .expect("load");

    // program-003 also imports host::check_credit and host::emit_audit, so
    // the policy still rejects one of them.
    let err = component
        .invoke("process_order", &[Val::S32(1)])
        .unwrap_err();
    match err {
        RuntimeError::UnauthorizedImport { module, name } => {
            assert_eq!(module, "host");
            assert_ne!(
                name, "kv_get",
                "kv_get is explicitly allowed and must not be the offender"
            );
        }
        other => panic!("expected UnauthorizedImport for a non-kv import, got {other:?}"),
    }
}

#[test]
fn allow_module_authorises_every_import_under_it() {
    let mut policy = HostPolicy::deny_all();
    policy.allow_module("host");

    let wasm = program_003_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component_with_policy(&wasm, &policy)
        .expect("load");

    // With all host imports allowed, the policy check passes. Instantiation
    // still fails because this WI does not supply host functions; INIT-06-01
    // provides them.
    let err = component
        .invoke("process_order", &[Val::S32(1)])
        .unwrap_err();
    assert!(
        matches!(err, RuntimeError::InstantiationFailed(_)),
        "expected InstantiationFailed once policy allows the imports, got {err:?}"
    );
}

#[test]
fn is_allowed_inspection_reflects_state() {
    let mut policy = HostPolicy::deny_all();
    assert!(!policy.is_allowed("host", "kv_get"));

    policy.allow("host", "kv_get");
    assert!(policy.is_allowed("host", "kv_get"));
    assert!(!policy.is_allowed("host", "check_credit"));

    policy.allow_module("wasi:cli");
    assert!(policy.is_allowed("wasi:cli", "stdout"));
    assert!(policy.is_allowed("wasi:cli", "stdin"));
    assert!(!policy.is_allowed("other", "anything"));
}
