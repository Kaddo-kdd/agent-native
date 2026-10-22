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
            (func (export "increment") (result i32)
                global.get $state
                i32.const 1
                i32.add
                global.set $state
                global.get $state)
            (func (export "identity") (param i32) (result i32)
                local.get 0)
            (func (export "spin")
                (loop $forever
                    br $forever))
        )"#,
    )
    .expect("valid Core WASM fixture")
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
        Err(WorkerPoolInitError::Config(WorkerPoolConfigError::ZeroWorkers))
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

    let report = pool.close(ShutdownMode::RejectQueued);
    assert_eq!(queued.recv(), Err(WorkerError::Cancelled));
    assert!(matches!(
        running.recv(),
        Err(WorkerError::Runtime(RuntimeError::FuelExhausted { .. }))
    ));
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

    let report = pool.close(ShutdownMode::Drain);
    assert!(matches!(
        running.recv(),
        Err(WorkerError::Runtime(RuntimeError::FuelExhausted { .. }))
    ));
    assert_eq!(
        queued.recv().expect("drained result").values,
        vec![Val::S32(9)]
    );
    assert_eq!(report.stats.completed, 1);
    assert_eq!(report.stats.cancelled, 0);
}
