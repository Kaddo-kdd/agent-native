//! AN-033: Declarative capability manifests and intersection policy tests.

use agent_native_compiler::{CompileContext, compile_ir_to_wasm_with_context};
use agent_native_runtime::{ComponentManifest, Engine, HostPolicy, PolicyError, RuntimeError, Val};
use std::path::PathBuf;

fn compiler_crate_ctx() -> CompileContext {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.push("compiler");
    CompileContext::new(root)
}

fn probe_wasm() -> Vec<u8> {
    let text = include_str!("../../compiler/tests/fixtures/ir/probe-use-types.ir.json");
    let ir: serde_json::Value = serde_json::from_str(text).expect("valid JSON");
    compile_ir_to_wasm_with_context(&ir, &compiler_crate_ctx()).expect("compile must succeed")
}

const PROBE_TYPES_IFACE: &str = "an:probe/types@0.1.0";
const EXPORTED_OPS_IFACE: &str = "an:probe/ops@0.1.0";

#[test]
fn test_parse_valid_yaml_and_json_manifests() {
    let yaml = r#"
version: "0.1"
name: "test-component"
capabilities:
  wit_interfaces:
    - "an:probe/types@0.1.0"
  wit_operations:
    - interface: "wasi:cli/environment@0.2.0"
      operation: "get-environment"
  core_imports:
    - module: "host"
      name: "log"
"#;
    let manifest_yaml = ComponentManifest::from_yaml_str(yaml).expect("valid yaml");
    assert_eq!(manifest_yaml.version, "0.1");
    assert_eq!(manifest_yaml.name.as_deref(), Some("test-component"));
    assert_eq!(manifest_yaml.capabilities.wit_interfaces.len(), 1);
    assert_eq!(manifest_yaml.capabilities.wit_operations.len(), 1);
    assert_eq!(manifest_yaml.capabilities.core_imports.len(), 1);

    let json = r#"{
  "version": "0.1",
  "name": "test-json",
  "capabilities": {
    "wit_interfaces": ["an:probe/types@0.1.0"],
    "wit_operations": [],
    "core_imports": []
  }
}"#;
    let manifest_json = ComponentManifest::from_json_str(json).expect("valid json");
    assert_eq!(manifest_json.version, "0.1");
    assert_eq!(manifest_json.name.as_deref(), Some("test-json"));
}

#[test]
fn test_manifest_rejects_unsupported_version() {
    let yaml = r#"
version: "0.2"
capabilities:
  wit_interfaces: []
"#;
    let err = ComponentManifest::from_yaml_str(yaml).unwrap_err();
    assert!(
        matches!(err, PolicyError::UnsupportedVersion(ref v) if v == "0.2"),
        "expected UnsupportedVersion, got {err:?}"
    );
}

#[test]
fn test_manifest_rejects_unknown_fields() {
    let yaml = r#"
version: "0.1"
unexpected_field: "should_fail"
capabilities:
  wit_interfaces: []
"#;
    let err = ComponentManifest::from_yaml_str(yaml).unwrap_err();
    assert!(
        matches!(err, PolicyError::ParseError(_)),
        "expected ParseError on unknown field, got {err:?}"
    );
}

#[test]
fn test_manifest_rejects_wildcards() {
    let yaml_iface = r#"
version: "0.1"
capabilities:
  wit_interfaces:
    - "an:probe/*"
"#;
    let err = ComponentManifest::from_yaml_str(yaml_iface).unwrap_err();
    assert!(
        matches!(err, PolicyError::WildcardForbidden(_)),
        "expected WildcardForbidden, got {err:?}"
    );

    let yaml_core = r#"
version: "0.1"
capabilities:
  core_imports:
    - module: "host"
      name: "*"
"#;
    let err = ComponentManifest::from_yaml_str(yaml_core).unwrap_err();
    assert!(
        matches!(err, PolicyError::WildcardForbidden(_)),
        "expected WildcardForbidden, got {err:?}"
    );
}

#[test]
fn test_manifest_rejects_duplicates() {
    let yaml = r#"
version: "0.1"
capabilities:
  wit_interfaces:
    - "an:probe/types@0.1.0"
    - "an:probe/types@0.1.0"
"#;
    let err = ComponentManifest::from_yaml_str(yaml).unwrap_err();
    assert!(
        matches!(err, PolicyError::DuplicateDeclaration(_)),
        "expected DuplicateDeclaration, got {err:?}"
    );
}

#[test]
fn test_effective_policy_intersection_logic() {
    // Component requests A and B
    let manifest_yaml = r#"
version: "0.1"
capabilities:
  wit_interfaces:
    - "an:probe/types@0.1.0"
    - "an:forbidden/iface@0.1.0"
  core_imports:
    - module: "host"
      name: "kv_get"
    - module: "host"
      name: "admin_override"
"#;
    let requested = ComponentManifest::from_yaml_str(manifest_yaml).unwrap();

    // Host grants A and C (never grants forbidden or admin_override)
    let mut granted = HostPolicy::deny_all();
    granted.allow_wit_interface("an:probe/types@0.1.0");
    granted.allow_wit_interface("an:unrequested/iface@0.1.0");
    granted.allow_core("host", "kv_get");
    granted.allow_core("host", "unrequested_call");

    let effective = requested.effective_policy(&granted);

    // Rule 1: A is both requested and granted -> allowed
    assert!(effective.is_wit_interface_allowed("an:probe/types@0.1.0"));
    assert!(effective.is_allowed_core("host", "kv_get"));

    // Rule 2: B is requested but NOT granted -> denied
    assert!(!effective.is_wit_interface_allowed("an:forbidden/iface@0.1.0"));
    assert!(!effective.is_allowed_core("host", "admin_override"));

    // Rule 3: C is granted but NOT requested -> denied (least privilege)
    assert!(!effective.is_wit_interface_allowed("an:unrequested/iface@0.1.0"));
    assert!(!effective.is_allowed_core("host", "unrequested_call"));
}

#[test]
fn test_load_time_validation_with_manifest() {
    let wasm = probe_wasm();
    let engine = Engine::new().expect("engine");

    // Case 1: Host grants it, but component manifest does not request it -> Fails at load time!
    let empty_manifest = ComponentManifest::from_yaml_str(
        r#"
version: "0.1"
capabilities:
  wit_interfaces: []
"#,
    )
    .unwrap();

    let mut host_policy = HostPolicy::deny_all();
    host_policy.allow_wit_interface(PROBE_TYPES_IFACE);

    let err = engine
        .load_component_with_manifest(&wasm, &empty_manifest, &host_policy)
        .expect_err("must fail at load time because component didn't request the import");
    assert!(
        matches!(err, RuntimeError::UnauthorizedImport { ref module, .. } if module == PROBE_TYPES_IFACE),
        "expected UnauthorizedImport, got {err:?}"
    );

    // Case 2: Component requests it, and host grants it -> Succeeds at load time and executes!
    let valid_manifest = ComponentManifest::from_yaml_str(&format!(
        r#"
version: "0.1"
capabilities:
  wit_interfaces:
    - "{PROBE_TYPES_IFACE}"
"#
    ))
    .unwrap();

    let component = engine
        .load_component_with_manifest(&wasm, &valid_manifest, &host_policy)
        .expect("must succeed at load time");

    let res = component
        .invoke(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(10), Val::S32(20)],
        )
        .expect("invocation must succeed");
    let Val::Record(fields) = &res[0] else {
        panic!("expected record");
    };
    assert_eq!(fields.len(), 2);
}

#[test]
fn test_cross_granularity_wit_intersection_interface_requested_operation_granted() {
    let iface = "an:service/api@0.1.0";
    let manifest_yaml = format!(
        r#"
version: "0.1"
capabilities:
  wit_interfaces:
    - "{iface}"
"#
    );
    let requested = ComponentManifest::from_yaml_str(&manifest_yaml).unwrap();

    // Host policy does NOT grant the entire interface, only a specific operation
    let mut granted = HostPolicy::deny_all();
    granted.allow_wit_operation(iface, "allowed_op");

    let effective = requested.effective_policy(&granted);

    // Cross-granularity: The intersection retains the granted operation
    assert!(effective.is_wit_operation_allowed(iface, "allowed_op"));

    // But operations that the host did not grant are denied
    assert!(!effective.is_wit_operation_allowed(iface, "unauthorized_op"));

    // And the interface as a whole is not unconditionally allowed
    assert!(!effective.is_wit_interface_allowed(iface));
}

#[test]
fn test_programmatic_manifest_validation_enforced_at_runtime_entry_points() {
    use agent_native_runtime::ManifestCapabilities;

    let wasm = probe_wasm();
    let engine = Engine::new().expect("engine");

    // Construct an invalid manifest directly in Rust memory (bypassing parser)
    let bad_manifest = ComponentManifest {
        version: "99.0".to_string(), // Invalid version
        name: Some("in-memory-bad".to_string()),
        capabilities: ManifestCapabilities::default(),
    };

    let policy = HostPolicy::deny_all();

    // 1. Engine load-time entry point must validate in-memory manifest
    let err_engine = engine
        .load_component_with_manifest(&wasm, &bad_manifest, &policy)
        .expect_err("must validate programmatic manifest");
    assert!(
        matches!(
            err_engine,
            RuntimeError::Policy(PolicyError::UnsupportedVersion(ref v)) if v == "99.0"
        ),
        "expected UnsupportedVersion, got {err_engine:?}"
    );

    // 2. HostPolicy::from_manifest must validate in-memory manifest
    let err_policy =
        HostPolicy::from_manifest(&bad_manifest).expect_err("must validate in-memory manifest");
    assert!(
        matches!(
            err_policy,
            PolicyError::UnsupportedVersion(ref v) if v == "99.0"
        ),
        "expected UnsupportedVersion, got {err_policy:?}"
    );
}
