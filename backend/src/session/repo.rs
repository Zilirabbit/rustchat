use async_trait::async_trait;
use sqlx::{Row, postgres::PgRow};

use crate::{
    common::error::AppResult,
    storage::repository::{Repository, RepositoryContext},
};

use super::model::{
    GroupSession, GroupSessionMember, PrivateSession, SessionMember, SessionReadState,
};

pub struct CreatePrivateSessionResult {
    pub session: PrivateSession,
    pub created: bool,
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn user_exists(&self, user_id: i64) -> AppResult<bool>;
    async fn is_session_member(&self, session_id: i64, user_id: i64) -> AppResult<bool>;
    async fn get_session_last_message_id(&self, session_id: i64) -> AppResult<Option<i64>>;
    async fn find_private_session_between(
        &self,
        user_id: i64,
        peer_user_id: i64,
    ) -> AppResult<Option<PrivateSession>>;
    async fn create_private_session(
        &self,
        created_by: i64,
        peer_user_id: i64,
    ) -> AppResult<CreatePrivateSessionResult>;
    async fn create_group_session(
        &self,
        created_by: i64,
        name: &str,
        member_user_ids: &[i64],
    ) -> AppResult<GroupSession>;
    async fn get_group_member(
        &self,
        session_id: i64,
        user_id: i64,
    ) -> AppResult<Option<SessionMember>>;
    async fn list_group_members(&self, session_id: i64) -> AppResult<Vec<GroupSessionMember>>;
    async fn add_group_member(&self, session_id: i64, user_id: i64) -> AppResult<SessionMember>;
    async fn leave_group_session(&self, session_id: i64, user_id: i64) -> AppResult<bool>;
    async fn mark_session_read(
        &self,
        session_id: i64,
        user_id: i64,
        last_read_message_id: Option<i64>,
    ) -> AppResult<SessionReadState>;
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

    async fn get_session_last_message_id(&self, session_id: i64) -> AppResult<Option<i64>> {
        let row = sqlx::query(
            r#"
            SELECT last_message_id
            FROM sessions
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(self.pool())
        .await?;

        Ok(row
            .map(|row| row.try_get::<Option<i64>, _>("last_message_id"))
            .transpose()?
            .flatten())
    }

    async fn find_private_session_between(
        &self,
        user_id: i64,
        peer_user_id: i64,
    ) -> AppResult<Option<PrivateSession>> {
        let (user_low_id, user_high_id) = ordered_user_pair(user_id, peer_user_id);
        let row = sqlx::query(
            r#"
            SELECT s.id, s.created_by, s.created_at::text AS created_at
            FROM sessions s
            JOIN private_session_pairs p
              ON p.session_id = s.id
            WHERE p.user_low_id = $1
              AND p.user_high_id = $2
              AND s.session_type = 'private'
            LIMIT 1
            "#,
        )
        .bind(user_low_id)
        .bind(user_high_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(|row| map_private_session(row, peer_user_id))
            .transpose()
    }

    async fn create_private_session(
        &self,
        created_by: i64,
        peer_user_id: i64,
    ) -> AppResult<CreatePrivateSessionResult> {
        let (user_low_id, user_high_id) = ordered_user_pair(created_by, peer_user_id);
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

        let pair_result = sqlx::query(
            r#"
            INSERT INTO private_session_pairs (session_id, user_low_id, user_high_id)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(session.session_id)
        .bind(user_low_id)
        .bind(user_high_id)
        .execute(&mut *tx)
        .await;

        if let Err(error) = pair_result {
            tx.rollback().await?;

            if is_private_session_pair_unique_violation(&error) {
                if let Some(existing_session) = self
                    .find_private_session_between(created_by, peer_user_id)
                    .await?
                {
                    return Ok(CreatePrivateSessionResult {
                        session: existing_session,
                        created: false,
                    });
                }
            }

            return Err(error.into());
        }

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

        Ok(CreatePrivateSessionResult {
            session,
            created: true,
        })
    }

    async fn create_group_session(
        &self,
        created_by: i64,
        name: &str,
        member_user_ids: &[i64],
    ) -> AppResult<GroupSession> {
        let mut tx = self.pool().begin().await?;

        let session_row = sqlx::query(
            r#"
            INSERT INTO sessions (session_type, name, created_by)
            VALUES ('group', $1, $2)
            RETURNING id, name, created_by, created_at::text AS created_at
            "#,
        )
        .bind(name)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        let session_id: i64 = session_row.try_get("id")?;
        for member_user_id in member_user_ids {
            let role = if *member_user_id == created_by {
                "owner"
            } else {
                "member"
            };

            sqlx::query(
                r#"
                INSERT INTO session_members (session_id, user_id, role)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(session_id)
            .bind(member_user_id)
            .bind(role)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(GroupSession {
            session_id,
            name: session_row.try_get("name")?,
            created_by: session_row.try_get("created_by")?,
            member_user_ids: member_user_ids.to_vec(),
            created_at: session_row.try_get("created_at")?,
        })
    }

    async fn get_group_member(
        &self,
        session_id: i64,
        user_id: i64,
    ) -> AppResult<Option<SessionMember>> {
        let row = sqlx::query(
            r#"
            SELECT
                sm.session_id,
                sm.user_id,
                sm.role,
                sm.joined_at::text AS joined_at
            FROM sessions s
            JOIN session_members sm
              ON sm.session_id = s.id
            WHERE s.id = $1
              AND s.session_type = 'group'
              AND sm.user_id = $2
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(map_session_member).transpose()
    }

    async fn add_group_member(&self, session_id: i64, user_id: i64) -> AppResult<SessionMember> {
        let row = sqlx::query(
            r#"
            INSERT INTO session_members (session_id, user_id, role)
            VALUES ($1, $2, 'member')
            RETURNING
                session_id,
                user_id,
                role,
                joined_at::text AS joined_at
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_one(self.pool())
        .await?;

        map_session_member(row)
    }

    async fn list_group_members(&self, session_id: i64) -> AppResult<Vec<GroupSessionMember>> {
        let rows = sqlx::query(
            r#"
            SELECT
                sm.user_id,
                u.username,
                sm.role,
                sm.joined_at::text AS joined_at
            FROM sessions s
            JOIN session_members sm
              ON sm.session_id = s.id
            JOIN users u
              ON u.id = sm.user_id
            WHERE s.id = $1
              AND s.session_type = 'group'
            ORDER BY
                CASE WHEN sm.role = 'owner' THEN 0 ELSE 1 END,
                sm.joined_at,
                sm.id
            "#,
        )
        .bind(session_id)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(map_group_session_member).collect()
    }

    async fn leave_group_session(&self, session_id: i64, user_id: i64) -> AppResult<bool> {
        let mut tx = self.pool().begin().await?;

        let deleted = sqlx::query(
            r#"
            DELETE FROM session_members sm
            USING sessions s
            WHERE sm.session_id = s.id
              AND s.session_type = 'group'
              AND sm.session_id = $1
              AND sm.user_id = $2
            RETURNING sm.role
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(deleted) = deleted else {
            tx.commit().await?;
            return Ok(false);
        };

        let deleted_role: String = deleted.try_get("role")?;
        let member_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM session_members
            WHERE session_id = $1
            "#,
        )
        .bind(session_id)
        .fetch_one(&mut *tx)
        .await?;

        if member_count == 0 {
            sqlx::query("DELETE FROM sessions WHERE id = $1")
                .bind(session_id)
                .execute(&mut *tx)
                .await?;
        } else if deleted_role == "owner" {
            let has_owner = sqlx::query(
                r#"
                SELECT 1
                FROM session_members
                WHERE session_id = $1
                  AND role = 'owner'
                LIMIT 1
                "#,
            )
            .bind(session_id)
            .fetch_optional(&mut *tx)
            .await?
            .is_some();

            if !has_owner {
                sqlx::query(
                    r#"
                    UPDATE session_members
                    SET role = 'owner'
                    WHERE id = (
                        SELECT id
                        FROM session_members
                        WHERE session_id = $1
                        ORDER BY joined_at, id
                        LIMIT 1
                    )
                    "#,
                )
                .bind(session_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(true)
    }

    async fn mark_session_read(
        &self,
        session_id: i64,
        user_id: i64,
        last_read_message_id: Option<i64>,
    ) -> AppResult<SessionReadState> {
        let row = sqlx::query(
            r#"
            INSERT INTO user_session_read_state (
                user_id,
                session_id,
                last_read_message_id,
                last_read_at
            )
            VALUES ($1, $2, $3::BIGINT, NOW())
            ON CONFLICT (user_id, session_id)
            DO UPDATE SET
                last_read_message_id = EXCLUDED.last_read_message_id,
                last_read_at = EXCLUDED.last_read_at
            RETURNING
                session_id,
                user_id,
                last_read_message_id,
                last_read_at::text AS last_read_at
            "#,
        )
        .bind(user_id)
        .bind(session_id)
        .bind(last_read_message_id)
        .fetch_one(self.pool())
        .await?;

        map_session_read_state(row)
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

fn ordered_user_pair(user_id: i64, peer_user_id: i64) -> (i64, i64) {
    if user_id < peer_user_id {
        (user_id, peer_user_id)
    } else {
        (peer_user_id, user_id)
    }
}

fn is_private_session_pair_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(|database_error| database_error.constraint())
        == Some("private_session_pairs_user_pair_uidx")
}

fn map_session_member(row: PgRow) -> AppResult<SessionMember> {
    Ok(SessionMember {
        session_id: row.try_get("session_id")?,
        user_id: row.try_get("user_id")?,
        role: row.try_get("role")?,
        joined_at: row.try_get("joined_at")?,
    })
}

fn map_group_session_member(row: PgRow) -> AppResult<GroupSessionMember> {
    Ok(GroupSessionMember {
        user_id: row.try_get("user_id")?,
        username: row.try_get("username")?,
        role: row.try_get("role")?,
        joined_at: row.try_get("joined_at")?,
    })
}

fn map_session_read_state(row: PgRow) -> AppResult<SessionReadState> {
    Ok(SessionReadState {
        session_id: row.try_get("session_id")?,
        user_id: row.try_get("user_id")?,
        last_read_message_id: row.try_get("last_read_message_id")?,
        last_read_at: row.try_get("last_read_at")?,
    })
}
