//! Content-addressed resolver for parsed WASM artifacts.
//!
//! The POC (OQ L147) re-parses each component for every invocation. AN-012
//! measured this at ~247 µs per load. A [`Resolver`] amortises that cost
//! across calls that reference the same bytes by caching the parsed
//! `wasmtime::Module` or `wasmtime::component::Component`, keyed by the
//! SHA-256 of the input.
//!
//! Per ADR-005 (baseline operativa), the resolver caches **only** parsed
//! artifacts, never `wasmtime::Store` or live `Instance`s — each
//! [`crate::Component::invoke`] still runs in a fresh store.
//! [`Resolver::reset`] clears the cache (e.g. at end of a tenant session).

use std::collections::HashMap;

use sha2::{Digest, Sha256};
use wasmtime::component::Component as WtComponent;

use crate::engine::{Component, Engine, Payload};
use crate::error::RuntimeError;
use crate::policy::HostPolicy;

/// A reusable resolver with a content-addressed cache of parsed artifacts.
pub struct Resolver {
    engine: Engine,
    cache: HashMap<[u8; 32], CachedPayload>,
}

impl std::fmt::Debug for Resolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resolver")
            .field("cached_count", &self.cached_count())
            .finish()
    }
}

#[derive(Clone)]
enum CachedPayload {
    Component(WtComponent),
    Module(wasmtime::Module),
}

impl CachedPayload {
    fn into_payload(self) -> Payload {
        match self {
            CachedPayload::Component(c) => Payload::Component(c),
            CachedPayload::Module(m) => Payload::Module(m),
        }
    }
}

impl Resolver {
    /// Constructs a new resolver with an empty cache.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::LoadFailed`] if the underlying engine cannot
    /// be constructed.
    pub fn new() -> Result<Self, RuntimeError> {
        Ok(Self {
            engine: Engine::new()?,
            cache: HashMap::new(),
        })
    }

    /// Loads a component under the default deny-all policy, reusing the
    /// cached parse when the input hash matches.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::LoadFailed`] on invalid WASM.
    pub fn load(&mut self, bytes: &[u8]) -> Result<Component, RuntimeError> {
        self.load_with_policy(bytes, &HostPolicy::deny_all())
    }

    /// Loads a component under an explicit policy. The parsed artifact is
    /// cached by SHA-256; the policy is applied fresh to the returned
    /// [`Component`] on every call, so distinct policies over the same
    /// bytes do not duplicate the cache entry.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::LoadFailed`] on invalid WASM.
    pub fn load_with_policy(
        &mut self,
        bytes: &[u8],
        policy: &HostPolicy,
    ) -> Result<Component, RuntimeError> {
        let hash: [u8; 32] = Sha256::digest(bytes).into();
        let cached = if let Some(existing) = self.cache.get(&hash) {
            existing.clone()
        } else {
            let parsed = parse_bytes(self.engine.inner(), bytes)?;
            self.cache.insert(hash, parsed.clone());
            parsed
        };
        Ok(Component {
            payload: cached.into_payload(),
            engine: self.engine.inner().clone(),
            policy: policy.clone(),
        })
    }

    /// Clears the cache. Intended for tenant isolation per ADR-005.
    pub fn reset(&mut self) {
        self.cache.clear();
    }

    /// Returns the number of distinct artifacts currently cached. Primary
    /// use is testing and diagnostics.
    #[must_use]
    pub fn cached_count(&self) -> usize {
        self.cache.len()
    }
}

fn parse_bytes(engine: &wasmtime::Engine, bytes: &[u8]) -> Result<CachedPayload, RuntimeError> {
    if let Ok(component) = WtComponent::new(engine, bytes) {
        return Ok(CachedPayload::Component(component));
    }
    wasmtime::Module::new(engine, bytes)
        .map(CachedPayload::Module)
        .map_err(|e| RuntimeError::LoadFailed(e.to_string()))
}
