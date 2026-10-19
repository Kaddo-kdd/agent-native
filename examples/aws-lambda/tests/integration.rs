//! Offline integration tests for the AWS Lambda adapter.
//!
//! Validates AC-01, AC-02, AC-03, AC-04, AC-05, and AC-07 completely offline,
//! without requiring AWS credentials or network connectivity.

use std::fs;
use std::path::PathBuf;

use agent_native_aws_lambda::handler::{AppState, HandlerError, Request, Response, handle_request};
use serde_json::json;

fn get_state() -> AppState {
    AppState::new().expect("AppState initialization with embedded WASM must succeed")
}

#[test]
fn test_double_value_success() {
    let state = get_state();
    let req = Request {
        operation: "double-value".to_string(),
        args: json!({ "x": 21 }),
    };

    let resp = handle_request(&state, req).expect("invocation must succeed");
    assert_eq!(resp, Response { result: json!(42) });
}

#[test]
fn test_double_value_zero_and_negative() {
    let state = get_state();

    let resp_zero = handle_request(
        &state,
        Request {
            operation: "double-value".to_string(),
            args: json!({ "x": 0 }),
        },
    )
    .expect("x=0 must succeed");
    assert_eq!(resp_zero, Response { result: json!(0) });

    let resp_neg = handle_request(
        &state,
        Request {
            operation: "double-value".to_string(),
            args: json!({ "x": -15 }),
        },
    )
    .expect("x=-15 must succeed");
    assert_eq!(resp_neg, Response { result: json!(-30) });
}

#[test]
fn test_make_point_sum_success() {
    let state = get_state();
    let req = Request {
        operation: "make-point-sum".to_string(),
        args: json!({ "x": 3, "y": 4 }),
    };

    let resp = handle_request(&state, req).expect("invocation must succeed");
    assert_eq!(
        resp,
        Response {
            result: json!({
                "sum": 7,
                "doubled": 14
            })
        }
    );
}

#[test]
fn test_make_point_sum_negative_values() {
    let state = get_state();
    let req = Request {
        operation: "make-point-sum".to_string(),
        args: json!({ "x": -10, "y": 5 }),
    };

    let resp = handle_request(&state, req).expect("invocation must succeed");
    assert_eq!(
        resp,
        Response {
            result: json!({
                "sum": -5,
                "doubled": -10
            })
        }
    );
}

#[test]
fn test_warm_invocation_reuse() {
    // AC-03: Repeated invocations on the same AppState (same Engine + Component)
    // run independently without leaking state across invocations.
    let state = get_state();

    for i in 1..=5 {
        let resp = handle_request(
            &state,
            Request {
                operation: "double-value".to_string(),
                args: json!({ "x": i }),
            },
        )
        .expect("warm invocation must succeed");
        assert_eq!(
            resp,
            Response {
                result: json!(i * 2)
            }
        );
    }
}

#[test]
fn test_invalid_operation() {
    let state = get_state();
    let req = Request {
        operation: "unknown-operation".to_string(),
        args: json!({ "x": 1 }),
    };

    let err = handle_request(&state, req).unwrap_err();
    match err {
        HandlerError::InvalidOperation(op) => assert_eq!(op, "unknown-operation"),
        other => panic!("expected InvalidOperation, got: {other:?}"),
    }
}

#[test]
fn test_missing_argument() {
    let state = get_state();

    // Missing 'x' in double-value
    let err_double = handle_request(
        &state,
        Request {
            operation: "double-value".to_string(),
            args: json!({}),
        },
    )
    .unwrap_err();
    assert!(matches!(
        err_double,
        HandlerError::MissingArgument("x", "double-value")
    ));

    // Missing 'y' in make-point-sum
    let err_point = handle_request(
        &state,
        Request {
            operation: "make-point-sum".to_string(),
            args: json!({ "x": 10 }),
        },
    )
    .unwrap_err();
    assert!(matches!(
        err_point,
        HandlerError::MissingArgument("y", "make-point-sum")
    ));
}

#[test]
fn test_invalid_argument_type() {
    let state = get_state();

    let err_type = handle_request(
        &state,
        Request {
            operation: "double-value".to_string(),
            args: json!({ "x": "twenty-one" }),
        },
    )
    .unwrap_err();
    assert!(matches!(err_type, HandlerError::InvalidArgumentType("x")));
}

#[test]
fn test_integer_out_of_range() {
    let state = get_state();

    let err_overflow = handle_request(
        &state,
        Request {
            operation: "double-value".to_string(),
            args: json!({ "x": 999999999999_i64 }),
        },
    )
    .unwrap_err();
    assert!(matches!(err_overflow, HandlerError::IntegerOutOfRange("x")));
}

#[test]
fn test_invalid_component_bytes() {
    let invalid_bytes = b"not a valid wasm file";
    let res = AppState::from_wasm_bytes(invalid_bytes);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(matches!(err, HandlerError::Load(_)));
}

#[test]
fn test_event_fixtures() {
    let state = get_state();
    let events_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("events");

    // 1. events/double-value.json
    let double_json = fs::read_to_string(events_dir.join("double-value.json"))
        .expect("double-value.json must exist");
    let req_double: Request = serde_json::from_str(&double_json).expect("valid json");
    let resp_double = handle_request(&state, req_double).expect("must succeed");
    assert_eq!(resp_double, Response { result: json!(42) });

    // 2. events/make-point-sum.json
    let point_json = fs::read_to_string(events_dir.join("make-point-sum.json"))
        .expect("make-point-sum.json must exist");
    let req_point: Request = serde_json::from_str(&point_json).expect("valid json");
    let resp_point = handle_request(&state, req_point).expect("must succeed");
    assert_eq!(
        resp_point,
        Response {
            result: json!({
                "sum": 7,
                "doubled": 14
            })
        }
    );

    // 3. events/invalid-operation.json
    let invalid_json = fs::read_to_string(events_dir.join("invalid-operation.json"))
        .expect("invalid-operation.json must exist");
    let req_invalid: Request = serde_json::from_str(&invalid_json).expect("valid json");
    let err_invalid = handle_request(&state, req_invalid).unwrap_err();
    assert!(matches!(err_invalid, HandlerError::InvalidOperation(_)));
}
