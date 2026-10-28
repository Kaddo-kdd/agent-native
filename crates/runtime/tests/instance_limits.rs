//! AN-036: Instance limits and `ResourceLimiter` verification tests.

use wasm_encoder::{
    BlockType, CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction,
    MemorySection, MemoryType, Module, TypeSection, ValType,
};
use wasma_runtime::{Engine, ExecutionLimits, InstanceLimits, RuntimeError, Val};

fn wasm_module_with_initial_memory(min_pages: u64, max_pages: Option<u64>) -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I32]);
    module.section(&types);

    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);

    let mut memories = MemorySection::new();
    memories.memory(MemoryType {
        minimum: min_pages,
        maximum: max_pages,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memories);

    let mut exports = ExportSection::new();
    exports.export("get_val", ExportKind::Func, 0);
    exports.export("memory", ExportKind::Memory, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut func = Function::new([]);
    func.instruction(&Instruction::I32Const(42));
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    module.finish()
}

fn wasm_module_with_memory_grow() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], [ValType::I32]); // grow_memory(delta) -> i32
    types.ty().function([ValType::I32], [ValType::I32]); // grow_or_trap(delta) -> i32
    types.ty().function([], [ValType::I32]); // current_pages() -> i32
    module.section(&types);

    let mut funcs = FunctionSection::new();
    funcs.function(0);
    funcs.function(1);
    funcs.function(2);
    module.section(&funcs);

    let mut memories = MemorySection::new();
    memories.memory(MemoryType {
        minimum: 1, // 1 page = 64 KiB
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memories);

    let mut exports = ExportSection::new();
    exports.export("grow_memory", ExportKind::Func, 0);
    exports.export("grow_or_trap", ExportKind::Func, 1);
    exports.export("current_pages", ExportKind::Func, 2);
    exports.export("memory", ExportKind::Memory, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();

    // Func 0: grow_memory(delta) -> i32
    let mut f0 = Function::new([]);
    f0.instruction(&Instruction::LocalGet(0));
    f0.instruction(&Instruction::MemoryGrow(0));
    f0.instruction(&Instruction::End);
    codes.function(&f0);

    // Func 1: grow_or_trap(delta) -> i32
    let mut f1 = Function::new([(1, ValType::I32)]);
    f1.instruction(&Instruction::LocalGet(0));
    f1.instruction(&Instruction::MemoryGrow(0));
    f1.instruction(&Instruction::LocalTee(1));
    f1.instruction(&Instruction::I32Const(-1));
    f1.instruction(&Instruction::I32Eq);
    f1.instruction(&Instruction::If(BlockType::Empty));
    f1.instruction(&Instruction::Unreachable);
    f1.instruction(&Instruction::End);
    f1.instruction(&Instruction::LocalGet(1));
    f1.instruction(&Instruction::End);
    codes.function(&f1);

    // Func 2: current_pages() -> i32
    let mut f2 = Function::new([]);
    f2.instruction(&Instruction::MemorySize(0));
    f2.instruction(&Instruction::End);
    codes.function(&f2);

    module.section(&codes);

    module.finish()
}

#[test]
fn test_initial_memory_exceeding_limit_fails_instantiation() {
    let engine = Engine::new().expect("engine");
    // Request initial 2 pages = 128 KiB
    let wasm = wasm_module_with_initial_memory(2, None);
    let component = engine.load_component(&wasm).expect("load");

    // Configure a limit of only 1 page = 64 KiB
    let tight_limits = ExecutionLimits::unmetered().with_max_memory_bytes(65_536);

    let err = component
        .invoke_with_limits("get_val", &[], &tight_limits)
        .expect_err("instantiation must fail when initial memory exceeds limit");

    match err {
        RuntimeError::MemoryLimitExceeded {
            requested_bytes,
            max_bytes,
        } => {
            assert_eq!(max_bytes, 65_536);
            assert_eq!(requested_bytes, 131_072);
        }
        other => panic!("expected MemoryLimitExceeded, got {other:?}"),
    }
}

#[test]
fn test_initial_memory_within_limit_succeeds() {
    let engine = Engine::new().expect("engine");
    // Initial 2 pages = 128 KiB
    let wasm = wasm_module_with_initial_memory(2, None);
    let component = engine.load_component(&wasm).expect("load");

    // Configure generous limit of 4 pages = 256 KiB
    let limits = ExecutionLimits::unmetered().with_max_memory_bytes(262_144);

    let report = component
        .invoke_with_limits("get_val", &[], &limits)
        .expect("invocation must succeed within limits");

    assert_eq!(report.values, vec![Val::S32(42)]);
}

#[test]
fn test_dynamic_memory_grow_returns_minus_one_on_host_denial() {
    let engine = Engine::new().expect("engine");
    let wasm = wasm_module_with_memory_grow();
    let component = engine.load_component(&wasm).expect("load");

    // Start with 1 page (64 KiB). Set limit to 2 pages (128 KiB = 131_072 bytes).
    let limits = ExecutionLimits::unmetered().with_max_memory_bytes(131_072);

    // Step 1: Grow by 1 page (total desired = 2 pages = 128 KiB <= 131_072). Allowed!
    let rep1 = component
        .invoke_with_limits("grow_memory", &[Val::S32(1)], &limits)
        .expect("first growth must succeed");
    // memory.grow returns previous page count: 1
    assert_eq!(rep1.values, vec![Val::S32(1)]);

    // Step 2: In a fresh invocation with the SAME limit (remember: fresh Store per invocation per ADR-005),
    // attempting to grow by 2 pages (1 + 2 = 3 pages = 192 KiB > 128 KiB).
    // Denied by ResourceLimiter -> memory.grow returns -1!
    let rep2 = component
        .invoke_with_limits("grow_memory", &[Val::S32(2)], &limits)
        .expect("invocation must complete; memory.grow returns -1 on denial per WebAssembly spec");
    assert_eq!(rep2.values, vec![Val::S32(-1)]);
}

#[test]
fn test_dynamic_memory_grow_classified_as_memory_limit_exceeded_on_guest_trap() {
    let engine = Engine::new().expect("engine");
    let wasm = wasm_module_with_memory_grow();
    let component = engine.load_component(&wasm).expect("load");

    // Limit to 128 KiB (2 pages)
    let limits = ExecutionLimits::unmetered().with_max_memory_bytes(131_072);

    // Call grow_or_trap with delta = 2 pages (1 + 2 = 3 pages = 192 KiB > 128 KiB).
    // memory.grow returns -1, the guest sees -1 and triggers `unreachable`.
    // The runtime correlates the trap with the ResourceLimiter denial and returns MemoryLimitExceeded.
    let err = component
        .invoke_with_limits("grow_or_trap", &[Val::S32(2)], &limits)
        .expect_err("guest must trap and be classified as MemoryLimitExceeded");

    match err {
        RuntimeError::MemoryLimitExceeded {
            requested_bytes,
            max_bytes,
        } => {
            assert_eq!(max_bytes, 131_072);
            assert_eq!(requested_bytes, 196_608);
        }
        other => panic!("expected MemoryLimitExceeded, got {other:?}"),
    }
}

#[test]
fn test_store_memory_isolation_adr005() {
    let engine = Engine::new().expect("engine");
    let wasm = wasm_module_with_initial_memory(2, None);
    let component = engine.load_component(&wasm).expect("load");

    // Run 1: Constrained store fails
    let tight_limits = ExecutionLimits::unmetered().with_max_memory_bytes(65_536);
    let err = component
        .invoke_with_limits("get_val", &[], &tight_limits)
        .expect_err("run 1 must fail");
    assert!(matches!(err, RuntimeError::MemoryLimitExceeded { .. }));

    // Run 2: Immediately on the SAME component, generous store succeeds
    let generous_limits = ExecutionLimits::unmetered().with_max_memory_bytes(262_144);
    let rep = component
        .invoke_with_limits("get_val", &[], &generous_limits)
        .expect("run 2 must succeed: fresh store, no memory limit leakage");
    assert_eq!(rep.values, vec![Val::S32(42)]);

    // Run 3: Constrained store fails again
    let err2 = component
        .invoke_with_limits("get_val", &[], &tight_limits)
        .expect_err("run 3 must fail again");
    assert!(matches!(err2, RuntimeError::MemoryLimitExceeded { .. }));
}

#[test]
fn test_instance_limits_builder_and_defaults() {
    let default_limits = InstanceLimits::default();
    assert_eq!(default_limits.max_memory_bytes, 16 * 1024 * 1024);
    assert_eq!(default_limits.max_table_elements, 10_000);

    let custom = InstanceLimits::new(32 * 1024 * 1024).with_max_table_elements(500);
    assert_eq!(custom.max_memory_bytes, 32 * 1024 * 1024);
    assert_eq!(custom.max_table_elements, 500);

    let exec_limits = ExecutionLimits::unmetered()
        .with_fuel(5_000)
        .with_instance_limits(custom);

    assert_eq!(exec_limits.fuel_budget, Some(5_000));
    assert_eq!(exec_limits.max_memory_bytes, Some(32 * 1024 * 1024));
    assert_eq!(exec_limits.instance_limits, Some(custom));
}
