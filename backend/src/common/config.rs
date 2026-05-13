use std::env;

use crate::common::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub app_host: String,
    pub app_port: u16,
    pub log_level: String,
    pub database: Option<DatabaseConfig>,
    pub jwt: JwtConfig,
    pub upload_dir: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expires_in_secs: u64,
    pub issuer: String,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        let app_host = env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let app_port = env::var("APP_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| AppError::Config("APP_PORT must be a valid u16".to_string()))?;

        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        let database = env::var("DATABASE_URL").ok().and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        let database = match database {
            Some(url) => Some(DatabaseConfig {
                url,
                max_connections: read_u32_env("DATABASE_MAX_CONNECTIONS", 10)?,
                min_connections: read_u32_env("DATABASE_MIN_CONNECTIONS", 1)?,
                acquire_timeout_secs: read_u64_env("DATABASE_ACQUIRE_TIMEOUT_SECS", 5)?,
            }),
            None => None,
        };

        let jwt = JwtConfig {
            secret: read_string_env("JWT_SECRET", "rustchat-dev-secret"),
            expires_in_secs: read_u64_env("JWT_EXPIRES_IN_SECS", 86_400)?,
            issuer: read_string_env("JWT_ISSUER", "rustchat"),
        };

        let upload_dir = read_string_env("UPLOAD_DIR", "./uploads");

        Ok(Self {
            app_host,
            app_port,
            log_level,
            database,
            jwt,
            upload_dir,
        })
    }
}

fn read_string_env(key: &str, default: &str) -> String {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn read_u32_env(key: &str, default: u32) -> AppResult<u32> {
    env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse::<u32>()
                .map_err(|_| AppError::Config(format!("{key} must be a valid u32")))
        })
        .transpose()
        .map(|value| value.unwrap_or(default))
}

fn read_u64_env(key: &str, default: u64) -> AppResult<u64> {
    env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| AppError::Config(format!("{key} must be a valid u64")))
        })
        .transpose()
        .map(|value| value.unwrap_or(default))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_reads_database_pool_settings() {
        unsafe {
            env::set_var("APP_HOST", "127.0.0.1");
            env::set_var("APP_PORT", "3001");
            env::set_var("RUST_LOG", "debug");
            env::set_var(
                "DATABASE_URL",
                "postgres://postgres:postgres@localhost:5432/rustchat",
            );
            env::set_var("DATABASE_MAX_CONNECTIONS", "20");
            env::set_var("DATABASE_MIN_CONNECTIONS", "2");
            env::set_var("DATABASE_ACQUIRE_TIMEOUT_SECS", "9");
            env::set_var("JWT_SECRET", "super-secret");
            env::set_var("JWT_EXPIRES_IN_SECS", "7200");
            env::set_var("JWT_ISSUER", "rustchat-test");
        }

        let config = AppConfig::from_env().unwrap();
        let database = config.database.expect("database config should exist");

        assert_eq!(config.app_host, "127.0.0.1");
        assert_eq!(config.app_port, 3001);
        assert_eq!(config.log_level, "debug");
        assert_eq!(
            database.url,
            "postgres://postgres:postgres@localhost:5432/rustchat"
        );
        assert_eq!(database.max_connections, 20);
        assert_eq!(database.min_connections, 2);
        assert_eq!(database.acquire_timeout_secs, 9);
        assert_eq!(config.jwt.secret, "super-secret");
        assert_eq!(config.jwt.expires_in_secs, 7200);
        assert_eq!(config.jwt.issuer, "rustchat-test");

        unsafe {
            env::remove_var("APP_HOST");
            env::remove_var("APP_PORT");
            env::remove_var("RUST_LOG");
            env::remove_var("DATABASE_URL");
            env::remove_var("DATABASE_MAX_CONNECTIONS");
            env::remove_var("DATABASE_MIN_CONNECTIONS");
            env::remove_var("DATABASE_ACQUIRE_TIMEOUT_SECS");
            env::remove_var("JWT_SECRET");
            env::remove_var("JWT_EXPIRES_IN_SECS");
            env::remove_var("JWT_ISSUER");
        }
    }
}
