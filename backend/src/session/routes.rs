use axum::{Router, middleware, routing::post};

use crate::{app::AppState, middleware::auth};

use super::handler;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/api/sessions/private",
            post(handler::create_private_session),
        )
        .route(
            "/api/sessions/{session_id}/read",
            post(handler::mark_session_read),
        )
        .route_layer(middleware::from_fn_with_state(state, auth::require_auth))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    use crate::{
        app::AppState,
        auth::{jwt::JwtService, types::CurrentUser},
        common::{config::JwtConfig, error::AppResult},
        message::service::UnavailableMessageService,
        user::service::UnavailableUserService,
    };

    use super::super::{
        dto::{CreatePrivateSessionRequest, CreatePrivateSessionResponse, MarkSessionReadResponse},
        service::SessionUseCase,
    };
    use super::router;

    struct StubSessionService;

    #[async_trait]
    impl SessionUseCase for StubSessionService {
        async fn create_private_session(
            &self,
            _current_user: &CurrentUser,
            request: CreatePrivateSessionRequest,
        ) -> AppResult<CreatePrivateSessionResponse> {
            Ok(CreatePrivateSessionResponse {
                session_id: 12,
                session_type: "private",
                peer_user_id: request.target_user_id,
                created_at: "2026-05-03 12:00:00+00".to_string(),
                created: true,
            })
        }

        async fn mark_session_read(
            &self,
            _current_user: &CurrentUser,
            session_id: i64,
        ) -> AppResult<MarkSessionReadResponse> {
            Ok(MarkSessionReadResponse {
                session_id,
                last_read_message_id: Some(99),
                last_read_at: "2026-05-07 12:00:00+00".to_string(),
            })
        }
    }

    fn test_jwt_service() -> JwtService {
        JwtService::new(JwtConfig {
            secret: "session-routes-test-secret".to_string(),
            expires_in_secs: 3_600,
            issuer: "rustchat-test".to_string(),
        })
    }

    #[tokio::test]
    async fn create_private_session_endpoint_returns_session() {
        let jwt = test_jwt_service();
        let token = jwt.issue_token(1, "alice").unwrap();
        let state = AppState::new_with_services(
            None,
            jwt,
            Arc::new(UnavailableUserService),
            Arc::new(StubSessionService),
            Arc::new(UnavailableMessageService),
        );

        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/sessions/private")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "target_user_id": 2 }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "private session ready");
        assert_eq!(body["data"]["session_id"], 12);
        assert_eq!(body["data"]["peer_user_id"], 2);
    }

    #[tokio::test]
    async fn create_private_session_endpoint_rejects_missing_token() {
        let state = AppState::new_with_services(
            None,
            test_jwt_service(),
            Arc::new(UnavailableUserService),
            Arc::new(StubSessionService),
            Arc::new(UnavailableMessageService),
        );

        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/sessions/private")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "target_user_id": 2 }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn mark_session_read_endpoint_returns_read_state() {
        let jwt = test_jwt_service();
        let token = jwt.issue_token(1, "alice").unwrap();
        let state = AppState::new_with_services(
            None,
            jwt,
            Arc::new(UnavailableUserService),
            Arc::new(StubSessionService),
            Arc::new(UnavailableMessageService),
        );

        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/sessions/12/read")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "session marked as read");
        assert_eq!(body["data"]["session_id"], 12);
        assert_eq!(body["data"]["last_read_message_id"], 99);
    }

    #[tokio::test]
    async fn mark_session_read_endpoint_rejects_missing_token() {
        let state = AppState::new_with_services(
            None,
            test_jwt_service(),
            Arc::new(UnavailableUserService),
            Arc::new(StubSessionService),
            Arc::new(UnavailableMessageService),
        );

        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/sessions/12/read")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
