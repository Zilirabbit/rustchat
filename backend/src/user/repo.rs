use async_trait::async_trait;
use sqlx::{Row, postgres::PgRow};

use crate::{
    common::error::{AppError, AppResult},
    storage::repository::{Repository, RepositoryContext},
};

use super::model::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, username: &str, password_hash: &str) -> AppResult<User>;
    async fn find_by_id(&self, user_id: i64) -> AppResult<Option<User>>;
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
}

#[derive(Clone)]
pub struct PostgresUserRepository {
    context: RepositoryContext,
}

impl PostgresUserRepository {
    pub fn new(context: RepositoryContext) -> Self {
        Self { context }
    }
}

impl Repository for PostgresUserRepository {
    fn context(&self) -> &RepositoryContext {
        &self.context
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create_user(&self, username: &str, password_hash: &str) -> AppResult<User> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (username, password_hash)
            VALUES ($1, $2)
            RETURNING id, username, password_hash, avatar_url
            "#,
        )
        .bind(username)
        .bind(password_hash)
        .fetch_one(self.pool())
        .await
        .map_err(map_create_user_error)?;

        map_user(row)
    }

    async fn find_by_id(&self, user_id: i64) -> AppResult<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, username, password_hash, avatar_url
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(map_user).transpose()
    }

    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, username, password_hash, avatar_url
            FROM users
            WHERE LOWER(username) = LOWER($1)
            "#,
        )
        .bind(username)
        .fetch_optional(self.pool())
        .await?;

        row.map(map_user).transpose()
    }
}

fn map_user(row: PgRow) -> AppResult<User> {
    Ok(User {
        id: row.try_get("id")?,
        username: row.try_get("username")?,
        password_hash: row.try_get("password_hash")?,
        avatar_url: row.try_get("avatar_url")?,
    })
}

fn map_create_user_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error.constraint() == Some("users_username_lower_uidx") {
            return AppError::Conflict("username already exists".to_string());
        }
    }

    AppError::Db(error)
}
