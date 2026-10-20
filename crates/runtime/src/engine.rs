//! Minimal engine wrapper around `wasmtime` with the Component Model
//! enabled.
//!
//! Per ADR-002, this module must not depend on `agent-native-compiler` nor
//! on any LLM adapter. It consumes WASM bytes produced elsewhere.
//!
//! Agent-Native IR v0.1-v0.3 compile to Core WASM modules; v0.4+ compile to
//! Component Model artifacts. The engine handles both transparently: it
//! tries the Component Model loader first and falls back to a Core module.

use wasmtime::Config;
use wasmtime::component::Component as WtComponent;

use crate::error::RuntimeError;
use crate::manifest::ComponentManifest;
use crate::policy::HostPolicy;

/// A reusable Wasmtime engine configured for Component Model execution and
/// Core WASM module loading.
#[derive(Clone)]
pub struct Engine {
    inner: wasmtime::Engine,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine").finish_non_exhaustive()
    }
}

impl Engine {
    /// Creates a new engine with Component Model enabled.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::LoadFailed`] if the underlying Wasmtime engine
    /// cannot be constructed.
    pub fn new() -> Result<Self, RuntimeError> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.consume_fuel(true);
        config.native_unwind_info(true);
        let inner =
            wasmtime::Engine::new(&config).map_err(|e| RuntimeError::LoadFailed(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Loads a WASM artifact under the default **deny-all** policy
    /// (ADR-004). Equivalent to `load_component_with_policy(bytes,
    /// &HostPolicy::deny_all())`.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::LoadFailed`] if the payload is neither a
    /// valid Component Model artifact nor a valid Core WASM module.
    pub fn load_component(&self, bytes: &[u8]) -> Result<Component, RuntimeError> {
        self.load_component_with_policy(bytes, &HostPolicy::deny_all())
    }

    /// Loads a WASM artifact under an explicit host-import policy. Accepts
    /// both Component Model and Core WASM modules.
    ///
    /// The policy is consulted pre-instantiate during
    /// [`Component::invoke`]; this method only parses and stores it with
    /// the loaded artifact.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::LoadFailed`] if the payload is neither a
    /// valid Component Model artifact nor a valid Core WASM module.
    pub fn load_component_with_policy(
        &self,
        bytes: &[u8],
        policy: &HostPolicy,
    ) -> Result<Component, RuntimeError> {
        let payload = if let Ok(component) = WtComponent::new(&self.inner, bytes) {
            Payload::Component(component)
        } else {
            let module = wasmtime::Module::new(&self.inner, bytes)
                .map_err(|e| RuntimeError::LoadFailed(e.to_string()))?;
            Payload::Module(module)
        };
        Ok(Component {
            payload,
            engine: self.inner.clone(),
            policy: policy.clone(),
        })
    }

    /// Loads a WASM artifact under an explicit policy AND validates all declared
    /// imports immediately at load time.
    ///
    /// Unlike [`Engine::load_component_with_policy`], which defers import checks
    /// until invocation/instantiation, this entry point guarantees that an artifact
    /// with unauthorized imports is rejected before returning a [`Component`].
    ///
    /// # Errors
    ///
    /// - Returns [`RuntimeError::LoadFailed`] if the payload is invalid WASM.
    /// - Returns [`RuntimeError::UnauthorizedImport`] if any declared import is not permitted.
    pub fn load_component_enforcing_policy(
        &self,
        bytes: &[u8],
        policy: &HostPolicy,
    ) -> Result<Component, RuntimeError> {
        let component = self.load_component_with_policy(bytes, policy)?;
        component.validate_imports()?;
        Ok(component)
    }

    /// Loads a WASM artifact with an untrusted [`ComponentManifest`] and an authoritative
    /// granted [`HostPolicy`]. Computes the effective policy (`Requested ∩ Granted`) and
    /// enforces all import validations immediately at load time.
    ///
    /// # Errors
    ///
    /// - Returns [`RuntimeError::LoadFailed`] if the payload is invalid WASM.
    /// - Returns [`RuntimeError::UnauthorizedImport`] if any declared import fails
    ///   the effective policy check.
    pub fn load_component_with_manifest(
        &self,
        bytes: &[u8],
        manifest: &ComponentManifest,
        granted_policy: &HostPolicy,
    ) -> Result<Component, RuntimeError> {
        manifest.validate()?;
        let effective = manifest.effective_policy(granted_policy);
        self.load_component_enforcing_policy(bytes, &effective)
    }

    /// Access to the inner Wasmtime engine. Used by `Resolver` to parse
    /// bytes directly into a cached payload.
    pub(crate) fn inner(&self) -> &wasmtime::Engine {
        &self.inner
    }

    /// Creates a new tenant execution context configured with the specified options.
    #[must_use]
    pub fn create_tenant(&self, config: crate::tenant::TenantConfig) -> crate::tenant::Tenant {
        crate::tenant::Tenant::new(config)
    }
}

/// A loaded WASM artifact (Component Model or Core module), ready for
/// invocation under a specific [`HostPolicy`].
pub struct Component {
    pub(crate) payload: Payload,
    pub(crate) engine: wasmtime::Engine,
    pub(crate) policy: HostPolicy,
}

impl std::fmt::Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Component")
            .field("is_core_module", &self.is_core_module())
            .field("is_component_model", &self.is_component_model())
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

impl Component {
    /// Returns `true` when the loaded artifact is a Core WASM module,
    /// `false` for a Component Model artifact. Lets callers pick the
    /// matching `invoke_*_with_host_ctx` variant (AN-019).
    #[must_use]
    pub fn is_core_module(&self) -> bool {
        matches!(self.payload, Payload::Module(_))
    }

    /// Returns `true` when the loaded artifact is a WebAssembly Component Model artifact.
    #[must_use]
    pub fn is_component_model(&self) -> bool {
        matches!(self.payload, Payload::Component(_))
    }

    /// Returns a reference to the [`HostPolicy`] governing this component.
    #[must_use]
    pub fn policy(&self) -> &HostPolicy {
        &self.policy
    }

    /// Validates all declared imports against the component's governing policy
    /// immediately, without instantiating or executing any code.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::UnauthorizedImport`] if any declared import in the
    /// artifact is not permitted under its governing policy.
    pub fn validate_imports(&self) -> Result<(), RuntimeError> {
        self.validate_imports_against_policy(&self.policy)
    }

    /// Validates all declared imports against an explicit policy without
    /// instantiating or executing any code.
    pub fn validate_imports_against_policy(&self, policy: &HostPolicy) -> Result<(), RuntimeError> {
        match &self.payload {
            Payload::Component(c) => {
                crate::invoke::precheck_component_imports(&self.engine, c, policy)
            }
            Payload::Module(m) => crate::invoke::precheck_core_imports(m, policy),
        }
    }
}

pub(crate) enum Payload {
    Component(WtComponent),
    Module(wasmtime::Module),
}
