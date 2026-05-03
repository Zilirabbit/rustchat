use std::time::Instant;

use axum::{extract::Request, middleware::Next, response::Response};

pub async fn log_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let started_at = Instant::now();

    tracing::info!(method = %method, path = %path, "request started");

    let response = next.run(request).await;
    let status = response.status();
    let elapsed_ms = started_at.elapsed().as_millis() as u64;

    if status.is_server_error() {
        tracing::error!(
            method = %method,
            path = %path,
            status = status.as_u16(),
            elapsed_ms,
            "request completed with server error"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            method = %method,
            path = %path,
            status = status.as_u16(),
            elapsed_ms,
            "request completed with client error"
        );
    } else {
        tracing::info!(
            method = %method,
            path = %path,
            status = status.as_u16(),
            elapsed_ms,
            "request completed"
        );
    }

    response
}
