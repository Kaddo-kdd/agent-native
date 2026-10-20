//! Invocation of exported functions on a loaded artifact.
//!
//! Per ADR-005 (baseline operativa), each invocation runs in a fresh
//! `wasmtime::Store`; instances are not reused across invocations in MVP.
//!
//! Imports are intentionally **not** linked in this baseline: any component
//! that declares a host import without the runtime providing it fails
//! pre-instantiate with [`RuntimeError::UnauthorizedImport`] (ADR-004,
//! fail-explicit). Allow-listed capability imports arrive in INIT-03-05.

use wasmtime::Store;
use wasmtime::component::{Linker as ComponentLinker, Val};

use crate::engine::{Component, Payload};
use crate::error::RuntimeError;
use crate::limits::{ExecutionLimits, GrowthDenial, InvocationReport, StoreResourceLimiter};
use crate::policy::HostPolicy;

pub(crate) const UNMETERED_FUEL_BUDGET: u64 = u64::MAX;

#[derive(Debug, Default)]
pub(crate) struct StoreData {
    pub(crate) limiter: StoreResourceLimiter,
}

fn map_instantiation_error(err: &wasmtime::Error, denial: Option<GrowthDenial>) -> RuntimeError {
    if let Some(denial) = denial {
        return RuntimeError::MemoryLimitExceeded {
            requested_bytes: denial.requested_bytes,
            max_bytes: denial.max_bytes,
        };
    }
    RuntimeError::InstantiationFailed(err.to_string())
}

fn map_execution_error(
    err: &wasmtime::Error,
    fuel_budget: Option<u64>,
    denial: Option<GrowthDenial>,
) -> RuntimeError {
    for cause in err.chain() {
        if let Some(trap) = cause.downcast_ref::<wasmtime::Trap>() {
            match trap {
                wasmtime::Trap::OutOfFuel => {
                    return RuntimeError::FuelExhausted {
                        limit: fuel_budget.unwrap_or(0),
                    };
                }
                wasmtime::Trap::UnreachableCodeReached => {
                    if let Some(denial) = denial {
                        return RuntimeError::MemoryLimitExceeded {
                            requested_bytes: denial.requested_bytes,
                            max_bytes: denial.max_bytes,
                        };
                    }
                    return RuntimeError::ExecutionFailed(format!("{trap}: {err}"));
                }
                wasmtime::Trap::IntegerOverflow
                | wasmtime::Trap::IntegerDivisionByZero
                | wasmtime::Trap::BadConversionToInteger
                | wasmtime::Trap::TableOutOfBounds
                | wasmtime::Trap::IndirectCallToNull
                | wasmtime::Trap::BadSignature
                | wasmtime::Trap::StackOverflow => {
                    return RuntimeError::ExecutionFailed(format!("{trap}: {err}"));
                }
                _ => {}
            }
        }
    }

    if let Some(denial) = denial {
        return RuntimeError::MemoryLimitExceeded {
            requested_bytes: denial.requested_bytes,
            max_bytes: denial.max_bytes,
        };
    }

    RuntimeError::ExecutionFailed(err.to_string())
}

impl Component {
    /// Invokes an exported function by name with the supplied arguments under
    /// unmetered execution limits.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] variants on load/instantiation/execution
    /// failures. See [`RuntimeError`] for the full taxonomy. In particular,
    /// a component with declared imports fails pre-instantiate with
    /// [`RuntimeError::UnauthorizedImport`].
    pub fn invoke(&self, func_name: &str, args: &[Val]) -> Result<Vec<Val>, RuntimeError> {
        self.invoke_with_limits(func_name, args, &ExecutionLimits::unmetered())
            .map(|report| report.values)
    }

    /// Invokes an exported function with explicit execution limits (fuel budget).
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::FuelExhausted`] if execution exceeds the configured
    /// fuel budget. Returns other [`RuntimeError`] variants on load/instantiation/execution
    /// failures.
    pub fn invoke_with_limits(
        &self,
        func_name: &str,
        args: &[Val],
        limits: &ExecutionLimits,
    ) -> Result<InvocationReport, RuntimeError> {
        self.invoke_with_policy_and_limits(&self.policy, limits, func_name, args)
    }

    /// Invokes an exported function under explicit policy and execution limits.
    pub fn invoke_with_policy_and_limits(
        &self,
        policy: &HostPolicy,
        limits: &ExecutionLimits,
        func_name: &str,
        args: &[Val],
    ) -> Result<InvocationReport, RuntimeError> {
        match &self.payload {
            Payload::Component(component) => {
                invoke_component(&self.engine, component, policy, func_name, args, limits)
            }
            Payload::Module(module) => {
                invoke_core(&self.engine, module, policy, func_name, args, limits)
            }
        }
    }

    /// Invokes an exported Core WASM function with a caller-provided host
    /// context and `wasmtime::Linker<T>` builder under unmetered execution limits.
    pub fn invoke_core_with_host_ctx<T: Send + 'static>(
        &self,
        func_name: &str,
        args: &[Val],
        host_ctx: T,
        build_linker: impl FnOnce(
            &wasmtime::Engine,
            &mut wasmtime::Linker<T>,
        ) -> Result<(), wasmtime::Error>,
    ) -> Result<(Vec<Val>, T), RuntimeError> {
        let (report, ctx) = self.invoke_core_with_host_ctx_and_limits(
            func_name,
            args,
            &ExecutionLimits::unmetered(),
            host_ctx,
            build_linker,
        )?;
        Ok((report.values, ctx))
    }

    /// Invokes an exported Core WASM function with a caller-provided host
    /// context and explicit execution limits.
    pub fn invoke_core_with_host_ctx_and_limits<T: Send + 'static>(
        &self,
        func_name: &str,
        args: &[Val],
        limits: &ExecutionLimits,
        host_ctx: T,
        build_linker: impl FnOnce(
            &wasmtime::Engine,
            &mut wasmtime::Linker<T>,
        ) -> Result<(), wasmtime::Error>,
    ) -> Result<(InvocationReport, T), RuntimeError> {
        self.invoke_core_with_host_ctx_and_policy_limits(
            &self.policy,
            limits,
            func_name,
            args,
            host_ctx,
            build_linker,
        )
    }

    /// Invokes an exported Core WASM function under explicit policy and execution limits.
    pub fn invoke_core_with_host_ctx_and_policy_limits<T: Send + 'static>(
        &self,
        policy: &HostPolicy,
        limits: &ExecutionLimits,
        func_name: &str,
        args: &[Val],
        host_ctx: T,
        build_linker: impl FnOnce(
            &wasmtime::Engine,
            &mut wasmtime::Linker<T>,
        ) -> Result<(), wasmtime::Error>,
    ) -> Result<(InvocationReport, T), RuntimeError> {
        let Payload::Module(module) = &self.payload else {
            return Err(RuntimeError::ExecutionFailed(
                "invoke_core_with_host_ctx called on a Component-Model payload".to_string(),
            ));
        };
        invoke_core_with_host_ctx_impl(
            &self.engine,
            module,
            policy,
            func_name,
            args,
            limits,
            host_ctx,
            build_linker,
        )
    }

    /// Invokes an exported Component-Model function with a caller-provided
    /// host context and `wasmtime::component::Linker<T>` builder under unmetered execution limits.
    pub fn invoke_component_with_host_ctx<T: Send + 'static>(
        &self,
        func_name: &str,
        args: &[Val],
        host_ctx: T,
        build_linker: impl FnOnce(
            &wasmtime::Engine,
            &mut ComponentLinker<T>,
        ) -> Result<(), wasmtime::Error>,
    ) -> Result<(Vec<Val>, T), RuntimeError> {
        let (report, ctx) = self.invoke_component_with_host_ctx_and_limits(
            func_name,
            args,
            &ExecutionLimits::unmetered(),
            host_ctx,
            build_linker,
        )?;
        Ok((report.values, ctx))
    }

    /// Invokes an exported Component-Model function with a caller-provided
    /// host context and explicit execution limits.
    pub fn invoke_component_with_host_ctx_and_limits<T: Send + 'static>(
        &self,
        func_name: &str,
        args: &[Val],
        limits: &ExecutionLimits,
        host_ctx: T,
        build_linker: impl FnOnce(
            &wasmtime::Engine,
            &mut ComponentLinker<T>,
        ) -> Result<(), wasmtime::Error>,
    ) -> Result<(InvocationReport, T), RuntimeError> {
        self.invoke_component_with_host_ctx_and_policy_limits(
            &self.policy,
            limits,
            func_name,
            args,
            host_ctx,
            build_linker,
        )
    }

    /// Invokes an exported Component-Model function under explicit policy and execution limits.
    pub fn invoke_component_with_host_ctx_and_policy_limits<T: Send + 'static>(
        &self,
        policy: &HostPolicy,
        limits: &ExecutionLimits,
        func_name: &str,
        args: &[Val],
        host_ctx: T,
        build_linker: impl FnOnce(
            &wasmtime::Engine,
            &mut ComponentLinker<T>,
        ) -> Result<(), wasmtime::Error>,
    ) -> Result<(InvocationReport, T), RuntimeError> {
        let Payload::Component(component) = &self.payload else {
            return Err(RuntimeError::ExecutionFailed(
                "invoke_component_with_host_ctx called on a Core-WASM payload".to_string(),
            ));
        };
        invoke_component_with_host_ctx_impl(
            &self.engine,
            component,
            policy,
            func_name,
            args,
            limits,
            host_ctx,
            build_linker,
        )
    }
}

/// Pre-instantiate policy check for a Component. Extracted so the AN-019
/// executor shares exactly the same authorisation logic as
/// [`Component::invoke`].
pub(crate) fn precheck_component_imports(
    engine: &wasmtime::Engine,
    component: &wasmtime::component::Component,
    policy: &HostPolicy,
) -> Result<(), RuntimeError> {
    for (name, item) in component.component_type().imports(engine) {
        use wasmtime::component::types::ComponentItem;
        match item {
            ComponentItem::ComponentInstance(inst_ty) => {
                if policy.is_wit_interface_allowed(name) {
                    continue;
                }
                for (op_name, _op_item) in inst_ty.exports(engine) {
                    if !policy.is_wit_operation_allowed(name, op_name) {
                        return Err(RuntimeError::UnauthorizedImport {
                            module: name.to_string(),
                            name: op_name.to_string(),
                        });
                    }
                }
            }
            ComponentItem::ComponentFunc(_) => {
                if !policy.is_wit_interface_allowed(name)
                    && !policy.is_wit_operation_allowed(name, "")
                {
                    return Err(RuntimeError::UnauthorizedImport {
                        module: name.to_string(),
                        name: String::new(),
                    });
                }
            }
            _ => {
                if !policy.is_wit_interface_allowed(name) {
                    return Err(RuntimeError::UnauthorizedImport {
                        module: name.to_string(),
                        name: component_item_name(&item).to_string(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Pre-instantiate policy check for a Core WASM module.
pub(crate) fn precheck_core_imports(
    module: &wasmtime::Module,
    policy: &HostPolicy,
) -> Result<(), RuntimeError> {
    for import in module.imports() {
        if !policy.is_allowed_core(import.module(), import.name()) {
            return Err(RuntimeError::UnauthorizedImport {
                module: import.module().to_string(),
                name: import.name().to_string(),
            });
        }
    }
    Ok(())
}

fn invoke_component(
    engine: &wasmtime::Engine,
    component: &wasmtime::component::Component,
    policy: &HostPolicy,
    func_name: &str,
    args: &[Val],
    limits: &ExecutionLimits,
) -> Result<InvocationReport, RuntimeError> {
    precheck_component_imports(engine, component, policy)?;

    let linker: ComponentLinker<StoreData> = ComponentLinker::new(engine);
    let mut store = Store::new(
        engine,
        StoreData {
            limiter: StoreResourceLimiter {
                limits: limits.instance_limits,
                last_growth_denial: None,
            },
        },
    );
    store.limiter(|data| &mut data.limiter);

    let initial_budget = limits.fuel_budget.unwrap_or(UNMETERED_FUEL_BUDGET);
    let _ = store.set_fuel(initial_budget);

    let instance = linker
        .instantiate(&mut store, component)
        .map_err(|e| map_instantiation_error(&e, store.data().limiter.last_growth_denial))?;

    // Function lookup supports two forms:
    //
    // - Top-level `name`: `instance.get_func(name)` — the only path AN-009
    //   supported.
    // - WIT interface `interface_id#operation`: resolves the interface as
    //   an exported instance first, then the operation inside it. The
    //   `interface_id` is the fully qualified WIT identifier Wasmtime
    //   exports (e.g. `an:test/ops@0.1.0`); the operation uses the WIT
    //   kebab-case name.
    let func = resolve_component_func(&instance, &mut store, func_name)
        .ok_or_else(|| RuntimeError::FunctionNotFound(func_name.to_string()))?;

    let params = func.params(&store);
    let param_count = params.len();
    if param_count != args.len() {
        return Err(RuntimeError::ArityMismatch {
            name: func_name.to_string(),
            expected: param_count,
            got: args.len(),
        });
    }

    for (index, ((_param_name, expected_ty), provided)) in
        params.iter().zip(args.iter()).enumerate()
    {
        if !component_val_matches_type(provided, expected_ty) {
            return Err(RuntimeError::ArgTypeMismatch {
                function: func_name.to_string(),
                index,
                expected: format!("{expected_ty:?}"),
                found: component_val_kind(provided).to_string(),
            });
        }
    }

    let mut results = vec![Val::Bool(false); func.results(&store).len()];
    func.call(&mut store, args, &mut results).map_err(|e| {
        map_execution_error(
            &e,
            limits.fuel_budget,
            store.data().limiter.last_growth_denial,
        )
    })?;
    func.post_return(&mut store).map_err(|e| {
        map_execution_error(
            &e,
            limits.fuel_budget,
            store.data().limiter.last_growth_denial,
        )
    })?;

    let remaining_fuel = store.get_fuel().unwrap_or(0);
    let fuel_consumed = limits.fuel_budget.map(|b| b.saturating_sub(remaining_fuel));

    Ok(InvocationReport {
        values: results,
        fuel_consumed,
    })
}

fn invoke_core(
    engine: &wasmtime::Engine,
    module: &wasmtime::Module,
    policy: &HostPolicy,
    func_name: &str,
    args: &[Val],
    limits: &ExecutionLimits,
) -> Result<InvocationReport, RuntimeError> {
    precheck_core_imports(module, policy)?;

    let linker: wasmtime::Linker<StoreData> = wasmtime::Linker::new(engine);
    let mut store = Store::new(
        engine,
        StoreData {
            limiter: StoreResourceLimiter {
                limits: limits.instance_limits,
                last_growth_denial: None,
            },
        },
    );
    store.limiter(|data| &mut data.limiter);

    let initial_budget = limits.fuel_budget.unwrap_or(UNMETERED_FUEL_BUDGET);
    let _ = store.set_fuel(initial_budget);

    let instance = linker
        .instantiate(&mut store, module)
        .map_err(|e| map_instantiation_error(&e, store.data().limiter.last_growth_denial))?;

    let func = instance
        .get_func(&mut store, func_name)
        .ok_or_else(|| RuntimeError::FunctionNotFound(func_name.to_string()))?;

    let func_ty = func.ty(&store);
    let param_count = func_ty.params().len();
    if param_count != args.len() {
        return Err(RuntimeError::ArityMismatch {
            name: func_name.to_string(),
            expected: param_count,
            got: args.len(),
        });
    }

    for (index, (expected_ty, provided)) in func_ty.params().zip(args.iter()).enumerate() {
        if !core_val_matches_type(provided, &expected_ty) {
            return Err(RuntimeError::ArgTypeMismatch {
                function: func_name.to_string(),
                index,
                expected: format!("{expected_ty:?}"),
                found: component_val_kind(provided).to_string(),
            });
        }
    }

    let core_args = component_vals_to_core(args, func_name)?;
    let mut core_results = vec![wasmtime::Val::I32(0); func_ty.results().len()];
    func.call(&mut store, &core_args, &mut core_results)
        .map_err(|e| {
            map_execution_error(
                &e,
                limits.fuel_budget,
                store.data().limiter.last_growth_denial,
            )
        })?;

    let lifted = core_results_to_component(&core_results)?;
    let remaining_fuel = store.get_fuel().unwrap_or(0);
    let fuel_consumed = limits.fuel_budget.map(|b| b.saturating_sub(remaining_fuel));

    Ok(InvocationReport {
        values: lifted,
        fuel_consumed,
    })
}

/// Resolves an exported function. Supports top-level exports and functions
/// nested inside an exported WIT interface, referenced as
/// `interface_id#operation`.
fn resolve_component_func<T>(
    instance: &wasmtime::component::Instance,
    store: &mut Store<T>,
    func_name: &str,
) -> Option<wasmtime::component::Func> {
    if let Some((iface, op)) = func_name.split_once('#') {
        let iface_idx = instance.get_export(&mut *store, None, iface)?;
        let fn_idx = instance.get_export(&mut *store, Some(&iface_idx), op)?;
        return instance.get_func(&mut *store, fn_idx);
    }
    instance.get_func(&mut *store, func_name)
}

/// Core-WASM variant that runs with a caller-populated `Linker<T>` and
/// returns the final `T` alongside the lifted results. Shares the policy
/// pre-check and function-resolution logic with [`invoke_core`].
#[allow(clippy::too_many_arguments)]
fn invoke_core_with_host_ctx_impl<T: Send + 'static>(
    engine: &wasmtime::Engine,
    module: &wasmtime::Module,
    policy: &HostPolicy,
    func_name: &str,
    args: &[Val],
    limits: &ExecutionLimits,
    host_ctx: T,
    build_linker: impl FnOnce(
        &wasmtime::Engine,
        &mut wasmtime::Linker<T>,
    ) -> Result<(), wasmtime::Error>,
) -> Result<(InvocationReport, T), RuntimeError> {
    precheck_core_imports(module, policy)?;

    let mut linker: wasmtime::Linker<T> = wasmtime::Linker::new(engine);
    build_linker(engine, &mut linker)
        .map_err(|e| RuntimeError::InstantiationFailed(e.to_string()))?;
    let mut store = Store::new(engine, host_ctx);
    let initial_budget = limits.fuel_budget.unwrap_or(UNMETERED_FUEL_BUDGET);
    let _ = store.set_fuel(initial_budget);

    let instance = linker
        .instantiate(&mut store, module)
        .map_err(|e| RuntimeError::InstantiationFailed(e.to_string()))?;

    let func = instance
        .get_func(&mut store, func_name)
        .ok_or_else(|| RuntimeError::FunctionNotFound(func_name.to_string()))?;

    let func_ty = func.ty(&store);
    let param_count = func_ty.params().len();
    if param_count != args.len() {
        return Err(RuntimeError::ArityMismatch {
            name: func_name.to_string(),
            expected: param_count,
            got: args.len(),
        });
    }

    for (index, (expected_ty, provided)) in func_ty.params().zip(args.iter()).enumerate() {
        if !core_val_matches_type(provided, &expected_ty) {
            return Err(RuntimeError::ArgTypeMismatch {
                function: func_name.to_string(),
                index,
                expected: format!("{expected_ty:?}"),
                found: component_val_kind(provided).to_string(),
            });
        }
    }

    let core_args = component_vals_to_core(args, func_name)?;
    let mut core_results = vec![wasmtime::Val::I32(0); func_ty.results().len()];
    func.call(&mut store, &core_args, &mut core_results)
        .map_err(|e| map_execution_error(&e, limits.fuel_budget, None))?;

    let lifted = core_results_to_component(&core_results)?;
    let remaining_fuel = store.get_fuel().unwrap_or(0);
    let fuel_consumed = limits.fuel_budget.map(|b| b.saturating_sub(remaining_fuel));

    Ok((
        InvocationReport {
            values: lifted,
            fuel_consumed,
        },
        store.into_data(),
    ))
}

/// Component-Model variant with host context (symmetric to the core one).
#[allow(clippy::too_many_arguments)]
fn invoke_component_with_host_ctx_impl<T: Send + 'static>(
    engine: &wasmtime::Engine,
    component: &wasmtime::component::Component,
    policy: &HostPolicy,
    func_name: &str,
    args: &[Val],
    limits: &ExecutionLimits,
    host_ctx: T,
    build_linker: impl FnOnce(&wasmtime::Engine, &mut ComponentLinker<T>) -> Result<(), wasmtime::Error>,
) -> Result<(InvocationReport, T), RuntimeError> {
    precheck_component_imports(engine, component, policy)?;

    let mut linker: ComponentLinker<T> = ComponentLinker::new(engine);
    build_linker(engine, &mut linker)
        .map_err(|e| RuntimeError::InstantiationFailed(e.to_string()))?;
    let mut store = Store::new(engine, host_ctx);
    let initial_budget = limits.fuel_budget.unwrap_or(UNMETERED_FUEL_BUDGET);
    let _ = store.set_fuel(initial_budget);

    let instance = linker
        .instantiate(&mut store, component)
        .map_err(|e| RuntimeError::InstantiationFailed(e.to_string()))?;

    let func = resolve_component_func(&instance, &mut store, func_name)
        .ok_or_else(|| RuntimeError::FunctionNotFound(func_name.to_string()))?;

    let params = func.params(&store);
    let param_count = params.len();
    if param_count != args.len() {
        return Err(RuntimeError::ArityMismatch {
            name: func_name.to_string(),
            expected: param_count,
            got: args.len(),
        });
    }

    for (index, ((_param_name, expected_ty), provided)) in
        params.iter().zip(args.iter()).enumerate()
    {
        if !component_val_matches_type(provided, expected_ty) {
            return Err(RuntimeError::ArgTypeMismatch {
                function: func_name.to_string(),
                index,
                expected: format!("{expected_ty:?}"),
                found: component_val_kind(provided).to_string(),
            });
        }
    }

    let mut results = vec![Val::Bool(false); func.results(&store).len()];
    func.call(&mut store, args, &mut results)
        .map_err(|e| map_execution_error(&e, limits.fuel_budget, None))?;
    func.post_return(&mut store)
        .map_err(|e| map_execution_error(&e, limits.fuel_budget, None))?;

    let remaining_fuel = store.get_fuel().unwrap_or(0);
    let fuel_consumed = limits.fuel_budget.map(|b| b.saturating_sub(remaining_fuel));

    Ok((
        InvocationReport {
            values: results,
            fuel_consumed,
        },
        store.into_data(),
    ))
}

/// Returns a short, stable kebab-style name for an imported component item,
/// used only for error messages.
fn component_item_name(item: &wasmtime::component::types::ComponentItem) -> &'static str {
    use wasmtime::component::types::ComponentItem;
    match item {
        ComponentItem::ComponentFunc(_) => "func",
        ComponentItem::CoreFunc(_) => "core-func",
        ComponentItem::Module(_) => "module",
        ComponentItem::Component(_) => "component",
        ComponentItem::ComponentInstance(_) => "instance",
        ComponentItem::Type(_) => "type",
        ComponentItem::Resource(_) => "resource",
    }
}

/// Returns a short kebab-style tag for a component `Val`, used in error
/// messages.
fn component_val_kind(v: &Val) -> &'static str {
    match v {
        Val::Bool(_) => "bool",
        Val::S8(_) => "s8",
        Val::U8(_) => "u8",
        Val::S16(_) => "s16",
        Val::U16(_) => "u16",
        Val::S32(_) => "s32",
        Val::U32(_) => "u32",
        Val::S64(_) => "s64",
        Val::U64(_) => "u64",
        Val::Float32(_) => "float32",
        Val::Float64(_) => "float64",
        Val::Char(_) => "char",
        Val::String(_) => "string",
        Val::List(_) => "list",
        Val::Record(_) => "record",
        Val::Tuple(_) => "tuple",
        Val::Variant(_, _) => "variant",
        Val::Enum(_) => "enum",
        Val::Option(_) => "option",
        Val::Result(_) => "result",
        Val::Flags(_) => "flags",
        Val::Resource(_) => "resource",
    }
}

fn component_val_matches_type(v: &Val, ty: &wasmtime::component::types::Type) -> bool {
    use wasmtime::component::types::Type;
    matches!(
        (v, ty),
        (Val::Bool(_), Type::Bool)
            | (Val::S32(_), Type::S32)
            | (Val::U32(_), Type::U32)
            | (Val::S64(_), Type::S64)
            | (Val::U64(_), Type::U64)
    )
}

fn core_val_matches_type(v: &Val, ty: &wasmtime::ValType) -> bool {
    matches!(
        (v, ty),
        (Val::S32(_) | Val::Bool(_), wasmtime::ValType::I32)
            | (Val::S64(_), wasmtime::ValType::I64)
    )
}

/// Lowers component-level `Val` arguments to core WASM `wasmtime::Val`.
/// Supports the primitive subset Agent-Native IR v0.5 uses: `S32` and
/// `Bool`.
fn component_vals_to_core(
    args: &[Val],
    func_name: &str,
) -> Result<Vec<wasmtime::Val>, RuntimeError> {
    args.iter()
        .map(|v| match v {
            Val::S32(i) => Ok(wasmtime::Val::I32(*i)),
            Val::Bool(b) => Ok(wasmtime::Val::I32(i32::from(*b))),
            other => Err(RuntimeError::ExecutionFailed(format!(
                "unsupported Core WASM argument type for function '{func_name}': {other:?}"
            ))),
        })
        .collect()
}

/// Lifts core WASM results back to component-level `Val`. Core WASM has no
/// `Bool`; booleans round-trip as `S32` and the caller interprets using the
/// function's declared IR signature.
///
/// AN-015 (ADR-004): any non-`I32` return type produces a typed
/// `RuntimeError::ExecutionFailed` instead of a silent `Val::S32(0)`
/// fallback. Agent-Native IR v0.5 Core WASM output never emits such types,
/// so this arm acts as an invariant guard rather than a silent coercion.
fn core_results_to_component(results: &[wasmtime::Val]) -> Result<Vec<Val>, RuntimeError> {
    results
        .iter()
        .map(|v| match v {
            wasmtime::Val::I32(i) => Ok(Val::S32(*i)),
            other => Err(RuntimeError::ExecutionFailed(format!(
                "unsupported core result type: {other:?}"
            ))),
        })
        .collect()
}
