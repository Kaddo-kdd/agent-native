//! AWS Lambda native Rust handler logic for benchmarking.
//!
//! Implements the exact same business logic as `examples/aws-lambda/src/handler.rs`
//! directly in native Rust, with ZERO dependencies on `agent-native-runtime`,
//! Wasmtime, or compiler crates.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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
}

/// Shared Lambda state initialized once per Lambda execution environment.
///
/// For pure native Rust, this state requires no engine or component allocations.
#[derive(Clone, Default)]
pub struct AppState;

impl AppState {
    pub fn new() -> Result<Self, HandlerError> {
        Ok(Self)
    }
}

/// Dispatches and executes a validated request natively in Rust.
pub fn handle_request(_state: &AppState, request: Request) -> Result<Response, HandlerError> {
    match request.operation.as_str() {
        "double-value" => handle_double_value(&request.args),
        "make-point-sum" => handle_make_point_sum(&request.args),
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

fn handle_double_value(args: &Value) -> Result<Response, HandlerError> {
    let x = extract_i32(args, "x", "double-value")?;
    let res = x.wrapping_mul(2);
    Ok(Response { result: json!(res) })
}

fn handle_make_point_sum(args: &Value) -> Result<Response, HandlerError> {
    let x = extract_i32(args, "x", "make-point-sum")?;
    let y = extract_i32(args, "y", "make-point-sum")?;

    let sum = x.wrapping_add(y);
    let doubled = sum.wrapping_mul(2);

    Ok(Response {
        result: json!({
            "sum": sum,
            "doubled": doubled,
        }),
    })
}
