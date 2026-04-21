use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::{app::AppState, common::error::AppError, storage::db::ping_db};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/db/ping", get(db_ping))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Serialize)]
struct DbPingResponse {
    message: String,
    value: i64,
}

async fn db_ping(State(state): State<AppState>) -> Result<Json<DbPingResponse>, AppError> {
    let pool = state.db.as_ref().ok_or(AppError::DbNotConfigured)?;
    let value = ping_db(pool).await?;

    Ok(Json(DbPingResponse {
        message: "database connected".to_string(),
        value,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt;

    fn test_state() -> AppState {
        AppState { db: None }
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
        assert_eq!(&body[..], b"ok");
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
