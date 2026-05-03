use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{app::AppState, common::error::AppError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentUser {
    pub user_id: i64,
    pub username: String,
}

impl From<super::jwt::AuthClaims> for CurrentUser {
    fn from(claims: super::jwt::AuthClaims) -> Self {
        Self {
            user_id: claims.sub,
            username: claims.username,
        }
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("authentication context missing".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        extract::FromRequestParts,
        http::{Request, request::Parts},
    };

    use super::CurrentUser;
    use crate::{
        app::AppState, auth::jwt::JwtService, common::config::JwtConfig,
        user::service::UnavailableUserService,
    };

    fn test_state() -> AppState {
        AppState::new(
            None,
            JwtService::new(JwtConfig {
                secret: "auth-types-test-secret".to_string(),
                expires_in_secs: 3_600,
                issuer: "rustchat-test".to_string(),
            }),
            Arc::new(UnavailableUserService),
        )
    }

    fn empty_parts() -> Parts {
        let (parts, _) = Request::builder()
            .uri("/api/me")
            .body(())
            .unwrap()
            .into_parts();
        parts
    }

    #[tokio::test]
    async fn current_user_is_loaded_from_request_extensions() {
        let mut parts = empty_parts();
        parts.extensions.insert(CurrentUser {
            user_id: 42,
            username: "alice".to_string(),
        });

        let current_user = CurrentUser::from_request_parts(&mut parts, &test_state())
            .await
            .unwrap();

        assert_eq!(current_user.user_id, 42);
        assert_eq!(current_user.username, "alice");
    }

    #[tokio::test]
    async fn missing_current_user_extension_is_rejected() {
        let mut parts = empty_parts();
        let error = CurrentUser::from_request_parts(&mut parts, &test_state())
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::UNAUTHORIZED);
    }
}
