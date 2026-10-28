//! AN-015: policy by WIT interface and operation.
//!
//! Exercises the fixture produced by the AN-014 probe
//! (`crates/compiler/tests/fixtures/{ir,wit}/probe-use-types.*`): a
//! component that imports its own type interface via `use types.{pt}`.
//! Wasmtime surfaces this as a `ComponentInstance` import named
//! `an:probe/types@0.1.0`, indistinguishable at the type level from a host
//! capability import.
//!
//! These tests verify the AN-015 precisions:
//!   - Imports are identified by their REAL names.
//!   - Policy authorises by interface or operation, never by `"func"`/
//!     `"instance"` labels.
//!   - Allow-listing a specific operation does not authorise siblings.
//!   - Deny-by-default is intact — the probe fixture (which looks "internal"
//!     because it uses the same package) still requires explicit allow.

use std::path::PathBuf;
use wasma_compiler::{CompileContext, compile_ir_to_wasm_with_context};
use wasma_runtime::{Engine, HostPolicy, RuntimeError, Val};

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

/// Even though the probe component only imports its own package, deny-all
/// still rejects it — AN-015 precision 1: there is no automatic
/// "same-package = trusted" rule.
#[test]
fn same_package_type_import_is_not_implicitly_allowed() {
    let wasm = probe_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine.load_component(&wasm).expect("load");

    let err = component
        .invoke(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(1), Val::S32(2)],
        )
        .unwrap_err();
    match err {
        RuntimeError::UnauthorizedImport { module, name } => {
            assert_eq!(module, PROBE_TYPES_IFACE);
            // The surfaced `name` is the REAL operation / item from inside
            // the imported interface, never a kind label like "func" or
            // "instance" (precision 3).
            assert!(
                !matches!(name.as_str(), "func" | "instance"),
                "policy must report real identifiers, not kind labels (got '{name}')"
            );
        }
        other => panic!("expected UnauthorizedImport, got {other:?}"),
    }
}

/// `allow_wit_interface` autorises every operation of the given interface.
/// For the probe fixture — whose single import `an:probe/types@0.1.0` is a
/// **type-only** interface — authorisation is sufficient: Wasmtime resolves
/// the type reference internally without needing a host function.
/// Execution runs end-to-end.
///
/// This reinforces AN-015 precision 2: identification, authorisation and
/// resolution are three concerns. For type-only imports, resolution is a
/// no-op; for function imports, a separate provider is required
/// (INIT-06-01 / INIT-07-01).
#[test]
fn allow_wit_interface_authorises_whole_interface_and_type_import_resolves() {
    let mut policy = HostPolicy::deny_all();
    policy.allow_wit_interface(PROBE_TYPES_IFACE);

    let wasm = probe_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component_with_policy(&wasm, &policy)
        .expect("load");

    let results = component
        .invoke(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(1), Val::S32(2)],
        )
        .expect("type-only import plus allow: must execute");

    let Val::Record(fields) = &results[0] else {
        panic!("expected a record, got {:?}", results[0]);
    };
    assert_eq!(fields.len(), 2);
}

/// `allow_wit_operation` autorises exactly one operation inside an
/// interface. Other operations of the same interface remain blocked.
#[test]
fn allow_wit_operation_is_granular() {
    // The probe interface only exports `pt` (a Type). Allowing a bogus
    // operation name must NOT satisfy the gate.
    let mut policy = HostPolicy::deny_all();
    policy.allow_wit_operation(PROBE_TYPES_IFACE, "not-the-real-op");

    let wasm = probe_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component_with_policy(&wasm, &policy)
        .expect("load");

    let err = component
        .invoke(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(1), Val::S32(2)],
        )
        .unwrap_err();
    assert!(
        matches!(err, RuntimeError::UnauthorizedImport { .. }),
        "a bogus operation allow must not unlock the interface, got {err:?}"
    );
}

/// `allow_wit_operation` with the EXACT real operation name authorises
/// just that one. The probe's imported `types` interface only exposes
/// the type `pt`; authorising it is enough for execution to proceed.
#[test]
fn allow_wit_operation_with_real_name_executes_type_import() {
    let mut policy = HostPolicy::deny_all();
    policy.allow_wit_operation(PROBE_TYPES_IFACE, "pt");

    let wasm = probe_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component_with_policy(&wasm, &policy)
        .expect("load");

    let results = component
        .invoke(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(10), Val::S32(20)],
        )
        .expect("single-op allow for the type import: must execute");
    assert!(matches!(results.first(), Some(Val::Record(_))));
}

/// `allow_module` remains the permissive wildcard. Documented in the API
/// docstring, verified here: authorising the module alone satisfies the
/// gate for any import under it.
#[test]
fn allow_module_is_permissive_wildcard_and_documented() {
    let mut policy = HostPolicy::deny_all();
    policy.allow_module(PROBE_TYPES_IFACE);

    let wasm = probe_wasm();
    let engine = Engine::new().expect("engine");
    let component = engine
        .load_component_with_policy(&wasm, &policy)
        .expect("load");

    let results = component
        .invoke(
            &format!("{EXPORTED_OPS_IFACE}#make-pt"),
            &[Val::S32(42), Val::S32(43)],
        )
        .expect("allow_module shortcut must satisfy the gate");
    assert!(matches!(results.first(), Some(Val::Record(_))));
}

/// Positive query API: `is_wit_interface_allowed` and
/// `is_wit_operation_allowed` behave as expected.
#[test]
fn host_policy_inspection_api_reflects_grants() {
    let mut policy = HostPolicy::deny_all();
    assert!(!policy.is_wit_interface_allowed(PROBE_TYPES_IFACE));
    assert!(!policy.is_wit_operation_allowed(PROBE_TYPES_IFACE, "pt"));

    policy.allow_wit_operation(PROBE_TYPES_IFACE, "pt");
    assert!(!policy.is_wit_interface_allowed(PROBE_TYPES_IFACE));
    assert!(policy.is_wit_operation_allowed(PROBE_TYPES_IFACE, "pt"));
    assert!(!policy.is_wit_operation_allowed(PROBE_TYPES_IFACE, "other"));

    policy.allow_wit_interface("an:example/iface");
    assert!(policy.is_wit_interface_allowed("an:example/iface"));
    assert!(policy.is_wit_operation_allowed("an:example/iface", "whatever"));

    policy.allow_module("wasi:cli");
    assert!(policy.is_wit_interface_allowed("wasi:cli"));
    assert!(policy.is_allowed_core("wasi:cli", "stdout"));
}
