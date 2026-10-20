//! Resource and execution limits for component invocation.
//!
//! # Deterministic Fuel vs. Real Wall-Clock Time
//!
//! Per RM-06 governance and AN-035 design invariants:
//! - Computational **fuel** measures abstract WebAssembly instructions and basic blocks.
//! - Execution is deterministic: identical inputs on identical architectures consume
//!   the exact same fuel count.
//! - Fuel does **not** map to a guaranteed number of physical CPU milliseconds. Tests
//!   and metrics record wall-clock time strictly as observed diagnostic values,
//!   never as universal real-time guarantees.

/// Execution quotas and limits applied to a single invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionLimits {
    /// Maximum computational fuel units allowed for this invocation.
    ///
    /// When `None`, execution runs with an unmetered (effectively unlimited) budget.
    pub fuel_budget: Option<u64>,

    /// Maximum linear memory bytes allowed for the guest instance.
    ///
    /// Managed via `wasmtime::ResourceLimiter` in AN-036.
    pub max_memory_bytes: Option<usize>,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self::unmetered()
    }
}

impl ExecutionLimits {
    /// Constructs an unmetered execution configuration without strict fuel quotas.
    #[must_use]
    pub fn unmetered() -> Self {
        Self {
            fuel_budget: None,
            max_memory_bytes: None,
        }
    }

    /// Configures a strict fuel budget in computational units.
    #[must_use]
    pub fn with_fuel(mut self, fuel: u64) -> Self {
        self.fuel_budget = Some(fuel);
        self
    }

    /// Configures a maximum linear memory limit in bytes.
    #[must_use]
    pub fn with_max_memory_bytes(mut self, bytes: usize) -> Self {
        self.max_memory_bytes = Some(bytes);
        self
    }
}

/// Invocation report containing returned values and resource accounting metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationReport {
    /// Values returned by the exported function.
    pub values: Vec<crate::Val>,

    /// Total units of computational fuel consumed during the invocation,
    /// present only when a finite fuel budget was configured.
    pub fuel_consumed: Option<u64>,
}
