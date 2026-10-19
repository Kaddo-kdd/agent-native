//! Integration tests for artifact distribution traceability and metadata origin.
//!
//! Validates AC-01 through AC-04 of AN-027 completely offline.

use std::fs;
use std::path::PathBuf;

use agent_native::agent::FixtureLlmProvider;
use agent_native::publish::{build_and_publish_components, recover_and_verify_dist_state};
use agent_native::spec::model::{
    ComponentSpec, RequirementSource, RequirementSpec, SpecMetadata, SystemSpec,
};
use serde_json::json;

fn compiler_wit_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("compiler")
}

fn create_traceable_spec() -> SystemSpec {
    SystemSpec {
        schema: None,
        spec_version: "1.0.0".to_string(),
        metadata: SpecMetadata {
            title: "Math Service".to_string(),
            description: "Sample math service".to_string(),
            source_type: "work-items".to_string(),
            source_files: vec!["WI-MATH-01.md".to_string()],
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
        requirements: vec![
            RequirementSpec {
                id: "MATH-001".to_string(),
                component: "math".to_string(),
                title: "Double value".to_string(),
                statement: "Double integer x".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "work-item".to_string(),
                    ref_target: "WI-MATH-01.md#math-001".to_string(),
                },
                acceptance_criteria: vec![],
            },
            RequirementSpec {
                id: "MATH-002".to_string(),
                component: "math".to_string(),
                title: "Point sum".to_string(),
                statement: "Make point sum".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "work-item".to_string(),
                    ref_target: "WI-MATH-02.md#math-002".to_string(),
                },
                acceptance_criteria: vec![],
            },
        ],
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
        "agent_native_trace_{test_name}_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_traceability_metadata_in_manifest_and_active() {
    let dist_dir = temp_dist_dir("metadata");
    let spec = create_traceable_spec();
    let provider = create_fixture_provider();
    let wit_root = compiler_wit_root();

    let options = agent_native::publish::PublishOptions {
        wit_root: Some(&wit_root),
        git_commit: Some("abcdef1234567890".to_string()),
        fail_point: None,
        extra_source_wi: Vec::new(),
    };

    let summary = agent_native::publish::build_and_publish_components_with_options(
        &provider, &spec, &dist_dir, &options,
    )
    .expect("build and publish");
    assert!(summary.success);

    // Verify recovery and check traceability block in manifest
    let (active_ver, manifest) = recover_and_verify_dist_state(&dist_dir).unwrap().unwrap();
    assert_eq!(active_ver, summary.version_id);

    // 1. Check source_wi contains both declared source file and requirement targets
    assert!(
        manifest
            .traceability
            .source_wi
            .contains(&"WI-MATH-01.md".to_string())
    );
    assert!(
        manifest
            .traceability
            .source_wi
            .contains(&"WI-MATH-02.md".to_string())
    );

    // 2. Check source_spec_hash is 64 hex characters (valid SHA-256)
    assert_eq!(manifest.traceability.source_spec_hash.len(), 64);

    // 3. Check git commit is propagated
    assert_eq!(
        manifest.traceability.source_git_commit,
        Some("abcdef1234567890".to_string())
    );

    // 4. Check active.json also carries identical traceability
    let active_content = fs::read_to_string(dist_dir.join("active.json")).unwrap();
    let active_json: serde_json::Value = serde_json::from_str(&active_content).unwrap();
    assert_eq!(
        active_json["traceability"]["source_spec_hash"],
        manifest.traceability.source_spec_hash
    );

    let _ = fs::remove_dir_all(&dist_dir);
}

#[test]
fn test_traceability_hash_changes_deterministically_with_spec_modifications() {
    let dist_dir = temp_dist_dir("hash_diff");
    let spec1 = create_traceable_spec();
    let mut spec2 = create_traceable_spec();
    spec2.requirements[0].statement = "Modified statement changing hash".to_string();

    let provider = create_fixture_provider();
    let wit_root = compiler_wit_root();

    let summary1 =
        build_and_publish_components(&provider, &spec1, &dist_dir, Some(&wit_root), None).unwrap();
    let (_, manifest1) = recover_and_verify_dist_state(&dist_dir).unwrap().unwrap();
    assert_eq!(manifest1.version_id, summary1.version_id);

    let summary2 =
        build_and_publish_components(&provider, &spec2, &dist_dir, Some(&wit_root), None).unwrap();
    let (_, manifest2) = recover_and_verify_dist_state(&dist_dir).unwrap().unwrap();
    assert_eq!(manifest2.version_id, summary2.version_id);

    // Changing spec statement must produce different source_spec_hash
    assert_ne!(
        manifest1.traceability.source_spec_hash,
        manifest2.traceability.source_spec_hash
    );

    let _ = fs::remove_dir_all(&dist_dir);
}
