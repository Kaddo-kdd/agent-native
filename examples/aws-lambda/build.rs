use std::env;
use std::fs;
use std::path::PathBuf;

use wasma_compiler::{CompileContext, compile_ir_to_wasm_with_context};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut repo_root = manifest_dir.clone();
    repo_root.pop(); // examples
    repo_root.pop(); // root

    let compiler_dir = repo_root.join("crates").join("compiler");
    let ir_path = compiler_dir
        .join("tests")
        .join("fixtures")
        .join("ir")
        .join("program-005.ir.json");
    let wit_path = compiler_dir
        .join("tests")
        .join("fixtures")
        .join("wit")
        .join("program-005.wit");

    println!("cargo:rerun-if-changed={}", ir_path.display());
    println!("cargo:rerun-if-changed={}", wit_path.display());
    println!("cargo:rerun-if-changed=build.rs");

    let ir_str = fs::read_to_string(&ir_path)
        .unwrap_or_else(|e| panic!("failed to read fixture IR at {}: {e}", ir_path.display()));
    let ir_value: serde_json::Value = serde_json::from_str(&ir_str)
        .unwrap_or_else(|e| panic!("invalid JSON in fixture IR at {}: {e}", ir_path.display()));

    let ctx = CompileContext::new(&compiler_dir);
    let wasm_bytes = compile_ir_to_wasm_with_context(&ir_value, &ctx)
        .unwrap_or_else(|diags| panic!("compilation failed for program-005: {diags:?}"));

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("program-005.wasm");
    fs::write(&dest_path, &wasm_bytes).unwrap_or_else(|e| {
        panic!(
            "failed to write compiled wasm to {}: {e}",
            dest_path.display()
        )
    });
}
