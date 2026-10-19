//! AWS Lambda custom runtime bootstrap for Agent-Native.
//!
//! Uses `lambda_runtime` to run as a native executable on `provided.al2023`.

use agent_native_aws_lambda::handler::{AppState, Request, handle_request};
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

    tracing::info!("Initializing Agent-Native Lambda runtime environment");

    // Initialize AppState (Engine + Component loading) once during Lambda cold start
    let state = AppState::new().map_err(|e| {
        tracing::error!("Failed to initialize Agent-Native AppState: {e}");
        Error::from(e)
    })?;

    tracing::info!(
        "Agent-Native Component Model loaded successfully. Starting lambda runtime loop"
    );

    // Pass shared AppState across warm invocations
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
