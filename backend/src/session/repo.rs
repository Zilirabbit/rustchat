use async_trait::async_trait;
use sqlx::{Row, postgres::PgRow};

use crate::{
    common::error::AppResult,
    storage::repository::{Repository, RepositoryContext},
};

use super::model::PrivateSession;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn user_exists(&self, user_id: i64) -> AppResult<bool>;
    async fn find_private_session_between(
        &self,
        user_id: i64,
        peer_user_id: i64,
    ) -> AppResult<Option<PrivateSession>>;
    async fn create_private_session(
        &self,
        created_by: i64,
        peer_user_id: i64,
    ) -> AppResult<PrivateSession>;
}

#[derive(Clone)]
pub struct PostgresSessionRepository {
    context: RepositoryContext,
}

impl PostgresSessionRepository {
    pub fn new(context: RepositoryContext) -> Self {
        Self { context }
    }
}

impl Repository for PostgresSessionRepository {
    fn context(&self) -> &RepositoryContext {
        &self.context
    }
}

#[async_trait]
impl SessionRepository for PostgresSessionRepository {
    async fn user_exists(&self, user_id: i64) -> AppResult<bool> {
        let row = sqlx::query("SELECT 1 FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(self.pool())
            .await?;

        Ok(row.is_some())
    }

    async fn find_private_session_between(
        &self,
        user_id: i64,
        peer_user_id: i64,
    ) -> AppResult<Option<PrivateSession>> {
        let row = sqlx::query(
            r#"
            SELECT s.id, s.created_by, s.created_at::text AS created_at
            FROM sessions s
            JOIN session_members sm1
              ON sm1.session_id = s.id
             AND sm1.user_id = $1
            JOIN session_members sm2
              ON sm2.session_id = s.id
             AND sm2.user_id = $2
            WHERE s.session_type = 'private'
              AND (
                    SELECT COUNT(*)
                    FROM session_members sm
                    WHERE sm.session_id = s.id
                  ) = 2
            ORDER BY s.id
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(peer_user_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(|row| map_private_session(row, peer_user_id))
            .transpose()
    }

    async fn create_private_session(
        &self,
        created_by: i64,
        peer_user_id: i64,
    ) -> AppResult<PrivateSession> {
        let mut tx = self.pool().begin().await?;

        let session_row = sqlx::query(
            r#"
            INSERT INTO sessions (session_type, created_by)
            VALUES ('private', $1)
            RETURNING id, created_by, created_at::text AS created_at
            "#,
        )
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        let session = map_private_session(session_row, peer_user_id)?;

        sqlx::query(
            r#"
            INSERT INTO session_members (session_id, user_id, role)
            VALUES
                ($1, $2, 'owner'),
                ($1, $3, 'member')
            "#,
        )
        .bind(session.session_id)
        .bind(created_by)
        .bind(peer_user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(session)
    }
}

fn map_private_session(row: PgRow, peer_user_id: i64) -> AppResult<PrivateSession> {
    Ok(PrivateSession {
        session_id: row.try_get("id")?,
        created_by: row.try_get("created_by")?,
        peer_user_id,
        created_at: row.try_get("created_at")?,
    })
}
