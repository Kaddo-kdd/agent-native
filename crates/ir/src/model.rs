//! Core IR data types for Semantic IR v0.5.
//!
//! Migrated verbatim from `_reference/poc-agent-native/crates/irtool/src/lib.rs`
//! (lines 23-108), per matriz de extracción C-01. Semantics are preserved
//! 1:1 with the POC; any evolution to v0.6 (see ADR-003) happens under
//! an explicit flag in a dedicated Work Item.

use std::collections::HashSet;

/// Primitive and user-defined types supported by Semantic IR v0.5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
    /// 32-bit signed integer.
    I32,
    /// Boolean.
    Bool,
    /// A record type defined in the current IR document, identified by name.
    Record(String),
}

impl IrType {
    /// Returns the canonical textual representation of the type.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            IrType::I32 => "i32",
            IrType::Bool => "bool",
            IrType::Record(name) => name.as_str(),
        }
    }

    /// Parses a type name against the set of record types known to the
    /// current verification context. Returns `None` for unknown names.
    #[must_use]
    pub fn from_str(s: &str, known_records: &HashSet<String>) -> Option<Self> {
        match s {
            "i32" => Some(IrType::I32),
            "bool" => Some(IrType::Bool),
            other if known_records.contains(other) => Some(IrType::Record(other.to_string())),
            _ => None,
        }
    }
}

/// Signature of a host import consumed by an IR module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSignature {
    pub module: String,
    pub name: String,
    pub capability: String,
    pub wit_interface: Option<String>,
    pub params: Vec<(String, IrType)>,
    pub returns: IrType,
    pub index: u32,
}

/// A record (struct-like) type defined in the IR document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordDef {
    pub name: String,
    pub fields: Vec<(String, IrType)>,
}

/// WIT contract metadata declared by the IR document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WitContractDef {
    pub package: String,
    pub world: String,
    pub export_interface: String,
    pub import_interfaces: Vec<String>,
    pub wit_file: Option<String>,
}

/// Signature of a function declared in the IR document.
#[derive(Debug, Clone)]
pub struct FuncSignature {
    pub name: String,
    pub export: bool,
    pub wit_interface: Option<String>,
    pub params: Vec<(String, IrType)>,
    pub returns: IrType,
    pub index: u32,
}
