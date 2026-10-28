//! Inventory guard for the real Rust `wasm32-wasip2` fixture (AN-050).
//!
//! This test deliberately does not instantiate the fixture. It records the
//! imports that must be resolved or explicitly rejected before command support
//! can be enabled.

use std::collections::BTreeSet;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use wasmtime::component::Component;

use wasma_runtime::{Engine, ExecutionLimits, HostPolicy, WasiIoConfig};

const HELLO: &[u8] = include_bytes!("fixtures/wasip2_hello.wasm");
const EXIT_SEVEN: &[u8] = include_bytes!("fixtures/wasip2_exit7.wasm");
const SOURCE: &[u8] = include_bytes!("fixtures/wasip2_hello.rs");
const EXIT_SOURCE: &[u8] = include_bytes!("fixtures/wasip2_exit7.rs");
const MANIFEST: &str = include_str!("fixtures/wasip2_hello.json");
const EXIT_MANIFEST: &str = include_str!("fixtures/wasip2_exit7.json");

#[derive(Debug, Deserialize)]
struct FixtureManifest {
    source_sha256: String,
    artifact_sha256: String,
    artifact_bytes: usize,
    #[serde(default)]
    imports: Vec<String>,
    #[serde(default)]
    exports: Vec<String>,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// The manifests pin UTF-8 source content with LF line endings. Git may check
/// those source files out with CRLF on Windows, which must not change their
/// content identity.
fn source_sha256(bytes: &[u8]) -> String {
    sha256(
        String::from_utf8_lossy(bytes)
            .replace("\r\n", "\n")
            .as_bytes(),
    )
}

fn command_policy_without_stdout() -> HostPolicy {
    let mut policy = HostPolicy::deny_all();
    for interface in [
        "wasi:cli/environment@0.2.0",
        "wasi:cli/exit@0.2.0",
        "wasi:cli/stderr@0.2.0",
        "wasi:cli/stdin@0.2.0",
        "wasi:clocks/wall-clock@0.2.0",
        "wasi:filesystem/preopens@0.2.0",
        "wasi:filesystem/types@0.2.0",
        "wasi:io/error@0.2.0",
        "wasi:io/streams@0.2.0",
    ] {
        policy.allow_wit_interface(interface);
    }
    policy
}

#[test]
fn real_wasip2_hello_has_the_reviewed_import_inventory() {
    let manifest: FixtureManifest = serde_json::from_str(MANIFEST).expect("valid fixture manifest");
    assert_eq!(source_sha256(SOURCE), manifest.source_sha256);
    assert_eq!(sha256(HELLO), manifest.artifact_sha256);
    assert_eq!(HELLO.len(), manifest.artifact_bytes);

    let engine = wasmtime::Engine::default();
    let component = Component::from_binary(&engine, HELLO).expect("fixture is a component");
    let imports = component
        .component_type()
        .imports(&engine)
        .map(|(name, _)| name.to_string())
        .collect::<BTreeSet<_>>();

    let expected = manifest.imports.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(imports, expected);

    let exports = component
        .component_type()
        .exports(&engine)
        .map(|(name, _)| name.to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        exports,
        manifest.exports.into_iter().collect::<BTreeSet<_>>()
    );
}

#[test]
fn real_wasip2_command_runs_with_an_empty_filesystem_and_fixed_clock() {
    let engine = Engine::new().expect("engine");
    let mut policy = HostPolicy::deny_all();
    policy.allow_wasi_command_empty_fs();
    let component = engine
        .load_component_enforcing_policy(HELLO, &policy)
        .expect("exact command interfaces are allowed");

    let report = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "wasi:cli/run@0.2.0#run",
            &[],
            WasiIoConfig::capturing(),
        )
        .expect("hello command runs");

    assert_eq!(
        report.io.stdout_utf8().expect("utf-8 stdout"),
        "hello agent-native\n"
    );
    assert!(report.io.stderr.is_empty());
}

#[test]
fn real_wasip2_command_rejects_stdout_before_instantiation() {
    let engine = Engine::new().expect("engine");
    let error = engine
        .load_component_enforcing_policy(HELLO, &command_policy_without_stdout())
        .expect_err("stdout must be denied before instantiation");

    assert_eq!(
        error,
        wasma_runtime::RuntimeError::UnauthorizedImport {
            module: "wasi:cli/stdout@0.2.0".to_string(),
            name: "output-stream".to_string(),
        }
    );
}

#[test]
fn real_wasip2_command_discards_every_calls_io_state() {
    let engine = Engine::new().expect("engine");
    let mut policy = HostPolicy::deny_all();
    policy.allow_wasi_command_empty_fs();
    let component = engine
        .load_component_enforcing_policy(HELLO, &policy)
        .expect("exact command interfaces are allowed");

    for _ in 0..2 {
        let report = component
            .invoke_with_wasi(
                &policy,
                &ExecutionLimits::unmetered(),
                "wasi:cli/run@0.2.0#run",
                &[],
                WasiIoConfig::capturing(),
            )
            .expect("hello command runs");
        assert_eq!(
            report.io.stdout_utf8().expect("utf-8 stdout"),
            "hello agent-native\n"
        );
        assert!(report.io.stderr.is_empty());
    }
}

#[test]
fn real_wasip2_command_maps_a_nonzero_exit_to_a_typed_error() {
    let manifest: FixtureManifest =
        serde_json::from_str(EXIT_MANIFEST).expect("valid nonzero-exit fixture manifest");
    assert_eq!(source_sha256(EXIT_SOURCE), manifest.source_sha256);
    assert_eq!(sha256(EXIT_SEVEN), manifest.artifact_sha256);
    assert_eq!(EXIT_SEVEN.len(), manifest.artifact_bytes);

    let engine = Engine::new().expect("engine");
    let mut policy = HostPolicy::deny_all();
    policy.allow_wasi_command_empty_fs();
    let component = engine
        .load_component_enforcing_policy(EXIT_SEVEN, &policy)
        .expect("exact command interfaces are allowed");

    let error = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "wasi:cli/run@0.2.0#run",
            &[],
            WasiIoConfig::capturing(),
        )
        .expect_err("nonzero command exit must fail");
    assert_eq!(
        error,
        wasma_runtime::RuntimeError::WasiCommandExited { code: 1 }
    );
}
