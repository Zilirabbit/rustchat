use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, header, request::Parts},
};

use crate::{
    app::AppState,
    common::error::{AppError, AppResult},
};

use super::jwt::AuthClaims;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentUser {
    pub user_id: i64,
    pub username: String,
}

impl From<AuthClaims> for CurrentUser {
    fn from(claims: AuthClaims) -> Self {
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
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_bearer_token(&parts.headers)?;
        let claims = state.auth.jwt.decode_token(&token)?;
        Ok(claims.into())
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> AppResult<String> {
    let header_value = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| AppError::Unauthorized("missing authorization header".to_string()))?;

    let header_value = header_value
        .to_str()
        .map_err(|_| AppError::Unauthorized("invalid authorization header".to_string()))?;

    let token = header_value
        .strip_prefix("Bearer ")
        .or_else(|| header_value.strip_prefix("bearer "))
        .ok_or_else(|| AppError::Unauthorized("invalid authorization header".to_string()))?;

    let token = token.trim();
    if token.is_empty() {
        return Err(AppError::Unauthorized(
            "invalid authorization header".to_string(),
        ));
    }

    Ok(token.to_string())
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, header};

    use super::extract_bearer_token;

    #[test]
    fn bearer_token_is_extracted() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer hello-world"),
        );

        let token = extract_bearer_token(&headers).unwrap();
        assert_eq!(token, "hello-world");
    }
}
