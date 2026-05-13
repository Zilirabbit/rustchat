use async_trait::async_trait;
use sqlx::Row;

use crate::common::error::AppResult;
use crate::storage::repository::{Repository, RepositoryContext};

use super::model::FileRecord;

#[async_trait]
pub trait FileRepository: Send + Sync {
    async fn create_file_upload_message(
        &self,
        session_id: i64,
        sender_id: i64,
        file_name: &str,
        file_size: i64,
        file_type: &str,
        file_hash: &str,
        storage_path: &str,
    ) -> AppResult<(FileRecord, i64, String)>;
    async fn get_file(&self, file_id: i64) -> AppResult<Option<FileRecord>>;
    async fn list_expired_files(&self) -> AppResult<Vec<FileRecord>>;
    async fn delete_file_record(&self, file_id: i64) -> AppResult<()>;
    async fn get_session_member_ids(
        &self,
        session_id: i64,
        exclude_user_id: i64,
    ) -> AppResult<Vec<i64>>;
    async fn is_session_member(&self, session_id: i64, user_id: i64) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct PostgresFileRepository {
    context: RepositoryContext,
}

impl PostgresFileRepository {
    pub fn new(context: RepositoryContext) -> Self {
        Self { context }
    }
}

impl Repository for PostgresFileRepository {
    fn context(&self) -> &RepositoryContext {
        &self.context
    }
}

#[async_trait]
impl FileRepository for PostgresFileRepository {
    async fn create_file_upload_message(
        &self,
        session_id: i64,
        sender_id: i64,
        file_name: &str,
        file_size: i64,
        file_type: &str,
        file_hash: &str,
        storage_path: &str,
    ) -> AppResult<(FileRecord, i64, String)> {
        let mut tx = self.pool().begin().await?;

        let row = sqlx::query(
            r#"
            INSERT INTO files (session_id, sender_id, file_name, file_size, file_type, file_hash, storage_path)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id AS file_id,
                session_id,
                sender_id,
                file_name,
                file_size,
                file_type,
                file_hash,
                storage_path,
                created_at::text AS created_at,
                expires_at::text AS expires_at
            "#,
        )
        .bind(session_id)
        .bind(sender_id)
        .bind(file_name)
        .bind(file_size)
        .bind(file_type)
        .bind(file_hash)
        .bind(storage_path)
        .fetch_one(&mut *tx)
        .await?;

        let file_record = map_file_record(row);
        let file_content = serde_json::json!({
            "file_id": file_record.file_id,
            "file_name": file_record.file_name,
            "file_size": file_record.file_size,
            "file_type": file_record.file_type,
        })
        .to_string();

        let row = sqlx::query(
            r#"
            INSERT INTO messages (session_id, sender_id, message_type, content)
            VALUES ($1, $2, 'file', $3)
            RETURNING id
            "#,
        )
        .bind(session_id)
        .bind(sender_id)
        .bind(&file_content)
        .fetch_one(&mut *tx)
        .await?;
        let message_id: i64 = row.get("id");

        sqlx::query(
            r#"
            UPDATE sessions
            SET last_message_id = $2,
                last_message_at = (SELECT created_at FROM messages WHERE session_id = $1 AND id = $2)
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .bind(message_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok((file_record, message_id, file_content))
    }

    async fn get_file(&self, file_id: i64) -> AppResult<Option<FileRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id AS file_id,
                session_id,
                sender_id,
                file_name,
                file_size,
                file_type,
                file_hash,
                storage_path,
                created_at::text AS created_at,
                expires_at::text AS expires_at
            FROM files
            WHERE id = $1 AND expires_at >= NOW()
            "#,
        )
        .bind(file_id)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.map(map_file_record))
    }

    async fn list_expired_files(&self) -> AppResult<Vec<FileRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id AS file_id,
                session_id,
                sender_id,
                file_name,
                file_size,
                file_type,
                file_hash,
                storage_path,
                created_at::text AS created_at,
                expires_at::text AS expires_at
            FROM files
            WHERE expires_at < NOW()
            "#,
        )
        .fetch_all(self.pool())
        .await?;

        Ok(rows.into_iter().map(map_file_record).collect())
    }

    async fn delete_file_record(&self, file_id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM files WHERE id = $1")
            .bind(file_id)
            .execute(self.pool())
            .await?;

        Ok(())
    }

    async fn get_session_member_ids(
        &self,
        session_id: i64,
        exclude_user_id: i64,
    ) -> AppResult<Vec<i64>> {
        let rows = sqlx::query(
            r#"
            SELECT user_id
            FROM session_members
            WHERE session_id = $1 AND user_id <> $2
            ORDER BY user_id
            "#,
        )
        .bind(session_id)
        .bind(exclude_user_id)
        .fetch_all(self.pool())
        .await?;

        Ok(rows.into_iter().map(|r| r.get("user_id")).collect())
    }

    async fn is_session_member(&self, session_id: i64, user_id: i64) -> AppResult<bool> {
        let row = sqlx::query(
            r#"
            SELECT 1
            FROM session_members
            WHERE session_id = $1 AND user_id = $2
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.is_some())
    }
}

fn map_file_record(row: sqlx::postgres::PgRow) -> FileRecord {
    FileRecord {
        file_id: row.get("file_id"),
        session_id: row.get("session_id"),
        sender_id: row.get("sender_id"),
        file_name: row.get("file_name"),
        file_size: row.get("file_size"),
        file_type: row.get("file_type"),
        file_hash: row.get("file_hash"),
        storage_path: row.get("storage_path"),
        created_at: row.get("created_at"),
        expires_at: row.get("expires_at"),
    }
}
