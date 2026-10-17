//! Diagnostics produced by the Semantic IR verifier.
//!
//! Each diagnostic carries a stable `code` (e.g. `E_SCHEMA_INVALID`), the
//! JSON path where it was raised and a human-readable `message`. The
//! optional `expected` and `found` fields narrate type mismatches without
//! coupling the verifier to a specific error-rendering strategy.
//!
//! Migrated verbatim from `_reference/poc-agent-native/crates/irtool/src/lib.rs`
//! (lines 11-21), per matriz de extracción C-01.

use serde::{Deserialize, Serialize};

/// A single diagnostic produced while verifying a Semantic IR document.
///
/// An empty `Vec<Diagnostic>` returned by the verifier means the document is
/// both structurally and semantically valid under the active IR schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Stable machine-readable code, e.g. `E_SCHEMA_INVALID`.
    pub code: String,
    /// Function name the diagnostic belongs to, when applicable.
    pub function: Option<String>,
    /// JSON path (dotted) where the diagnostic was raised.
    pub path: String,
    /// Human-readable message describing the issue.
    pub message: String,
    /// Expected type or value, when the diagnostic reports a mismatch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    /// Actual type or value found, when the diagnostic reports a mismatch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub found: Option<String>,
}
