use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::{app::AppState, middleware::auth};

use super::handler;

pub fn router(state: AppState) -> Router<AppState> {
    let protected_routes = Router::new()
        .route("/api/me", get(handler::me))
        .route("/api/users", get(handler::search_users))
        .route_layer(middleware::from_fn_with_state(state, auth::require_auth));

    Router::new()
        .route("/api/register", post(handler::register))
        .route("/api/login", post(handler::login))
        .merge(protected_routes)
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
        auth::jwt::JwtService,
        common::{config::JwtConfig, error::AppResult},
    };

    use super::super::{
        dto::{AuthResponse, LoginRequest, RegisterRequest, SearchUsersQuery, UserSearchItem},
        model::UserProfile,
        service::UserUseCase,
    };
    use super::router;

    struct StubUserService;

    #[async_trait]
    impl UserUseCase for StubUserService {
        async fn register(&self, request: RegisterRequest) -> AppResult<UserProfile> {
            Ok(UserProfile {
                user_id: 1,
                username: request.username.trim().to_string(),
                avatar_url: None,
            })
        }

        async fn login(&self, request: LoginRequest) -> AppResult<AuthResponse> {
            Ok(AuthResponse {
                token: format!("token-for-{}", request.username.trim()),
                user: UserProfile {
                    user_id: 1,
                    username: request.username.trim().to_string(),
                    avatar_url: None,
                },
            })
        }

        async fn get_user_by_id(&self, user_id: i64) -> AppResult<UserProfile> {
            Ok(UserProfile {
                user_id,
                username: "alice".to_string(),
                avatar_url: None,
            })
        }

        async fn search_users(
            &self,
            _current_user: &crate::auth::types::CurrentUser,
            query: SearchUsersQuery,
        ) -> AppResult<Vec<UserSearchItem>> {
            Ok(vec![UserSearchItem {
                user_id: 2,
                username: format!("{}-bob", query.keyword.trim()),
            }])
        }
    }

    fn test_jwt_service() -> JwtService {
        JwtService::new(JwtConfig {
            secret: "user-routes-test-secret".to_string(),
            expires_in_secs: 3_600,
            issuer: "rustchat-test".to_string(),
        })
    }

    fn test_state() -> AppState {
        AppState::new(None, test_jwt_service(), Arc::new(StubUserService))
    }

    #[tokio::test]
    async fn register_endpoint_returns_user_profile() {
        let state = test_state();
        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/register")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "username": "alice",
                            "password": "secret123"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "user registered");
        assert_eq!(body["data"]["user_id"], 1);
        assert_eq!(body["data"]["username"], "alice");
    }

    #[tokio::test]
    async fn login_endpoint_returns_token() {
        let state = test_state();
        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "username": "alice",
                            "password": "secret123"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "login succeeded");
        assert_eq!(body["data"]["token"], "token-for-alice");
        assert_eq!(body["data"]["user"]["username"], "alice");
    }

    #[tokio::test]
    async fn me_endpoint_returns_current_user() {
        let jwt = test_jwt_service();
        let token = jwt.issue_token(7, "alice").unwrap();
        let state = AppState::new(None, jwt, Arc::new(StubUserService));

        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/me")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["data"]["user_id"], 7);
        assert_eq!(body["data"]["username"], "alice");
    }

    #[tokio::test]
    async fn me_endpoint_rejects_missing_authorization_header() {
        let state = test_state();
        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn me_endpoint_rejects_invalid_token() {
        let state = test_state();
        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/me")
                    .header(header::AUTHORIZATION, "Bearer invalid-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn search_users_endpoint_returns_matches() {
        let jwt = test_jwt_service();
        let token = jwt.issue_token(1, "alice").unwrap();
        let state = AppState::new(None, jwt, Arc::new(StubUserService));

        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/users?keyword=bo")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "users searched");
        assert_eq!(body["data"][0]["user_id"], 2);
        assert_eq!(body["data"][0]["username"], "bo-bob");
    }

    #[tokio::test]
    async fn search_users_endpoint_rejects_missing_token() {
        let state = test_state();
        let response = router(state.clone())
            .with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/users?keyword=bo")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
