//! Multi-tenant isolation and invocation lifecycle management.
//!
//! # Fundamental Security Invariant (ADR-005)
//!
//! - **Strictly ZERO instance reuse between tenants.**
//! - Every invocation executed by a [`Tenant`] runs in a fresh, ephemeral
//!   [`wasmtime::Store`] created specifically for that invocation and destroyed
//!   immediately upon return.
//! - Linear memory and instance state are never retained, cached, or shared
//!   across invocations or tenants.
//!
//! # Governance & Homogeneous Quotas
//!
//! A [`Tenant`] binds:
//! - An identity: [`TenantId`]
//! - An authorization boundary: [`HostPolicy`]
//! - Resource limits: [`ExecutionLimits`] (fuel budget, linear memory limits, table bounds)
//!
//! All invocations through a [`Tenant`] enforce this triplet homogeneously.

use std::fmt;
use wasmtime::component::Val;

use crate::engine::Component;
use crate::error::RuntimeError;
use crate::limits::{ExecutionLimits, InstanceLimits, InvocationReport};
use crate::policy::HostPolicy;

/// Immutable identifier of a tenant.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TenantId(pub String);

impl TenantId {
    /// Creates a new tenant identifier.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the string slice of the tenant identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for TenantId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for TenantId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&TenantId> for TenantId {
    fn from(id: &TenantId) -> Self {
        id.clone()
    }
}

/// Comprehensive configuration and quotas assigned to a [`Tenant`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantConfig {
    /// Identifier of the tenant.
    pub id: TenantId,

    /// Authoritative host capability policy.
    pub policy: HostPolicy,

    /// Execution limits (fuel and memory quotas).
    pub limits: ExecutionLimits,
}

impl TenantConfig {
    /// Creates a new tenant configuration with default deny-all policy and unmetered limits.
    #[must_use]
    pub fn new(id: impl Into<TenantId>) -> Self {
        Self {
            id: id.into(),
            policy: HostPolicy::deny_all(),
            limits: ExecutionLimits::unmetered(),
        }
    }

    /// Sets the authoritative host policy for this tenant.
    #[must_use]
    pub fn with_policy(mut self, policy: HostPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Sets the execution limits for this tenant.
    #[must_use]
    pub fn with_execution_limits(mut self, limits: ExecutionLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Configures a fuel budget for this tenant.
    #[must_use]
    pub fn with_fuel_budget(mut self, fuel: u64) -> Self {
        self.limits = self.limits.with_fuel(fuel);
        self
    }

    /// Configures maximum linear memory in bytes for this tenant.
    #[must_use]
    pub fn with_max_memory_bytes(mut self, bytes: usize) -> Self {
        self.limits = self.limits.with_max_memory_bytes(bytes);
        self
    }

    /// Configures full instance limits for this tenant.
    #[must_use]
    pub fn with_instance_limits(mut self, limits: InstanceLimits) -> Self {
        self.limits = self.limits.with_instance_limits(limits);
        self
    }
}

/// Tenant execution context.
///
/// Encapsulates tenant identity and governance policies. In adherence to ADR-005,
/// a `Tenant` does NOT store or pool `Store` or `Instance` state.
#[derive(Debug, Clone)]
pub struct Tenant {
    config: TenantConfig,
}

impl Tenant {
    /// Creates a new tenant context with the provided configuration.
    #[must_use]
    pub fn new(config: TenantConfig) -> Self {
        Self { config }
    }

    /// Returns a reference to the tenant identifier.
    #[must_use]
    pub fn id(&self) -> &TenantId {
        &self.config.id
    }

    /// Returns a reference to the tenant configuration.
    #[must_use]
    pub fn config(&self) -> &TenantConfig {
        &self.config
    }

    /// Validates whether a loaded component conforms to the tenant's policy
    /// without instantiating or executing it.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::UnauthorizedImport`] if any declared import in the
    /// component is not granted by the tenant's policy.
    pub fn validate_component(&self, component: &Component) -> Result<(), RuntimeError> {
        component.validate_imports_against_policy(&self.config.policy)
    }

    /// Invokes an exported function on a Component under this tenant's policy and limits.
    ///
    /// Each invocation creates a fresh, ephemeral `wasmtime::Store` that is discarded
    /// immediately upon return (ADR-005 zero instance reuse invariant).
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::UnauthorizedImport`] if ungranted imports are required,
    /// [`RuntimeError::FuelExhausted`] if the tenant's fuel budget is exceeded,
    /// [`RuntimeError::MemoryLimitExceeded`] if memory limits are breached, or other
    /// [`RuntimeError`] variants on failure.
    pub fn invoke(
        &self,
        component: &Component,
        func_name: &str,
        args: &[Val],
    ) -> Result<InvocationReport, RuntimeError> {
        component.invoke_with_policy_and_limits(
            &self.config.policy,
            &self.config.limits,
            func_name,
            args,
        )
    }

    /// Invokes a Core WASM function with a custom host context under this tenant's policy and limits.
    pub fn invoke_core_with_host_ctx<T: Send + 'static>(
        &self,
        component: &Component,
        func_name: &str,
        args: &[Val],
        host_ctx: T,
        build_linker: impl FnOnce(
            &wasmtime::Engine,
            &mut wasmtime::Linker<T>,
        ) -> Result<(), wasmtime::Error>,
    ) -> Result<(InvocationReport, T), RuntimeError> {
        component.invoke_core_with_host_ctx_and_policy_limits(
            &self.config.policy,
            &self.config.limits,
            func_name,
            args,
            host_ctx,
            build_linker,
        )
    }

    /// Invokes a Component Model function with a custom host context under this tenant's policy and limits.
    pub fn invoke_component_with_host_ctx<T: Send + 'static>(
        &self,
        component: &Component,
        func_name: &str,
        args: &[Val],
        host_ctx: T,
        build_linker: impl FnOnce(
            &wasmtime::Engine,
            &mut wasmtime::component::Linker<T>,
        ) -> Result<(), wasmtime::Error>,
    ) -> Result<(InvocationReport, T), RuntimeError> {
        component.invoke_component_with_host_ctx_and_policy_limits(
            &self.config.policy,
            &self.config.limits,
            func_name,
            args,
            host_ctx,
            build_linker,
        )
    }
}
