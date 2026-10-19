//! Canonical example of embedding `agent-native-runtime` in an external Rust application.
//!
//! # Architecture & Guarantees
//!
//! 1. **Standalone Execution (ADR-001 & ADR-002)**:
//!    The runtime executes WebAssembly components (Core WASM or Component Model)
//!    independently. In production, consumers only depend on `agent-native-runtime`.
//!    No Rust compiler or LLM connectivity is required at runtime.
//!
//! 2. **Deny-by-Default Security (ADR-002)**:
//!    Every component executes under a strict `HostPolicy`. Capabilities (host calls,
//!    WIT interfaces, external modules) are denied by default unless explicitly allowed.
//!
//! 3. **Session & Tenant Isolation (ADR-005)**:
//!    Invocations run in freshly scoped stores. When using `Resolver` for content-addressed
//!    caching, `Resolver::reset()` safely clears cached artifacts between sessions/tenants.
//!
//! # Usage
//!
//! Run with default embedded Component Model fixture:
//! ```bash
//! cargo run --example embed_runtime
//! ```
//!
//! Run with an external compiled `.wasm` file:
//! ```bash
//! cargo run --example embed_runtime -- path/to/component.wasm
//! ```

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use agent_native_runtime::{Engine, HostPolicy, Resolver, RuntimeError, Val};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Agent-Native Runtime Embedding Example ===\n");

    // 1. Resolve WASM bytes (either from CLI argument or default Component Model fixture)
    let (wasm_bytes, is_default_fixture) = get_or_compile_wasm()?;
    println!("✓ Loaded WASM artifact: {} bytes", wasm_bytes.len());

    // 2. Initialize the Runtime Engine
    // The Engine encapsulates Wasmtime's engine and configuration.
    let engine = Engine::new()?;
    println!("✓ Runtime engine initialized successfully");

    // 3. Configure Security Policy (ADR-002: Deny-by-default)
    // By default, HostPolicy::deny_all() blocks all undeclared host calls.
    let mut policy = HostPolicy::deny_all();
    policy.allow_wit_interface("an:test/ops@0.1.0");
    println!("✓ Security policy configured: deny-all with granular WIT authorization");

    // 4. Load and Validate Component
    // The runtime validates the binary (either Core WASM or WebAssembly Component Model)
    // and binds it to the specified security policy.
    let component = engine.load_component_with_policy(&wasm_bytes, &policy)?;
    println!("✓ Component loaded and bound to security policy");
    println!("  - Component Model: {}", component.is_component_model());
    println!("  - Core WASM Module: {}", component.is_core_module());

    // 5. Execute Exported Operations
    println!("\n--- Invoking Component Operations ---");

    if is_default_fixture && component.is_component_model() {
        // Component Model: Invoking 'double-value(21)' via WIT interface
        let op_primitive = "an:test/ops@0.1.0#double-value";
        println!("Invoking '{op_primitive}' with argument 21...");
        let results = component.invoke(op_primitive, &[Val::S32(21)])?;
        println!("Result: {results:?}");
        assert_eq!(
            results,
            vec![Val::S32(42)],
            "double_value(21) must equal 42"
        );

        // Component Model: Invoking 'make-point-sum(10, 25)' returning a record
        let op_record = "an:test/ops@0.1.0#make-point-sum";
        println!("Invoking '{op_record}' with arguments (10, 25)...");
        let record_results = component.invoke(op_record, &[Val::S32(10), Val::S32(25)])?;
        println!("Result: {record_results:?}");

        if let Some(Val::Record(fields)) = record_results.first() {
            println!("  Extracted Record fields:");
            for (name, val) in fields {
                println!("    • {name}: {val:?}");
            }
        }
    } else if is_default_fixture {
        // Core WASM fallback
        println!("Invoking default export on Core WASM component...");
        let results = component.invoke("identity", &[Val::S32(42)])?;
        println!("Result: {results:?}");
    } else {
        println!("Custom WASM component successfully loaded under host security policy.");
        println!("Pass custom operation calls or inspect exports through Component API.");
    }

    // 6. Demonstrate Explicit Error Handling (ADR-004: Fail-explicit taxonomy)
    println!("\n--- Demonstrating Explicit Error Handling ---");
    let missing_op = "an:test/ops@0.1.0#nonexistent-operation";
    println!("Attempting to invoke nonexistent operation '{missing_op}'...");
    match component.invoke(missing_op, &[]) {
        Ok(_) => panic!("Expected invocation to fail for missing function"),
        Err(RuntimeError::FunctionNotFound(name)) => {
            println!("✓ Caught expected typed error: FunctionNotFound('{name}')");
        }
        Err(other) => {
            println!("✓ Caught typed runtime error: {other:?}");
        }
    }

    // 7. Session Caching & Tenant Isolation with Resolver (ADR-005)
    println!("\n--- Demonstrating Content-Addressed Caching and Session Reset ---");
    let mut resolver = Resolver::new()?;

    // First load computes SHA-256 and compiles/caches the artifact
    let _cached_comp1 = resolver.load(&wasm_bytes)?;
    println!("Cached artifacts in resolver: {}", resolver.cached_count());
    assert_eq!(resolver.cached_count(), 1);

    // Second load hits content-addressed cache
    let _cached_comp2 = resolver.load(&wasm_bytes)?;
    println!(
        "Cached artifacts after repeated load: {}",
        resolver.cached_count()
    );
    assert_eq!(
        resolver.cached_count(),
        1,
        "Cache hit must not duplicate artifact"
    );

    // Session reset clears cached instances/components for tenant isolation
    resolver.reset();
    println!(
        "Cached artifacts after resolver.reset(): {}",
        resolver.cached_count()
    );
    assert_eq!(
        resolver.cached_count(),
        0,
        "Resolver reset must clear cache"
    );

    println!("\n=== Embedding Example Completed Successfully ===");
    Ok(())
}

/// Helper function to obtain WASM bytes:
/// - If a file path is provided via CLI argument, reads bytes from disk.
/// - If no argument is provided, compiles the standard Component Model fixture.
fn get_or_compile_wasm() -> Result<(Vec<u8>, bool), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && !args[1].starts_with('-') {
        let path = Path::new(&args[1]);
        println!(
            "Loading component from user-specified path: {}",
            path.display()
        );
        let bytes = fs::read(path)?;
        return Ok((bytes, false));
    }

    println!("No path specified; synthesizing canonical Component Model fixture (program-005)...");
    let bytes = compile_fixture_component_model()?;
    Ok((bytes, true))
}

/// Compiles the canonical `program-005` Component Model fixture for testing and examples.
///
/// In an external consumer project, pre-compiled `.wasm` binaries are used directly,
/// eliminating the need for `agent-native-compiler`.
fn compile_fixture_component_model() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use agent_native_compiler::{CompileContext, compile_ir_to_wasm_with_context};

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut compiler_dir = manifest_dir;
    compiler_dir.pop(); // crates/
    compiler_dir.push("compiler");

    let ir_path = compiler_dir
        .join("tests")
        .join("fixtures")
        .join("ir")
        .join("program-005.ir.json");

    let ir_str = fs::read_to_string(&ir_path).map_err(|e| {
        format!(
            "Failed to read fixture IR at {}: {}. Run from repo root.",
            ir_path.display(),
            e
        )
    })?;

    let ir_val: serde_json::Value = serde_json::from_str(&ir_str)?;
    let ctx = CompileContext::new(compiler_dir);

    let wasm = compile_ir_to_wasm_with_context(&ir_val, &ctx)
        .map_err(|diags| format!("Compilation of fixture failed: {diags:?}"))?;

    Ok(wasm)
}
