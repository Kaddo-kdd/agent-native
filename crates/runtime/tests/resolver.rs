//! `Resolver` tests — content-addressed cache, policy independence and
//! reset semantics.

use agent_native_compiler::compile_ir_to_wasm;
use agent_native_runtime::{HostPolicy, Resolver, RuntimeError, Val};
use serde_json::json;

fn identity_wasm_a() -> Vec<u8> {
    let ir = json!({
        "ir_version": "0.1",
        "spec_version": 1,
        "functions": [
            {
                "name": "identity",
                "export": true,
                "params": [{ "name": "value", "type": "i32" }],
                "returns": "i32",
                "requirements": ["SMOKE-A"],
                "body": { "op": "param", "name": "value" }
            }
        ]
    });
    compile_ir_to_wasm(&ir).expect("compile must succeed")
}

fn identity_wasm_b() -> Vec<u8> {
    // Different function name → different exported symbol → different bytes.
    let ir = json!({
        "ir_version": "0.1",
        "spec_version": 1,
        "functions": [
            {
                "name": "echo",
                "export": true,
                "params": [{ "name": "value", "type": "i32" }],
                "returns": "i32",
                "requirements": ["SMOKE-B"],
                "body": { "op": "param", "name": "value" }
            }
        ]
    });
    compile_ir_to_wasm(&ir).expect("compile must succeed")
}

fn program_003_wasm() -> Vec<u8> {
    let ir_text = include_str!("../../compiler/tests/fixtures/ir/program-003.ir.json");
    let ir: serde_json::Value = serde_json::from_str(ir_text).expect("valid JSON");
    compile_ir_to_wasm(&ir).expect("compile must succeed")
}

#[test]
fn same_bytes_hit_the_cache() {
    let wasm = identity_wasm_a();
    let mut resolver = Resolver::new().expect("resolver");

    let component_1 = resolver.load(&wasm).expect("first load");
    assert_eq!(resolver.cached_count(), 1);

    let component_2 = resolver.load(&wasm).expect("second load");
    assert_eq!(
        resolver.cached_count(),
        1,
        "identical bytes must not grow the cache"
    );

    // Both components execute correctly from the cached payload.
    assert_eq!(
        component_1.invoke("identity", &[Val::S32(1)]).unwrap(),
        vec![Val::S32(1)]
    );
    assert_eq!(
        component_2.invoke("identity", &[Val::S32(2)]).unwrap(),
        vec![Val::S32(2)]
    );
}

#[test]
fn distinct_bytes_grow_the_cache() {
    let wasm_a = identity_wasm_a();
    let wasm_b = identity_wasm_b();
    assert_ne!(wasm_a, wasm_b, "fixtures must differ");

    let mut resolver = Resolver::new().expect("resolver");
    resolver.load(&wasm_a).expect("load a");
    resolver.load(&wasm_b).expect("load b");

    assert_eq!(resolver.cached_count(), 2);
}

#[test]
fn reset_clears_the_cache() {
    let wasm_a = identity_wasm_a();
    let wasm_b = identity_wasm_b();

    let mut resolver = Resolver::new().expect("resolver");
    resolver.load(&wasm_a).expect("load a");
    resolver.load(&wasm_b).expect("load b");
    assert_eq!(resolver.cached_count(), 2);

    resolver.reset();
    assert_eq!(resolver.cached_count(), 0);
}

#[test]
fn same_bytes_distinct_policies_share_cache_but_enforce_own_policy() {
    let wasm = program_003_wasm();

    let mut resolver = Resolver::new().expect("resolver");

    let deny_all = HostPolicy::deny_all();
    let mut allow_host = HostPolicy::deny_all();
    allow_host.allow_module("host");

    let component_deny = resolver
        .load_with_policy(&wasm, &deny_all)
        .expect("load deny-all");
    let component_allow = resolver
        .load_with_policy(&wasm, &allow_host)
        .expect("load allow-host");

    assert_eq!(
        resolver.cached_count(),
        1,
        "policies must not duplicate cache entries"
    );

    // Deny-all variant surfaces UnauthorizedImport.
    let err = component_deny
        .invoke("process_order", &[Val::S32(1)])
        .unwrap_err();
    assert!(matches!(err, RuntimeError::UnauthorizedImport { .. }));

    // Allow-host variant passes the policy gate and fails downstream on
    // instantiation (no host functions provided — expected).
    let err = component_allow
        .invoke("process_order", &[Val::S32(1)])
        .unwrap_err();
    assert!(matches!(err, RuntimeError::InstantiationFailed(_)));
}
