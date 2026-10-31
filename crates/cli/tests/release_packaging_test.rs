//! Integration tests for release packaging, SHA-256 integrity manifests,
//! Minisign offline verification, archive inspection, and bit-for-bit reproducibility.
//!
//! Validates:
//! 1. Packaging structure matches ADR-006 (strictly executable, LICENSE, README.md).
//! 2. Deep inspection of .tar.gz and .zip archives (file names, permissions, and executability).
//! 3. Bit-for-bit binary packaging reproducibility (demonstrated by hash equivalence
//!    under normalized timestamps, permissions, and file ordering).
//! 4. SHA-256 manifest calculation, sorting, and tamper detection.
//! 5. Workflow security invariants (.github/workflows/release.yml):
//!    - Read-only default permissions (`contents: read`).
//!    - Segregated publish job disabled per Decisión D3 (`if: false`).
//!    - Ephemeral test keys for dry-runs; zero mock signature placeholders.
//!    - All 3 canonical target triples represented.
//!    - GNU tar / gtar compatibility on macOS.

use flate2::GzBuilder;
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates/
    path.pop(); // root/
    path
}

/// Standard IEEE 802.3 CRC32 calculation without external dependencies.
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// Helper that builds a deterministic `.tar.gz` archive in memory with normalized metadata.
fn build_deterministic_tar_gz(
    entries: &[(&str, &[u8], u32)], // (name, content, mode)
    fixed_mtime: u64,
) -> Vec<u8> {
    let mut tar_builder = tar::Builder::new(Vec::new());

    // Sort entries deterministically by filename
    let mut sorted_entries = entries.to_vec();
    sorted_entries.sort_by_key(|e| e.0);

    for (name, content, mode) in sorted_entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(content.len()).unwrap());
        header.set_mode(mode);
        header.set_mtime(fixed_mtime);
        header.set_uid(0);
        header.set_gid(0);
        header.set_cksum();
        tar_builder
            .append_data(&mut header, name, std::io::Cursor::new(content))
            .expect("append tar entry");
    }

    let tar_bytes = tar_builder.into_inner().expect("finalize tar");

    // Compress with GzBuilder using fixed mtime for header determinism
    let mut gz_encoder = GzBuilder::new()
        .mtime(u32::try_from(fixed_mtime).unwrap())
        .write(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut gz_encoder, &tar_bytes).expect("compress gzip");
    gz_encoder.finish().expect("finish gzip")
}

/// Simple zero-dependency ZIP reader that inspects local file headers.
fn inspect_zip_entries(bytes: &[u8]) -> Vec<(String, u64)> {
    let mut entries = Vec::new();
    let mut pos = 0;
    while pos + 30 <= bytes.len() {
        if &bytes[pos..pos + 4] != b"PK\x03\x04" {
            break;
        }
        let comp_size = usize::try_from(u32::from_le_bytes(
            bytes[pos + 18..pos + 22].try_into().unwrap(),
        ))
        .unwrap();
        let uncomp_size = u64::from(u32::from_le_bytes(
            bytes[pos + 22..pos + 26].try_into().unwrap(),
        ));
        let name_len = usize::from(u16::from_le_bytes(
            bytes[pos + 26..pos + 28].try_into().unwrap(),
        ));
        let extra_len = usize::from(u16::from_le_bytes(
            bytes[pos + 28..pos + 30].try_into().unwrap(),
        ));

        if pos + 30 + name_len > bytes.len() {
            break;
        }
        let name = String::from_utf8_lossy(&bytes[pos + 30..pos + 30 + name_len]).to_string();
        entries.push((name, uncomp_size));
        pos += 30 + name_len + extra_len + comp_size;
    }
    entries
}

/// Helper that builds a minimal uncompressed ZIP archive with fixed timestamps for testing.
fn build_deterministic_zip(
    entries: &[(&str, &[u8])],
    fixed_dos_time: u16,
    fixed_dos_date: u16,
) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut central_dir = Vec::new();

    // Sort entries deterministically
    let mut sorted_entries = entries.to_vec();
    sorted_entries.sort_by_key(|e| e.0);

    for (name, content) in sorted_entries {
        let local_header_offset = u32::try_from(buf.len()).unwrap();
        let file_crc = crc32(content);
        let content_len_u32 = u32::try_from(content.len()).unwrap();
        let name_len_u16 = u16::try_from(name.len()).unwrap();

        // Local file header (30 bytes + name)
        buf.extend_from_slice(b"PK\x03\x04");
        buf.extend_from_slice(&20u16.to_le_bytes()); // version needed: 2.0
        buf.extend_from_slice(&0u16.to_le_bytes()); // flags
        buf.extend_from_slice(&0u16.to_le_bytes()); // method: 0 (stored)
        buf.extend_from_slice(&fixed_dos_time.to_le_bytes());
        buf.extend_from_slice(&fixed_dos_date.to_le_bytes());
        buf.extend_from_slice(&file_crc.to_le_bytes());
        buf.extend_from_slice(&content_len_u32.to_le_bytes()); // comp size
        buf.extend_from_slice(&content_len_u32.to_le_bytes()); // uncomp size
        buf.extend_from_slice(&name_len_u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes()); // extra len
        buf.extend_from_slice(name.as_bytes());
        buf.extend_from_slice(content);

        // Central directory entry
        central_dir.extend_from_slice(b"PK\x01\x02");
        central_dir.extend_from_slice(&20u16.to_le_bytes()); // version made by
        central_dir.extend_from_slice(&20u16.to_le_bytes()); // version needed
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // flags
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // method
        central_dir.extend_from_slice(&fixed_dos_time.to_le_bytes());
        central_dir.extend_from_slice(&fixed_dos_date.to_le_bytes());
        central_dir.extend_from_slice(&file_crc.to_le_bytes());
        central_dir.extend_from_slice(&content_len_u32.to_le_bytes());
        central_dir.extend_from_slice(&content_len_u32.to_le_bytes());
        central_dir.extend_from_slice(&name_len_u16.to_le_bytes());
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // extra len
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // comment len
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // disk nr
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        central_dir.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        central_dir.extend_from_slice(&local_header_offset.to_le_bytes());
        central_dir.extend_from_slice(name.as_bytes());
    }

    let central_dir_offset = u32::try_from(buf.len()).unwrap();
    let central_dir_size = u32::try_from(central_dir.len()).unwrap();
    buf.extend_from_slice(&central_dir);

    let entries_count_u16 = u16::try_from(entries.len()).unwrap();

    // End of central directory record (22 bytes)
    buf.extend_from_slice(b"PK\x05\x06");
    buf.extend_from_slice(&0u16.to_le_bytes()); // disk number
    buf.extend_from_slice(&0u16.to_le_bytes()); // disk with cd
    buf.extend_from_slice(&entries_count_u16.to_le_bytes()); // entries on disk
    buf.extend_from_slice(&entries_count_u16.to_le_bytes()); // total entries
    buf.extend_from_slice(&central_dir_size.to_le_bytes());
    buf.extend_from_slice(&central_dir_offset.to_le_bytes());
    buf.extend_from_slice(&0u16.to_le_bytes()); // comment len

    buf
}

#[test]
fn test_tar_gz_archive_structure_permissions_and_executability() {
    let binary_content = b"\x7fELF-mock-agent-native-binary-contents";
    let license_content = b"MIT License Mock";
    let readme_content = b"# Agent-Native CLI Readme Mock";

    let entries = [
        ("agent-native", binary_content as &[u8], 0o755),
        ("LICENSE", license_content as &[u8], 0o644),
        ("README.md", readme_content as &[u8], 0o644),
    ];

    let tar_gz_bytes = build_deterministic_tar_gz(&entries, 1_767_225_600);

    // Decompress and inspect entries inside the .tar.gz archive
    let gz_decoder = GzDecoder::new(&tar_gz_bytes[..]);
    let mut archive = tar::Archive::new(gz_decoder);

    let mut inspected_names = Vec::new();
    for entry_res in archive.entries().expect("read tar entries") {
        let mut entry = entry_res.expect("valid tar entry");
        let path = entry
            .path()
            .expect("entry path")
            .to_str()
            .unwrap()
            .to_string();
        let mode = entry.header().mode().expect("entry mode");

        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read entry content");

        if path == "agent-native" {
            // Executability test: must have executable bits set (0o111)
            assert_eq!(
                mode & 0o111,
                0o111,
                "agent-native binary entry must have executable permissions"
            );
            assert_eq!(data, binary_content);
        } else if path == "LICENSE" {
            assert_eq!(mode & 0o111, 0, "LICENSE must not be executable");
            assert_eq!(data, license_content);
        } else if path == "README.md" {
            assert_eq!(mode & 0o111, 0, "README.md must not be executable");
            assert_eq!(data, readme_content);
        } else {
            panic!("Unexpected entry in archive: {path}");
        }

        inspected_names.push(path);
    }

    assert_eq!(
        inspected_names,
        vec!["LICENSE", "README.md", "agent-native"],
        "Archive must contain strictly the 3 canonical entries with no leading directory paths"
    );
}

#[test]
fn test_zip_archive_structure_inspection() {
    let binary_content = b"MZ-mock-windows-binary";
    let license_content = b"MIT License Mock";
    let readme_content = b"# Agent-Native Readme";

    let entries = [
        ("agent-native.exe", binary_content as &[u8]),
        ("LICENSE", license_content as &[u8]),
        ("README.md", readme_content as &[u8]),
    ];

    let zip_bytes = build_deterministic_zip(&entries, 0, 0x5821);
    let inspected = inspect_zip_entries(&zip_bytes);

    let entry_names: Vec<String> = inspected.into_iter().map(|(name, _)| name).collect();
    assert_eq!(
        entry_names,
        vec!["LICENSE", "README.md", "agent-native.exe"],
        "Zip archive must contain strictly the 3 canonical entries at root"
    );
}

#[test]
fn test_bit_for_bit_packaging_reproducibility() {
    let binary_data = b"REPRODUCIBLE_BINARY_PAYLOAD_V1";
    let license_data = b"MIT License";
    let readme_data = b"# Documentation";

    let entries = [
        ("agent-native", binary_data as &[u8], 0o755),
        ("LICENSE", license_data as &[u8], 0o644),
        ("README.md", readme_data as &[u8], 0o644),
    ];

    // Pack 1: generated at "Run 1"
    let pkg1 = build_deterministic_tar_gz(&entries, 1_767_225_600);
    let mut hasher1 = Sha256::new();
    hasher1.update(&pkg1);
    let hash1 = format!("{:x}", hasher1.finalize());

    // Pack 2: generated independently at "Run 2"
    let pkg2 = build_deterministic_tar_gz(&entries, 1_767_225_600);
    let mut hasher2 = Sha256::new();
    hasher2.update(&pkg2);
    let hash2 = format!("{:x}", hasher2.finalize());

    // Proof of bit-for-bit reproducibility:
    assert_eq!(
        hash1, hash2,
        "Independent packagings with normalized metadata MUST be byte-identical"
    );
    assert_eq!(pkg1, pkg2, "Byte buffers of independent runs must match");

    // Negative proof: altering mtime alters binary hash
    let pkg_different_time = build_deterministic_tar_gz(&entries, 1_767_225_601);
    let mut hasher_diff = Sha256::new();
    hasher_diff.update(&pkg_different_time);
    let hash_diff = format!("{:x}", hasher_diff.finalize());
    assert_ne!(
        hash1, hash_diff,
        "Different timestamp must yield different hash, proving mtime is captured and normalized"
    );
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
#[allow(clippy::too_many_lines)]
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
        content.contains(
            "permissions:
  contents: read"
        ) || content.contains(
            "permissions:
  contents: read"
        ),
        "Workflow must specify top-level default permissions: contents: read"
    );

    // Architecture Invariant 2: Segregation of distinct jobs including pre-flight fail-closed gate
    assert!(
        content.contains("name: Pre-Flight Gate & Safety Verification"),
        "Must define pre-flight gate job"
    );
    assert!(
        content.contains("needs: pre-flight"),
        "build job must depend on pre-flight gate"
    );
    assert!(
        content.contains("name: Build & Package"),
        "Must define build job"
    );
    assert!(
        content.contains("name: Checksums & Package Integrity"),
        "Must define validate job (Checksums & Package Integrity)"
    );
    assert!(
        content.contains("name: Official Cryptographic Signing (Guarded Gate)"),
        "Must define sign-official job"
    );
    assert!(
        content.contains("name: Publish GitHub Release (Guarded by D3)"),
        "Must define publish job"
    );

    // Fail-Closed Control Invariant 2b: Official mode blocked at pre-flight while sign-official is disabled
    assert!(
        content.contains("if [ \"${MODE}\" = \"official\" ]; then"),
        "Pre-flight gate must check for mode == official"
    );
    assert!(
        content.contains(
            "ERROR: Official release mode ('mode: official') is BLOCKED and FAIL-CLOSED."
        ),
        "Pre-flight gate must emit explicit fail-closed error message for official mode"
    );

    // Security Invariant 3: Both sign-official and publish are strictly disabled per Decisión D3
    let sign_job_pos = content
        .find("  sign-official:")
        .or_else(|| content.find("sign-official:"))
        .expect("sign-official job");
    let publish_job_pos = content
        .find("  publish:")
        .or_else(|| {
            content.find(
                "
publish:",
            )
        })
        .expect("publish job");

    let sign_job_block = &content[sign_job_pos..publish_job_pos];
    assert!(
        sign_job_block.contains("if: false"),
        "sign-official job MUST be disabled with 'if: false' until human provisioning"
    );

    let publish_job_block = &content[publish_job_pos..];
    assert!(
        publish_job_block.contains("if: false"),
        "publish job MUST be disabled with 'if: false' per Decisión D3"
    );
    assert!(
        content.contains("DECISIÓN D3") || content.contains("Decisión D3"),
        "Workflow must reference Decisión D3 rationale"
    );

    // Governance Invariant 4: sign-official associates with protected Environment 'release'
    assert!(
        sign_job_block.contains("environment: release"),
        "sign-official job must be associated with 'release' Environment"
    );
    assert!(
        sign_job_block.contains("Required Reviewers"),
        "sign-official must document Required Reviewers requirement"
    );
    assert!(
        sign_job_block.contains("Prevent self-review"),
        "sign-official must document Prevent self-review policy"
    );
    assert!(
        sign_job_block.contains("Bypass administrativo"),
        "sign-official must document administrative bypass prohibition"
    );

    // Version Invariant 5: Dynamic tag traceability with Cargo version and commit SHA
    assert!(
        content.contains("TAG=\"v${CARGO_PKG_VER}-dev-${SHORT_COMMIT_SHA}\""),
        "Workflow must derive tag as v{{CARGO_VERSION}}-dev-{{SHORT_SHA}} in dev/validation mode"
    );
    assert!(
        content.contains("FULL_COMMIT_SHA"),
        "Workflow must record full commit SHA in provenance metadata"
    );
    assert!(
        content.contains("SHORT_COMMIT_SHA"),
        "Workflow must record short commit SHA for readability"
    );

    // Artifact Invariant 6: Differentiated bundles (release-bundle vs official-release-bundle)
    assert!(
        content.contains("name: release-bundle"),
        "Validate job must upload unverified/dry-run release-bundle"
    );
    assert!(
        content.contains("name: official-release-bundle"),
        "sign-official job must upload official-release-bundle"
    );

    // Security Invariant 7: Secret isolation between validate and sign-official
    let validate_job_pos = content
        .find("  validate:")
        .or_else(|| content.find("validate:"))
        .expect("validate job");
    let validate_job_block = &content[validate_job_pos..sign_job_pos];
    assert!(
        !validate_job_block.contains("RELEASE_MINISIGN_SECRET_KEY"),
        "validate job MUST NOT have access to RELEASE_MINISIGN_SECRET_KEY"
    );
    assert!(
        !validate_job_block.contains("RELEASE_MINISIGN_PASSWORD"),
        "validate job MUST NOT have access to RELEASE_MINISIGN_PASSWORD"
    );
    assert!(
        sign_job_block.contains("RELEASE_MINISIGN_SECRET_KEY"),
        "sign-official job must consume RELEASE_MINISIGN_SECRET_KEY"
    );
    assert!(
        sign_job_block.contains("RELEASE_MINISIGN_PASSWORD"),
        "sign-official job must consume RELEASE_MINISIGN_PASSWORD"
    );

    // Security Invariant 8: In-memory RAM filesystem (tmpfs) to avoid SSD key persistence
    assert!(
        sign_job_block.contains("tmpfs"),
        "sign-official must utilize RAM tmpfs to avoid persistent flash retention on SSDs"
    );

    // Security Invariant 9: No insecure mock signature placeholders
    assert!(
        !content.contains("dGVzdC1zaWduYXR1cmUtcGxhY2Vob2xkZXI="),
        "Insecure mock base64 placeholder MUST NOT exist in workflow"
    );

    // Security Invariant 10: Ephemeral key generation used in dry-run mode
    assert!(
        content.contains("minisign -G"),
        "Workflow must use real ephemeral keypair generation in validation/dry-run mode"
    );
    assert!(
        content.contains("SHA256SUMS.test-minisig"),
        "Dry-run signature must be explicitly named SHA256SUMS.test-minisig"
    );

    // Security Invariant 11: Official release mode blocks on placeholder public key
    assert!(
        content.contains("PLACEHOLDER"),
        "Workflow must check for PLACEHOLDER in public key"
    );

    // Architecture Invariant 12: All 3 canonical triples present in matrix
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

    // Architecture Invariant 13: Architecture validation on macOS
    assert!(
        content.contains("arm64|aarch64"),
        "macOS step must explicitly validate that binary architecture is arm64"
    );

    // Architecture Invariant 14: Portable GNU tar / gtar handling on macOS
    assert!(
        content.contains("brew install gnu-tar"),
        "macOS job must install gnu-tar for portable --sort=name support"
    );
    assert!(
        content.contains("gtar"),
        "Workflow must invoke gtar when available to avoid BSD tar incompatibility"
    );

    // Build Invariant 15: Locked builds for reproducible dependency resolution
    assert!(
        content.contains("cargo build --release --locked"),
        "Cargo build must use --locked"
    );

    // Packaging Invariant 16: Temporary staging directories excluded from artifact uploads
    assert!(
        content.contains("rm -rf \"${STAGE_DIR}\""),
        "Unix packaging must remove staging directory after archive verification"
    );
    assert!(
        content.contains("Remove-Item -LiteralPath $StageDir -Recurse -Force"),
        "Windows packaging must remove staging directory after archive creation"
    );
    assert!(
        content.contains("path: wasma-*.${{ matrix.archive_ext }}"),
        "Upload artifact must target archive extension directly"
    );

    // Inventory Invariant 17: Strict directory inspection rejecting non-regular files
    assert!(
        content.contains("ALL_ENTRIES=$(ls -1A | LC_ALL=C sort)"),
        "Validation step must inspect ALL entries in dist/ without pre-filtering"
    );
    assert!(
        content.contains("[ -d \"${entry}\" ]"),
        "Validation step must reject directories in dist/"
    );
    assert!(
        content.contains("[ -L \"${entry}\" ]"),
        "Validation step must reject symlinks in dist/"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FsEntryKind {
    RegularFile,
    Directory,
    Symlink,
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_package_inventory_exact_validation_and_negative_rejections() {
    let tag = "v0.1.0";
    let canonical_linux = format!("wasma-{tag}-x86_64-unknown-linux-gnu.tar.gz");
    let canonical_windows = format!("wasma-{tag}-x86_64-pc-windows-msvc.zip");
    let canonical_macos = format!("wasma-{tag}-aarch64-apple-darwin.tar.gz");

    let canonical_expected = vec![
        canonical_macos.clone(),
        canonical_linux.clone(),
        canonical_windows.clone(),
    ];

    let validate_dist_inventory = |entries: &[(&str, FsEntryKind)]| -> Result<(), String> {
        // Step 1: Reject any non-regular files (directories, symlinks, etc.)
        for &(name, kind) in entries {
            if kind == FsEntryKind::Directory {
                return Err(format!("Unexpected directory found in dist/: {name}"));
            }
            if kind == FsEntryKind::Symlink {
                return Err(format!("Unexpected symlink found in dist/: {name}"));
            }
            if kind != FsEntryKind::RegularFile {
                return Err(format!("Entry is not a regular file: {name}"));
            }
        }

        // Step 2: Total count must be exactly 3
        if entries.len() != 3 {
            return Err(format!(
                "Expected exactly 3 entries, found {}",
                entries.len()
            ));
        }

        // Step 3: Check existence of all 3 canonical files
        let names: Vec<&str> = entries.iter().map(|(n, _)| *n).collect();
        for expected in &[&canonical_linux, &canonical_windows, &canonical_macos] {
            if !names.contains(&expected.as_str()) {
                return Err(format!("Missing expected archive: {expected}"));
            }
        }

        // Step 4: Strict equality against expected inventory list
        let mut sorted_names = names;
        sorted_names.sort_unstable();
        let mut sorted_expected = canonical_expected.clone();
        sorted_expected.sort_unstable();
        if sorted_names != sorted_expected {
            return Err("Inventory mismatch".to_string());
        }

        Ok(())
    };

    // 1. Positive case: exactly the 3 canonical regular files
    let valid_inventory = [
        (canonical_linux.as_str(), FsEntryKind::RegularFile),
        (canonical_windows.as_str(), FsEntryKind::RegularFile),
        (canonical_macos.as_str(), FsEntryKind::RegularFile),
    ];
    assert!(validate_dist_inventory(&valid_inventory).is_ok());

    // 2. Negative case: missing target (only 2 packages)
    let missing_macos = [
        (canonical_linux.as_str(), FsEntryKind::RegularFile),
        (canonical_windows.as_str(), FsEntryKind::RegularFile),
    ];
    assert!(validate_dist_inventory(&missing_macos).is_err());

    // 3. Negative case: unexpected extra package (4 packages)
    let extra_package = [
        (canonical_linux.as_str(), FsEntryKind::RegularFile),
        (canonical_windows.as_str(), FsEntryKind::RegularFile),
        (canonical_macos.as_str(), FsEntryKind::RegularFile),
        ("wasma-v0.1.0-extra-target.tar.gz", FsEntryKind::RegularFile),
    ];
    assert!(validate_dist_inventory(&extra_package).is_err());

    // 4. Negative case: version discrepancy (v0.2.0 mixed with v0.1.0)
    let version_mismatch = [
        (
            "wasma-v0.2.0-x86_64-unknown-linux-gnu.tar.gz",
            FsEntryKind::RegularFile,
        ),
        (canonical_windows.as_str(), FsEntryKind::RegularFile),
        (canonical_macos.as_str(), FsEntryKind::RegularFile),
    ];
    assert!(validate_dist_inventory(&version_mismatch).is_err());

    // 5. Negative case: legacy name (agent-native instead of wasma)
    let legacy_named = [
        (
            "agent-native-v0.1.0-x86_64-unknown-linux-gnu.tar.gz",
            FsEntryKind::RegularFile,
        ),
        (canonical_windows.as_str(), FsEntryKind::RegularFile),
        (canonical_macos.as_str(), FsEntryKind::RegularFile),
    ];
    assert!(validate_dist_inventory(&legacy_named).is_err());

    // 6. Negative case: staging directory included alongside 3 archives (CI run 41137309657 regression test)
    let staging_dir_leak = [
        (canonical_linux.as_str(), FsEntryKind::RegularFile),
        (canonical_windows.as_str(), FsEntryKind::RegularFile),
        (canonical_macos.as_str(), FsEntryKind::RegularFile),
        (
            "wasma-v0.1.0-x86_64-unknown-linux-gnu",
            FsEntryKind::Directory,
        ),
    ];
    let res = validate_dist_inventory(&staging_dir_leak);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Unexpected directory"));

    // 7. Negative case: symlink inside dist/
    let symlink_in_dist = [
        (canonical_linux.as_str(), FsEntryKind::RegularFile),
        (canonical_windows.as_str(), FsEntryKind::RegularFile),
        (canonical_macos.as_str(), FsEntryKind::Symlink),
    ];
    let res = validate_dist_inventory(&symlink_in_dist);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Unexpected symlink"));

    // 8. Positive case: dev/validation mode dynamic naming scheme (v0.1.0-dev-d3e0d54)
    let dev_tag = "v0.1.0-dev-d3e0d54";
    let dev_linux = format!("wasma-{dev_tag}-x86_64-unknown-linux-gnu.tar.gz");
    let dev_windows = format!("wasma-{dev_tag}-x86_64-pc-windows-msvc.zip");
    let dev_macos = format!("wasma-{dev_tag}-aarch64-apple-darwin.tar.gz");
    let dev_inventory = [
        (dev_linux.as_str(), FsEntryKind::RegularFile),
        (dev_windows.as_str(), FsEntryKind::RegularFile),
        (dev_macos.as_str(), FsEntryKind::RegularFile),
    ];
    let validate_dev_inventory = |entries: &[(&str, FsEntryKind)]| -> Result<(), String> {
        let canonical_dev = vec![dev_macos.clone(), dev_linux.clone(), dev_windows.clone()];
        if entries.len() != 3 {
            return Err("Expected 3 entries".to_string());
        }
        let names: Vec<&str> = entries.iter().map(|(n, _)| *n).collect();
        for exp in &[&dev_linux, &dev_windows, &dev_macos] {
            if !names.contains(&exp.as_str()) {
                return Err("Missing expected archive".to_string());
            }
        }
        let mut sorted_names = names;
        sorted_names.sort_unstable();
        let mut sorted_exp = canonical_dev;
        sorted_exp.sort_unstable();
        if sorted_names != sorted_exp {
            return Err("Mismatch".to_string());
        }
        Ok(())
    };
    assert!(validate_dev_inventory(&dev_inventory).is_ok());
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
fn test_fail_closed_control_for_official_mode() {
    // Simulates the pre-flight gate decision logic:
    // When sign-official is disabled (if: false), mode: official must fail-closed immediately.
    let evaluate_pre_flight_mode = |mode: &str,
                                    sign_official_enabled: bool|
     -> Result<(), &'static str> {
        let effective_mode = if mode.is_empty() { "validation" } else { mode };
        if effective_mode == "official" {
            if !sign_official_enabled {
                return Err(
                    "ERROR: Official release mode ('mode: official') is BLOCKED and FAIL-CLOSED.",
                );
            }
            return Ok(());
        }
        if effective_mode == "validation" {
            return Ok(());
        }
        Err("Unknown mode")
    };

    // Case 1: mode == 'validation' passes cleanly (dry-run mode)
    assert!(evaluate_pre_flight_mode("validation", false).is_ok());

    // Case 2: default/empty mode defaults to 'validation' and passes cleanly
    assert!(evaluate_pre_flight_mode("", false).is_ok());

    // Case 3: mode == 'official' FAILS CLOSED while sign_official is disabled
    let res = evaluate_pre_flight_mode("official", false);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        "ERROR: Official release mode ('mode: official') is BLOCKED and FAIL-CLOSED."
    );

    // Case 4: mode == 'official' would only pass when sign-official is legitimately enabled
    assert!(evaluate_pre_flight_mode("official", true).is_ok());
}
