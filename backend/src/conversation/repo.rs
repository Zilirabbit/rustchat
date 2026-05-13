use async_trait::async_trait;
use sqlx::{Row, postgres::PgRow};

use crate::{
    common::error::AppResult,
    storage::repository::{Repository, RepositoryContext},
};

use super::model::ConversationSummary;

#[async_trait]
pub trait ConversationRepository: Send + Sync {
    async fn list_for_user(&self, user_id: i64) -> AppResult<Vec<ConversationSummary>>;
}

#[derive(Clone)]
pub struct PostgresConversationRepository {
    context: RepositoryContext,
}

impl PostgresConversationRepository {
    pub fn new(context: RepositoryContext) -> Self {
        Self { context }
    }
}

impl Repository for PostgresConversationRepository {
    fn context(&self) -> &RepositoryContext {
        &self.context
    }
}

#[async_trait]
impl ConversationRepository for PostgresConversationRepository {
    async fn list_for_user(&self, user_id: i64) -> AppResult<Vec<ConversationSummary>> {
        let rows = sqlx::query(
            r#"
            SELECT
                s.id AS session_id,
                s.session_type,
                COALESCE(
                    CASE
                        WHEN s.session_type = 'private' THEN peer.username
                        ELSE s.name
                    END,
                    'unknown session'
                ) AS session_name,
                CASE
                    WHEN last_message.message_type = 'file'
                    THEN COALESCE(
                        last_message.content::json ->> 'file_name',
                        last_message.content
                    )
                    ELSE last_message.content
                END AS last_message,
                s.last_message_at::text AS last_message_time,
                COALESCE(unread.unread_count, 0) AS unread_count
            FROM session_members sm
            JOIN sessions s
              ON s.id = sm.session_id
            LEFT JOIN messages last_message
              ON last_message.session_id = s.id
             AND last_message.id = s.last_message_id
            LEFT JOIN session_members peer_member
              ON s.session_type = 'private'
             AND peer_member.session_id = s.id
             AND peer_member.user_id <> $1
            LEFT JOIN users peer
              ON peer.id = peer_member.user_id
            LEFT JOIN user_session_read_state read_state
              ON read_state.user_id = $1
             AND read_state.session_id = s.id
            LEFT JOIN LATERAL (
                SELECT COUNT(*)::BIGINT AS unread_count
                FROM messages unread_message
                WHERE unread_message.session_id = s.id
                  AND unread_message.sender_id <> $1
                  AND (
                        read_state.last_read_message_id IS NULL
                        OR unread_message.id > read_state.last_read_message_id
                  )
            ) unread ON TRUE
            WHERE sm.user_id = $1
            ORDER BY s.last_message_at DESC NULLS LAST, s.id DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(map_conversation_summary).collect()
    }
}

fn map_conversation_summary(row: PgRow) -> AppResult<ConversationSummary> {
    Ok(ConversationSummary {
        session_id: row.try_get("session_id")?,
        session_type: row.try_get("session_type")?,
        session_name: row.try_get("session_name")?,
        last_message: row.try_get("last_message")?,
        last_message_time: row.try_get("last_message_time")?,
        unread_count: row.try_get("unread_count")?,
    })
}
