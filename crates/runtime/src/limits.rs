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
//!
//! # Memory Limits and `ResourceLimiter`
//!
//! Per AN-036 design invariants:
//! - Memory bounds limit guest linear memory pages (64 KiB each) and table elements.
//! - If `memory.grow` requests growth exceeding the configured limit, the host
//!   rejects the request and causes `memory.grow` to return `-1` per the WebAssembly spec.
//! - If the guest traps as a result of denied memory growth, or if initial memory requirements
//!   exceed the limit, the runtime returns [`crate::RuntimeError::MemoryLimitExceeded`].

/// Detailed instance and resource quotas enforced by [`wasmtime::ResourceLimiter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstanceLimits {
    /// Maximum linear memory size in bytes across all memories in the instance.
    pub max_memory_bytes: usize,

    /// Maximum number of elements in a table.
    pub max_table_elements: u32,

    /// Maximum number of instances that can be created in the Store.
    pub max_instances: usize,

    /// Maximum number of tables that can be created in the Store.
    pub max_tables: usize,

    /// Maximum number of linear memories that can be created in the Store.
    pub max_memories: usize,
}

impl Default for InstanceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 16 * 1024 * 1024, // 16 MB default
            max_table_elements: 10_000,
            max_instances: 10,
            max_tables: 10,
            max_memories: 5,
        }
    }
}

impl InstanceLimits {
    /// Creates an instance limits configuration with the specified maximum memory bytes.
    #[must_use]
    pub fn new(max_memory_bytes: usize) -> Self {
        Self {
            max_memory_bytes,
            ..Default::default()
        }
    }

    /// Sets the maximum linear memory limit in bytes.
    #[must_use]
    pub fn with_max_memory_bytes(mut self, bytes: usize) -> Self {
        self.max_memory_bytes = bytes;
        self
    }

    /// Sets the maximum number of table elements allowed.
    #[must_use]
    pub fn with_max_table_elements(mut self, elements: u32) -> Self {
        self.max_table_elements = elements;
        self
    }
}

/// Execution quotas and limits applied to a single invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionLimits {
    /// Maximum computational fuel units allowed for this invocation.
    ///
    /// When `None`, execution runs with an unmetered (effectively unlimited) budget.
    pub fuel_budget: Option<u64>,

    /// Maximum linear memory bytes allowed for the guest instance.
    pub max_memory_bytes: Option<usize>,

    /// Instance and table resource quotas enforced via [`wasmtime::ResourceLimiter`].
    pub instance_limits: Option<InstanceLimits>,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self::unmetered()
    }
}

impl ExecutionLimits {
    /// Constructs an unmetered execution configuration without strict fuel quotas or memory limits.
    #[must_use]
    pub fn unmetered() -> Self {
        Self {
            fuel_budget: None,
            max_memory_bytes: None,
            instance_limits: None,
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
        let mut il = self.instance_limits.unwrap_or_default();
        il.max_memory_bytes = bytes;
        self.instance_limits = Some(il);
        self
    }

    /// Configures full instance limits (memory, tables, instances).
    #[must_use]
    pub fn with_instance_limits(mut self, limits: InstanceLimits) -> Self {
        self.max_memory_bytes = Some(limits.max_memory_bytes);
        self.instance_limits = Some(limits);
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

/// Internal record of a resource denial event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GrowthDenial {
    pub requested_bytes: usize,
    pub max_bytes: usize,
}

/// Internal `wasmtime::ResourceLimiter` implementation.
#[derive(Debug, Default)]
pub(crate) struct StoreResourceLimiter {
    pub limits: Option<InstanceLimits>,
    pub last_growth_denial: Option<GrowthDenial>,
}

impl wasmtime::ResourceLimiter for StoreResourceLimiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        if let Some(limits) = &self.limits {
            if desired > limits.max_memory_bytes {
                self.last_growth_denial = Some(GrowthDenial {
                    requested_bytes: desired,
                    max_bytes: limits.max_memory_bytes,
                });
                return Ok(false);
            }
        }
        if let Some(max) = maximum {
            if desired > max {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        if let Some(limits) = &self.limits {
            if desired > limits.max_table_elements as usize {
                return Ok(false);
            }
        }
        if let Some(max) = maximum {
            if desired > max {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn instances(&self) -> usize {
        self.limits.map_or(10_000, |l| l.max_instances)
    }

    fn tables(&self) -> usize {
        self.limits.map_or(10_000, |l| l.max_tables)
    }

    fn memories(&self) -> usize {
        self.limits.map_or(10_000, |l| l.max_memories)
    }
}
