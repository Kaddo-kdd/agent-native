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

/// A reusable Wasmtime engine configured for Component Model execution and
/// Core WASM module loading.
pub struct Engine {
    inner: wasmtime::Engine,
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
        let inner =
            wasmtime::Engine::new(&config).map_err(|e| RuntimeError::LoadFailed(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Loads a WASM artifact from its serialized bytes. Accepts both
    /// Component Model and Core WASM modules.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::LoadFailed`] if the payload is neither a
    /// valid Component Model artifact nor a valid Core WASM module.
    pub fn load_component(&self, bytes: &[u8]) -> Result<Component, RuntimeError> {
        if let Ok(component) = WtComponent::new(&self.inner, bytes) {
            return Ok(Component {
                payload: Payload::Component(component),
                engine: self.inner.clone(),
            });
        }
        let module = wasmtime::Module::new(&self.inner, bytes)
            .map_err(|e| RuntimeError::LoadFailed(e.to_string()))?;
        Ok(Component {
            payload: Payload::Module(module),
            engine: self.inner.clone(),
        })
    }
}

/// A loaded WASM artifact (Component Model or Core module), ready for
/// invocation.
pub struct Component {
    pub(crate) payload: Payload,
    pub(crate) engine: wasmtime::Engine,
}

pub(crate) enum Payload {
    Component(WtComponent),
    Module(wasmtime::Module),
}
