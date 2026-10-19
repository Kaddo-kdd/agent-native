//! Integration tests for atomic publication, version manifests, and integrity verification.
//!
//! Validates AC-01 through AC-05 of AN-026 completely offline.

use std::fs;
use std::path::PathBuf;

use agent_native::agent::FixtureLlmProvider;
use agent_native::publish::{
    BuildFailPoint, PublishError, build_and_publish_components, recover_and_verify_dist_state,
    resolve_active_component_artifact,
};
use agent_native::spec::model::{ComponentSpec, SpecMetadata, SystemSpec};
use serde_json::json;

fn create_test_spec() -> SystemSpec {
    SystemSpec {
        schema: None,
        spec_version: "1.0.0".to_string(),
        metadata: SpecMetadata {
            title: "Math Service".to_string(),
            description: "Sample math service".to_string(),
            source_type: "narrative".to_string(),
            source_files: vec!["math.md".to_string()],
            last_updated: "2026-10-19".to_string(),
        },
        components: vec![ComponentSpec {
            id: "math".to_string(),
            name: "Math Component".to_string(),
            description: "Math operations".to_string(),
            wit_file: "tests/fixtures/wit/program-005.wit".to_string(),
            world: "math".to_string(),
            export_interfaces: vec!["ops".to_string()],
            import_interfaces: vec![],
            capabilities: vec![],
            dependencies: vec![],
        }],
        requirements: vec![],
        entities: vec![],
        operations: vec![],
    }
}

fn create_fixture_provider() -> FixtureLlmProvider {
    let mut provider = FixtureLlmProvider::new("test-fixture-provider");
    let golden_ir = json!({
        "ir_version": "0.5",
        "spec_version": 1,
        "wit_contract": {
            "package": "an:test@0.1.0",
            "world": "math",
            "export_interface": "ops",
            "import_interfaces": [],
            "wit_file": "tests/fixtures/wit/program-005.wit"
        },
        "types": [
            {
                "name": "point_sum",
                "fields": [
                    { "name": "sum", "type": "i32" },
                    { "name": "doubled", "type": "i32" }
                ]
            }
        ],
        "functions": [
            {
                "name": "double_value",
                "export": true,
                "wit_interface": "ops",
                "params": [{ "name": "x", "type": "i32" }],
                "returns": "i32",
                "requirements": ["MATH-001"],
                "body": {
                    "op": "mul",
                    "left": { "op": "param", "name": "x" },
                    "right": { "op": "const", "type": "i32", "value": 2 }
                }
            },
            {
                "name": "make_point_sum",
                "export": true,
                "wit_interface": "ops",
                "params": [
                    { "name": "x", "type": "i32" },
                    { "name": "y", "type": "i32" }
                ],
                "returns": "point_sum",
                "requirements": ["MATH-002"],
                "body": {
                    "op": "record.construct",
                    "type": "point_sum",
                    "fields": {
                        "sum": {
                            "op": "add",
                            "left": { "op": "param", "name": "x" },
                            "right": { "op": "param", "name": "y" }
                        },
                        "doubled": {
                            "op": "mul",
                            "left": {
                                "op": "add",
                                "left": { "op": "param", "name": "x" },
                                "right": { "op": "param", "name": "y" }
                            },
                            "right": { "op": "const", "type": "i32", "value": 2 }
                        }
                    }
                }
            }
        ]
    });
    provider.register_fixture("math", golden_ir);
    provider
}

fn temp_dist_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "agent_native_test_{test_name}_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn compiler_wit_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("compiler")
}

#[test]
fn test_atomic_publish_lifecycle_and_promotion() {
    let dist_dir = temp_dist_dir("lifecycle");
    let spec = create_test_spec();
    let provider = create_fixture_provider();
    let wit_root = compiler_wit_root();

    // 1. Initially no active version published
    let state_initial = recover_and_verify_dist_state(&dist_dir).unwrap();
    assert!(state_initial.is_none());

    // 2. Build and publish version 1
    let summary1 = build_and_publish_components(&provider, &spec, &dist_dir, Some(&wit_root), None)
        .expect("Publication of v1 should succeed");
    assert!(summary1.success);
    assert_eq!(summary1.artifacts.len(), 1);

    // Verify disk layout: versions/build-{id}/manifest.json and active.json exist
    let active_json = dist_dir.join("active.json");
    assert!(active_json.exists());
    let active_wasm = dist_dir.join(format!("versions/{}/math.wasm", summary1.version_id));
    assert!(active_wasm.exists());
    let active_ir = dist_dir.join(format!("versions/{}/math.ir.json", summary1.version_id));
    assert!(active_ir.exists());

    // Verify recovery and state inspection
    let (active_ver, manifest) = recover_and_verify_dist_state(&dist_dir).unwrap().unwrap();
    assert_eq!(active_ver, summary1.version_id);
    assert_eq!(manifest.version_id, summary1.version_id);
    assert_eq!(manifest.artifacts.len(), 1);

    // Resolve active artifact path directly
    let resolved_path = resolve_active_component_artifact(&dist_dir, "math").unwrap();
    assert_eq!(resolved_path, active_wasm);

    // 3. Promote a new version 2
    let summary2 = build_and_publish_components(&provider, &spec, &dist_dir, Some(&wit_root), None)
        .expect("Publication of v2 should succeed");
    assert_ne!(summary1.version_id, summary2.version_id);

    // Active version should now point to version 2
    let (active_ver2, manifest2) = recover_and_verify_dist_state(&dist_dir).unwrap().unwrap();
    assert_eq!(active_ver2, summary2.version_id);
    assert_eq!(manifest2.version_id, summary2.version_id);

    // Version 1 must still be preserved in the store (immutable version history)
    assert!(active_wasm.exists());

    let _ = fs::remove_dir_all(&dist_dir);
}

#[test]
fn test_publish_integrity_detection_on_tampered_artifact() {
    let dist_dir = temp_dist_dir("integrity");
    let spec = create_test_spec();
    let provider = create_fixture_provider();
    let wit_root = compiler_wit_root();

    let summary =
        build_and_publish_components(&provider, &spec, &dist_dir, Some(&wit_root), None).unwrap();
    let wasm_path = dist_dir.join(format!("versions/{}/math.wasm", summary.version_id));
    assert!(wasm_path.exists());

    // Tamper with published bytes
    fs::write(&wasm_path, b"corrupted bytes").unwrap();

    // Verification must detect integrity mismatch
    let err = recover_and_verify_dist_state(&dist_dir).unwrap_err();
    match err {
        PublishError::IntegrityMismatch(path, expected, found) => {
            assert!(path.contains("math.wasm"));
            assert_ne!(expected, found);
        }
        other => panic!("Expected IntegrityMismatch error, got {other:?}"),
    }

    let _ = fs::remove_dir_all(&dist_dir);
}

#[test]
fn test_atomic_rollback_on_injected_fail_point() {
    let dist_dir = temp_dist_dir("rollback");
    let spec = create_test_spec();
    let provider = create_fixture_provider();
    let wit_root = compiler_wit_root();

    // 1. Establish initial active version 1
    let summary1 =
        build_and_publish_components(&provider, &spec, &dist_dir, Some(&wit_root), None).unwrap();

    // 2. Attempt version 2 with failure during copy
    let err_copy = build_and_publish_components(
        &provider,
        &spec,
        &dist_dir,
        Some(&wit_root),
        Some(BuildFailPoint::DuringVersionCopy),
    )
    .unwrap_err();
    assert_eq!(
        err_copy,
        PublishError::InjectedFailPoint("DuringVersionCopy".to_string())
    );

    // Active version must still be version 1 completely intact
    let (active_ver, _) = recover_and_verify_dist_state(&dist_dir).unwrap().unwrap();
    assert_eq!(active_ver, summary1.version_id);

    // Staging must have been cleaned up
    assert!(!dist_dir.join(".staging").exists());

    // 3. Attempt version 3 with failure during active pointer promotion
    let err_promote = build_and_publish_components(
        &provider,
        &spec,
        &dist_dir,
        Some(&wit_root),
        Some(BuildFailPoint::DuringActivePromote),
    )
    .unwrap_err();
    assert_eq!(
        err_promote,
        PublishError::InjectedFailPoint("DuringActivePromote".to_string())
    );

    // Active version must STILL be version 1
    let (active_ver_after, _) = recover_and_verify_dist_state(&dist_dir).unwrap().unwrap();
    assert_eq!(active_ver_after, summary1.version_id);

    let _ = fs::remove_dir_all(&dist_dir);
}
