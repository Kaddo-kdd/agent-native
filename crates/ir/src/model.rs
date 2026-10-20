//! Core IR data types for Semantic IR v0.5.
//!
//! Migrated verbatim from `_reference/poc-agent-native/crates/irtool/src/lib.rs`
//! (lines 23-108), per matriz de extracción C-01. Semantics are preserved
//! 1:1 with the POC; any evolution to v0.6 (see ADR-003) happens under
//! an explicit flag in a dedicated Work Item.

use std::collections::HashSet;

/// Primitive and user-defined types supported by Semantic IR (v0.5 baseline + v0.6 experimental).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
    /// 32-bit signed integer.
    I32,
    /// Boolean.
    Bool,
    /// UTF-8 string (Semantic IR v0.6, experimental).
    String,
    /// Typed result `result<ok, err>` (Semantic IR v0.6, experimental).
    Result {
        ok: Box<IrType>,
        err: Box<IrType>,
        repr: String,
    },
    /// A record type defined in the current IR document, identified by name.
    Record(String),
}

impl IrType {
    /// Constructs a canonical `result<ok, err>` IR type.
    #[must_use]
    pub fn result(ok: IrType, err: IrType) -> Self {
        let repr = format!("result<{}, {}>", ok.as_str(), err.as_str());
        Self::Result {
            ok: Box::new(ok),
            err: Box::new(err),
            repr,
        }
    }

    /// Returns the canonical textual representation of the type.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            IrType::I32 => "i32",
            IrType::Bool => "bool",
            IrType::String => "string",
            IrType::Result { repr, .. } => repr.as_str(),
            IrType::Record(name) => name.as_str(),
        }
    }

    /// Parses a type name against the set of record types known to the
    /// current v0.5 verification context. Returns `None` for unknown names.
    #[must_use]
    pub fn from_str(s: &str, known_records: &HashSet<String>) -> Option<Self> {
        Self::from_str_for_version(s, known_records, "0.5")
    }

    /// Parses a type name for a specific `ir_version`.
    ///
    /// - In `v0.1..=v0.5`, only `i32`, `bool`, and declared record names are accepted.
    /// - In `v0.6` (`--experimental`), `string` and `result<ok, err>` are also accepted.
    #[must_use]
    pub fn from_str_for_version(
        s: &str,
        known_records: &HashSet<String>,
        ir_version: &str,
    ) -> Option<Self> {
        let trimmed = s.trim();
        match trimmed {
            "i32" => Some(IrType::I32),
            "bool" => Some(IrType::Bool),
            "string" if ir_version == "0.6" => Some(IrType::String),
            other
                if ir_version == "0.6" && other.starts_with("result<") && other.ends_with('>') =>
            {
                let inner = &other["result<".len()..other.len() - 1];
                let (ok_raw, err_raw) = inner.split_once(',')?;
                let ok_ty = Self::parse_v06_payload_type(ok_raw.trim(), known_records)?;
                let err_ty = Self::parse_v06_payload_type(err_raw.trim(), known_records)?;
                Some(IrType::result(ok_ty, err_ty))
            }
            other if known_records.contains(other) => Some(IrType::Record(other.to_string())),
            _ => None,
        }
    }

    fn parse_v06_payload_type(s: &str, known_records: &HashSet<String>) -> Option<Self> {
        match s {
            "i32" => Some(IrType::I32),
            "bool" => Some(IrType::Bool),
            "string" => Some(IrType::String),
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
