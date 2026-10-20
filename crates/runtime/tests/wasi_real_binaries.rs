//! Inventory guard for the real Rust `wasm32-wasip2` fixture (AN-050).
//!
//! This test deliberately does not instantiate the fixture. It records the
//! imports that must be resolved or explicitly rejected before command support
//! can be enabled.

use std::collections::BTreeSet;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use wasmtime::component::Component;

const HELLO: &[u8] = include_bytes!("fixtures/wasip2_hello.wasm");
const SOURCE: &[u8] = include_bytes!("fixtures/wasip2_hello.rs");
const MANIFEST: &str = include_str!("fixtures/wasip2_hello.json");

#[derive(Debug, Deserialize)]
struct FixtureManifest {
    source_sha256: String,
    artifact_sha256: String,
    artifact_bytes: usize,
    imports: Vec<String>,
    exports: Vec<String>,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn real_wasip2_hello_has_the_reviewed_import_inventory() {
    let manifest: FixtureManifest = serde_json::from_str(MANIFEST).expect("valid fixture manifest");
    assert_eq!(sha256(SOURCE), manifest.source_sha256);
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
    assert_eq!(exports, manifest.exports.into_iter().collect::<BTreeSet<_>>());
}
