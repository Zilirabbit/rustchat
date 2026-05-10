use async_trait::async_trait;
use sqlx::{Row, postgres::PgRow};

use crate::{
    common::error::AppResult,
    storage::repository::{Repository, RepositoryContext},
};

use super::model::{HistoryMessage, SessionMessageAccess, StoredMessage};

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn is_session_member(&self, session_id: i64, user_id: i64) -> AppResult<bool>;
    async fn get_session_message_access(
        &self,
        session_id: i64,
        sender_id: i64,
    ) -> AppResult<Option<SessionMessageAccess>>;
    async fn create_text_message(
        &self,
        session_id: i64,
        sender_id: i64,
        content: &str,
    ) -> AppResult<StoredMessage>;
    async fn list_session_messages(
        &self,
        session_id: i64,
        before_message_id: Option<i64>,
        limit: i64,
    ) -> AppResult<Vec<HistoryMessage>>;
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
    async fn is_session_member(&self, session_id: i64, user_id: i64) -> AppResult<bool> {
        let row = sqlx::query(
            r#"
            SELECT 1
            FROM session_members
            WHERE session_id = $1
              AND user_id = $2
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.is_some())
    }

    async fn get_session_message_access(
        &self,
        session_id: i64,
        sender_id: i64,
    ) -> AppResult<Option<SessionMessageAccess>> {
        let row = sqlx::query(
            r#"
            SELECT
                s.id AS session_id,
                COALESCE(
                    ARRAY_AGG(sm_other.user_id ORDER BY sm_other.user_id)
                        FILTER (WHERE sm_other.user_id IS NOT NULL),
                    ARRAY[]::BIGINT[]
                ) AS recipient_user_ids
            FROM sessions s
            JOIN session_members sm_self
              ON sm_self.session_id = s.id
             AND sm_self.user_id = $2
            LEFT JOIN session_members sm_other
              ON sm_other.session_id = s.id
             AND sm_other.user_id <> $2
            WHERE s.id = $1
            GROUP BY s.id
            "#,
        )
        .bind(session_id)
        .bind(sender_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(map_session_message_access).transpose()
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

    async fn list_session_messages(
        &self,
        session_id: i64,
        before_message_id: Option<i64>,
        limit: i64,
    ) -> AppResult<Vec<HistoryMessage>> {
        let rows = sqlx::query(
            r#"
            SELECT
                m.id AS message_id,
                m.session_id,
                m.sender_id,
                u.username AS sender_username,
                m.message_type,
                m.content,
                m.created_at::text AS created_at
            FROM messages m
            JOIN users u
              ON u.id = m.sender_id
            WHERE m.session_id = $1
              AND ($2::BIGINT IS NULL OR m.id < $2)
            ORDER BY m.id DESC
            LIMIT $3
            "#,
        )
        .bind(session_id)
        .bind(before_message_id)
        .bind(limit)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(map_history_message).collect()
    }
}

fn map_session_message_access(row: PgRow) -> AppResult<SessionMessageAccess> {
    Ok(SessionMessageAccess {
        session_id: row.try_get("session_id")?,
        recipient_user_ids: row.try_get("recipient_user_ids")?,
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

fn map_history_message(row: PgRow) -> AppResult<HistoryMessage> {
    Ok(HistoryMessage {
        message_id: row.try_get("message_id")?,
        session_id: row.try_get("session_id")?,
        sender_id: row.try_get("sender_id")?,
        sender_username: row.try_get("sender_username")?,
        message_type: row.try_get("message_type")?,
        content: row.try_get("content")?,
        created_at: row.try_get("created_at")?,
    })
}
