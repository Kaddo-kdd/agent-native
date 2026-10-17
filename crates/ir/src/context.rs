//! Verification context — bookkeeping accumulated while walking an IR
//! document.
//!
//! Migrated verbatim from `_reference/poc-agent-native/crates/irtool/src/lib.rs`
//! (lines 75-96), per matriz de extracción C-01.
//!
//! The verifier (AN-006) populates this `Context` as it processes the
//! document and uses it to resolve type references, imports and function
//! calls.

use std::collections::{HashMap, HashSet};

use crate::model::{FuncSignature, ImportSignature, RecordDef, WitContractDef};

/// Mutable state carried through verification of a single IR document.
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// IR version string declared in the document (e.g. "0.5").
    pub ir_version: String,
    /// Optional WIT contract metadata, if present in the document.
    pub wit_contract: Option<WitContractDef>,
    /// Set of host capabilities granted to the module via its imports.
    pub capabilities: HashSet<String>,
    /// Import signatures resolved during verification, keyed by local name.
    pub imports: HashMap<String, ImportSignature>,
    /// Insertion order of imports — preserved for stable WASM lowering.
    pub import_order: Vec<String>,
    /// Record definitions, keyed by name.
    pub records: HashMap<String, RecordDef>,
    /// Function signatures, keyed by name.
    pub functions: HashMap<String, FuncSignature>,
    /// Number of imports accumulated so far — used to assign stable
    /// `ImportSignature::index` values during lowering.
    pub num_imports: u32,
}
