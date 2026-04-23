use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use crate::common::error::{AppError, AppResult};

#[derive(Clone)]
pub struct PasswordService {
    algorithm: Argon2<'static>,
}

impl PasswordService {
    pub fn new() -> Self {
        Self {
            algorithm: Argon2::default(),
        }
    }

    pub fn hash_password(&self, password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);

        self.algorithm
            .hash_password(password.as_bytes(), &salt)
            .map(|password_hash| password_hash.to_string())
            .map_err(|error| {
                AppError::internal(anyhow::anyhow!("failed to hash password: {error}"))
            })
    }

    pub fn verify_password(&self, password: &str, password_hash: &str) -> AppResult<bool> {
        let parsed_hash = PasswordHash::new(password_hash).map_err(|error| {
            AppError::internal(anyhow::anyhow!(
                "failed to parse stored password hash: {error}"
            ))
        })?;

        Ok(self
            .algorithm
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

impl Default for PasswordService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::PasswordService;

    #[test]
    fn hash_and_verify_password() {
        let service = PasswordService::new();
        let password_hash = service.hash_password("secret123").unwrap();

        assert_ne!(password_hash, "secret123");
        assert!(
            service
                .verify_password("secret123", &password_hash)
                .unwrap()
        );
        assert!(
            !service
                .verify_password("wrong-pass", &password_hash)
                .unwrap()
        );
    }
}
