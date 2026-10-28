//! Integration test suite for load-time verification in `agent-native run`
//! (INIT-06-04, AN-041, AN-042).
//!
//! Covers:
//! - SHA-256 integrity check via `--expected-sha256` and `--manifest`.
//! - Fail-explicit rejection when a component is tampered or corrupt.
//! - Minisign Ed25519 signature verification via `--signature` and `--public-key`.
//! - Convention-based signature resolution (`<component>.minisig`).
//! - Tampered component rejection post-signing.
//! - Invalid or mismatched public keys.
//! - JSON machine-readable output mode.

use minisign::{KeyPair, SecretKeyBox};
use minisign_verify::{PublicKey, Signature};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

fn cli_binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_wasma"))
}

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let temp = std::env::temp_dir().join(format!(
            "agent_native_verify_test_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
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

fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn build_test_component(dir: &TestDir, name: &str) -> (PathBuf, Vec<u8>, String) {
    let ir_file = dir.path().join(format!("{name}.json"));
    let wasm_file = dir.path().join(format!("{name}.wasm"));
    fs::write(&ir_file, VALID_IDENTITY_IR).expect("must write IR file");

    let build_out = Command::new(cli_binary_path())
        .arg("build")
        .arg(&ir_file)
        .arg("-o")
        .arg(&wasm_file)
        .output()
        .expect("must execute build");
    assert_eq!(build_out.status.code(), Some(0), "build must succeed");

    let bytes = fs::read(&wasm_file).expect("must read wasm");
    let sha = compute_sha256(&bytes);
    (wasm_file, bytes, sha)
}

struct GeneratedMinisignKey {
    pub_file: PathBuf,
    pub_base64: String,
    decrypted_sk: minisign::SecretKey,
}

fn generate_test_minisign_key(dir: &TestDir, key_name: &str) -> GeneratedMinisignKey {
    let password = Some("test_password".to_string());
    let KeyPair { pk, sk } = KeyPair::generate_encrypted_keypair(password.clone()).unwrap();

    let pk_box_str = pk.to_box().unwrap().to_string();
    let sk_box_str = sk.to_box(Some("test key")).unwrap().to_string();

    let pub_file = dir.path().join(format!("{key_name}.pub"));
    fs::write(&pub_file, &pk_box_str).expect("must write pub file");

    let loaded_sk_box = SecretKeyBox::from_string(&sk_box_str).unwrap();
    let decrypted_sk = loaded_sk_box.into_secret_key(password).unwrap();

    let pk_lines: Vec<&str> = pk_box_str.lines().collect();
    let pub_base64 = pk_lines.last().unwrap().trim().to_string();

    GeneratedMinisignKey {
        pub_file,
        pub_base64,
        decrypted_sk,
    }
}

fn sign_file(key: &GeneratedMinisignKey, data: &[u8], sig_file: &Path) {
    let data_reader = Cursor::new(data);
    let signature_box = minisign::sign(
        None,
        &key.decrypted_sk,
        data_reader,
        Some("trusted test comment"),
        Some("untrusted test comment"),
    )
    .unwrap();
    fs::write(sig_file, signature_box.to_string()).expect("must write sig file");
}

#[test]
fn test_minisign_sign_and_verify_roundtrip() {
    let dir = TestDir::new("roundtrip");
    let key = generate_test_minisign_key(&dir, "roundtrip");
    let data = b"\x00asm\x01\x00\x00\x00dummy component bytes";
    let sig_file = dir.path().join("dummy.minisig");
    sign_file(&key, data, &sig_file);

    let sig = Signature::from_file(&sig_file).unwrap();
    let pk = PublicKey::from_base64(&key.pub_base64).unwrap();
    assert!(pk.verify(&data[..], &sig, false).is_ok());

    let tampered = b"\x00asm\x01\x00\x00\x00tampered bytes";
    assert!(pk.verify(&tampered[..], &sig, false).is_err());
}

#[test]
fn test_run_verify_sha256_success() {
    let dir = TestDir::new("sha256_success");
    let (wasm_file, _bytes, sha) = build_test_component(&dir, "identity");

    // Human output with --verify and --expected-sha256
    let output = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("42")
        .arg("--verify")
        .arg("--expected-sha256")
        .arg(&sha)
        .output()
        .expect("must run CLI");

    assert_eq!(output.status.code(), Some(0));
    let out_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out_text.contains("[VERIFIED] SHA-256 integrity check passed"));
    assert!(out_text.contains("42"));
}

#[test]
fn test_run_verify_sha256_mismatch_fails_explicit() {
    let dir = TestDir::new("sha256_mismatch");
    let (wasm_file, _bytes, _sha) = build_test_component(&dir, "identity");

    let wrong_sha = "0000000000000000000000000000000000000000000000000000000000000000";
    let output = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("42")
        .arg("--verify")
        .arg("--expected-sha256")
        .arg(wrong_sha)
        .output()
        .expect("must run CLI");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Integrity failure must exit with code 1"
    );
    let out_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        out_text.contains("Integrity verification FAILED"),
        "stdout must mention integrity failure: {out_text}"
    );
}

#[test]
fn test_run_verify_tampered_component_fails() {
    let dir = TestDir::new("tampered_wasm");
    let (wasm_file, mut bytes, original_sha) = build_test_component(&dir, "identity");

    // Tamper 1 byte in the WASM file
    let last_idx = bytes.len() - 1;
    bytes[last_idx] ^= 0xFF;
    fs::write(&wasm_file, &bytes).expect("must write tampered wasm");

    let output = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("42")
        .arg("--verify")
        .arg("--expected-sha256")
        .arg(&original_sha)
        .output()
        .expect("must run CLI");

    assert_eq!(output.status.code(), Some(1));
    let out_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out_text.contains("Integrity verification FAILED"));
}

#[test]
fn test_run_verify_via_manifest() {
    let dir = TestDir::new("manifest_verify");
    let (wasm_file, _bytes, sha) = build_test_component(&dir, "identity");

    let manifest_path = dir.path().join("manifest.json");
    let manifest_content = serde_json::json!({
        "version_id": "v1.0.0",
        "artifacts": [
            {
                "component_id": "identity",
                "wasm_file": "identity.wasm",
                "sha256": sha
            }
        ]
    });
    fs::write(&manifest_path, manifest_content.to_string()).expect("must write manifest");

    let output = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("100")
        .arg("--verify")
        .arg("--manifest")
        .arg(&manifest_path)
        .output()
        .expect("must run CLI");

    assert_eq!(output.status.code(), Some(0));
    let out_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out_text.contains("[VERIFIED] SHA-256 integrity check passed"));
}

#[test]
fn test_run_verify_minisign_signature_success() {
    let dir = TestDir::new("minisig_success");
    let (wasm_file, bytes, _sha) = build_test_component(&dir, "identity");
    let key = generate_test_minisign_key(&dir, "signing_key");

    // Convention-based: <wasm_file>.minisig
    let sig_file = dir.path().join("identity.wasm.minisig");
    sign_file(&key, &bytes, &sig_file);

    // 1. Verify using public key file path (auto-discovering .minisig)
    let output_file_pk = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("77")
        .arg("--verify")
        .arg("--public-key")
        .arg(&key.pub_file)
        .output()
        .expect("must run CLI");

    assert_eq!(output_file_pk.status.code(), Some(0));
    let stdout1 = String::from_utf8_lossy(&output_file_pk.stdout);
    assert!(stdout1.contains("[VERIFIED] Cryptographic signature check passed"));
    assert!(stdout1.contains("77"));

    // 2. Verify using inline base64 public key and explicit --signature
    let output_inline_pk = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("88")
        .arg("--verify")
        .arg("--signature")
        .arg(&sig_file)
        .arg("--public-key")
        .arg(&key.pub_base64)
        .output()
        .expect("must run CLI");

    assert_eq!(output_inline_pk.status.code(), Some(0));
    let stdout2 = String::from_utf8_lossy(&output_inline_pk.stdout);
    assert!(stdout2.contains("[VERIFIED] Cryptographic signature check passed"));
    assert!(stdout2.contains("88"));
}

#[test]
fn test_run_verify_minisign_tampered_component_fails() {
    let dir = TestDir::new("minisig_tampered");
    let (wasm_file, mut bytes, _sha) = build_test_component(&dir, "identity");
    let key = generate_test_minisign_key(&dir, "signing_key");

    let sig_file = dir.path().join("identity.wasm.minisig");
    sign_file(&key, &bytes, &sig_file);

    // Tamper component bytes post-signature
    let last = bytes.len() - 1;
    bytes[last] ^= 0x42;
    fs::write(&wasm_file, &bytes).expect("must write tampered wasm");

    let output = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("77")
        .arg("--verify")
        .arg("--public-key")
        .arg(&key.pub_file)
        .output()
        .expect("must run CLI");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Tampered component must fail signature verification with exit 1"
    );
    let out_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out_text.contains("signature verification FAILED"));
}

#[test]
fn test_run_verify_minisign_wrong_key_fails() {
    let dir = TestDir::new("wrong_key");
    let (wasm_file, bytes, _sha) = build_test_component(&dir, "identity");
    let key_a = generate_test_minisign_key(&dir, "key_a");
    let key_b = generate_test_minisign_key(&dir, "key_b");

    let sig_file = dir.path().join("identity.wasm.minisig");
    sign_file(&key_a, &bytes, &sig_file);

    // Attempt verification with Key B
    let output = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("77")
        .arg("--verify")
        .arg("--public-key")
        .arg(&key_b.pub_file)
        .output()
        .expect("must run CLI");

    assert_eq!(output.status.code(), Some(1));
    let out_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out_text.contains("signature verification FAILED"));
}

#[test]
fn test_run_verify_missing_signature_file_fails() {
    let dir = TestDir::new("missing_sig");
    let (wasm_file, _bytes, _sha) = build_test_component(&dir, "identity");
    let key = generate_test_minisign_key(&dir, "key");

    // Do NOT create .minisig file, but supply --public-key
    let output = Command::new(cli_binary_path())
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("77")
        .arg("--verify")
        .arg("--public-key")
        .arg(&key.pub_file)
        .output()
        .expect("must run CLI");

    assert_eq!(output.status.code(), Some(1));
    let out_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out_text.contains("Signature file not found"));
}

#[test]
fn test_run_verify_json_output() {
    let dir = TestDir::new("json_verify");
    let (wasm_file, bytes, sha) = build_test_component(&dir, "identity");
    let key = generate_test_minisign_key(&dir, "key");
    let sig_file = dir.path().join("identity.wasm.minisig");
    sign_file(&key, &bytes, &sig_file);

    let output = Command::new(cli_binary_path())
        .arg("--format")
        .arg("json")
        .arg("run")
        .arg("-c")
        .arg(&wasm_file)
        .arg("-f")
        .arg("identity")
        .arg("999")
        .arg("--verify")
        .arg("--expected-sha256")
        .arg(&sha)
        .arg("--public-key")
        .arg(&key.pub_file)
        .output()
        .expect("must run CLI");

    assert_eq!(output.status.code(), Some(0));
    let json_val: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("output must be valid JSON");

    assert_eq!(json_val["success"], true);
    assert_eq!(json_val["results"][0], 999);

    let verification = &json_val["verification"];
    assert_eq!(verification["integrity_verified"], true);
    assert_eq!(verification["signature_verified"], true);
    assert_eq!(verification["computed_sha256"], sha);
    assert_eq!(verification["expected_sha256"], sha);
}
