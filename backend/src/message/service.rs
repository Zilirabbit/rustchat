use async_trait::async_trait;

use crate::{
    auth::types::CurrentUser,
    common::error::{AppError, AppResult},
};

use super::{
    dto::{ChatMessagePayload, SendMessageRequest},
    repo::MessageRepository,
};

const MAX_TEXT_MESSAGE_LENGTH: usize = 1_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSendResult {
    pub recipient_user_id: i64,
    pub message: ChatMessagePayload,
}

#[async_trait]
pub trait MessageUseCase: Send + Sync {
    async fn send_text_message(
        &self,
        current_user: &CurrentUser,
        request: SendMessageRequest,
    ) -> AppResult<MessageSendResult>;
}

pub struct MessageService<R> {
    repo: R,
}

impl<R> MessageService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R> MessageUseCase for MessageService<R>
where
    R: MessageRepository,
{
    async fn send_text_message(
        &self,
        current_user: &CurrentUser,
        request: SendMessageRequest,
    ) -> AppResult<MessageSendResult> {
        if request.session_id <= 0 {
            return Err(AppError::BadRequest(
                "session id must be a positive integer".to_string(),
            ));
        }

        let content = request.content.trim();
        if content.is_empty() {
            return Err(AppError::BadRequest(
                "message content cannot be blank".to_string(),
            ));
        }

        if content.chars().count() > MAX_TEXT_MESSAGE_LENGTH {
            return Err(AppError::BadRequest(format!(
                "message content must be at most {MAX_TEXT_MESSAGE_LENGTH} characters"
            )));
        }

        let access = self
            .repo
            .get_private_session_access(request.session_id, current_user.user_id)
            .await?
            .ok_or_else(|| {
                AppError::Forbidden("you are not a member of this session".to_string())
            })?;

        let stored_message = self
            .repo
            .create_text_message(request.session_id, current_user.user_id, content)
            .await?;

        Ok(MessageSendResult {
            recipient_user_id: access.recipient_user_id,
            message: ChatMessagePayload {
                message_id: stored_message.message_id,
                session_id: stored_message.session_id,
                sender_id: stored_message.sender_id,
                sender_username: current_user.username.clone(),
                content: stored_message.content,
                created_at: stored_message.created_at,
            },
        })
    }
}

#[derive(Default)]
pub struct UnavailableMessageService;

#[async_trait]
impl MessageUseCase for UnavailableMessageService {
    async fn send_text_message(
        &self,
        _current_user: &CurrentUser,
        _request: SendMessageRequest,
    ) -> AppResult<MessageSendResult> {
        Err(AppError::DbNotConfigured)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use async_trait::async_trait;

    use super::*;
    use crate::message::{
        model::{PrivateSessionAccess, StoredMessage},
        repo::MessageRepository,
    };

    #[derive(Default)]
    struct FakeMessageRepository {
        access: Mutex<HashMap<(i64, i64), PrivateSessionAccess>>,
        next_message_id: Mutex<i64>,
        stored_messages: Mutex<Vec<StoredMessage>>,
    }

    #[async_trait]
    impl MessageRepository for FakeMessageRepository {
        async fn get_private_session_access(
            &self,
            session_id: i64,
            sender_id: i64,
        ) -> AppResult<Option<PrivateSessionAccess>> {
            Ok(self
                .access
                .lock()
                .unwrap()
                .get(&(session_id, sender_id))
                .cloned())
        }

        async fn create_text_message(
            &self,
            session_id: i64,
            sender_id: i64,
            content: &str,
        ) -> AppResult<StoredMessage> {
            let mut next_message_id = self.next_message_id.lock().unwrap();
            *next_message_id += 1;

            let message = StoredMessage {
                message_id: *next_message_id,
                session_id,
                sender_id,
                content: content.to_string(),
                created_at: "2026-05-03 12:00:00+00".to_string(),
            };

            self.stored_messages.lock().unwrap().push(message.clone());
            Ok(message)
        }
    }

    fn current_user() -> CurrentUser {
        CurrentUser {
            user_id: 1,
            username: "alice".to_string(),
        }
    }

    #[tokio::test]
    async fn send_text_message_persists_message_for_private_session() {
        let repo = FakeMessageRepository::default();
        repo.access.lock().unwrap().insert(
            (12, 1),
            PrivateSessionAccess {
                session_id: 12,
                recipient_user_id: 2,
            },
        );

        let service = MessageService::new(repo);
        let result = service
            .send_text_message(
                &current_user(),
                SendMessageRequest {
                    session_id: 12,
                    content: " hello ".to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(result.recipient_user_id, 2);
        assert_eq!(result.message.message_id, 1);
        assert_eq!(result.message.content, "hello");
        assert_eq!(result.message.sender_username, "alice");
    }

    #[tokio::test]
    async fn send_text_message_rejects_non_member() {
        let service = MessageService::new(FakeMessageRepository::default());
        let error = service
            .send_text_message(
                &current_user(),
                SendMessageRequest {
                    session_id: 12,
                    content: "hello".to_string(),
                },
            )
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn send_text_message_rejects_blank_content() {
        let service = MessageService::new(FakeMessageRepository::default());
        let error = service
            .send_text_message(
                &current_user(),
                SendMessageRequest {
                    session_id: 12,
                    content: "   ".to_string(),
                },
            )
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::BAD_REQUEST);
    }
}
