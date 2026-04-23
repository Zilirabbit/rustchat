use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::common::{
    config::JwtConfig,
    error::{AppError, AppResult},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthClaims {
    pub sub: i64,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
}

#[derive(Clone)]
pub struct JwtService {
    decoding_key: DecodingKey,
    encoding_key: EncodingKey,
    expires_in_secs: u64,
    issuer: String,
}

impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        Self {
            decoding_key: DecodingKey::from_secret(config.secret.as_bytes()),
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            expires_in_secs: config.expires_in_secs,
            issuer: config.issuer,
        }
    }

    pub fn issue_token(&self, user_id: i64, username: &str) -> AppResult<String> {
        let issued_at = unix_timestamp_now();
        let claims = AuthClaims {
            sub: user_id,
            username: username.to_string(),
            exp: issued_at + self.expires_in_secs as usize,
            iat: issued_at,
            iss: self.issuer.clone(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|error| AppError::internal(anyhow::anyhow!("failed to issue jwt: {error}")))
    }

    pub fn decode_token(&self, token: &str) -> AppResult<AuthClaims> {
        decode::<AuthClaims>(token, &self.decoding_key, &self.validation())
            .map(|data| data.claims)
            .map_err(|_| AppError::Unauthorized("invalid or expired token".to_string()))
    }

    fn validation(&self) -> Validation {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.set_issuer(&[self.issuer.as_str()]);
        validation
    }
}

fn unix_timestamp_now() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_secs() as usize
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use super::JwtService;
    use crate::common::config::JwtConfig;

    #[test]
    fn issue_and_decode_token() {
        let service = JwtService::new(JwtConfig {
            secret: "jwt-test-secret".to_string(),
            expires_in_secs: 3_600,
            issuer: "rustchat-test".to_string(),
        });

        let token = service.issue_token(42, "alice").unwrap();
        let claims = service.decode_token(&token).unwrap();

        assert_eq!(claims.sub, 42);
        assert_eq!(claims.username, "alice");
        assert_eq!(claims.iss, "rustchat-test");
    }

    #[test]
    fn invalid_token_is_rejected() {
        let service = JwtService::new(JwtConfig {
            secret: "jwt-test-secret".to_string(),
            expires_in_secs: 3_600,
            issuer: "rustchat-test".to_string(),
        });

        let error = service.decode_token("not-a-token").unwrap_err();
        assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
    }
}
