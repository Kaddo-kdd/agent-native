//! AN-063 contract tests for a future warm-pool experiment.
//!
//! These tests specify the non-negotiable baseline that any pool must preserve:
//! every current invocation begins with a new Store and instance. They deliberately
//! exercise mutable guest globals and memory, a trap, fuel exhaustion, and tenant
//! policy. AN-063 introduces no pool or production reuse path.

use agent_native_runtime::{Engine, HostPolicy, RuntimeError, TenantConfig, Val};

fn stateful_wasm() -> Vec<u8> {
    wat::parse_str(
        r#"(module
            (global $state (mut i32) (i32.const 0))
            (memory 1)
            (func (export "increment") (result i32)
                global.get $state
                i32.const 1
                i32.add
                global.set $state
                global.get $state)
            (func (export "read_state") (result i32)
                global.get $state)
            (func (export "mutate_then_trap")
                i32.const 99
                global.set $state
                unreachable)
            (func (export "write_then_trap")
                i32.const 0
                i32.const 77
                i32.store
                unreachable)
            (func (export "read_memory") (result i32)
                i32.const 0
                i32.load)
            (func (export "spin")
                (loop $forever
                    br $forever))
        )"#,
    )
    .expect("valid stateful Core WASM fixture")
}

fn imported_wasm() -> Vec<u8> {
    wat::parse_str(
        r#"(module
            (import "env" "read" (func $read (param i32) (result i32)))
            (func (export "query") (param i32) (result i32)
                local.get 0
                call $read)
        )"#,
    )
    .expect("valid imported Core WASM fixture")
}

#[test]
fn fresh_instance_contract_discards_guest_globals_and_memory_after_success_and_trap() {
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component(&stateful_wasm())
        .expect("load stateful fixture");
    let tenant_a = engine.create_tenant(TenantConfig::new("tenant-a"));
    let tenant_b = engine.create_tenant(TenantConfig::new("tenant-b"));

    // A mutable global starts from zero in every invocation, including the same tenant.
    assert_eq!(
        tenant_a
            .invoke(&component, "increment", &[])
            .expect("first increment")
            .values,
        vec![Val::S32(1)]
    );
    assert_eq!(
        tenant_a
            .invoke(&component, "increment", &[])
            .expect("second increment uses a fresh instance")
            .values,
        vec![Val::S32(1)]
    );

    // A trap after mutating guest global state must not leave residue for another tenant.
    assert!(matches!(
        tenant_a.invoke(&component, "mutate_then_trap", &[]),
        Err(RuntimeError::ExecutionFailed(_))
    ));
    assert_eq!(
        tenant_b
            .invoke(&component, "read_state", &[])
            .expect("tenant B sees a clean global")
            .values,
        vec![Val::S32(0)]
    );

    // The same rule applies to linear memory even when the failing call wrote first.
    assert!(matches!(
        tenant_a.invoke(&component, "write_then_trap", &[]),
        Err(RuntimeError::ExecutionFailed(_))
    ));
    assert_eq!(
        tenant_b
            .invoke(&component, "read_memory", &[])
            .expect("tenant B sees clean linear memory")
            .values,
        vec![Val::S32(0)]
    );
}

#[test]
fn fresh_instance_contract_discards_fuel_exhausted_execution() {
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component(&stateful_wasm())
        .expect("load stateful fixture");
    let exhausted = engine.create_tenant(TenantConfig::new("exhausted").with_fuel_budget(1_000));
    let healthy = engine.create_tenant(TenantConfig::new("healthy").with_fuel_budget(50_000));

    assert!(matches!(
        exhausted.invoke(&component, "spin", &[]),
        Err(RuntimeError::FuelExhausted { limit: 1_000 })
    ));
    assert_eq!(
        healthy
            .invoke(&component, "read_state", &[])
            .expect("fuel exhaustion does not retain guest state")
            .values,
        vec![Val::S32(0)]
    );
}

#[test]
fn fresh_instance_contract_rechecks_policy_for_every_tenant() {
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component(&imported_wasm())
        .expect("load imported fixture");
    let denied = engine.create_tenant(TenantConfig::new("denied"));

    assert!(matches!(
        denied.invoke(&component, "query", &[Val::S32(7)]),
        Err(RuntimeError::UnauthorizedImport { .. })
    ));

    let mut allowed_policy = HostPolicy::deny_all();
    allowed_policy.allow_core("env", "read");
    let allowed = engine.create_tenant(TenantConfig::new("allowed").with_policy(allowed_policy));
    let (report, _) = allowed
        .invoke_core_with_host_ctx(
            &component,
            "query",
            &[Val::S32(7)],
            (),
            |_engine, linker| {
                linker.func_wrap(
                    "env",
                    "read",
                    |_caller: wasmtime::Caller<'_, ()>, value: i32| value * 3,
                )?;
                Ok(())
            },
        )
        .expect("authorised tenant can invoke with its own host context");
    assert_eq!(report.values, vec![Val::S32(21)]);
}
