//! AN-065 contract tests for experimental bounded host-side workers.

#![cfg(feature = "experimental-workers")]

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use agent_native_runtime::workers::{
    CancelError, ShutdownMode, SubmitError, WorkerError, WorkerPool, WorkerPoolConfig,
    WorkerPoolConfigError, WorkerPoolInitError, WorkerRequest,
};
use agent_native_runtime::{
    Component, Engine, InvocationReport, RuntimeError, Tenant, TenantConfig, Val,
};

fn fixture() -> Vec<u8> {
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
            (func (export "identity") (param i32) (result i32)
                local.get 0)
            (func (export "write_then_read") (param i32) (result i32)
                i32.const 0
                local.get 0
                i32.store
                i32.const 0
                i32.load)
            (func (export "read_memory") (result i32)
                i32.const 0
                i32.load)
            (func (export "mutate_then_trap")
                i32.const 99
                global.set $state
                unreachable)
            (func (export "grow_then_trap")
                i32.const 1
                memory.grow
                drop
                unreachable)
            (func (export "spin")
                (loop $forever
                    br $forever))
        )"#,
    )
    .expect("valid Core WASM fixture")
}

fn imported_component(engine: &Engine) -> Arc<Component> {
    let bytes = wat::parse_str(
        r#"(module
            (import "env" "read" (func $read (param i32) (result i32)))
            (func (export "query") (param i32) (result i32)
                local.get 0
                call $read)
        )"#,
    )
    .expect("valid imported Core WASM fixture");
    Arc::new(engine.load_component(&bytes).expect("imported component"))
}

fn component() -> (Engine, Arc<Component>) {
    let engine = Engine::new().expect("engine");
    let component = Arc::new(engine.load_component(&fixture()).expect("component"));
    (engine, component)
}

fn request(
    component: Arc<Component>,
    tenant: Tenant,
    function: &str,
    arguments: Vec<Val>,
) -> WorkerRequest {
    WorkerRequest::new(component, tenant, function, arguments)
}

fn wait_for_running(pool: &WorkerPool, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while pool.stats().running != expected {
        assert!(
            Instant::now() < deadline,
            "worker pool did not reach {expected} running request(s)"
        );
        thread::yield_now();
    }
}

#[test]
fn worker_messages_are_thread_safe_without_unsafe_adapters() {
    fn assert_send_sync<T: Send + Sync>() {}
    fn assert_send<T: Send>() {}

    assert_send_sync::<Component>();
    assert_send_sync::<Tenant>();
    assert_send_sync::<Val>();
    assert_send_sync::<InvocationReport>();
    assert_send_sync::<RuntimeError>();
    assert_send::<WorkerRequest>();
}

#[test]
fn worker_pool_rejects_zero_capacities() {
    assert!(matches!(
        WorkerPool::new(WorkerPoolConfig::new(0, 1)),
        Err(WorkerPoolInitError::Config(
            WorkerPoolConfigError::ZeroWorkers
        ))
    ));
    assert!(matches!(
        WorkerPool::new(WorkerPoolConfig::new(1, 0)),
        Err(WorkerPoolInitError::Config(
            WorkerPoolConfigError::ZeroQueueCapacity
        ))
    ));
}

#[test]
fn worker_pool_rejects_requests_without_a_finite_fuel_budget() {
    let (engine, component) = component();
    let pool = WorkerPool::new(WorkerPoolConfig::new(1, 1)).expect("pool");
    let tenant = engine.create_tenant(TenantConfig::new("unbounded"));

    assert!(matches!(
        pool.submit(request(component, tenant, "identity", vec![Val::S32(1)])),
        Err(SubmitError::UnboundedExecution)
    ));
    let _ = pool.close(ShutdownMode::RejectQueued);
}

#[test]
fn each_worker_request_uses_a_fresh_guest_instance() {
    let (engine, component) = component();
    let pool = WorkerPool::new(WorkerPoolConfig::new(1, 2)).expect("pool");
    let tenant_a = engine.create_tenant(TenantConfig::new("tenant-a").with_fuel_budget(10_000));
    let tenant_b = engine.create_tenant(TenantConfig::new("tenant-b").with_fuel_budget(10_000));

    let first = pool
        .submit(request(component.clone(), tenant_a, "increment", vec![]))
        .expect("first request")
        .recv()
        .expect("first result");
    let second = pool
        .submit(request(component, tenant_b, "increment", vec![]))
        .expect("second request")
        .recv()
        .expect("second result");

    assert_eq!(first.values, vec![Val::S32(1)]);
    assert_eq!(second.values, vec![Val::S32(1)]);
    let report = pool.close(ShutdownMode::Drain);
    assert_eq!(report.stats.completed, 2);
}

#[test]
fn concurrent_tenants_keep_globals_and_memory_isolated() {
    let (engine, component) = component();
    let pool = WorkerPool::new(WorkerPoolConfig::new(2, 2)).expect("pool");
    let tenant_a = engine.create_tenant(TenantConfig::new("tenant-a").with_fuel_budget(10_000_000));
    let tenant_b = engine.create_tenant(TenantConfig::new("tenant-b").with_fuel_budget(10_000_000));

    let spinning_a = pool
        .submit(request(component.clone(), tenant_a.clone(), "spin", vec![]))
        .expect("first spin accepted");
    wait_for_running(&pool, 1);
    let spinning_b = pool
        .submit(request(component.clone(), tenant_b.clone(), "spin", vec![]))
        .expect("second spin accepted");
    wait_for_running(&pool, 2);
    assert!(matches!(
        spinning_a.recv(),
        Err(WorkerError::Runtime(RuntimeError::FuelExhausted { .. }))
    ));
    assert!(matches!(
        spinning_b.recv(),
        Err(WorkerError::Runtime(RuntimeError::FuelExhausted { .. }))
    ));

    let global_a = pool
        .submit(request(
            component.clone(),
            tenant_a.clone(),
            "increment",
            vec![],
        ))
        .expect("tenant A increment accepted");
    let global_b = pool
        .submit(request(
            component.clone(),
            tenant_b.clone(),
            "increment",
            vec![],
        ))
        .expect("tenant B increment accepted");
    assert_eq!(
        global_a.recv().expect("tenant A global").values,
        vec![Val::S32(1)]
    );
    assert_eq!(
        global_b.recv().expect("tenant B global").values,
        vec![Val::S32(1)]
    );

    let memory_a = pool
        .submit(request(
            component.clone(),
            tenant_a,
            "write_then_read",
            vec![Val::S32(41)],
        ))
        .expect("tenant A memory write accepted");
    let memory_b = pool
        .submit(request(
            component.clone(),
            tenant_b,
            "write_then_read",
            vec![Val::S32(7)],
        ))
        .expect("tenant B memory write accepted");
    assert_eq!(
        memory_a.recv().expect("tenant A memory").values,
        vec![Val::S32(41)]
    );
    assert_eq!(
        memory_b.recv().expect("tenant B memory").values,
        vec![Val::S32(7)]
    );

    let clean_memory = pool
        .submit(request(
            component,
            engine.create_tenant(TenantConfig::new("tenant-clean").with_fuel_budget(10_000)),
            "read_memory",
            vec![],
        ))
        .expect("clean memory read accepted");
    assert_eq!(
        clean_memory.recv().expect("fresh memory").values,
        vec![Val::S32(0)]
    );
    let report = pool.close(ShutdownMode::Drain);
    assert_eq!(report.stats.running, 0);
}

#[test]
fn tenant_policy_and_memory_failures_do_not_leak_to_next_request() {
    let (engine, component) = component();
    let imported = imported_component(&engine);
    let pool = WorkerPool::new(WorkerPoolConfig::new(1, 2)).expect("pool");
    let denied = engine.create_tenant(TenantConfig::new("denied").with_fuel_budget(10_000));
    let tight_memory = engine.create_tenant(
        TenantConfig::new("tight-memory")
            .with_fuel_budget(10_000)
            .with_max_memory_bytes(65_536),
    );
    let healthy = engine.create_tenant(TenantConfig::new("healthy").with_fuel_budget(10_000));

    let denied_result = pool
        .submit(request(imported, denied, "query", vec![Val::S32(3)]))
        .expect("denied request accepted")
        .recv();
    assert!(matches!(
        denied_result,
        Err(WorkerError::Runtime(
            RuntimeError::UnauthorizedImport { .. }
        ))
    ));

    let trap_result = pool
        .submit(request(
            component.clone(),
            healthy.clone(),
            "mutate_then_trap",
            vec![],
        ))
        .expect("trapping request accepted")
        .recv();
    assert!(matches!(
        trap_result,
        Err(WorkerError::Runtime(RuntimeError::ExecutionFailed(_)))
    ));

    let memory_result = pool
        .submit(request(
            component.clone(),
            tight_memory,
            "grow_then_trap",
            vec![],
        ))
        .expect("memory-limited request accepted")
        .recv();
    assert!(matches!(
        memory_result,
        Err(WorkerError::Runtime(
            RuntimeError::MemoryLimitExceeded { .. }
        ))
    ));

    assert_eq!(
        pool.submit(request(component, healthy, "identity", vec![Val::S32(5)]))
            .expect("healthy request accepted after failures")
            .recv()
            .expect("healthy result")
            .values,
        vec![Val::S32(5)]
    );
    let report = pool.close(ShutdownMode::Drain);
    assert_eq!(report.stats.failed, 3);
    assert_eq!(report.stats.completed, 1);
}

#[test]
fn runtime_failures_are_local_and_capacity_recovers() {
    let (engine, component) = component();
    let pool = WorkerPool::new(WorkerPoolConfig::new(1, 2)).expect("pool");
    let tenant = engine.create_tenant(TenantConfig::new("tenant").with_fuel_budget(10_000));

    let failed = pool
        .submit(request(
            component.clone(),
            tenant.clone(),
            "missing",
            vec![],
        ))
        .expect("accepted failing request")
        .recv()
        .expect_err("missing export must fail");
    assert!(matches!(
        failed,
        WorkerError::Runtime(RuntimeError::FunctionNotFound(_))
    ));

    let healthy = pool
        .submit(request(component, tenant, "identity", vec![Val::S32(7)]))
        .expect("subsequent request accepted")
        .recv()
        .expect("subsequent result");
    assert_eq!(healthy.values, vec![Val::S32(7)]);

    let report = pool.close(ShutdownMode::Drain);
    assert_eq!(report.stats.failed, 1);
    assert_eq!(report.stats.completed, 1);
}

#[test]
fn queue_backpressure_and_queued_cancellation_release_capacity() {
    let (engine, component) = component();
    let pool = WorkerPool::new(WorkerPoolConfig::new(1, 1)).expect("pool");
    let bounded = engine.create_tenant(TenantConfig::new("bounded").with_fuel_budget(50_000_000));
    let ordinary = engine.create_tenant(TenantConfig::new("ordinary").with_fuel_budget(10_000));

    let running = pool
        .submit(request(component.clone(), bounded, "spin", vec![]))
        .expect("spin accepted");
    let deadline = Instant::now() + Duration::from_secs(3);
    while pool.stats().running != 1 {
        assert!(
            Instant::now() < deadline,
            "worker did not start the bounded request"
        );
        thread::yield_now();
    }

    let queued = pool
        .submit(request(
            component.clone(),
            ordinary.clone(),
            "identity",
            vec![Val::S32(2)],
        ))
        .expect("queued request accepted");
    assert!(matches!(
        pool.submit(request(
            component.clone(),
            ordinary.clone(),
            "identity",
            vec![Val::S32(3)],
        )),
        Err(SubmitError::QueueFull { queue_capacity: 1 })
    ));

    queued.try_cancel().expect("queued ticket is removable");
    assert_eq!(queued.recv(), Err(WorkerError::Cancelled));

    let admitted_after_cancel = pool
        .submit(request(component, ordinary, "identity", vec![Val::S32(4)]))
        .expect("cancelled slot is immediately reusable");
    assert_eq!(
        running.recv(),
        Err(WorkerError::Runtime(RuntimeError::FuelExhausted {
            limit: 50_000_000
        }))
    );
    assert_eq!(
        admitted_after_cancel
            .recv()
            .expect("new request result")
            .values,
        vec![Val::S32(4)]
    );
    let report = pool.close(ShutdownMode::Drain);
    assert_eq!(report.stats.rejected, 1);
    assert_eq!(report.stats.cancelled, 1);
}

#[test]
fn running_work_is_not_force_cancelled() {
    let (engine, component) = component();
    let pool = WorkerPool::new(WorkerPoolConfig::new(1, 1)).expect("pool");
    let tenant = engine.create_tenant(TenantConfig::new("bounded").with_fuel_budget(50_000_000));
    let running = pool
        .submit(request(component, tenant, "spin", vec![]))
        .expect("spin accepted");
    let deadline = Instant::now() + Duration::from_secs(3);
    while pool.stats().running != 1 {
        assert!(
            Instant::now() < deadline,
            "worker did not start the bounded request"
        );
        thread::yield_now();
    }

    assert_eq!(running.try_cancel(), Err(CancelError::AlreadyRunning));
    assert!(matches!(
        running.recv(),
        Err(WorkerError::Runtime(RuntimeError::FuelExhausted { .. }))
    ));
    let _ = pool.close(ShutdownMode::Drain);
}

#[test]
fn reject_queued_shutdown_cancels_only_work_that_has_not_started() {
    let (engine, component) = component();
    let pool = WorkerPool::new(WorkerPoolConfig::new(1, 1)).expect("pool");
    let bounded = engine.create_tenant(TenantConfig::new("bounded").with_fuel_budget(50_000_000));
    let ordinary = engine.create_tenant(TenantConfig::new("ordinary").with_fuel_budget(10_000));
    let running = pool
        .submit(request(component.clone(), bounded, "spin", vec![]))
        .expect("spin accepted");
    let deadline = Instant::now() + Duration::from_secs(3);
    while pool.stats().running != 1 {
        assert!(
            Instant::now() < deadline,
            "worker did not start the bounded request"
        );
        thread::yield_now();
    }
    let queued = pool
        .submit(request(component, ordinary, "identity", vec![Val::S32(8)]))
        .expect("queued request accepted");

    let closer = thread::spawn(move || pool.close(ShutdownMode::RejectQueued));
    assert_eq!(queued.recv(), Err(WorkerError::Cancelled));
    assert!(matches!(
        running.recv(),
        Err(WorkerError::Runtime(RuntimeError::FuelExhausted { .. }))
    ));
    let report = closer.join().expect("close thread must not panic");
    assert_eq!(report.stats.cancelled, 1);
    assert_eq!(report.stats.running, 0);
}

#[test]
fn drain_shutdown_finishes_work_already_in_the_queue() {
    let (engine, component) = component();
    let pool = WorkerPool::new(WorkerPoolConfig::new(1, 1)).expect("pool");
    let bounded = engine.create_tenant(TenantConfig::new("bounded").with_fuel_budget(50_000_000));
    let ordinary = engine.create_tenant(TenantConfig::new("ordinary").with_fuel_budget(10_000));
    let running = pool
        .submit(request(component.clone(), bounded, "spin", vec![]))
        .expect("spin accepted");
    let deadline = Instant::now() + Duration::from_secs(3);
    while pool.stats().running != 1 {
        assert!(
            Instant::now() < deadline,
            "worker did not start the bounded request"
        );
        thread::yield_now();
    }
    let queued = pool
        .submit(request(component, ordinary, "identity", vec![Val::S32(9)]))
        .expect("queued request accepted");

    let closer = thread::spawn(move || pool.close(ShutdownMode::Drain));
    assert!(matches!(
        running.recv(),
        Err(WorkerError::Runtime(RuntimeError::FuelExhausted { .. }))
    ));
    assert_eq!(
        queued.recv().expect("drained result").values,
        vec![Val::S32(9)]
    );
    let report = closer.join().expect("close thread must not panic");
    assert_eq!(report.stats.completed, 1);
    assert_eq!(report.stats.cancelled, 0);
}
