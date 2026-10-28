//! AN-034: Comprehensive negative tests and security boundary verification.
//!
//! Audits:
//! 1. Deny-by-default on `Component::invoke()`.
//! 2. Precheck enforcement on `invoke_core_with_host_ctx()` and `invoke_component_with_host_ctx()`.
//! 3. Absence of privilege leakage across `Resolver` cache hits.
//! 4. Typed error distinction: `UnauthorizedImport` (permission denied) vs
//!    `InstantiationFailed` (missing host provider).
//! 5. Strict WIT operation semantics: partial authorization rejects interface
//!    when unauthorized operations are imported.
//! 6. Self-granting defeat: untrusted manifest requesting permissions denied
//!    by host policy yields an empty effective policy and fails.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use wasma_compiler::{CompileContext, compile_ir_to_wasm, compile_ir_to_wasm_with_context};
use wasma_runtime::{ComponentManifest, Engine, HostPolicy, Resolver, RuntimeError, Val};

fn compiler_crate_ctx() -> CompileContext {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.push("compiler");
    CompileContext::new(root)
}

fn probe_cm_wasm() -> Vec<u8> {
    let text = include_str!("../../compiler/tests/fixtures/ir/probe-use-types.ir.json");
    let ir: serde_json::Value = serde_json::from_str(text).expect("valid JSON");
    compile_ir_to_wasm_with_context(&ir, &compiler_crate_ctx()).expect("compile must succeed")
}

fn program_003_core_wasm() -> Vec<u8> {
    let text = include_str!("../../compiler/tests/fixtures/ir/program-003.ir.json");
    let ir: serde_json::Value = serde_json::from_str(text).expect("valid JSON");
    compile_ir_to_wasm(&ir).expect("compile must succeed")
}

const PROBE_TYPES_IFACE: &str = "an:probe/types@0.1.0";
const EXPORTED_OPS_IFACE: &str = "an:probe/ops@0.1.0";

/// 1. Deny-by-default on `Component::invoke()`
#[test]
fn test_invoke_denies_unauthorized_component_model_import() {
    let wasm = probe_cm_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component_with_policy(&wasm, &HostPolicy::deny_all())
        .expect("load");

    let err = component
        .invoke(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(1), Val::S32(2)],
        )
        .expect_err("must be rejected pre-instantiate");

    assert!(
        matches!(err, RuntimeError::UnauthorizedImport { ref module, .. } if module == PROBE_TYPES_IFACE),
        "expected UnauthorizedImport, got {err:?}"
    );
}

/// 2a. Precheck on `invoke_core_with_host_ctx()` prevents host linker/stub execution
#[test]
fn test_invoke_core_with_host_ctx_enforces_policy_before_stub_execution() {
    let wasm = program_003_core_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component_with_policy(&wasm, &HostPolicy::deny_all())
        .expect("load");

    let linker_called = AtomicBool::new(false);

    let err = component
        .invoke_core_with_host_ctx("process_order", &[Val::S32(1)], (), |_eng, _linker| {
            linker_called.store(true, Ordering::SeqCst);
            Ok(())
        })
        .expect_err("must fail precheck before instantiating or executing stubs");

    assert!(
        matches!(err, RuntimeError::UnauthorizedImport { ref module, .. } if module == "host"),
        "expected UnauthorizedImport, got {err:?}"
    );
    // Crucial security invariant: precheck fails before any host linker closure is invoked
    assert!(
        !linker_called.load(Ordering::SeqCst),
        "linker must not be configured when precheck fails"
    );
}

/// 2b. Precheck on `invoke_component_with_host_ctx()`
#[test]
fn test_invoke_component_with_host_ctx_enforces_policy_before_instantiation() {
    let wasm = probe_cm_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component_with_policy(&wasm, &HostPolicy::deny_all())
        .expect("load");

    let linker_called = AtomicBool::new(false);

    let err = component
        .invoke_component_with_host_ctx(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(1), Val::S32(2)],
            (),
            |_eng, _linker| {
                linker_called.store(true, Ordering::SeqCst);
                Ok(())
            },
        )
        .expect_err("must fail precheck before component linker configuration");

    assert!(
        matches!(err, RuntimeError::UnauthorizedImport { ref module, .. } if module == PROBE_TYPES_IFACE),
        "expected UnauthorizedImport, got {err:?}"
    );
    assert!(
        !linker_called.load(Ordering::SeqCst),
        "component linker must not be configured when precheck fails"
    );
}

/// 3. Resolver cache isolation: no privilege leakage across cache hits
#[test]
fn test_resolver_cache_does_not_leak_permissions_to_subsequent_loads() {
    let wasm = probe_cm_wasm();
    let mut resolver = Resolver::new().expect("resolver");

    // Session 1: Permissive policy
    let mut permissive = HostPolicy::deny_all();
    permissive.allow_wit_interface(PROBE_TYPES_IFACE);

    let c_allowed = resolver
        .load_with_policy(&wasm, &permissive)
        .expect("load allowed");
    assert_eq!(resolver.cached_count(), 1);

    // Invocations under permissive policy succeed
    let res = c_allowed
        .invoke(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(3), Val::S32(4)],
        )
        .expect("invocation must succeed under permissive policy");
    assert!(matches!(res[0], Val::Record(_)));

    // Session 2: Restrictive deny-all policy on the EXACT same cached bytes
    let c_restricted = resolver
        .load_with_policy(&wasm, &HostPolicy::deny_all())
        .expect("load under deny-all");
    assert_eq!(resolver.cached_count(), 1, "must hit cache");

    // Invocations under restricted policy MUST FAIL (no authorization inherited from cache)
    let err = c_restricted
        .invoke(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(3), Val::S32(4)],
        )
        .expect_err("must be rejected under restricted policy despite cache hit");

    assert!(
        matches!(err, RuntimeError::UnauthorizedImport { ref module, .. } if module == PROBE_TYPES_IFACE),
        "expected UnauthorizedImport, got {err:?}"
    );

    // Also verify load-time enforcement through resolver
    let err_load = resolver
        .load_enforcing_policy(&wasm, &HostPolicy::deny_all())
        .expect_err("load-time enforcement must reject unauthorized import");
    assert!(matches!(err_load, RuntimeError::UnauthorizedImport { .. }));
}

/// 4. Distinction between `UnauthorizedImport` and `InstantiationFailed`
#[test]
fn test_distinction_unauthorized_import_vs_instantiation_failed() {
    let wasm = program_003_core_wasm();
    let engine = Engine::new().expect("engine");

    // Scenario A: Permission denied (HostPolicy::deny_all)
    let c_denied = engine
        .load_component_with_policy(&wasm, &HostPolicy::deny_all())
        .expect("load");
    let err_denied = c_denied
        .invoke("process_order", &[Val::S32(1)])
        .expect_err("must fail");
    assert!(
        matches!(err_denied, RuntimeError::UnauthorizedImport { ref module, .. } if module == "host"),
        "must be UnauthorizedImport when policy denies it, got {err_denied:?}"
    );

    // Scenario B: Permission granted, but NO host implementation provided
    let mut policy_granted = HostPolicy::deny_all();
    policy_granted.allow_core("host", "kv_get");
    policy_granted.allow_core("host", "check_credit");
    policy_granted.allow_core("host", "emit_audit");

    let c_granted = engine
        .load_component_with_policy(&wasm, &policy_granted)
        .expect("load");

    // invoke() without stubs reaches Wasmtime instantiation and fails because no stub exists
    let err_missing_impl = c_granted
        .invoke("process_order", &[Val::S32(1)])
        .expect_err("must fail instantiation because host provided no stub");

    assert!(
        matches!(err_missing_impl, RuntimeError::InstantiationFailed(_)),
        "must be InstantiationFailed when import is authorized but unimplemented, got {err_missing_impl:?}"
    );
}

/// 5. Strict WIT operation semantics: partial allow does not permit missing operations
#[test]
fn test_wit_operation_partial_authorization_rejects_missing_operations() {
    let wasm = probe_cm_wasm();
    let engine = Engine::new().expect("engine");

    // The probe imports interface `an:probe/types@0.1.0` which defines type `pt`.
    // Allowing an unrelated operation name within that interface must NOT authorize `pt`.
    let mut partial_policy = HostPolicy::deny_all();
    partial_policy.allow_wit_operation(PROBE_TYPES_IFACE, "unrelated_op");

    let component = engine
        .load_component_with_policy(&wasm, &partial_policy)
        .expect("load");

    let err = component
        .invoke(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(1), Val::S32(2)],
        )
        .expect_err("must reject because required import was not authorized");

    assert!(
        matches!(err, RuntimeError::UnauthorizedImport { ref module, .. } if module == PROBE_TYPES_IFACE),
        "expected UnauthorizedImport, got {err:?}"
    );
}

/// 6. Self-granting defeat: untrusted manifest requesting permissions denied by host
#[test]
fn test_self_granting_manifest_defeated_by_effective_policy() {
    let wasm = probe_cm_wasm();
    let engine = Engine::new().expect("engine");

    // Untrusted manifest created by guest/agent claiming access to everything
    let untrusted_manifest_yaml = format!(
        r#"
version: "0.1"
name: "malicious-self-granting-agent"
capabilities:
  wit_interfaces:
    - "{PROBE_TYPES_IFACE}"
    - "wasi:http/outgoing-handler@0.2.0"
    - "wasi:filesystem/types@0.2.0"
  core_imports:
    - module: "host"
      name: "root_exec"
"#
    );
    let requested = ComponentManifest::from_yaml_str(&untrusted_manifest_yaml).unwrap();

    // Host policy is strictly deny_all
    let granted = HostPolicy::deny_all();

    // Effective policy must be completely empty: Requested ∩ Granted = ∅
    let effective = requested.effective_policy(&granted);
    assert!(!effective.is_wit_interface_allowed(PROBE_TYPES_IFACE));
    assert!(!effective.is_wit_interface_allowed("wasi:http/outgoing-handler@0.2.0"));
    assert!(!effective.is_allowed_core("host", "root_exec"));

    // Attempting to load with manifest and host policy fails immediately at load time
    let err = engine
        .load_component_with_manifest(&wasm, &requested, &granted)
        .expect_err("must reject self-granting component at load time");

    assert!(
        matches!(err, RuntimeError::UnauthorizedImport { ref module, .. } if module == PROBE_TYPES_IFACE),
        "expected UnauthorizedImport, got {err:?}"
    );
}
