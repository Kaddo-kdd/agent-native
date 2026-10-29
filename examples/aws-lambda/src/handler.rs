//! AWS Lambda handler logic for Agent-Native.
//!
//! Exclusively consumes `agent-native-runtime` (`Engine`, `Component`, `Val`,
//! `HostPolicy`). Compiles and executes completely offline without LLMs or
//! compiler dependencies during execution.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use wasma_runtime::{Component, Engine, RuntimeError, Val};

pub const EMBEDDED_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/program-005.wasm"));

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Request {
    pub operation: String,
    #[serde(default)]
    pub args: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Response {
    pub result: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("Invalid operation '{0}'. Allowed operations are 'double-value' and 'make-point-sum'")]
    InvalidOperation(String),

    #[error("Missing argument '{0}' for operation '{1}'")]
    MissingArgument(&'static str, &'static str),

    #[error("Invalid argument '{0}': expected an integer")]
    InvalidArgumentType(&'static str),

    #[error("Integer out of range for argument '{0}': value must fit in i32")]
    IntegerOutOfRange(&'static str),

    #[error("Runtime error during component invocation: {0}")]
    Runtime(#[from] RuntimeError),

    #[error("Engine initialization or component load failed: {0}")]
    Load(String),

    #[error("Invalid return value from component: {0}")]
    InvalidReturnValue(String),
}

/// Shared Lambda state initialized once per Lambda execution environment.
///
/// Keeps `Engine` and the parsed `Component` cached across warm invocations.
/// Per ADR-005, each call to `Component::invoke` runs in a brand new
/// `wasmtime::Store`, preventing memory/state leaks across invocations.
#[derive(Clone)]
pub struct AppState {
    component: Arc<Component>,
}

impl AppState {
    /// Creates a new `AppState` by initializing the `Engine` and loading the
    /// embedded component bytes under default deny-all host policy.
    pub fn new() -> Result<Self, HandlerError> {
        Self::from_wasm_bytes(EMBEDDED_WASM)
    }

    /// Creates an `AppState` from specific WASM bytes (useful for testing or custom payloads).
    pub fn from_wasm_bytes(bytes: &[u8]) -> Result<Self, HandlerError> {
        let engine = Engine::new().map_err(|e| HandlerError::Load(e.to_string()))?;
        let component = engine
            .load_component(bytes)
            .map_err(|e| HandlerError::Load(e.to_string()))?;
        Ok(Self {
            component: Arc::new(component),
        })
    }

    /// Access to the underlying loaded component.
    #[must_use]
    pub fn component(&self) -> &Component {
        &self.component
    }
}

/// Dispatches and executes a validated request against the loaded component.
pub fn handle_request(state: &AppState, request: Request) -> Result<Response, HandlerError> {
    match request.operation.as_str() {
        "double-value" => handle_double_value(state, &request.args),
        "make-point-sum" => handle_make_point_sum(state, &request.args),
        other => Err(HandlerError::InvalidOperation(other.to_string())),
    }
}

fn extract_i32(args: &Value, field: &'static str, op: &'static str) -> Result<i32, HandlerError> {
    let obj = args
        .as_object()
        .ok_or(HandlerError::MissingArgument(field, op))?;
    let val = obj
        .get(field)
        .ok_or(HandlerError::MissingArgument(field, op))?;
    let int_val = val
        .as_i64()
        .ok_or(HandlerError::InvalidArgumentType(field))?;
    i32::try_from(int_val).map_err(|_| HandlerError::IntegerOutOfRange(field))
}

fn handle_double_value(state: &AppState, args: &Value) -> Result<Response, HandlerError> {
    let x = extract_i32(args, "x", "double-value")?;
    let results = state
        .component
        .invoke("an:test/ops@0.1.0#double-value", &[Val::S32(x)])?;

    if results.len() != 1 {
        return Err(HandlerError::InvalidReturnValue(format!(
            "expected 1 return value, got {}",
            results.len()
        )));
    }

    match results[0] {
        Val::S32(val) => Ok(Response { result: json!(val) }),
        ref other => Err(HandlerError::InvalidReturnValue(format!(
            "expected S32 return value, got {other:?}"
        ))),
    }
}

fn handle_make_point_sum(state: &AppState, args: &Value) -> Result<Response, HandlerError> {
    let x = extract_i32(args, "x", "make-point-sum")?;
    let y = extract_i32(args, "y", "make-point-sum")?;

    let results = state.component.invoke(
        "an:test/ops@0.1.0#make-point-sum",
        &[Val::S32(x), Val::S32(y)],
    )?;

    if results.len() != 1 {
        return Err(HandlerError::InvalidReturnValue(format!(
            "expected 1 return value, got {}",
            results.len()
        )));
    }

    let Val::Record(fields) = &results[0] else {
        return Err(HandlerError::InvalidReturnValue(format!(
            "expected Record return value, got {:?}",
            results[0]
        )));
    };

    let mut sum: Option<i32> = None;
    let mut doubled: Option<i32> = None;

    for (name, val) in fields {
        match (name.as_str(), val) {
            ("sum", Val::S32(v)) => sum = Some(*v),
            ("doubled", Val::S32(v)) => doubled = Some(*v),
            _ => {}
        }
    }

    let sum = sum.ok_or_else(|| {
        HandlerError::InvalidReturnValue("field 'sum' missing from point-sum record".to_string())
    })?;
    let doubled = doubled.ok_or_else(|| {
        HandlerError::InvalidReturnValue(
            "field 'doubled' missing from point-sum record".to_string(),
        )
    })?;

    Ok(Response {
        result: json!({
            "sum": sum,
            "doubled": doubled,
        }),
    })
}
