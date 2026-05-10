use axum::{
    Router, middleware,
    routing::{delete, post},
};

use crate::{app::AppState, middleware::auth};

use super::handler;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/api/sessions/private",
            post(handler::create_private_session),
        )
        .route("/api/sessions/group", post(handler::create_group_session))
        .route(
            "/api/sessions/{session_id}/members",
            post(handler::add_group_member).get(handler::list_group_members),
        )
        .route(
            "/api/sessions/{session_id}/members/me",
            delete(handler::leave_group_session),
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
        dto::{
            AddGroupMemberRequest, AddGroupMemberResponse, CreateGroupSessionRequest,
            CreateGroupSessionResponse, CreatePrivateSessionRequest, CreatePrivateSessionResponse,
            GroupMemberListItem, LeaveGroupSessionResponse, ListGroupMembersResponse,
            MarkSessionReadResponse,
        },
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

        async fn create_group_session(
            &self,
            current_user: &CurrentUser,
            request: CreateGroupSessionRequest,
        ) -> AppResult<CreateGroupSessionResponse> {
            let mut member_user_ids = vec![current_user.user_id];
            member_user_ids.extend(request.member_user_ids);
            Ok(CreateGroupSessionResponse {
                session_id: 22,
                session_type: "group",
                name: request.name,
                created_by: current_user.user_id,
                member_user_ids,
                created_at: "2026-05-10 12:00:00+00".to_string(),
            })
        }

        async fn add_group_member(
            &self,
            _current_user: &CurrentUser,
            session_id: i64,
            request: AddGroupMemberRequest,
        ) -> AppResult<AddGroupMemberResponse> {
            Ok(AddGroupMemberResponse {
                session_id,
                user_id: request.user_id,
                role: "member".to_string(),
                joined_at: "2026-05-10 12:00:00+00".to_string(),
                added: true,
            })
        }

        async fn list_group_members(
            &self,
            _current_user: &CurrentUser,
            session_id: i64,
        ) -> AppResult<ListGroupMembersResponse> {
            Ok(ListGroupMembersResponse {
                session_id,
                members: vec![
                    GroupMemberListItem {
                        user_id: 1,
                        username: "alice".to_string(),
                        role: "owner".to_string(),
                        joined_at: "2026-05-10 12:00:00+00".to_string(),
                    },
                    GroupMemberListItem {
                        user_id: 2,
                        username: "bob".to_string(),
                        role: "member".to_string(),
                        joined_at: "2026-05-10 12:01:00+00".to_string(),
                    },
                ],
            })
        }

        async fn leave_group_session(
            &self,
            current_user: &CurrentUser,
            session_id: i64,
        ) -> AppResult<LeaveGroupSessionResponse> {
            Ok(LeaveGroupSessionResponse {
                session_id,
                user_id: current_user.user_id,
                left: true,
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
    async fn create_group_session_endpoint_returns_session() {
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
                    .uri("/api/sessions/group")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({ "name": "team", "member_user_ids": [2, 3] }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "group session created");
        assert_eq!(body["data"]["session_id"], 22);
        assert_eq!(body["data"]["session_type"], "group");
        assert_eq!(body["data"]["name"], "team");
    }

    #[tokio::test]
    async fn add_group_member_endpoint_returns_member() {
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
                    .uri("/api/sessions/22/members")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "user_id": 2 }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "group member ready");
        assert_eq!(body["data"]["session_id"], 22);
        assert_eq!(body["data"]["user_id"], 2);
        assert_eq!(body["data"]["added"], true);
    }

    #[tokio::test]
    async fn leave_group_session_endpoint_returns_left_state() {
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
                    .method("DELETE")
                    .uri("/api/sessions/22/members/me")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "group session left");
        assert_eq!(body["data"]["session_id"], 22);
        assert_eq!(body["data"]["user_id"], 1);
        assert_eq!(body["data"]["left"], true);
    }

    #[tokio::test]
    async fn list_group_members_endpoint_returns_members() {
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
                    .method("GET")
                    .uri("/api/sessions/22/members")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "group members ready");
        assert_eq!(body["data"]["session_id"], 22);
        assert_eq!(body["data"]["members"].as_array().unwrap().len(), 2);
        assert_eq!(body["data"]["members"][0]["user_id"], 1);
        assert_eq!(body["data"]["members"][0]["username"], "alice");
        assert_eq!(body["data"]["members"][0]["role"], "owner");
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
