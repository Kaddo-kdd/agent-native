//! Declarative host-import policy.
//!
//! A [`HostPolicy`] carries an explicit allow-list of imports the runtime is
//! permitted to resolve. The default state is **deny-all** (ADR-004): any
//! declared import of the component is rejected unless the caller has
//! explicitly allowed it.
//!
//! # Three concerns, three APIs
//!
//! Per AN-015, authorisation is strictly separated from identification and
//! resolution:
//!
//! - **Identification** is Wasmtime's job: it exposes imports as
//!   `(name, ComponentItem)` tuples where `name` is the real WIT identifier
//!   (e.g. `an:test/ops@0.1.0`) or Core WASM pair (`module, name`).
//! - **Authorisation** is this module's job: the policy decides whether a
//!   given import is allowed.
//! - **Resolution** is not this module's job: allowing an import does not
//!   provide an implementation. Allowed-but-unprovided imports fail
//!   downstream with [`crate::RuntimeError::InstantiationFailed`] (see
//!   INIT-06-01 and INIT-07-01 for providers).
//!
//! # Three granularities
//!
//! - [`HostPolicy::allow_core`] authorises a single Core WASM
//!   `(module, name)` import.
//! - [`HostPolicy::allow_wit_interface`] authorises every operation of a
//!   WIT interface by its real identifier (e.g. `an:test/ops@0.1.0`).
//! - [`HostPolicy::allow_wit_operation`] authorises a single operation
//!   inside a WIT interface, identified by `(interface, operation)` using
//!   the real names Wasmtime exposes.
//!
//! The policy **never** stores or compares generic item-kind labels
//! (`"func"`, `"instance"`, …) — only real identifiers.
//!
//! # Compatibility shims
//!
//! - [`HostPolicy::allow`] is kept for backwards compatibility with
//!   AN-010/011 call sites. It is interpreted as [`HostPolicy::allow_core`]
//!   because its original intent was Core WASM imports.
//! - [`HostPolicy::allow_module`] is kept as a permissive wildcard across
//!   both Core WASM and Component Model. Its scope is broad: authorising
//!   `allow_module("host")` lets any symbol under `host` through. Prefer
//!   the granular methods unless there is a reason not to.

use std::collections::HashSet;

use crate::error::PolicyError;
use crate::manifest::ComponentManifest;

/// A declarative policy over host imports.
///
/// Consulted pre-instantiate for every declared import of a loaded artifact.
/// An import is allowed when it matches any of the configured allow-lists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostPolicy {
    allow_core_pairs: HashSet<(String, String)>,
    allow_modules: HashSet<String>,
    allow_wit_interfaces: HashSet<String>,
    allow_wit_operations: HashSet<(String, String)>,
}

impl HostPolicy {
    /// Creates a policy that denies every host import.
    ///
    /// Backwards-compatible default used by [`crate::Engine::load_component`].
    #[must_use]
    pub fn deny_all() -> Self {
        Self::default()
    }

    /// Constructs an authoritative [`HostPolicy`] from an authoritative capability manifest.
    ///
    /// # Security Warning
    ///
    /// This method grants all capabilities declared in the manifest as an authoritative
    /// host policy. It **MUST ONLY** be called with manifests supplied by an authoritative
    /// host operator / configuration. It **MUST NEVER** be invoked directly on an untrusted
    /// [`ComponentManifest`] supplied by a guest component, as doing so would allow
    /// self-granting of privileges.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError`] if the manifest fails validation (unsupported version,
    /// malformed identifiers, wildcards, or duplicates).
    pub fn from_manifest(manifest: &ComponentManifest) -> Result<Self, PolicyError> {
        manifest.validate()?;
        let mut policy = Self::deny_all();
        for iface in &manifest.capabilities.wit_interfaces {
            policy.allow_wit_interface(iface);
        }
        for op in &manifest.capabilities.wit_operations {
            policy.allow_wit_operation(&op.interface, &op.operation);
        }
        for core in &manifest.capabilities.core_imports {
            policy.allow_core(&core.module, &core.name);
        }
        Ok(policy)
    }

    /// Parses a YAML manifest string and constructs an authoritative [`HostPolicy`].
    ///
    /// See [`HostPolicy::from_manifest`] for security warnings regarding authoritative data sources.
    pub fn from_manifest_yaml(yaml_str: &str) -> Result<Self, PolicyError> {
        let manifest = ComponentManifest::from_yaml_str(yaml_str)?;
        Self::from_manifest(&manifest)
    }

    /// Parses a JSON manifest string and constructs an authoritative [`HostPolicy`].
    ///
    /// See [`HostPolicy::from_manifest`] for security warnings regarding authoritative data sources.
    pub fn from_manifest_json(json_str: &str) -> Result<Self, PolicyError> {
        let manifest = ComponentManifest::from_json_str(json_str)?;
        Self::from_manifest(&manifest)
    }

    /// Computes the effective [`HostPolicy`] from an untrusted requested manifest
    /// and an authoritative granted policy:
    ///
    /// \[ \text{Effective} = \text{Requested} \cap \text{Granted} \]
    #[must_use]
    pub fn effective(requested: &ComponentManifest, granted: &HostPolicy) -> Self {
        requested.effective_policy(granted)
    }

    /// Returns an iterator over all granular WIT operations explicitly allowed for
    /// the given interface identifier.
    pub fn allowed_wit_operations_for_interface<'a>(
        &'a self,
        interface_id: &'a str,
    ) -> impl Iterator<Item = &'a str> {
        self.allow_wit_operations
            .iter()
            .filter_map(move |(iface, op)| {
                if iface == interface_id {
                    Some(op.as_str())
                } else {
                    None
                }
            })
    }

    /// Allows a Core WASM `(module, name)` import. Chainable.
    pub fn allow_core(&mut self, module: impl Into<String>, name: impl Into<String>) -> &mut Self {
        self.allow_core_pairs.insert((module.into(), name.into()));
        self
    }

    /// Allows every operation of a WIT interface by its real identifier
    /// (e.g. `an:test/ops@0.1.0`). Chainable.
    pub fn allow_wit_interface(&mut self, interface_id: impl Into<String>) -> &mut Self {
        self.allow_wit_interfaces.insert(interface_id.into());
        self
    }

    /// Allows a single operation inside a WIT interface. Chainable.
    pub fn allow_wit_operation(
        &mut self,
        interface_id: impl Into<String>,
        operation: impl Into<String>,
    ) -> &mut Self {
        self.allow_wit_operations
            .insert((interface_id.into(), operation.into()));
        self
    }

    /// Legacy compatibility shim preserved for AN-010/011 call sites.
    /// Interpreted as [`HostPolicy::allow_core`].
    ///
    /// **Prefer** the granular alternatives for new code:
    /// - [`HostPolicy::allow_core`] for Core WASM imports.
    /// - [`HostPolicy::allow_wit_operation`] for a single WIT operation.
    /// - [`HostPolicy::allow_wit_interface`] for an entire WIT interface.
    pub fn allow(&mut self, module: impl Into<String>, name: impl Into<String>) -> &mut Self {
        self.allow_core(module, name)
    }

    /// **Permissive** wildcard across both Core WASM and Component Model.
    /// Authorises **any** name under the given module (or interface id),
    /// regardless of whether it is a type, a function or an instance.
    ///
    /// # When to use
    ///
    /// A bring-up / testing shortcut. Convenient to unblock a fixture
    /// without enumerating every operation.
    ///
    /// # When to prefer the granular alternatives
    ///
    /// For production policy declarations, **prefer** the granular
    /// alternatives — they encode exactly what the host authorises and
    /// make audits meaningful:
    /// - [`HostPolicy::allow_core`] for Core WASM imports.
    /// - [`HostPolicy::allow_wit_operation`] for a single WIT operation.
    /// - [`HostPolicy::allow_wit_interface`] for an entire WIT interface.
    pub fn allow_module(&mut self, module: impl Into<String>) -> &mut Self {
        self.allow_modules.insert(module.into());
        self
    }

    /// Returns `true` iff the given Core WASM import is permitted.
    #[must_use]
    pub fn is_allowed_core(&self, module: &str, name: &str) -> bool {
        if self.allow_modules.contains(module) {
            return true;
        }
        self.allow_core_pairs
            .contains(&(module.to_string(), name.to_string()))
    }

    /// Returns `true` iff every operation of the given WIT interface is
    /// permitted (interface-wide allow or module wildcard).
    #[must_use]
    pub fn is_wit_interface_allowed(&self, interface_id: &str) -> bool {
        self.allow_modules.contains(interface_id)
            || self.allow_wit_interfaces.contains(interface_id)
    }

    /// Returns `true` iff the given operation inside a WIT interface is
    /// permitted (operation-level, interface-wide, or module wildcard).
    #[must_use]
    pub fn is_wit_operation_allowed(&self, interface_id: &str, operation: &str) -> bool {
        if self.is_wit_interface_allowed(interface_id) {
            return true;
        }
        self.allow_wit_operations
            .contains(&(interface_id.to_string(), operation.to_string()))
    }

    /// Backwards-compatible query used by AN-010/011 tests: treats the pair
    /// as a Core WASM `(module, name)` lookup. New call sites should use
    /// [`HostPolicy::is_allowed_core`] directly.
    #[must_use]
    pub fn is_allowed(&self, module: &str, name: &str) -> bool {
        self.is_allowed_core(module, name)
    }
}
