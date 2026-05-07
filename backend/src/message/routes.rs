use axum::{Router, middleware, routing::get};

use crate::{app::AppState, middleware::auth};

use super::handler;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/messages", get(handler::list_history_messages))
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
        conversation::service::UnavailableConversationService,
        message::{
            dto::{HistoryMessagesQuery, MessageListItem, MessageListPage},
            service::{MessageSendResult, MessageUseCase},
        },
        session::service::UnavailableSessionService,
        user::service::UnavailableUserService,
    };

    use super::router;

    struct StubMessageService;

    #[async_trait]
    impl MessageUseCase for StubMessageService {
        async fn send_text_message(
            &self,
            _current_user: &CurrentUser,
            _request: crate::message::dto::SendMessageRequest,
        ) -> AppResult<MessageSendResult> {
            unreachable!("route test does not send websocket messages")
        }

        async fn list_history_messages(
            &self,
            current_user: &CurrentUser,
            query: HistoryMessagesQuery,
        ) -> AppResult<MessageListPage> {
            Ok(MessageListPage {
                session_id: query.session_id,
                limit: query.limit,
                before_message_id: query.before_message_id,
                next_before_message_id: Some(9),
                has_more: true,
                messages: vec![MessageListItem {
                    message_id: 10,
                    session_id: query.session_id,
                    sender_id: current_user.user_id,
                    sender_username: current_user.username.clone(),
                    message_type: "text".to_string(),
                    content: "hello".to_string(),
                    created_at: "2026-05-03 12:00:00+00".to_string(),
                }],
            })
        }
    }

    fn test_jwt_service() -> JwtService {
        JwtService::new(JwtConfig {
            secret: "message-routes-test-secret".to_string(),
            expires_in_secs: 3_600,
            issuer: "rustchat-test".to_string(),
        })
    }

    fn test_state(jwt: JwtService) -> AppState {
        AppState::new_with_all_services(
            None,
            jwt,
            Arc::new(UnavailableUserService),
            Arc::new(UnavailableSessionService),
            Arc::new(StubMessageService),
            Arc::new(UnavailableConversationService),
        )
    }

    #[tokio::test]
    async fn list_history_messages_endpoint_returns_page() {
        let jwt = test_jwt_service();
        let token = jwt.issue_token(1, "alice").unwrap();
        let state = test_state(jwt);

        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/messages?session_id=12&limit=20")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "history messages fetched");
        assert_eq!(body["data"]["session_id"], 12);
        assert_eq!(body["data"]["limit"], 20);
        assert_eq!(body["data"]["messages"][0]["message_id"], 10);
        assert_eq!(body["data"]["messages"][0]["sender_username"], "alice");
    }

    #[tokio::test]
    async fn list_history_messages_endpoint_rejects_missing_token() {
        let state = test_state(test_jwt_service());

        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/messages?session_id=12&limit=20")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn list_history_messages_endpoint_requires_query_params() {
        let jwt = test_jwt_service();
        let token = jwt.issue_token(1, "alice").unwrap();
        let state = test_state(jwt);

        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/messages")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
