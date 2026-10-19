//! Comprehensive public API audit test suite for `agent-native-runtime`.
//!
//! Validates:
//! - Only root exports of `agent_native_runtime` are needed by external consumers.
//! - The 6 canonical types (`Engine`, `Component`, `HostPolicy`, `RuntimeError`, `Val`, `Resolver`)
//!   operate without requiring compiler or LLM dependencies.
//! - Invariants: Deny-by-default, fail-explicit on missing exports, deterministic caching.

use agent_native_compiler::compile_ir_to_wasm;
use agent_native_runtime::{Component, Engine, HostPolicy, Resolver, RuntimeError, Val};
use serde_json::json;

fn sample_math_wasm() -> Vec<u8> {
    let ir = json!({
        "ir_version": "0.1",
        "spec_version": 1,
        "functions": [
            {
                "name": "add_numbers",
                "export": true,
                "params": [{ "name": "a", "type": "i32" }, { "name": "b", "type": "i32" }],
                "returns": "i32",
                "requirements": ["MATH-001"],
                "body": {
                    "op": "add",
                    "left": { "op": "param", "name": "a" },
                    "right": { "op": "param", "name": "b" }
                }
            },
            {
                "name": "identity",
                "export": true,
                "params": [{ "name": "val", "type": "i32" }],
                "returns": "i32",
                "requirements": ["MATH-002"],
                "body": { "op": "param", "name": "val" }
            }
        ]
    });
    compile_ir_to_wasm(&ir).expect("compile must succeed")
}

#[test]
fn test_public_api_surface_completeness_and_isolation() {
    // 1. Engine construction
    let engine = Engine::new().expect("Engine construction must succeed");

    // Assemble Core WASM bytes from WAT
    let wasm_bytes = sample_math_wasm();

    // 2. Default deny-all load
    let component: Component = engine
        .load_component(&wasm_bytes)
        .expect("load_component must succeed on valid wasm bytes");

    assert!(component.is_core_module());
    assert!(!component.is_component_model());

    // 3. Inspection of component policy
    assert!(!component.policy().is_allowed_core("any_module", "any_name"));

    // 4. Execution via Component::invoke
    let results = component
        .invoke("add_numbers", &[Val::S32(10), Val::S32(32)])
        .expect("invoke must succeed");
    assert_eq!(results, vec![Val::S32(42)]);

    let id_res = component
        .invoke("identity", &[Val::S32(100)])
        .expect("invoke identity");
    assert_eq!(id_res, vec![Val::S32(100)]);
}

#[test]
fn test_public_api_host_policy_builder_and_inspection() {
    let mut policy = HostPolicy::deny_all();

    // Verify initial deny-all state
    assert!(!policy.is_allowed_core("env", "read"));
    assert!(!policy.is_wit_interface_allowed("an:test/ops@0.1.0"));
    assert!(!policy.is_wit_operation_allowed("an:test/ops@0.1.0", "calculate"));

    // Allow single Core pair
    policy.allow_core("env", "read");
    assert!(policy.is_allowed_core("env", "read"));
    assert!(!policy.is_allowed_core("env", "write"));

    // Allow single WIT operation
    policy.allow_wit_operation("an:test/ops@0.1.0", "calculate");
    assert!(policy.is_wit_operation_allowed("an:test/ops@0.1.0", "calculate"));
    assert!(!policy.is_wit_operation_allowed("an:test/ops@0.1.0", "delete"));
    assert!(!policy.is_wit_interface_allowed("an:test/ops@0.1.0"));

    // Allow entire WIT interface
    policy.allow_wit_interface("an:host/logging@0.1.0");
    assert!(policy.is_wit_interface_allowed("an:host/logging@0.1.0"));
    assert!(policy.is_wit_operation_allowed("an:host/logging@0.1.0", "log_info"));
}

#[test]
fn test_public_api_runtime_error_taxonomy_matching() {
    let engine = Engine::new().expect("Engine");
    let wasm_bytes = sample_math_wasm();
    let component = engine.load_component(&wasm_bytes).expect("load");

    // 1. Missing function -> RuntimeError::FunctionNotFound
    let err_not_found = component
        .invoke("non_existent_fn", &[])
        .expect_err("must fail");
    match err_not_found {
        RuntimeError::FunctionNotFound(name) => assert_eq!(name, "non_existent_fn"),
        other => panic!("expected FunctionNotFound, got {other:?}"),
    }

    // 2. Arity mismatch -> RuntimeError::ArityMismatch
    let err_arity = component
        .invoke("add_numbers", &[Val::S32(1)])
        .expect_err("must fail on arity");
    match err_arity {
        RuntimeError::ArityMismatch {
            name,
            expected,
            got,
        } => {
            assert_eq!(name, "add_numbers");
            assert_eq!(expected, 2);
            assert_eq!(got, 1);
        }
        other => panic!("expected ArityMismatch, got {other:?}"),
    }

    // 3. Invalid WASM -> RuntimeError::LoadFailed
    let err_load = engine
        .load_component(b"invalid wasm magic")
        .expect_err("must fail on bad bytes");
    assert!(matches!(err_load, RuntimeError::LoadFailed(_)));
}

#[test]
fn test_public_api_resolver_caching_and_session_reset() {
    let wasm_bytes = sample_math_wasm();
    let mut resolver = Resolver::new().expect("Resolver construction");

    assert_eq!(resolver.cached_count(), 0);

    // Load first time (cache miss -> stores parsed artifact)
    let c1 = resolver.load(&wasm_bytes).expect("first load");
    assert_eq!(resolver.cached_count(), 1);

    // Load second time with same bytes (cache hit -> reuses parsed artifact)
    let c2 = resolver.load(&wasm_bytes).expect("second load");
    assert_eq!(resolver.cached_count(), 1);

    // Invocations from both components produce identical results
    assert_eq!(
        c1.invoke("identity", &[Val::S32(7)]).unwrap(),
        vec![Val::S32(7)]
    );
    assert_eq!(
        c2.invoke("identity", &[Val::S32(7)]).unwrap(),
        vec![Val::S32(7)]
    );

    // Reset clears cache for tenant/session isolation
    resolver.reset();
    assert_eq!(resolver.cached_count(), 0);
}
