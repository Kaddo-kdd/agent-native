//! AN-040: Cross-tenant blast radius containment and isolation verification test suite.
//!
//! Validates empirical guarantees under ADR-005 (zero instance reuse, ephemeral Stores):
//! - E1: Fuel exhaustion in Tenant A does not degrade or disrupt Tenant B.
//! - E2: Memory limits and growth denial in Tenant A do not leak or pollute Tenant B.
//! - E3: Host capability policies strictly enforce isolation per tenant.
//! - E4: Rapid alternating interleaved executions (A-B-A-B) maintain 100% isolation.
//! - E5: Concurrent multi-threaded execution preserves isolation without data races or panics.

use agent_native_runtime::{Engine, HostPolicy, InstanceLimits, RuntimeError, TenantConfig, Val};
use std::sync::Arc;
use std::thread;
use wasm_encoder::{
    BlockType, CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection,
    Instruction, MemorySection, MemoryType, Module, TypeSection, ValType,
};

/// Emits a WASM module with an infinite loop: `infinite_loop() -> i32`.
fn wasm_module_with_infinite_loop() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I32]);
    module.section(&types);

    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);

    let mut exports = ExportSection::new();
    exports.export("infinite_loop", ExportKind::Func, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut func = Function::new([]);
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::Br(0));
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::I32Const(0));
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    module.finish()
}

/// Emits a math module: `calculate(a: i32, b: i32) -> i32` computing `(a * 2) + b`.
fn wasm_module_math() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types
        .ty()
        .function([ValType::I32, ValType::I32], [ValType::I32]);
    module.section(&types);

    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);

    let mut exports = ExportSection::new();
    exports.export("calculate", ExportKind::Func, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut func = Function::new([]);
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Const(2));
    func.instruction(&Instruction::I32Mul);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    module.finish()
}

/// Emits a module with memory grow and store operations:
/// - Func 0: `grow_or_trap(delta: i32) -> i32`
/// - Func 1: `alloc_and_fill(delta: i32, val: i32) -> i32`
fn wasm_module_memory() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], [ValType::I32]); // Func 0
    types
        .ty()
        .function([ValType::I32, ValType::I32], [ValType::I32]); // Func 1
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
    exports.export("alloc_and_fill", ExportKind::Func, 1);
    exports.export("memory", ExportKind::Memory, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();

    // Func 0: grow_or_trap(delta) -> i32
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

    // Func 1: alloc_and_fill(delta, val) -> i32
    // Grows by delta, stores val at byte offset 0, and returns the val read back.
    let mut f1 = Function::new([]);
    f1.instruction(&Instruction::LocalGet(0));
    f1.instruction(&Instruction::MemoryGrow(0));
    f1.instruction(&Instruction::Drop);
    f1.instruction(&Instruction::I32Const(0)); // address 0
    f1.instruction(&Instruction::LocalGet(1)); // val
    f1.instruction(&Instruction::I32Store(wasm_encoder::MemArg {
        offset: 0,
        align: 2,
        memory_index: 0,
    }));
    f1.instruction(&Instruction::I32Const(0));
    f1.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
        offset: 0,
        align: 2,
        memory_index: 0,
    }));
    f1.instruction(&Instruction::End);
    codes.function(&f1);

    module.section(&codes);
    module.finish()
}

/// Emits a module declaring an import `(import "storage" "get" (func (param i32) (result i32)))`.
fn wasm_module_with_storage_import() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], [ValType::I32]);
    module.section(&types);

    let mut imports = ImportSection::new();
    imports.import("storage", "get", wasm_encoder::EntityType::Function(0));
    module.section(&imports);

    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);

    let mut exports = ExportSection::new();
    exports.export("query", ExportKind::Func, 1);
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

#[derive(Debug, Default)]
struct MockStorageContext {
    authorized_key: i32,
    stored_value: i32,
}

// -----------------------------------------------------------------------------
// Scenario E1: CPU Blast Radius
// -----------------------------------------------------------------------------

#[test]
fn test_scenario_e1_cpu_blast_radius_containment() {
    let engine = Engine::new().expect("engine");
    let loop_wasm = wasm_module_with_infinite_loop();
    let math_wasm = wasm_module_math();

    let loop_component = engine.load_component(&loop_wasm).expect("load loop");
    let math_component = engine.load_component(&math_wasm).expect("load math");

    // Tenant A: Malicious/infinite loop with tight fuel limit
    let tenant_a =
        engine.create_tenant(TenantConfig::new("tenant-a-hostile").with_fuel_budget(5_000));

    // Tenant B: Benign tenant with standard fuel budget
    let tenant_b =
        engine.create_tenant(TenantConfig::new("tenant-b-benign").with_fuel_budget(50_000));

    // 1. Tenant A exhausts fuel and traps deterministically
    let err_a = tenant_a
        .invoke(&loop_component, "infinite_loop", &[])
        .expect_err("Tenant A must fail with FuelExhausted");
    match err_a {
        RuntimeError::FuelExhausted { limit } => assert_eq!(limit, 5_000),
        other => panic!("expected FuelExhausted, got {other:?}"),
    }

    // 2. Tenant B immediately runs in its fresh Store without degradation or residue
    let report_b = tenant_b
        .invoke(&math_component, "calculate", &[Val::S32(10), Val::S32(5)])
        .expect("Tenant B must succeed immediately");
    assert_eq!(report_b.values, vec![Val::S32(25)]); // (10 * 2) + 5 = 25
    let consumed_b = report_b.fuel_consumed.expect("Tenant B fuel consumed");
    assert!(consumed_b > 0 && consumed_b <= 50_000);
}

// -----------------------------------------------------------------------------
// Scenario E2: Memory Blast Radius
// -----------------------------------------------------------------------------

#[test]
fn test_scenario_e2_memory_blast_radius_containment() {
    let engine = Engine::new().expect("engine");
    let mem_wasm = wasm_module_memory();
    let component = engine
        .load_component(&mem_wasm)
        .expect("load mem component");

    // Tenant A: strictly limited to 2 pages (128 KiB = 131_072 bytes)
    let limits_a = InstanceLimits::new(131_072);
    let tenant_a = engine.create_tenant(
        TenantConfig::new("tenant-a-greedy")
            .with_instance_limits(limits_a)
            .with_fuel_budget(100_000),
    );

    // Tenant B: standard allocation allowance of 8 pages (512 KiB)
    let limits_b = InstanceLimits::new(524_288);
    let tenant_b = engine.create_tenant(
        TenantConfig::new("tenant-b-standard")
            .with_instance_limits(limits_b)
            .with_fuel_budget(100_000),
    );

    // 1. Tenant A attempts to grow by 500 pages (32 MiB > 128 KiB).
    // Denied by ResourceLimiter and guest traps unreachable.
    let err_a = tenant_a
        .invoke(&component, "grow_or_trap", &[Val::S32(500)])
        .expect_err("Tenant A must fail with MemoryLimitExceeded");
    match err_a {
        RuntimeError::MemoryLimitExceeded {
            requested_bytes,
            max_bytes,
        } => {
            assert_eq!(max_bytes, 131_072);
            assert_eq!(requested_bytes, 501 * 65_536);
        }
        other => panic!("expected MemoryLimitExceeded, got {other:?}"),
    }

    // 2. Tenant B immediately runs in its fresh Store, allocates 2 pages, writes 1337, reads 1337 back.
    let report_b = tenant_b
        .invoke(&component, "alloc_and_fill", &[Val::S32(2), Val::S32(1337)])
        .expect("Tenant B must succeed with clean memory");
    assert_eq!(report_b.values, vec![Val::S32(1337)]);
}

// -----------------------------------------------------------------------------
// Scenario E3: Capability & Policy Isolation
// -----------------------------------------------------------------------------

#[test]
fn test_scenario_e3_capability_policy_isolation() {
    let engine = Engine::new().expect("engine");
    let wasm = wasm_module_with_storage_import();
    let component = engine
        .load_component(&wasm)
        .expect("load storage component");

    // Tenant A: Deny-all policy (no storage authorization)
    let tenant_a = engine.create_tenant(TenantConfig::new("tenant-a-unauthorized"));

    // Tenant B: Authorizes ("storage", "get")
    let mut policy_b = HostPolicy::deny_all();
    policy_b.allow_core("storage", "get");
    let tenant_b = engine.create_tenant(
        TenantConfig::new("tenant-b-authorized")
            .with_policy(policy_b)
            .with_fuel_budget(50_000),
    );

    // Tenant A is pre-checked and rejected
    let check_a = tenant_a.validate_component(&component);
    assert!(matches!(
        check_a,
        Err(RuntimeError::UnauthorizedImport { .. })
    ));

    let invoke_a = tenant_a.invoke(&component, "query", &[Val::S32(1)]);
    assert!(matches!(
        invoke_a,
        Err(RuntimeError::UnauthorizedImport { .. })
    ));

    // Tenant B validates and executes with its private host context
    assert!(tenant_b.validate_component(&component).is_ok());

    let ctx_b = MockStorageContext {
        authorized_key: 101,
        stored_value: 9999,
    };

    let (report_b, final_ctx) = tenant_b
        .invoke_core_with_host_ctx(
            &component,
            "query",
            &[Val::S32(101)],
            ctx_b,
            |_engine, linker| {
                linker.func_wrap(
                    "storage",
                    "get",
                    |caller: wasmtime::Caller<'_, MockStorageContext>, key: i32| -> i32 {
                        let data = caller.data();
                        if key == data.authorized_key {
                            data.stored_value
                        } else {
                            -1
                        }
                    },
                )?;
                Ok(())
            },
        )
        .expect("Tenant B invoke must succeed");

    assert_eq!(report_b.values, vec![Val::S32(9999)]);
    assert_eq!(final_ctx.stored_value, 9999);
}

// -----------------------------------------------------------------------------
// Scenario E4: Rapid Interleaving (A-B-A-B Sequence)
// -----------------------------------------------------------------------------

#[test]
fn test_scenario_e4_rapid_blast_interleaving() {
    let engine = Engine::new().expect("engine");
    let loop_wasm = wasm_module_with_infinite_loop();
    let math_wasm = wasm_module_math();

    let loop_component = engine.load_component(&loop_wasm).expect("load loop");
    let math_component = engine.load_component(&math_wasm).expect("load math");

    let tenant_a =
        engine.create_tenant(TenantConfig::new("interleaved-hostile").with_fuel_budget(1_000));
    let tenant_b =
        engine.create_tenant(TenantConfig::new("interleaved-benign").with_fuel_budget(50_000));

    // 100 alternating iterations
    for i in 0..100 {
        // Step A: Tenant A fails with FuelExhausted
        let err_a = tenant_a
            .invoke(&loop_component, "infinite_loop", &[])
            .expect_err("iteration A must fail");
        assert!(matches!(
            err_a,
            RuntimeError::FuelExhausted { limit: 1_000 }
        ));

        // Step B: Tenant B succeeds with exact math
        let report_b = tenant_b
            .invoke(&math_component, "calculate", &[Val::S32(i), Val::S32(1)])
            .expect("iteration B must succeed");
        let expected = (i * 2) + 1;
        assert_eq!(report_b.values, vec![Val::S32(expected)]);
    }
}

// -----------------------------------------------------------------------------
// Scenario E5: Multi-Threaded Concurrency
// -----------------------------------------------------------------------------

#[test]
fn test_scenario_e5_multithreaded_concurrency() {
    let engine = Arc::new(Engine::new().expect("engine"));
    let loop_wasm = Arc::new(wasm_module_with_infinite_loop());
    let math_wasm = Arc::new(wasm_module_math());

    let loop_component = Arc::new(engine.load_component(&loop_wasm).expect("load loop"));
    let math_component = Arc::new(engine.load_component(&math_wasm).expect("load math"));

    let tenant_a = Arc::new(
        engine.create_tenant(TenantConfig::new("threaded-hostile").with_fuel_budget(2_000)),
    );
    let tenant_b = Arc::new(
        engine.create_tenant(TenantConfig::new("threaded-benign").with_fuel_budget(50_000)),
    );

    let mut handles = Vec::new();

    // Spawn 2 threads running Tenant A (failing repeatedly)
    for _ in 0..2 {
        let t_a = Arc::clone(&tenant_a);
        let c_loop = Arc::clone(&loop_component);
        handles.push(thread::spawn(move || {
            for _ in 0..50 {
                let err = t_a
                    .invoke(&c_loop, "infinite_loop", &[])
                    .expect_err("Thread A must trap");
                assert!(matches!(err, RuntimeError::FuelExhausted { .. }));
            }
        }));
    }

    // Spawn 2 threads running Tenant B (succeeding repeatedly)
    for t_idx in 0..2 {
        let t_b = Arc::clone(&tenant_b);
        let c_math = Arc::clone(&math_component);
        handles.push(thread::spawn(move || {
            for i in 0..50 {
                let report = t_b
                    .invoke(&c_math, "calculate", &[Val::S32(i), Val::S32(t_idx)])
                    .expect("Thread B must compute");
                assert_eq!(report.values, vec![Val::S32((i * 2) + t_idx)]);
            }
        }));
    }

    for handle in handles {
        handle.join().expect("thread join must not panic");
    }
}
