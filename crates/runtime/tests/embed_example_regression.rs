//! Regression test to verify that `embed_runtime` example compiles and executes
//! successfully in automated CI.

use std::process::Command;

#[test]
fn embed_runtime_example_runs_successfully() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo)
        .args(["run", "--example", "embed_runtime", "-p", "wasma-runtime"])
        .output()
        .expect("cargo run --example embed_runtime must execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "embed_runtime example failed with status {:?}.\nSTDOUT:\n{}\nSTDERR:\n{}",
        output.status.code(),
        stdout,
        stderr
    );

    assert!(
        stdout.contains("=== Embedding Example Completed Successfully ==="),
        "Expected success sentinel in stdout, got:\n{stdout}"
    );
    assert!(
        stdout.contains("double-value"),
        "Expected double-value invocation in stdout"
    );
    assert!(
        stdout.contains("make-point-sum"),
        "Expected make-point-sum invocation in stdout"
    );
    assert!(
        stdout.contains("FunctionNotFound"),
        "Expected error handling demonstration in stdout"
    );
    assert!(
        stdout.contains("Cached artifacts after resolver.reset(): 0"),
        "Expected session reset demonstration in stdout"
    );
}
