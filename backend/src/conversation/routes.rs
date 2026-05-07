use axum::{Router, middleware, routing::get};

use crate::{app::AppState, middleware::auth};

use super::handler;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/conversations", get(handler::list_conversations))
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
    use serde_json::Value;
    use tower::util::ServiceExt;

    use crate::{
        app::AppState,
        auth::{jwt::JwtService, types::CurrentUser},
        common::{config::JwtConfig, error::AppResult},
        conversation::{dto::ConversationItem, service::ConversationUseCase},
        message::service::UnavailableMessageService,
        session::service::UnavailableSessionService,
        user::service::UnavailableUserService,
    };

    use super::router;

    struct StubConversationService;

    #[async_trait]
    impl ConversationUseCase for StubConversationService {
        async fn list_conversations(
            &self,
            current_user: &CurrentUser,
        ) -> AppResult<Vec<ConversationItem>> {
            Ok(vec![ConversationItem {
                session_id: 12,
                session_type: "private".to_string(),
                session_name: format!("{}-peer", current_user.username),
                last_message: Some("hello".to_string()),
                last_message_time: Some("2026-05-03 12:00:00+00".to_string()),
                unread_count: 3,
            }])
        }
    }

    fn test_jwt_service() -> JwtService {
        JwtService::new(JwtConfig {
            secret: "conversation-routes-test-secret".to_string(),
            expires_in_secs: 3_600,
            issuer: "rustchat-test".to_string(),
        })
    }

    #[tokio::test]
    async fn list_conversations_endpoint_returns_items() {
        let jwt = test_jwt_service();
        let token = jwt.issue_token(1, "alice").unwrap();
        let state = AppState::new_with_all_services(
            None,
            jwt,
            Arc::new(UnavailableUserService),
            Arc::new(UnavailableSessionService),
            Arc::new(UnavailableMessageService),
            Arc::new(StubConversationService),
        );

        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/conversations")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "conversations fetched");
        assert_eq!(body["data"][0]["session_id"], 12);
        assert_eq!(body["data"][0]["session_name"], "alice-peer");
        assert_eq!(body["data"][0]["unread_count"], 3);
    }

    #[tokio::test]
    async fn list_conversations_endpoint_rejects_missing_token() {
        let state = AppState::new_with_all_services(
            None,
            test_jwt_service(),
            Arc::new(UnavailableUserService),
            Arc::new(UnavailableSessionService),
            Arc::new(UnavailableMessageService),
            Arc::new(StubConversationService),
        );

        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/conversations")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
