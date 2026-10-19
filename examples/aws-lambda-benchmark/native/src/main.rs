//! AWS Lambda custom runtime bootstrap for native Rust benchmark.
//!
//! Uses `lambda_runtime` to run as a native executable on `provided.al2023`.

use agent_native_aws_lambda_native::handler::{AppState, Request, handle_request};
use lambda_runtime::{Error, LambdaEvent, service_fn};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .without_time()
        .init();

    tracing::info!("Initializing native Rust Lambda runtime environment");

    let state = AppState::new().map_err(|e| {
        tracing::error!("Failed to initialize AppState: {e}");
        Error::from(e)
    })?;

    tracing::info!("Native Rust environment ready. Starting lambda runtime loop");

    let handler_fn = service_fn(|event: LambdaEvent<Request>| {
        let state = state.clone();
        async move {
            let (request, _ctx) = event.into_parts();
            tracing::info!(operation = %request.operation, "Processing request");
            let response = handle_request(&state, request).map_err(|e| {
                tracing::warn!("Request failed: {e}");
                Error::from(e)
            })?;
            Ok::<_, Error>(response)
        }
    });

    lambda_runtime::run(handler_fn).await
}
