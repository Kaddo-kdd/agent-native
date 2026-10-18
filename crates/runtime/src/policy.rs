//! Declarative host-import policy.
//!
//! A [`HostPolicy`] carries an explicit allow-list of imports the runtime is
//! permitted to resolve. The default state is **deny-all** (ADR-004): any
//! declared import of the component is rejected unless the caller has
//! explicitly allowed it.
//!
//! This WI (AN-011) wires the policy into the pre-instantiate check added
//! by AN-010. It does **not** provide host function implementations:
//! allowing an import authorises the host to supply it, but if no host
//! function is actually linked the instantiation still fails with
//! [`crate::RuntimeError::InstantiationFailed`]. INIT-06-01 extends the
//! policy to carry stubs/providers; INIT-07-01 adds WASI stdio.

use std::collections::HashSet;

/// A declarative policy over host imports.
///
/// The policy is consulted pre-instantiate for every declared import of a
/// loaded artifact. An import is allowed when either its exact
/// `(module, name)` pair is on the allow-list, or its `module` is on the
/// module-wildcard list.
#[derive(Debug, Clone, Default)]
pub struct HostPolicy {
    allow_pairs: HashSet<(String, String)>,
    allow_modules: HashSet<String>,
}

impl HostPolicy {
    /// Creates a policy that denies every host import.
    ///
    /// This is the backwards-compatible default used by
    /// [`crate::Engine::load_component`].
    #[must_use]
    pub fn deny_all() -> Self {
        Self::default()
    }

    /// Allows an explicit `(module, name)` import. Chainable.
    pub fn allow(&mut self, module: impl Into<String>, name: impl Into<String>) -> &mut Self {
        self.allow_pairs.insert((module.into(), name.into()));
        self
    }

    /// Allows every import under the given module. Chainable.
    ///
    /// Useful during bring-up or when the host provides a well-known
    /// surface such as `host` or `wasi:cli`. INIT-06-01 may replace or
    /// refine this with capability-level grouping once the artifact
    /// manifest is formalised.
    pub fn allow_module(&mut self, module: impl Into<String>) -> &mut Self {
        self.allow_modules.insert(module.into());
        self
    }

    /// Returns `true` iff the given import is permitted by this policy.
    #[must_use]
    pub fn is_allowed(&self, module: &str, name: &str) -> bool {
        if self.allow_modules.contains(module) {
            return true;
        }
        self.allow_pairs
            .contains(&(module.to_string(), name.to_string()))
    }
}
