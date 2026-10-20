//! AN-039: Tenant API, governance, and unified execution lifecycle under homogeneous limits.

use agent_native_runtime::{
    Engine, ExecutionLimits, HostPolicy, InstanceLimits, RuntimeError, Tenant, TenantConfig,
    TenantId, Val,
};
use wasm_encoder::{
    BlockType, CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection,
    Instruction, MemorySection, MemoryType, Module, TypeSection, ValType,
};

/// Emits a simple math module with `add(a, b) -> a + b`.
fn sample_math_wasm() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([ValType::I32, ValType::I32], [ValType::I32]);
    module.section(&types);

    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);

    let mut exports = ExportSection::new();
    exports.export("add", ExportKind::Func, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut func = Function::new([]);
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    module.finish()
}

/// Emits a module with:
/// - Func 0: `grow_or_trap(delta: i32) -> i32` (if grow == -1, traps with unreachable)
/// - Func 1: `grow_and_div_zero(delta: i32) -> i32` (grows, ignores return, then divides 1 / 0)
fn sample_memory_trap_wasm() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], [ValType::I32]); // Func 0
    types.ty().function([ValType::I32], [ValType::I32]); // Func 1
    module.section(&types);

    let mut funcs = FunctionSection::new();
    funcs.function(0);
    funcs.function(1);
    module.section(&funcs);

    let mut memories = MemorySection::new();
    memories.memory(MemoryType {
        minimum: 1, // 64 KiB
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memories);

    let mut exports = ExportSection::new();
    exports.export("grow_or_trap", ExportKind::Func, 0);
    exports.export("grow_and_div_zero", ExportKind::Func, 1);
    exports.export("memory", ExportKind::Memory, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();

    // Func 0: grow_or_trap
    let mut f0 = Function::new([(1, ValType::I32)]);
    f0.instruction(&Instruction::LocalGet(0));
    f0.instruction(&Instruction::MemoryGrow(0));
    f0.instruction(&Instruction::LocalTee(1));
    f0.instruction(&Instruction::I32Const(-1));
    f0.instruction(&Instruction::I32Eq);
    f0.instruction(&Instruction::If(BlockType::Empty));
    f0.instruction(&Instruction::Unreachable);
    f0.instruction(&Instruction::End);
    f0.instruction(&Instruction::LocalGet(1));
    f0.instruction(&Instruction::End);
    codes.function(&f0);

    // Func 1: grow_and_div_zero
    let mut f1 = Function::new([]);
    f1.instruction(&Instruction::LocalGet(0));
    f1.instruction(&Instruction::MemoryGrow(0));
    f1.instruction(&Instruction::Drop); // ignore memory.grow result!
    f1.instruction(&Instruction::I32Const(42));
    f1.instruction(&Instruction::I32Const(0));
    f1.instruction(&Instruction::I32DivS); // trap: division by zero!
    f1.instruction(&Instruction::End);
    codes.function(&f1);

    module.section(&codes);
    module.finish()
}

/// Emits a module declaring an import `(import "env" "read" (func (param i32) (result i32)))`.
fn sample_imported_wasm() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], [ValType::I32]); // type 0
    module.section(&types);

    let mut imports = ImportSection::new();
    imports.import("env", "read", wasm_encoder::EntityType::Function(0));
    module.section(&imports);

    let mut funcs = FunctionSection::new();
    funcs.function(0); // type 0
    module.section(&funcs);

    let mut exports = ExportSection::new();
    exports.export("run", ExportKind::Func, 1);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut func = Function::new([]);
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::Call(0));
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    module.finish()
}

#[test]
fn test_tenant_identity_and_config_builder() {
    let tenant_id = TenantId::new("tenant-alpha");
    assert_eq!(tenant_id.as_str(), "tenant-alpha");
    assert_eq!(format!("{tenant_id}"), "tenant-alpha");

    let mut policy = HostPolicy::deny_all();
    policy.allow_core("env", "read");

    let limits = ExecutionLimits::unmetered()
        .with_fuel(50_000)
        .with_max_memory_bytes(64 * 1024 * 1024);

    let config = TenantConfig::new(&tenant_id)
        .with_policy(policy.clone())
        .with_execution_limits(limits)
        .with_fuel_budget(100_000)
        .with_max_memory_bytes(32 * 1024 * 1024);

    assert_eq!(config.id, tenant_id);
    assert_eq!(config.limits.fuel_budget, Some(100_000));
    assert_eq!(config.limits.max_memory_bytes, Some(32 * 1024 * 1024));

    let tenant = Tenant::new(config);
    assert_eq!(tenant.id().as_str(), "tenant-alpha");
    assert_eq!(tenant.config().limits.fuel_budget, Some(100_000));
}

#[test]
fn test_tenant_invoke_with_homogeneous_limits_and_fuel_accounting() {
    let engine = Engine::new().expect("engine");
    let wasm = sample_math_wasm();
    let component = engine.load_component(&wasm).expect("load component");

    // Tenant A: finite fuel budget of 50,000 units
    let config_a = TenantConfig::new("tenant-a").with_fuel_budget(50_000);
    let tenant_a = engine.create_tenant(config_a);

    // Tenant B: unmetered execution
    let config_b = TenantConfig::new("tenant-b");
    let tenant_b = engine.create_tenant(config_b);

    // Both tenants invoke the SAME shared component:
    let report_a = tenant_a
        .invoke(&component, "add", &[Val::S32(10), Val::S32(25)])
        .expect("tenant A invoke");
    assert_eq!(report_a.values, vec![Val::S32(35)]);
    let fuel_a = report_a.fuel_consumed.expect("fuel consumed for tenant A");
    assert!(fuel_a > 0 && fuel_a <= 50_000);

    let report_b = tenant_b
        .invoke(&component, "add", &[Val::S32(100), Val::S32(200)])
        .expect("tenant B invoke");
    assert_eq!(report_b.values, vec![Val::S32(300)]);
    assert_eq!(report_b.fuel_consumed, None, "unmetered tenant reports None fuel");
}

#[test]
fn test_tenant_fuel_exhaustion_does_not_affect_next_tenant() {
    let engine = Engine::new().expect("engine");
    let wasm = sample_math_wasm();
    let component = engine.load_component(&wasm).expect("load component");

    // Tenant A: starvation quota (budget = 1 unit)
    let tenant_a = engine.create_tenant(TenantConfig::new("starved-tenant").with_fuel_budget(1));

    // Tenant B: normal quota (budget = 50,000 units)
    let tenant_b = engine.create_tenant(TenantConfig::new("normal-tenant").with_fuel_budget(50_000));

    // Tenant A fails immediately with FuelExhausted
    let err_a = tenant_a
        .invoke(&component, "add", &[Val::S32(2), Val::S32(3)])
        .expect_err("tenant A must exhaust fuel");
    match err_a {
        RuntimeError::FuelExhausted { limit } => assert_eq!(limit, 1),
        other => panic!("expected FuelExhausted, got {other:?}"),
    }

    // Tenant B runs immediately in its own fresh Store (ADR-005) without degradation
    let report_b = tenant_b
        .invoke(&component, "add", &[Val::S32(2), Val::S32(3)])
        .expect("tenant B must succeed normally");
    assert_eq!(report_b.values, vec![Val::S32(5)]);
    assert!(report_b.fuel_consumed.is_some());
}

#[test]
fn test_tenant_policy_enforcement_per_tenant() {
    let engine = Engine::new().expect("engine");
    let wasm = sample_imported_wasm();
    let component = engine.load_component(&wasm).expect("load component");

    // Tenant A: Deny-all policy (default)
    let tenant_a = engine.create_tenant(TenantConfig::new("tenant-unauthorized"));

    // Tenant B: Explicitly authorized for ("env", "read")
    let mut policy_b = HostPolicy::deny_all();
    policy_b.allow_core("env", "read");
    let tenant_b = engine.create_tenant(TenantConfig::new("tenant-authorized").with_policy(policy_b));

    // Pre-execution validation check:
    let check_a = tenant_a.validate_component(&component);
    assert!(matches!(check_a, Err(RuntimeError::UnauthorizedImport { .. })));

    let check_b = tenant_b.validate_component(&component);
    assert!(check_b.is_ok(), "tenant B policy authorizes the import");

    // Invocation check:
    let invoke_a = tenant_a.invoke(&component, "run", &[Val::S32(10)]);
    assert!(matches!(invoke_a, Err(RuntimeError::UnauthorizedImport { .. })));
}

#[test]
fn test_tenant_decoupled_memory_trap_diagnosis() {
    let engine = Engine::new().expect("engine");
    let wasm = sample_memory_trap_wasm();
    let component = engine.load_component(&wasm).expect("load component");

    // Set memory ceiling to 1 page (64 KiB = 65,536 bytes)
    // Initial memory is 1 page, so any growth attempt by 1 or more pages is DENIED.
    let inst_limits = InstanceLimits::new(65_536);
    let tenant = engine.create_tenant(
        TenantConfig::new("memory-tenant")
            .with_instance_limits(inst_limits)
            .with_fuel_budget(50_000),
    );

    // Case 1: grow_or_trap requests 2 pages. memory.grow returns -1. Guest triggers `unreachable`.
    // Correctly classified as MemoryLimitExceeded!
    let err1 = tenant
        .invoke(&component, "grow_or_trap", &[Val::S32(2)])
        .expect_err("grow_or_trap must fail");
    match err1 {
        RuntimeError::MemoryLimitExceeded {
            requested_bytes,
            max_bytes,
        } => {
            assert_eq!(max_bytes, 65_536);
            assert_eq!(requested_bytes, 196_608); // 3 pages = 192 KiB
        }
        other => panic!("expected MemoryLimitExceeded, got {other:?}"),
    }

    // Case 2: grow_and_div_zero requests 2 pages (growth is denied), but guest ignores return and divides by zero.
    // Must be classified as ExecutionFailed (divide by zero trap), NOT MemoryLimitExceeded!
    let err2 = tenant
        .invoke(&component, "grow_and_div_zero", &[Val::S32(2)])
        .expect_err("grow_and_div_zero must trap");
    match err2 {
        RuntimeError::ExecutionFailed(msg) => {
            assert!(
                msg.contains("integer divide by zero") || msg.contains("divide by zero"),
                "expected division by zero trap message, got: {msg}"
            );
        }
        RuntimeError::MemoryLimitExceeded { .. } => {
            panic!("REGRESSION: divide by zero was misclassified as MemoryLimitExceeded!");
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn test_tenant_invoke_core_with_host_ctx_and_limits() {
    let engine = Engine::new().expect("engine");
    let wasm = sample_imported_wasm();

    let mut policy = HostPolicy::deny_all();
    policy.allow_core("env", "read");

    let tenant = engine.create_tenant(
        TenantConfig::new("tenant-host-ctx")
            .with_policy(policy)
            .with_fuel_budget(25_000),
    );

    let component = engine.load_component(&wasm).expect("load component");

    #[derive(Debug, Default)]
    struct CustomHostCtx {
        calls: Vec<i32>,
    }

    let initial_ctx = CustomHostCtx::default();

    let (report, ctx) = tenant
        .invoke_core_with_host_ctx(
            &component,
            "run",
            &[Val::S32(42)],
            initial_ctx,
            |_engine, linker| {
                linker.func_wrap("env", "read", |mut caller: wasmtime::Caller<'_, CustomHostCtx>, arg: i32| -> i32 {
                    caller.data_mut().calls.push(arg);
                    arg * 2
                })?;
                Ok(())
            },
        )
        .expect("invoke_core_with_host_ctx under tenant");

    assert_eq!(report.values, vec![Val::S32(84)]);
    assert_eq!(ctx.calls, vec![42]);
    let fuel = report.fuel_consumed.expect("fuel consumed must be reported");
    assert!(fuel > 0 && fuel <= 25_000);
}
