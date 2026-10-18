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
    // Pre-instantiate check (ADR-004, deny-by-default). Imports are
    // identified by their REAL names and authorised by interface or
    // operation, never by item-kind labels like "func" or "instance"
    // (AN-015 precision 3).
    for (name, item) in component.component_type().imports(engine) {
        use wasmtime::component::types::ComponentItem;
        match item {
            ComponentItem::ComponentInstance(inst_ty) => {
                // Interface-wide allow short-circuits the per-operation check.
                if policy.is_wit_interface_allowed(name) {
                    continue;
                }
                // Otherwise every exported operation in the imported
                // interface must be explicitly authorised; the first
                // unauthorised one is surfaced with its REAL name.
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
                // Top-level function import. The interface identifier
                // Wasmtime gives us is `name`; we treat the operation name
                // as empty since the import is itself a function (rare in
                // practice for Agent-Native, where WIT interfaces wrap
                // operations).
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
                // Modules, components, resources and types imported at the
                // top level follow the interface-wide rule by default.
                if !policy.is_wit_interface_allowed(name) {
                    return Err(RuntimeError::UnauthorizedImport {
                        module: name.to_string(),
                        name: component_item_name(&item).to_string(),
                    });
                }
            }
        }
    }

    let linker: ComponentLinker<()> = ComponentLinker::new(engine);
    let mut store = Store::new(engine, ());

    let instance = linker
        .instantiate(&mut store, component)
        .map_err(|e| RuntimeError::InstantiationFailed(e.to_string()))?;

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
        if !policy.is_allowed_core(import.module(), import.name()) {
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

    core_results_to_component(&core_results)
}

/// Resolves an exported function. Supports top-level exports and functions
/// nested inside an exported WIT interface, referenced as
/// `interface_id#operation`.
fn resolve_component_func(
    instance: &wasmtime::component::Instance,
    store: &mut Store<()>,
    func_name: &str,
) -> Option<wasmtime::component::Func> {
    if let Some((iface, op)) = func_name.split_once('#') {
        let iface_idx = instance.get_export(&mut *store, None, iface)?;
        let fn_idx = instance.get_export(&mut *store, Some(&iface_idx), op)?;
        return instance.get_func(&mut *store, fn_idx);
    }
    instance.get_func(&mut *store, func_name)
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
