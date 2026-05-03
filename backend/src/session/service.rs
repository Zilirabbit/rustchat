use async_trait::async_trait;

use crate::{
    auth::types::CurrentUser,
    common::error::{AppError, AppResult},
};

use super::{
    dto::{CreatePrivateSessionRequest, CreatePrivateSessionResponse},
    repo::SessionRepository,
};

#[async_trait]
pub trait SessionUseCase: Send + Sync {
    async fn create_private_session(
        &self,
        current_user: &CurrentUser,
        request: CreatePrivateSessionRequest,
    ) -> AppResult<CreatePrivateSessionResponse>;
}

pub struct SessionService<R> {
    repo: R,
}

impl<R> SessionService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R> SessionUseCase for SessionService<R>
where
    R: SessionRepository,
{
    async fn create_private_session(
        &self,
        current_user: &CurrentUser,
        request: CreatePrivateSessionRequest,
    ) -> AppResult<CreatePrivateSessionResponse> {
        if request.target_user_id == current_user.user_id {
            return Err(AppError::BadRequest(
                "cannot create private session with yourself".to_string(),
            ));
        }

        if request.target_user_id <= 0 {
            return Err(AppError::BadRequest(
                "target user id must be a positive integer".to_string(),
            ));
        }

        if !self.repo.user_exists(request.target_user_id).await? {
            return Err(AppError::BadRequest(
                "target user does not exist".to_string(),
            ));
        }

        if let Some(session) = self
            .repo
            .find_private_session_between(current_user.user_id, request.target_user_id)
            .await?
        {
            return Ok(CreatePrivateSessionResponse {
                session_id: session.session_id,
                session_type: "private",
                peer_user_id: session.peer_user_id,
                created_at: session.created_at,
                created: false,
            });
        }

        let session = self
            .repo
            .create_private_session(current_user.user_id, request.target_user_id)
            .await?;

        Ok(CreatePrivateSessionResponse {
            session_id: session.session_id,
            session_type: "private",
            peer_user_id: session.peer_user_id,
            created_at: session.created_at,
            created: true,
        })
    }
}

#[derive(Default)]
pub struct UnavailableSessionService;

#[async_trait]
impl SessionUseCase for UnavailableSessionService {
    async fn create_private_session(
        &self,
        _current_user: &CurrentUser,
        _request: CreatePrivateSessionRequest,
    ) -> AppResult<CreatePrivateSessionResponse> {
        Err(AppError::DbNotConfigured)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use async_trait::async_trait;

    use super::*;
    use crate::session::{model::PrivateSession, repo::SessionRepository};

    #[derive(Default)]
    struct FakeSessionRepository {
        next_session_id: Mutex<i64>,
        users: Mutex<HashMap<i64, bool>>,
        sessions: Mutex<HashMap<(i64, i64), PrivateSession>>,
    }

    impl FakeSessionRepository {
        fn key(user_id: i64, peer_user_id: i64) -> (i64, i64) {
            if user_id < peer_user_id {
                (user_id, peer_user_id)
            } else {
                (peer_user_id, user_id)
            }
        }
    }

    #[async_trait]
    impl SessionRepository for FakeSessionRepository {
        async fn user_exists(&self, user_id: i64) -> AppResult<bool> {
            Ok(self
                .users
                .lock()
                .unwrap()
                .get(&user_id)
                .copied()
                .unwrap_or(false))
        }

        async fn find_private_session_between(
            &self,
            user_id: i64,
            peer_user_id: i64,
        ) -> AppResult<Option<PrivateSession>> {
            Ok(self
                .sessions
                .lock()
                .unwrap()
                .get(&Self::key(user_id, peer_user_id))
                .cloned())
        }

        async fn create_private_session(
            &self,
            created_by: i64,
            peer_user_id: i64,
        ) -> AppResult<PrivateSession> {
            let mut next_session_id = self.next_session_id.lock().unwrap();
            *next_session_id += 1;

            let session = PrivateSession {
                session_id: *next_session_id,
                created_by,
                peer_user_id,
                created_at: "2026-05-03 12:00:00+00".to_string(),
            };

            self.sessions
                .lock()
                .unwrap()
                .insert(Self::key(created_by, peer_user_id), session.clone());

            Ok(session)
        }
    }

    fn current_user() -> CurrentUser {
        CurrentUser {
            user_id: 1,
            username: "alice".to_string(),
        }
    }

    #[tokio::test]
    async fn create_private_session_creates_new_session() {
        let repo = FakeSessionRepository::default();
        repo.users.lock().unwrap().insert(2, true);
        let service = SessionService::new(repo);

        let response = service
            .create_private_session(
                &current_user(),
                CreatePrivateSessionRequest { target_user_id: 2 },
            )
            .await
            .unwrap();

        assert_eq!(response.session_id, 1);
        assert_eq!(response.peer_user_id, 2);
        assert!(response.created);
    }

    #[tokio::test]
    async fn create_private_session_reuses_existing_session() {
        let repo = FakeSessionRepository::default();
        repo.users.lock().unwrap().insert(2, true);
        repo.sessions.lock().unwrap().insert(
            FakeSessionRepository::key(1, 2),
            PrivateSession {
                session_id: 9,
                created_by: 1,
                peer_user_id: 2,
                created_at: "2026-05-03 12:00:00+00".to_string(),
            },
        );

        let service = SessionService::new(repo);
        let response = service
            .create_private_session(
                &current_user(),
                CreatePrivateSessionRequest { target_user_id: 2 },
            )
            .await
            .unwrap();

        assert_eq!(response.session_id, 9);
        assert!(!response.created);
    }

    #[tokio::test]
    async fn create_private_session_rejects_self_chat() {
        let service = SessionService::new(FakeSessionRepository::default());
        let error = service
            .create_private_session(
                &current_user(),
                CreatePrivateSessionRequest { target_user_id: 1 },
            )
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::BAD_REQUEST);
    }
}
