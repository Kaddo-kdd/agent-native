//! AN-037: Comprehensive stress, limits, and regression test suite.

use agent_native_runtime::{Engine, ExecutionLimits, InstanceLimits, RuntimeError, Val};
use wasm_encoder::{
    BlockType, CodeSection, ElementSection, Elements, ExportKind, ExportSection, Function,
    FunctionSection, Instruction, MemorySection, MemoryType, Module, RefType, TableSection,
    TableType, TypeSection, ValType,
};

/// Emits a WASM module with an infinite loop: `(loop (br 0))`.
fn wasm_module_with_infinite_loop() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I32]);
    module.section(&types);

    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);

    let mut exports = ExportSection::new();
    exports.export("spin", ExportKind::Func, 0);
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

/// Emits a WASM module that declares a minimum initial linear memory.
fn wasm_module_with_min_memory(min_pages: u64) -> Vec<u8> {
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
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memories);

    let mut exports = ExportSection::new();
    exports.export("query", ExportKind::Func, 0);
    exports.export("memory", ExportKind::Memory, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut func = Function::new([]);
    func.instruction(&Instruction::I32Const(777));
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    module.finish()
}

/// Emits a WASM module that dynamically attempts memory growth until a limit is reached.
fn wasm_module_with_dynamic_allocation_loop() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    // allocate_until_fail(max_pages_to_request: i32) -> i32
    types.ty().function([ValType::I32], [ValType::I32]);
    module.section(&types);

    let mut funcs = FunctionSection::new();
    funcs.function(0);
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
    exports.export("allocate_until_fail", ExportKind::Func, 0);
    exports.export("memory", ExportKind::Memory, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();
    // locals: [pages_grown: i32, grow_res: i32]
    let mut func = Function::new([(2, ValType::I32)]);
    // loop { if (memory.grow(1) == -1) { unreachable } else { pages_grown += 1 } }
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::I32Const(1));
    func.instruction(&Instruction::MemoryGrow(0));
    func.instruction(&Instruction::LocalTee(2));
    func.instruction(&Instruction::I32Const(-1));
    func.instruction(&Instruction::I32Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::Unreachable);
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I32Const(1));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::LocalTee(1));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32LtS);
    func.instruction(&Instruction::BrIf(0));
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    module.finish()
}

/// Emits a WASM module that attempts to grow a table.
fn wasm_module_with_table_grow() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    // grow_table(delta: i32) -> i32
    types.ty().function([ValType::I32], [ValType::I32]);
    module.section(&types);

    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);

    let mut tables = TableSection::new();
    tables.table(TableType {
        element_type: RefType::FUNCREF,
        minimum: 5,
        maximum: None,
        table64: false,
        shared: false,
    });
    module.section(&tables);

    let mut exports = ExportSection::new();
    exports.export("grow_table", ExportKind::Func, 0);
    exports.export("table", ExportKind::Table, 0);
    module.section(&exports);

    let mut elements = ElementSection::new();
    elements.declared(Elements::Functions(std::borrow::Cow::Borrowed(&[0])));
    module.section(&elements);

    let mut codes = CodeSection::new();
    let mut func = Function::new([]);
    func.instruction(&Instruction::RefNull(wasm_encoder::HeapType::Abstract {
        shared: false,
        ty: wasm_encoder::AbstractHeapType::Func,
    }));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::TableGrow(0));
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    module.finish()
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "Wasmtime 27 upstream issue #10923: libcall out_of_gas is declared as extern \"C\" instead of extern \"C-unwind\", causing MSVC RtlUnwindEx to abort on Windows"
)]
fn test_infinite_loop_fuel_exhaustion() {
    let engine = Engine::new().expect("engine");
    let wasm = wasm_module_with_infinite_loop();
    let component = engine.load_component(&wasm).expect("load");

    let limits = ExecutionLimits::unmetered().with_fuel(5_000);

    let err = component
        .invoke_with_limits("spin", &[], &limits)
        .expect_err("infinite loop must exhaust fuel");

    match err {
        RuntimeError::FuelExhausted { limit } => {
            assert_eq!(limit, 5_000);
        }
        other => panic!("expected FuelExhausted, got {other:?}"),
    }
}

#[test]
fn test_initial_memory_stress_rejection() {
    let engine = Engine::new().expect("engine");
    // Request 512 pages = 32 MB initial memory
    let wasm = wasm_module_with_min_memory(512);
    let component = engine.load_component(&wasm).expect("load");

    // Ceiling set to 16 MB (256 pages = 16,777,216 bytes)
    let limits = ExecutionLimits::unmetered().with_max_memory_bytes(16 * 1024 * 1024);

    let err = component
        .invoke_with_limits("query", &[], &limits)
        .expect_err("32 MB initial memory must be rejected under 16 MB ceiling");

    match err {
        RuntimeError::MemoryLimitExceeded {
            requested_bytes,
            max_bytes,
        } => {
            assert_eq!(max_bytes, 16 * 1024 * 1024);
            assert_eq!(requested_bytes, 512 * 65_536);
        }
        other => panic!("expected MemoryLimitExceeded, got {other:?}"),
    }
}

#[test]
fn test_dynamic_memory_leak_growth_denial_and_trap() {
    let engine = Engine::new().expect("engine");
    let wasm = wasm_module_with_dynamic_allocation_loop();
    let component = engine.load_component(&wasm).expect("load");

    // Ceiling of 256 KiB (4 pages)
    let limits = ExecutionLimits::unmetered().with_max_memory_bytes(256 * 1024);

    // Attempt to allocate up to 50 pages (3.2 MB >> 256 KiB)
    let err = component
        .invoke_with_limits("allocate_until_fail", &[Val::S32(50)], &limits)
        .expect_err("dynamic allocation must hit ceiling and trap");

    match err {
        RuntimeError::MemoryLimitExceeded {
            requested_bytes,
            max_bytes,
        } => {
            assert_eq!(max_bytes, 256 * 1024);
            // 5th page request = 5 * 64 KiB = 327,680 bytes > 262,144 bytes
            assert_eq!(requested_bytes, 5 * 65_536);
        }
        other => panic!("expected MemoryLimitExceeded, got {other:?}"),
    }
}

#[test]
fn test_table_growth_limit_enforcement() {
    let engine = Engine::new().expect("engine");
    let wasm = wasm_module_with_table_grow();
    let component = engine.load_component(&wasm).expect("load");

    // Initial table is 5 elements. Ceiling set to 10 elements.
    let inst_limits = InstanceLimits::default().with_max_table_elements(10);
    let limits = ExecutionLimits::unmetered().with_instance_limits(inst_limits);

    // Attempt to grow table by 20 elements (5 + 20 = 25 > 10).
    // WebAssembly table.grow returns -1 on denial!
    let rep = component
        .invoke_with_limits("grow_table", &[Val::S32(20)], &limits)
        .expect("table.grow returns -1 on host denial per WebAssembly specification");

    assert_eq!(rep.values, vec![Val::S32(-1)]);
}

#[test]
fn test_consecutive_multi_store_lifecycle_isolation() {
    let engine = Engine::new().expect("engine");
    let wasm = wasm_module_with_min_memory(2); // 2 pages = 128 KiB
    let component = engine.load_component(&wasm).expect("load");

    // Step 1: Constrained invocation fails
    let tight = ExecutionLimits::unmetered().with_max_memory_bytes(65_536);
    let err1 = component
        .invoke_with_limits("query", &[], &tight)
        .expect_err("step 1 must fail due to memory limits");
    assert!(matches!(err1, RuntimeError::MemoryLimitExceeded { .. }));

    // Step 2: Next invocation on the SAME component with generous limits succeeds
    let generous = ExecutionLimits::unmetered().with_max_memory_bytes(256 * 1024);
    let rep2 = component
        .invoke_with_limits("query", &[], &generous)
        .expect("step 2 must succeed: completely fresh Store");
    assert_eq!(rep2.values, vec![Val::S32(777)]);

    // Step 3: Invocation with tight limits fails again immediately
    let err3 = component
        .invoke_with_limits("query", &[], &tight)
        .expect_err("step 3 must fail again");
    assert!(matches!(err3, RuntimeError::MemoryLimitExceeded { .. }));

    // Step 4: Unmetered invocation succeeds again
    let rep4 = component
        .invoke("query", &[])
        .expect("step 4 unmetered invoke must succeed");
    assert_eq!(rep4, vec![Val::S32(777)]);
}

#[test]
fn test_combined_fuel_and_memory_contention() {
    let engine = Engine::new().expect("engine");
    let wasm = wasm_module_with_min_memory(2); // 128 KiB
    let component = engine.load_component(&wasm).expect("load");

    // Both limits configured and sufficient
    let combined_limits = ExecutionLimits::unmetered()
        .with_fuel(50_000)
        .with_max_memory_bytes(256 * 1024);

    let report = component
        .invoke_with_limits("query", &[], &combined_limits)
        .expect("combined limits within bounds must succeed");

    assert_eq!(report.values, vec![Val::S32(777)]);
    let consumed = report.fuel_consumed.expect("fuel consumed reported");
    assert!(consumed > 0);
    assert!(consumed <= 50_000);
}
