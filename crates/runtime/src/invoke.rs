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

impl Component {
    /// Invokes an exported function by name with the supplied arguments.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] variants on load/instantiation/execution
    /// failures. See [`RuntimeError`] for the full taxonomy. In particular,
    /// a component with declared imports fails pre-instantiate with
    /// [`RuntimeError::UnauthorizedImport`].
    pub fn invoke(&self, func_name: &str, args: &[Val]) -> Result<Vec<Val>, RuntimeError> {
        match &self.payload {
            Payload::Component(component) => {
                invoke_component(&self.engine, component, &self.policy, func_name, args)
            }
            Payload::Module(module) => {
                invoke_core(&self.engine, module, &self.policy, func_name, args)
            }
        }
    }
}

fn invoke_component(
    engine: &wasmtime::Engine,
    component: &wasmtime::component::Component,
    policy: &crate::policy::HostPolicy,
    func_name: &str,
    args: &[Val],
) -> Result<Vec<Val>, RuntimeError> {
    // Pre-instantiate check: every declared import must be allowed by the
    // policy. The first offender is surfaced as a typed error (ADR-004,
    // deny-by-default).
    for (name, item) in component.component_type().imports(engine) {
        let item_label = component_item_name(&item);
        if !policy.is_allowed(name, item_label) {
            return Err(RuntimeError::UnauthorizedImport {
                module: name.to_string(),
                name: item_label.to_string(),
            });
        }
    }

    let linker: ComponentLinker<()> = ComponentLinker::new(engine);
    let mut store = Store::new(engine, ());

    let instance = linker
        .instantiate(&mut store, component)
        .map_err(|e| RuntimeError::InstantiationFailed(e.to_string()))?;

    let func = instance
        .get_func(&mut store, func_name)
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

    for (index, (expected_ty, provided)) in params.iter().zip(args.iter()).enumerate() {
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
        .map_err(|e| RuntimeError::ExecutionFailed(e.to_string()))?;
    func.post_return(&mut store)
        .map_err(|e| RuntimeError::ExecutionFailed(e.to_string()))?;

    Ok(results)
}

fn invoke_core(
    engine: &wasmtime::Engine,
    module: &wasmtime::Module,
    policy: &crate::policy::HostPolicy,
    func_name: &str,
    args: &[Val],
) -> Result<Vec<Val>, RuntimeError> {
    // Pre-instantiate check for Core WASM modules: every declared import
    // must be allowed by the policy (ADR-004, deny-by-default).
    for import in module.imports() {
        if !policy.is_allowed(import.module(), import.name()) {
            return Err(RuntimeError::UnauthorizedImport {
                module: import.module().to_string(),
                name: import.name().to_string(),
            });
        }
    }

    let linker: wasmtime::Linker<()> = wasmtime::Linker::new(engine);
    let mut store = Store::new(engine, ());

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
        .map_err(|e| RuntimeError::ExecutionFailed(e.to_string()))?;

    Ok(core_results_to_component(&core_results))
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
/// Non-`I32` return types are not produced by Agent-Native IR v0.5 Core WASM
/// output; if a future IR extension changes this, the arm must be
/// revisited.
fn core_results_to_component(results: &[wasmtime::Val]) -> Vec<Val> {
    results
        .iter()
        .map(|v| match v {
            wasmtime::Val::I32(i) => Val::S32(*i),
            // Unreachable under IR v0.5 which only emits i32-returning
            // Core WASM functions.
            _ => Val::S32(0),
        })
        .collect()
}
