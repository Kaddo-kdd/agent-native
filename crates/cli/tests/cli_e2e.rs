//! End-to-end process integration tests for `wasma` CLI.
//!
//! Validates:
//! - All 5 canonical subcommands: `build`, `verify`, `test`, `run`, `publish`.
//! - Output formats: `--format=human` and `--format=json`.
//! - Exit codes: 0 (`Success`), 1 (`DomainFailure`), 2 (`UsageOrIoError`).
//! - Subprocess execution via real independent OS processes (`std::process::Command`).
//! - Offline operation without external network or LLM dependencies.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Returns the path to the compiled `wasma` binary.
fn cli_binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_wasma"))
}

/// Helper to create an isolated temporary directory for test artifacts.
struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let temp = std::env::temp_dir().join(format!(
            "agent_native_cli_test_{name}_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("must create test temp dir");
        Self { path: temp }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

const VALID_IDENTITY_IR: &str = r#"{
    "ir_version": "0.1",
    "spec_version": 1,
    "functions": [
        {
            "name": "identity",
            "export": true,
            "params": [{ "name": "value", "type": "i32" }],
            "returns": "i32",
            "requirements": ["SMOKE-001"],
            "body": { "op": "param", "name": "value" }
        }
    ]
}"#;

const INVALID_IR_SYNTAX: &str = r#"{ "ir_version": "0.1", "functions": [ incomplete"#;

const INVALID_IR_SEMANTICS: &str = r#"{
    "ir_version": "0.5",
    "spec_version": 1,
    "functions": [
        {
            "name": "invalid_func",
            "export": true,
            "params": [],
            "returns": "i32",
            "body": { "op": "param", "name": "nonexistent" }
        }
    ]
}"#;

const VALID_NARRATIVE_SPEC: &str = r"# Sistema de Perfil de Usuario

## 1. Visión general del producto
Servicio para gestión de perfiles de usuario.
1. **Administrador de Perfil (`profile-manager`):** Maneja puntuaciones y perfiles.

## 2. Entidades de Dominio
### Puntuación de Usuario (`UserScore`)
- `score_id`: Identificador de la puntuación.
- `subtotal_cents`: Monto neto base.
- `total_cents`: Monto total acumulado.

## 3. Componente A: Perfil (`profile-manager`)
### Reglas
- **`PROF-001` (Cálculo de puntaje):** Asigna puntos base según actividad.

## 4. Contratos de Interfaz (WIT)
### Interfaz Exportada por `profile-manager`
```wit
interface ops {
    calculate-score: func(subtotal-cents: s32) -> s32;
}
```
";

#[test]
fn test_cli_help_flag() {
    let output = Command::new(cli_binary_path())
        .arg("--help")
        .output()
        .expect("must run CLI");

    assert!(output.status.success(), "help must exit with 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("build"), "must list build subcommand");
    assert!(stdout.contains("verify"), "must list verify subcommand");
    assert!(stdout.contains("test"), "must list test subcommand");
    assert!(stdout.contains("run"), "must list run subcommand");
    assert!(stdout.contains("publish"), "must list publish subcommand");
    assert!(stdout.contains("--format"), "must explain format option");
}

#[test]
fn test_cli_version_flag() {
    let output = Command::new(cli_binary_path())
        .arg("--version")
        .output()
        .expect("must run CLI");

    assert!(output.status.success(), "version must exit with 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wasma"), "must print binary name");
}

#[test]
fn test_cli_unknown_subcommand_exit_code_2() {
    let output = Command::new(cli_binary_path())
        .arg("unknown-command-xyz")
        .output()
        .expect("must run CLI");

    assert_eq!(
        output.status.code(),
        Some(2),
        "unknown subcommand must exit with code 2"
    );
}

#[test]
fn test_cli_verify_semantic_ir_valid() {
    let dir = TestDir::new("verify_valid_ir");
    let file = dir.path().join("valid_ir.json");
    fs::write(&file, VALID_IDENTITY_IR).unwrap();

    // Human output
    let output = Command::new(cli_binary_path())
        .arg("verify")
        .arg(&file)
        .output()
        .expect("must run CLI");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Verification SUCCESSFUL"));

    // JSON output
    let output_json = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("verify")
        .arg(&file)
        .output()
        .expect("must run CLI");

    assert_eq!(output_json.status.code(), Some(0));
    let json_val: serde_json::Value = serde_json::from_slice(&output_json.stdout).unwrap();
    assert_eq!(json_val["valid"], true);
    assert_eq!(json_val["error_count"], 0);
}

#[test]
fn test_cli_verify_semantic_ir_invalid() {
    let dir = TestDir::new("verify_invalid_ir");
    let file = dir.path().join("invalid_ir.json");
    fs::write(&file, INVALID_IR_SEMANTICS).unwrap();

    let output = Command::new(cli_binary_path())
        .arg("verify")
        .arg(&file)
        .output()
        .expect("must run CLI");

    assert_eq!(
        output.status.code(),
        Some(1),
        "semantic error must exit with code 1"
    );

    let output_json = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("verify")
        .arg(&file)
        .output()
        .expect("must run CLI");

    assert_eq!(output_json.status.code(), Some(1));
    let json_val: serde_json::Value = serde_json::from_slice(&output_json.stdout).unwrap();
    assert_eq!(json_val["valid"], false);
    assert!(json_val["error_count"].as_u64().unwrap() > 0);
}

#[test]
fn test_cli_verify_syntax_error_exit_code_1() {
    let dir = TestDir::new("verify_syntax_err");
    let file = dir.path().join("syntax_err.json");
    fs::write(&file, INVALID_IR_SYNTAX).unwrap();

    let output = Command::new(cli_binary_path())
        .arg("verify")
        .arg(&file)
        .output()
        .expect("must run CLI");

    assert_eq!(
        output.status.code(),
        Some(1),
        "bad JSON syntax must exit with code 1"
    );
}

#[test]
fn test_cli_verify_markdown_spec() {
    let dir = TestDir::new("verify_markdown");
    let file = dir.path().join("spec.md");
    fs::write(&file, VALID_NARRATIVE_SPEC).unwrap();

    let output = Command::new(cli_binary_path())
        .arg("verify")
        .arg(&file)
        .output()
        .expect("must run CLI");

    assert_eq!(output.status.code(), Some(0));

    let output_json = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("verify")
        .arg(&file)
        .output()
        .expect("must run CLI");

    assert_eq!(output_json.status.code(), Some(0));
    let json_val: serde_json::Value = serde_json::from_slice(&output_json.stdout).unwrap();
    assert_eq!(json_val["valid"], true);
}

#[test]
fn test_cli_missing_file_exit_code_2() {
    let output = Command::new(cli_binary_path())
        .arg("verify")
        .arg("non_existent_file_12345.json")
        .output()
        .expect("must run CLI");

    assert_eq!(
        output.status.code(),
        Some(2),
        "missing file must exit with code 2"
    );
}

#[test]
fn test_cli_build_from_ir_json() {
    let dir = TestDir::new("build_ir");
    let ir_file = dir.path().join("identity.json");
    let wasm_file = dir.path().join("identity.wasm");
    fs::write(&ir_file, VALID_IDENTITY_IR).unwrap();

    let output = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("build")
        .arg(&ir_file)
        .arg("-o")
        .arg(&wasm_file)
        .output()
        .expect("must run CLI");

    assert_eq!(output.status.code(), Some(0), "build must succeed");
    assert!(wasm_file.exists(), "wasm file must be created");
    assert!(fs::metadata(&wasm_file).unwrap().len() > 0);

    let json_val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json_val["success"], true);
    assert_eq!(json_val["sha256"].as_str().unwrap().len(), 64);
}

#[test]
fn test_cli_build_from_markdown_synthesizer() {
    let dir = TestDir::new("build_md");
    let spec_file = dir.path().join("spec.md");
    let wasm_file = dir.path().join("calc.wasm");
    fs::write(&spec_file, VALID_NARRATIVE_SPEC).unwrap();

    let output = Command::new(cli_binary_path())
        .arg("build")
        .arg(&spec_file)
        .arg("-o")
        .arg(&wasm_file)
        .arg("--mode")
        .arg("synthesizer")
        .output()
        .expect("must run CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "synthesizer build must succeed. stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(wasm_file.exists());
}

#[test]
fn test_cli_build_invalid_mode_exit_code_2() {
    let dir = TestDir::new("build_bad_mode");
    let spec_file = dir.path().join("spec.md");
    fs::write(&spec_file, VALID_NARRATIVE_SPEC).unwrap();

    let output = Command::new(cli_binary_path())
        .arg("build")
        .arg(&spec_file)
        .arg("--mode")
        .arg("unsupported-mode-xyz")
        .output()
        .expect("must run CLI");

    assert_eq!(
        output.status.code(),
        Some(2),
        "unsupported mode must exit with code 2"
    );
}

#[test]
fn test_cli_build_real_mode_fails_explicitly() {
    let dir = TestDir::new("build_real_fail");
    let spec_file = dir.path().join("spec.md");
    fs::write(&spec_file, VALID_NARRATIVE_SPEC).unwrap();

    let output = Command::new(cli_binary_path())
        .arg("build")
        .arg(&spec_file)
        .arg("--mode")
        .arg("real")
        .output()
        .expect("must run CLI");

    assert_eq!(
        output.status.code(),
        Some(1),
        "real mode without network must fail explicitly with code 1"
    );
}

#[test]
fn test_cli_run_exported_function() {
    let dir = TestDir::new("run_component");
    let ir_file = dir.path().join("identity.json");
    let wasm_file = dir.path().join("identity.wasm");
    fs::write(&ir_file, VALID_IDENTITY_IR).unwrap();

    // 1. Build
    let build_out = Command::new(cli_binary_path())
        .arg("build")
        .arg(&ir_file)
        .arg("-o")
        .arg(&wasm_file)
        .output()
        .unwrap();
    assert_eq!(build_out.status.code(), Some(0));

    // 2. Run human
    let run_human = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("42")
        .output()
        .unwrap();

    assert_eq!(run_human.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&run_human.stdout);
    assert!(stdout.contains("42"), "output must contain return value 42");

    // 3. Run JSON
    let run_json = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("100")
        .output()
        .unwrap();

    assert_eq!(run_json.status.code(), Some(0));
    let json_val: serde_json::Value = serde_json::from_slice(&run_json.stdout).unwrap();
    assert_eq!(json_val["success"], true);
    assert_eq!(json_val["results"][0], 100);
}

#[test]
fn test_cli_run_missing_function_exit_code_1() {
    let dir = TestDir::new("run_missing_fn");
    let ir_file = dir.path().join("identity.json");
    let wasm_file = dir.path().join("identity.wasm");
    fs::write(&ir_file, VALID_IDENTITY_IR).unwrap();

    Command::new(cli_binary_path())
        .arg("build")
        .arg(&ir_file)
        .arg("-o")
        .arg(&wasm_file)
        .output()
        .unwrap();

    let output = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("nonexistent_func")
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(1),
        "nonexistent function must exit with code 1"
    );
}

#[test]
fn test_cli_test_acceptance_suite() {
    let dir = TestDir::new("test_suite");
    let ir_file = dir.path().join("identity.json");
    let wasm_file = dir.path().join("identity.wasm");
    fs::write(&ir_file, VALID_IDENTITY_IR).unwrap();

    Command::new(cli_binary_path())
        .arg("build")
        .arg(&ir_file)
        .arg("-o")
        .arg(&wasm_file)
        .output()
        .unwrap();

    let suite_file =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../runtime/tests/suites/smoke.yaml");
    assert!(suite_file.exists(), "smoke.yaml fixture must exist");

    // Human test run
    let output_human = Command::new(cli_binary_path())
        .arg("test")
        .arg("-s")
        .arg(&suite_file)
        .arg("-c")
        .arg(&wasm_file)
        .output()
        .unwrap();

    assert_eq!(output_human.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output_human.stdout);
    assert!(stdout.contains("PASSED"));

    // JSON test run
    let output_json = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("test")
        .arg("-s")
        .arg(&suite_file)
        .arg("-c")
        .arg(&wasm_file)
        .output()
        .unwrap();

    assert_eq!(output_json.status.code(), Some(0));
    let json_val: serde_json::Value = serde_json::from_slice(&output_json.stdout).unwrap();
    assert_eq!(json_val["success"], true);
    assert_eq!(json_val["report"]["passed"], 3);
    assert_eq!(json_val["report"]["failed"], 0);
}

#[test]
fn test_cli_test_failing_suite_exit_code_1() {
    let dir = TestDir::new("test_failing_suite");
    let ir_file = dir.path().join("identity.json");
    let wasm_file = dir.path().join("identity.wasm");
    fs::write(&ir_file, VALID_IDENTITY_IR).unwrap();

    Command::new(cli_binary_path())
        .arg("build")
        .arg(&ir_file)
        .arg("-o")
        .arg(&wasm_file)
        .output()
        .unwrap();

    let failing_suite = r"suite: failing-smoke
spec_version: 1
cases:
  - id: smoke-wrong-expectation
    requirement: SMOKE-001
    function: identity
    args: [42]
    expected: 999
";
    let suite_path = dir.path().join("failing.yaml");
    fs::write(&suite_path, failing_suite).unwrap();

    let output = Command::new(cli_binary_path())
        .arg("test")
        .arg("-s")
        .arg(&suite_path)
        .arg("-c")
        .arg(&wasm_file)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(1),
        "failing test case must exit with code 1"
    );
}

#[test]
fn test_cli_publish_spec() {
    let dir = TestDir::new("publish_cmd");
    let spec_file = dir.path().join("spec.md");
    let dist_dir = dir.path().join("dist");
    fs::write(&spec_file, VALID_NARRATIVE_SPEC).unwrap();

    let output = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("publish")
        .arg("-s")
        .arg(&spec_file)
        .arg("-d")
        .arg(&dist_dir)
        .arg("--source-wi")
        .arg("AN-028")
        .arg("--git-commit")
        .arg("a1b2c3d4e5")
        .output()
        .expect("must run CLI");

    assert_eq!(output.status.code(), Some(0), "publish must exit with 0");
    assert!(
        dist_dir.join("active.json").exists(),
        "active.json must exist in dist store"
    );

    let json_val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json_val["success"], true);
    assert!(
        json_val["version_id"]
            .as_str()
            .unwrap()
            .starts_with("build-")
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_cli_unified_single_spec_pipeline() {
    let dir = TestDir::new("unified_pipeline");
    let spec_file = dir.path().join("pipeline_spec.md");
    let wasm_file = dir.path().join("profile-manager.wasm");
    let suite_file = dir.path().join("pipeline_suite.yaml");
    let dist_dir = dir.path().join("dist");

    // Exact same narrative specification across all stages
    fs::write(&spec_file, VALID_NARRATIVE_SPEC).unwrap();

    // Acceptance suite specifically tailored to the spec contract and synthesizer logic:
    // `calculate-score(subtotal-cents: 21)` -> returns `21 * 2 = 42` under synthesizer pass
    let suite_content = r"suite: profile-pipeline-suite
spec_version: 1
cases:
  - id: PROF-CASE-001
    requirement: PROF-001
    function: an:synthesized/ops@0.1.0#calculate-score
    args: [21]
    expected: 42
";
    fs::write(&suite_file, suite_content).unwrap();

    // 1. VERIFY (verify the Markdown specification directly)
    let verify_out = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("verify")
        .arg(&spec_file)
        .output()
        .expect("must execute verify");
    assert_eq!(verify_out.status.code(), Some(0), "verify must succeed");
    let verify_json: serde_json::Value =
        serde_json::from_slice(&verify_out.stdout).expect("verify output must be valid JSON");
    assert_eq!(verify_json["valid"], true);
    assert_eq!(verify_json["error_count"], 0);

    // 2. BUILD (build the WASM component from the Markdown specification using synthesizer mode)
    let build_out = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("build")
        .arg(&spec_file)
        .arg("-o")
        .arg(&wasm_file)
        .arg("--mode")
        .arg("synthesizer")
        .output()
        .expect("must execute build");
    assert_eq!(build_out.status.code(), Some(0), "build must succeed");
    assert!(wasm_file.exists(), "wasm file must be produced");
    let build_json: serde_json::Value =
        serde_json::from_slice(&build_out.stdout).expect("build output must be valid JSON");
    assert_eq!(build_json["success"], true);
    let build_sha = build_json["sha256"].as_str().unwrap().to_string();

    // 3. RUN (run the exported function calculate-score with arg 25 -> 25 * 2 = 50)
    let run_out = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("calculate-score")
        .arg("25")
        .output()
        .expect("must execute run");
    assert_eq!(run_out.status.code(), Some(0), "run must succeed");
    let run_json: serde_json::Value =
        serde_json::from_slice(&run_out.stdout).expect("run output must be valid JSON");
    assert_eq!(run_json["success"], true);
    assert_eq!(
        run_json["results"][0], 50,
        "synthesizer 25 * 2 must equal 50"
    );

    // 4. TEST (run test suite against the built WASM component)
    let test_out = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("test")
        .arg("-s")
        .arg(&suite_file)
        .arg("-c")
        .arg(&wasm_file)
        .output()
        .expect("must execute test");
    let test_stdout = String::from_utf8_lossy(&test_out.stdout);
    let test_stderr = String::from_utf8_lossy(&test_out.stderr);
    assert_eq!(
        test_out.status.code(),
        Some(0),
        "test suite must succeed. stdout:\n{test_stdout}\nstderr:\n{test_stderr}"
    );
    let test_json: serde_json::Value =
        serde_json::from_slice(&test_out.stdout).expect("test output must be valid JSON");
    assert_eq!(test_json["success"], true);
    assert_eq!(test_json["report"]["passed"], 1);
    assert_eq!(test_json["report"]["failed"], 0);

    // 5. PUBLISH (publish the exact same specification to the local distribution store)
    let pub_out = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("publish")
        .arg("-s")
        .arg(&spec_file)
        .arg("-d")
        .arg(&dist_dir)
        .arg("--source-wi")
        .arg("AN-028")
        .arg("--git-commit")
        .arg("deadbeef01")
        .arg("--mode")
        .arg("synthesizer")
        .output()
        .expect("must execute publish");
    assert_eq!(pub_out.status.code(), Some(0), "publish must succeed");
    let pub_json: serde_json::Value =
        serde_json::from_slice(&pub_out.stdout).expect("publish output must be valid JSON");
    assert_eq!(pub_json["success"], true);
    assert!(
        pub_json["published_components"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("profile-manager"))
    );

    // Verify published artifact matches the component
    let version_id = pub_json["version_id"].as_str().unwrap();
    let published_wasm = dist_dir
        .join("versions")
        .join(version_id)
        .join("profile-manager.wasm");
    assert!(
        published_wasm.exists(),
        "published wasm artifact must exist in dist/versions/<version_id>"
    );
    let pub_bytes = fs::read(&published_wasm).unwrap();
    let local_bytes = fs::read(&wasm_file).unwrap();
    assert_eq!(
        pub_bytes, local_bytes,
        "published artifact bytes must be identical to built WASM bytes"
    );
    assert_eq!(wasma_compiler::sha256_hex(&local_bytes), build_sha);
}

#[test]
fn test_cli_json_purity_and_contract_invariants() {
    let dir = TestDir::new("json_purity");
    let spec_file = dir.path().join("spec.md");
    fs::write(&spec_file, VALID_NARRATIVE_SPEC).unwrap();

    // Verify stdout contains ONLY valid JSON (no human text leaked before or after)
    let output = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("verify")
        .arg(&spec_file)
        .output()
        .unwrap();

    let stdout_str = std::str::from_utf8(&output.stdout).expect("stdout must be utf-8");
    let trimmed = stdout_str.trim();
    assert!(
        trimmed.starts_with('{') && trimmed.ends_with('}'),
        "pure JSON required"
    );
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(trimmed);
    assert!(parsed.is_ok(), "stdout must parse cleanly as JSON");

    // Test structured error format on exit code 1
    let bad_ir_file = dir.path().join("bad.json");
    fs::write(&bad_ir_file, INVALID_IR_SEMANTICS).unwrap();
    let err_out = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("verify")
        .arg(&bad_ir_file)
        .output()
        .unwrap();

    assert_eq!(err_out.status.code(), Some(1));
    let err_stdout = std::str::from_utf8(&err_out.stdout).expect("stdout must be utf-8");
    let err_json: serde_json::Value =
        serde_json::from_str(err_stdout.trim()).expect("error must be valid JSON");
    assert_eq!(err_json["valid"], false);
    assert!(err_json["diagnostics"].is_array());
    assert!(!err_json["diagnostics"].as_array().unwrap().is_empty());
}

#[test]
fn test_cli_publish_incorporates_explicit_source_wi() {
    let dir = TestDir::new("publish_explicit_wi");
    let spec_file = dir.path().join("spec.md");
    let dist_dir = dir.path().join("dist");
    fs::write(&spec_file, VALID_NARRATIVE_SPEC).unwrap();

    let output = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("publish")
        .arg("-s")
        .arg(&spec_file)
        .arg("-d")
        .arg(&dist_dir)
        .arg("--source-wi")
        .arg("AN-028")
        .arg("--source-wi")
        .arg("AN-CUSTOM-999")
        .output()
        .expect("must run CLI");

    assert_eq!(output.status.code(), Some(0));

    let active_path = dist_dir.join("active.json");
    assert!(active_path.exists());
    let active_content = fs::read_to_string(&active_path).unwrap();
    let active_json: serde_json::Value = serde_json::from_str(&active_content).unwrap();

    let source_wis = active_json["traceability"]["source_wi"]
        .as_array()
        .expect("source_wi must be an array");

    let source_wi_strings: Vec<&str> = source_wis.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        source_wi_strings.contains(&"AN-028"),
        "active.json must contain explicit --source-wi AN-028"
    );
    assert!(
        source_wi_strings.contains(&"AN-CUSTOM-999"),
        "active.json must contain explicit --source-wi AN-CUSTOM-999"
    );
}

#[test]
fn test_cli_clap_parse_error_emits_json_when_format_json_requested() {
    let output = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("unknown-subcommand-xyz")
        .output()
        .expect("must run CLI");

    assert_eq!(output.status.code(), Some(2));
    let stdout_str = std::str::from_utf8(&output.stdout).expect("stdout must be utf-8");
    let trimmed = stdout_str.trim();
    assert!(
        trimmed.starts_with('{') && trimmed.ends_with('}'),
        "clap parse error must be pure JSON when --format json is passed. Got:\n{trimmed}"
    );

    let parsed: serde_json::Value =
        serde_json::from_str(trimmed).expect("clap parse error must parse as JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["exit_code"], 2);
    assert!(
        parsed["error"]
            .as_str()
            .unwrap()
            .contains("unknown-subcommand-xyz")
    );
}
