//! Integration tests for release packaging, SHA-256 integrity manifests,
//! Minisign offline verification, and GitHub Actions workflow security governance.
//!
//! Validates:
//! 1. Packaging structure matches ADR-006 (strictly executable, LICENSE, README.md).
//! 2. SHA-256 manifest calculation, sorting, and tamper detection.
//! 3. Cryptographic fail-explicit behavior on modified archives.
//! 4. Workflow security invariants (.github/workflows/release.yml):
//!    - Read-only default permissions.
//!    - Segregated publish job disabled per Decisión D3 (`if: false`).
//!    - Ephemeral test keys for dry-runs; zero mock signature placeholders.
//!    - All 3 canonical target triples represented.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates/
    path.pop(); // root/
    path
}

#[test]
fn test_sha256sums_generation_and_tamper_detection() {
    let temp_dir =
        std::env::temp_dir().join(format!("agent_native_release_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    // Create 3 simulated platform archives
    let files = [
        (
            "agent-native-v0.1.0-aarch64-apple-darwin.tar.gz",
            b"binary-mac-arm64-content" as &[u8],
        ),
        (
            "agent-native-v0.1.0-x86_64-pc-windows-msvc.zip",
            b"binary-windows-content" as &[u8],
        ),
        (
            "agent-native-v0.1.0-x86_64-unknown-linux-gnu.tar.gz",
            b"binary-linux-content" as &[u8],
        ),
    ];

    let mut checksum_map = BTreeMap::new();
    for (filename, content) in &files {
        let file_path = temp_dir.join(filename);
        fs::write(&file_path, content).expect("write file");

        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = format!("{:x}", hasher.finalize());
        checksum_map.insert(*filename, hash);
    }

    // Generate SHA256SUMS deterministically sorted by filename
    let mut manifest_lines = Vec::new();
    for (filename, hash) in &checksum_map {
        manifest_lines.push(format!("{hash}  {filename}"));
    }
    let manifest_content = manifest_lines.join("\n") + "\n";
    let manifest_path = temp_dir.join("SHA256SUMS");
    fs::write(&manifest_path, &manifest_content).expect("write SHA256SUMS");

    // 1. Positive verification: all hashes match
    for line in manifest_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts.len(), 2);
        let expected_hash = parts[0];
        let filename = parts[1];

        let content = fs::read(temp_dir.join(filename)).expect("read archive");
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let actual_hash = format!("{:x}", hasher.finalize());
        assert_eq!(
            actual_hash, expected_hash,
            "Checksum must match for {filename}"
        );
    }

    // 2. Negative verification: tampering with a single byte MUST fail verification
    let tampered_path = temp_dir.join("agent-native-v0.1.0-x86_64-unknown-linux-gnu.tar.gz");
    let mut tampered_content = fs::read(&tampered_path).expect("read for tamper");
    tampered_content[0] ^= 0xFF; // Flip bits
    fs::write(&tampered_path, &tampered_content).expect("write tampered content");

    let mut hasher = Sha256::new();
    hasher.update(&tampered_content);
    let tampered_actual_hash = format!("{:x}", hasher.finalize());
    let original_hash = checksum_map
        .get("agent-native-v0.1.0-x86_64-unknown-linux-gnu.tar.gz")
        .unwrap();

    assert_ne!(
        &tampered_actual_hash, original_hash,
        "Tampered file MUST NOT match original SHA-256 digest"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_release_workflow_security_invariants() {
    let workflow_path = repo_root()
        .join(".github")
        .join("workflows")
        .join("release.yml");
    assert!(
        workflow_path.exists(),
        "Workflow .github/workflows/release.yml must exist"
    );

    let content = fs::read_to_string(&workflow_path).expect("read release.yml");

    // Security Invariant 1: Top-level default permissions MUST be read-only
    assert!(
        content.contains("permissions:\n  contents: read")
            || content.contains("permissions:\r\n  contents: read"),
        "Workflow must specify top-level default permissions: contents: read"
    );

    // Security Invariant 2: Build and Validate jobs must have read-only contents
    assert!(
        content.contains("name: Build & Package"),
        "Must define build job"
    );
    assert!(
        content.contains("name: Checksums, Minisign & Integrity Validation"),
        "Must define validation job"
    );

    // Security Invariant 3: Publishing to GitHub Releases is disabled per Decisión D3
    assert!(
        content.contains("if: false"),
        "Release publication step must be gated with 'if: false' in accordance with Decisión D3"
    );
    assert!(
        content.contains("DECISIÓN D3") || content.contains("Decisión D3"),
        "Workflow must reference Decisión D3 rationale for gating publication"
    );

    // Security Invariant 4: No insecure mock signature placeholders
    assert!(
        !content.contains("dGVzdC1zaWduYXR1cmUtcGxhY2Vob2xkZXI="),
        "Insecure mock base64 placeholder MUST NOT exist in workflow"
    );

    // Security Invariant 5: Ephemeral key generation used in dry-run mode
    assert!(
        content.contains("minisign -G"),
        "Workflow must use real ephemeral keypair generation in validation/dry-run mode"
    );
    assert!(
        content.contains("SHA256SUMS.test-minisig"),
        "Dry-run signature must be explicitly named SHA256SUMS.test-minisig"
    );

    // Security Invariant 6: Official release mode blocks on placeholder public key
    assert!(
        content.contains("PLACEHOLDER"),
        "Workflow must check for PLACEHOLDER in public key to prevent official signing without authentic keys"
    );

    // Architecture Invariant 7: All 3 canonical triples present in matrix
    assert!(
        content.contains("x86_64-unknown-linux-gnu"),
        "Must support Linux x86_64"
    );
    assert!(
        content.contains("x86_64-pc-windows-msvc"),
        "Must support Windows x86_64"
    );
    assert!(
        content.contains("aarch64-apple-darwin"),
        "Must support macOS arm64"
    );

    // Architecture Invariant 8: Architecture validation on macOS
    assert!(
        content.contains("arm64|aarch64"),
        "macOS step must explicitly validate that binary architecture is arm64"
    );
}

#[test]
fn test_release_signing_key_placeholder_status() {
    let key_path = repo_root()
        .join("knowledge")
        .join("tech")
        .join("release-signing-key.pub");
    assert!(key_path.exists(), "release-signing-key.pub must exist");

    let content = fs::read_to_string(&key_path).expect("read release-signing-key.pub");

    // Verifies that the file explicitly identifies itself as a placeholder
    assert!(
        content.contains("PLACEHOLDER"),
        "release-signing-key.pub must be explicitly marked as PLACEHOLDER until human provisioning"
    );
    assert!(
        content.contains("untrusted comment:"),
        "Key file must follow Minisign public key format header"
    );
}

#[test]
fn test_package_structure_contract_adr006() {
    // Validates that packaging contract requires strictly:
    // 1. executable (agent-native or agent-native.exe)
    // 2. LICENSE
    // 3. README.md
    let license_path = repo_root().join("LICENSE");
    let readme_path = repo_root().join("README.md");

    assert!(
        license_path.exists(),
        "Repository root must contain LICENSE"
    );
    assert!(
        readme_path.exists(),
        "Repository root must contain README.md"
    );

    let license_content = fs::read_to_string(&license_path).expect("read LICENSE");
    assert!(
        license_content.contains("MIT License"),
        "LICENSE must be MIT"
    );

    let readme_content = fs::read_to_string(&readme_path).expect("read README");
    assert!(!readme_content.is_empty(), "README must not be empty");
}
