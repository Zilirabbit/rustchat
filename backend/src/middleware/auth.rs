use axum::{
    extract::{Request, State},
    http::{HeaderMap, header},
    middleware::Next,
    response::Response,
};

use crate::{
    app::AppState,
    auth::types::CurrentUser,
    common::error::{AppError, AppResult},
};

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer_token(request.headers())?;
    let claims = state.auth.jwt.decode_token(&token)?;
    request.extensions_mut().insert(CurrentUser::from(claims));

    Ok(next.run(request).await)
}

pub fn extract_bearer_token(headers: &HeaderMap) -> AppResult<String> {
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
