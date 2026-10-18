//! Invocation of exported functions on a loaded artifact.
//!
//! Per ADR-005 (baseline operativa), each invocation runs in a fresh
//! `wasmtime::Store`; instances are not reused across invocations in MVP.
//!
//! Imports are intentionally **not** linked: this WI only accepts artifacts
//! with no imports or with imports the host does not need to satisfy for
//! the call to succeed. Allow-listed capability imports arrive in
//! INIT-03-05.

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
    /// failures. See [`RuntimeError`] for the full taxonomy.
    pub fn invoke(&self, func_name: &str, args: &[Val]) -> Result<Vec<Val>, RuntimeError> {
        match &self.payload {
            Payload::Component(component) => {
                invoke_component(&self.engine, component, func_name, args)
            }
            Payload::Module(module) => invoke_core(&self.engine, module, func_name, args),
        }
    }
}

fn invoke_component(
    engine: &wasmtime::Engine,
    component: &wasmtime::component::Component,
    func_name: &str,
    args: &[Val],
) -> Result<Vec<Val>, RuntimeError> {
    let linker: ComponentLinker<()> = ComponentLinker::new(engine);
    let mut store = Store::new(engine, ());

    let instance = linker
        .instantiate(&mut store, component)
        .map_err(|e| RuntimeError::InstantiationFailed(e.to_string()))?;

    let func = instance
        .get_func(&mut store, func_name)
        .ok_or_else(|| RuntimeError::FunctionNotFound(func_name.to_string()))?;

    let param_count = func.params(&store).len();
    if param_count != args.len() {
        return Err(RuntimeError::ArityMismatch {
            name: func_name.to_string(),
            expected: param_count,
            got: args.len(),
        });
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
    func_name: &str,
    args: &[Val],
) -> Result<Vec<Val>, RuntimeError> {
    let linker: wasmtime::Linker<()> = wasmtime::Linker::new(engine);
    let mut store = Store::new(engine, ());

    let instance = linker
        .instantiate(&mut store, module)
        .map_err(|e| RuntimeError::InstantiationFailed(e.to_string()))?;

    let func = instance
        .get_func(&mut store, func_name)
        .ok_or_else(|| RuntimeError::FunctionNotFound(func_name.to_string()))?;

    let core_args = component_vals_to_core(args, func_name)?;
    let func_ty = func.ty(&store);
    let param_count = func_ty.params().len();
    if param_count != args.len() {
        return Err(RuntimeError::ArityMismatch {
            name: func_name.to_string(),
            expected: param_count,
            got: args.len(),
        });
    }

    let mut core_results = vec![wasmtime::Val::I32(0); func_ty.results().len()];
    func.call(&mut store, &core_args, &mut core_results)
        .map_err(|e| RuntimeError::ExecutionFailed(e.to_string()))?;

    Ok(core_results_to_component(&core_results))
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
