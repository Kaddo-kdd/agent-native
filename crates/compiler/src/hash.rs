//! Canonical SHA-256 digest for compiled WASM artifacts.
//!
//! Migrated from `_reference/poc-agent-native/crates/irtool/src/lib.rs`
//! (lines 2348-2358), per matriz de extracción C-01 and ADR-004 (hash as a
//! transversal minimal control).

use sha2::{Digest, Sha256};

/// Computes the lowercase hex SHA-256 digest of a byte slice.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().fold(String::with_capacity(64), |mut acc, b| {
        let _ = write!(acc, "{b:02x}");
        acc
    })
}
