use axum::{Json, Router, extract::State, middleware, routing::get};
use serde::Serialize;

use crate::{
    app::AppState,
    common::{
        error::AppError,
        response::{ApiResponse, ok},
    },
    middleware::logging,
    user,
};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/db/ping", get(db_ping))
        .merge(user::routes::router(state.clone()))
        .layer(middleware::from_fn(logging::log_request))
        .with_state(state)
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<ApiResponse<HealthResponse>> {
    ok("service is healthy", HealthResponse { status: "ok" })
}

#[derive(Serialize)]
struct DbPingResponse {
    message: String,
    value: i64,
}

async fn db_ping(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<DbPingResponse>>, AppError> {
    let storage = state.storage.as_ref().ok_or(AppError::DbNotConfigured)?;
    let value = storage.ping().await?;

    Ok(ok(
        "database connected",
        DbPingResponse {
            message: "database connected".to_string(),
            value,
        },
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt;

    use crate::{
        auth::jwt::JwtService, common::config::JwtConfig, user::service::UnavailableUserService,
    };

    fn test_state() -> AppState {
        AppState::new(
            None,
            JwtService::new(JwtConfig {
                secret: "router-test-secret".to_string(),
                expires_in_secs: 3_600,
                issuer: "router-test".to_string(),
            }),
            Arc::new(UnavailableUserService),
        )
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let response = create_router(test_state())
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("\"message\":\"service is healthy\""));
        assert!(body.contains("\"status\":\"ok\""));
    }

    #[tokio::test]
    async fn db_ping_returns_service_unavailable_without_database() {
        let response = create_router(test_state())
            .oneshot(
                Request::builder()
                    .uri("/db/ping")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("database is not configured"));
    }
}
