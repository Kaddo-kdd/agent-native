//! Declarative capability manifests for components.
//!
//! # Untrusted Component Manifests vs. Authoritative Host Policies
//!
//! A [`ComponentManifest`] is provided alongside or embedded inside a WASM
//! artifact. Per RM-06 governance, **a component manifest is untrusted metadata**.
//! It is strictly a declarative statement of requested capabilities, **not**
//! a proof of provenance or cryptographic integrity (verification of binary
//! integrity and release provenance is governed separately under INIT-06-04).
//!
//! A component cannot grant permissions to itself.
//!
//! The authoritative source of permissions is the [`crate::HostPolicy`] configured
//! by the host operator. The runtime enforces:
//!
//! \[ \forall i \in \text{Imports}(WASM): i \in \text{Requested} \land i \in \text{Granted} \]
//!
//! If any declared import of the WASM binary fails to satisfy both conditions,
//! execution is rejected pre-instantiate with [`crate::RuntimeError::UnauthorizedImport`].

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::error::PolicyError;
use crate::policy::HostPolicy;

/// Supported schema version for capability manifests.
pub const CURRENT_MANIFEST_VERSION: &str = "0.1";

/// Declarative manifest specifying requested capabilities for a component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentManifest {
    /// Schema version (must be "0.1").
    pub version: String,

    /// Optional component name or identifier.
    #[serde(default)]
    pub name: Option<String>,

    /// Requested capabilities section.
    pub capabilities: ManifestCapabilities,
}

/// The set of capabilities declared by a component.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestCapabilities {
    /// Full WIT interfaces requested (e.g. `"an:test/ops@0.1.0"`).
    #[serde(default)]
    pub wit_interfaces: Vec<String>,

    /// Granular WIT operations requested inside an interface.
    #[serde(default)]
    pub wit_operations: Vec<WitOperation>,

    /// Core WASM (module, name) pairs requested.
    #[serde(default)]
    pub core_imports: Vec<CoreImport>,
}

/// Granular operation inside a WIT interface.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WitOperation {
    /// Interface identifier (e.g. `"wasi:cli/environment@0.2.0"`).
    pub interface: String,
    /// Function/operation name (e.g. `"get-environment"`).
    pub operation: String,
}

/// Core WASM import specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreImport {
    /// Module name (e.g. `"host"`).
    pub module: String,
    /// Function or symbol name (e.g. `"log"`).
    pub name: String,
}

impl ComponentManifest {
    /// Parses a manifest from a YAML string, enforcing strict schema validation.
    pub fn from_yaml_str(yaml_str: &str) -> Result<Self, PolicyError> {
        let manifest: Self =
            serde_yaml2::from_str(yaml_str).map_err(|e| PolicyError::ParseError(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Parses a manifest from a JSON string, enforcing strict schema validation.
    pub fn from_json_str(json_str: &str) -> Result<Self, PolicyError> {
        let manifest: Self =
            serde_json::from_str(json_str).map_err(|e| PolicyError::ParseError(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validates version, identifiers, duplicate declarations, and wildcard restrictions.
    pub fn validate(&self) -> Result<(), PolicyError> {
        if self.version != CURRENT_MANIFEST_VERSION {
            return Err(PolicyError::UnsupportedVersion(self.version.clone()));
        }

        let mut seen_interfaces = HashSet::new();
        for iface in &self.capabilities.wit_interfaces {
            validate_wit_identifier(iface)?;
            if !seen_interfaces.insert(iface) {
                return Err(PolicyError::DuplicateDeclaration(format!(
                    "duplicate wit_interface '{iface}'"
                )));
            }
        }

        let mut seen_operations = HashSet::new();
        for op in &self.capabilities.wit_operations {
            validate_wit_identifier(&op.interface)?;
            validate_operation_name(&op.operation)?;
            if !seen_operations.insert((&op.interface, &op.operation)) {
                return Err(PolicyError::DuplicateDeclaration(format!(
                    "duplicate wit_operation '{}:{}'",
                    op.interface, op.operation
                )));
            }
        }

        let mut seen_core = HashSet::new();
        for core in &self.capabilities.core_imports {
            validate_core_identifier(&core.module, &core.name)?;
            if !seen_core.insert((&core.module, &core.name)) {
                return Err(PolicyError::DuplicateDeclaration(format!(
                    "duplicate core_import '{}:{}'",
                    core.module, core.name
                )));
            }
        }

        Ok(())
    }

    /// Computes the effective [`HostPolicy`] by intersecting this untrusted
    /// request with the authoritative granted [`HostPolicy`].
    ///
    /// An import is present in the resulting policy if and only if:
    /// 1. It was explicitly requested in this manifest (`Requested`), AND
    /// 2. It was explicitly permitted by the host policy (`Granted`).
    #[must_use]
    pub fn effective_policy(&self, granted: &HostPolicy) -> HostPolicy {
        let mut effective = HostPolicy::deny_all();

        for iface in &self.capabilities.wit_interfaces {
            if granted.is_wit_interface_allowed(iface) {
                effective.allow_wit_interface(iface);
            } else {
                // Cross-granularity intersection:
                // When the component requests an entire interface and the host
                // only grants specific operations within that interface, the
                // intersection retains those granted operations.
                for op in granted.allowed_wit_operations_for_interface(iface) {
                    effective.allow_wit_operation(iface, op);
                }
            }
        }

        for op in &self.capabilities.wit_operations {
            if granted.is_wit_operation_allowed(&op.interface, &op.operation) {
                effective.allow_wit_operation(&op.interface, &op.operation);
            }
        }

        for core in &self.capabilities.core_imports {
            if granted.is_allowed_core(&core.module, &core.name) {
                effective.allow_core(&core.module, &core.name);
            }
        }

        effective
    }
}

fn validate_wit_identifier(id: &str) -> Result<(), PolicyError> {
    let trimmed = id.trim();
    if trimmed.is_empty() || trimmed != id {
        return Err(PolicyError::InvalidIdentifier(format!(
            "wit identifier cannot be empty or contain whitespace: '{id}'"
        )));
    }
    if id.contains('*') {
        return Err(PolicyError::WildcardForbidden(id.to_string()));
    }
    // A valid WIT identifier conforms to namespace:package/interface[@version]
    if !id.contains(':') || !id.contains('/') {
        return Err(PolicyError::InvalidIdentifier(format!(
            "wit identifier must contain ':' and '/': '{id}'"
        )));
    }
    Ok(())
}

fn validate_operation_name(name: &str) -> Result<(), PolicyError> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed != name {
        return Err(PolicyError::InvalidIdentifier(format!(
            "operation name cannot be empty or contain whitespace: '{name}'"
        )));
    }
    if name.contains('*') {
        return Err(PolicyError::WildcardForbidden(name.to_string()));
    }
    Ok(())
}

fn validate_core_identifier(module: &str, name: &str) -> Result<(), PolicyError> {
    let mod_trimmed = module.trim();
    let name_trimmed = name.trim();
    if mod_trimmed.is_empty() || mod_trimmed != module {
        return Err(PolicyError::InvalidIdentifier(format!(
            "core module cannot be empty or contain whitespace: '{module}'"
        )));
    }
    if name_trimmed.is_empty() || name_trimmed != name {
        return Err(PolicyError::InvalidIdentifier(format!(
            "core import name cannot be empty or contain whitespace: '{name}'"
        )));
    }
    if module.contains('*') || name.contains('*') {
        return Err(PolicyError::WildcardForbidden(format!("{module}::{name}")));
    }
    Ok(())
}
