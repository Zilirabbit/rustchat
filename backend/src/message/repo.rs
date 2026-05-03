use async_trait::async_trait;
use sqlx::{Row, postgres::PgRow};

use crate::{
    common::error::AppResult,
    storage::repository::{Repository, RepositoryContext},
};

use super::model::{PrivateSessionAccess, StoredMessage};

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn get_private_session_access(
        &self,
        session_id: i64,
        sender_id: i64,
    ) -> AppResult<Option<PrivateSessionAccess>>;
    async fn create_text_message(
        &self,
        session_id: i64,
        sender_id: i64,
        content: &str,
    ) -> AppResult<StoredMessage>;
}

#[derive(Clone)]
pub struct PostgresMessageRepository {
    context: RepositoryContext,
}

impl PostgresMessageRepository {
    pub fn new(context: RepositoryContext) -> Self {
        Self { context }
    }
}

impl Repository for PostgresMessageRepository {
    fn context(&self) -> &RepositoryContext {
        &self.context
    }
}

#[async_trait]
impl MessageRepository for PostgresMessageRepository {
    async fn get_private_session_access(
        &self,
        session_id: i64,
        sender_id: i64,
    ) -> AppResult<Option<PrivateSessionAccess>> {
        let row = sqlx::query(
            r#"
            SELECT s.id AS session_id, sm_other.user_id AS recipient_user_id
            FROM sessions s
            JOIN session_members sm_self
              ON sm_self.session_id = s.id
             AND sm_self.user_id = $2
            JOIN session_members sm_other
              ON sm_other.session_id = s.id
             AND sm_other.user_id <> $2
            WHERE s.id = $1
              AND s.session_type = 'private'
            LIMIT 1
            "#,
        )
        .bind(session_id)
        .bind(sender_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(map_private_session_access).transpose()
    }

    async fn create_text_message(
        &self,
        session_id: i64,
        sender_id: i64,
        content: &str,
    ) -> AppResult<StoredMessage> {
        let mut tx = self.pool().begin().await?;

        let message_row = sqlx::query(
            r#"
            INSERT INTO messages (session_id, sender_id, message_type, content)
            VALUES ($1, $2, 'text', $3)
            RETURNING id, session_id, sender_id, content, created_at::text AS created_at
            "#,
        )
        .bind(session_id)
        .bind(sender_id)
        .bind(content)
        .fetch_one(&mut *tx)
        .await?;

        let message = map_stored_message(message_row)?;

        sqlx::query(
            r#"
            UPDATE sessions
            SET last_message_id = $2,
                last_message_at = (
                    SELECT created_at
                    FROM messages
                    WHERE session_id = $1
                      AND id = $2
                )
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .bind(message.message_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(message)
    }
}

fn map_private_session_access(row: PgRow) -> AppResult<PrivateSessionAccess> {
    Ok(PrivateSessionAccess {
        session_id: row.try_get("session_id")?,
        recipient_user_id: row.try_get("recipient_user_id")?,
    })
}

fn map_stored_message(row: PgRow) -> AppResult<StoredMessage> {
    Ok(StoredMessage {
        message_id: row.try_get("id")?,
        session_id: row.try_get("session_id")?,
        sender_id: row.try_get("sender_id")?,
        content: row.try_get("content")?,
        created_at: row.try_get("created_at")?,
    })
}
