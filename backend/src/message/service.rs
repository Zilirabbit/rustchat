use async_trait::async_trait;

use crate::{
    auth::types::CurrentUser,
    common::error::{AppError, AppResult},
};

use super::{
    dto::{
        ChatMessagePayload, HistoryMessagesQuery, MessageListItem, MessageListPage,
        SendMessageRequest,
    },
    repo::MessageRepository,
};

const MAX_TEXT_MESSAGE_LENGTH: usize = 1_000;
const MAX_HISTORY_MESSAGES_LIMIT: i64 = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSendResult {
    pub recipient_user_ids: Vec<i64>,
    pub message: ChatMessagePayload,
}

#[async_trait]
pub trait MessageUseCase: Send + Sync {
    async fn send_text_message(
        &self,
        current_user: &CurrentUser,
        request: SendMessageRequest,
    ) -> AppResult<MessageSendResult>;
    async fn list_history_messages(
        &self,
        current_user: &CurrentUser,
        query: HistoryMessagesQuery,
    ) -> AppResult<MessageListPage>;
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
            .get_session_message_access(request.session_id, current_user.user_id)
            .await?
            .ok_or_else(|| {
                AppError::Forbidden("you are not a member of this session".to_string())
            })?;

        let stored_message = self
            .repo
            .create_text_message(request.session_id, current_user.user_id, content)
            .await?;

        Ok(MessageSendResult {
            recipient_user_ids: access.recipient_user_ids,
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

    async fn list_history_messages(
        &self,
        current_user: &CurrentUser,
        query: HistoryMessagesQuery,
    ) -> AppResult<MessageListPage> {
        if query.session_id <= 0 {
            return Err(AppError::BadRequest(
                "session id must be a positive integer".to_string(),
            ));
        }

        if query.limit <= 0 || query.limit > MAX_HISTORY_MESSAGES_LIMIT {
            return Err(AppError::BadRequest(format!(
                "limit must be between 1 and {MAX_HISTORY_MESSAGES_LIMIT}"
            )));
        }

        if query
            .before_message_id
            .is_some_and(|message_id| message_id <= 0)
        {
            return Err(AppError::BadRequest(
                "before_message_id must be a positive integer".to_string(),
            ));
        }

        if !self
            .repo
            .is_session_member(query.session_id, current_user.user_id)
            .await?
        {
            return Err(AppError::Forbidden(
                "you are not a member of this session".to_string(),
            ));
        }

        let mut messages = self
            .repo
            .list_session_messages(query.session_id, query.before_message_id, query.limit + 1)
            .await?;
        let has_more = messages.len() as i64 > query.limit;
        if has_more {
            messages.truncate(query.limit as usize);
        }

        let next_before_message_id = if has_more {
            messages.last().map(|message| message.message_id)
        } else {
            None
        };

        Ok(MessageListPage {
            session_id: query.session_id,
            limit: query.limit,
            before_message_id: query.before_message_id,
            next_before_message_id,
            has_more,
            messages: messages
                .into_iter()
                .map(|message| MessageListItem {
                    message_id: message.message_id,
                    session_id: message.session_id,
                    sender_id: message.sender_id,
                    sender_username: message.sender_username,
                    message_type: message.message_type,
                    content: message.content,
                    created_at: message.created_at,
                })
                .collect(),
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

    async fn list_history_messages(
        &self,
        _current_user: &CurrentUser,
        _query: HistoryMessagesQuery,
    ) -> AppResult<MessageListPage> {
        Err(AppError::DbNotConfigured)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use async_trait::async_trait;

    use super::*;
    use crate::message::{
        model::{HistoryMessage, SessionMessageAccess, StoredMessage},
        repo::MessageRepository,
    };

    #[derive(Default)]
    struct FakeMessageRepository {
        access: Mutex<HashMap<(i64, i64), SessionMessageAccess>>,
        members: Mutex<HashMap<(i64, i64), bool>>,
        history_messages: Mutex<HashMap<i64, Vec<HistoryMessage>>>,
        next_message_id: Mutex<i64>,
        stored_messages: Mutex<Vec<StoredMessage>>,
    }

    #[async_trait]
    impl MessageRepository for FakeMessageRepository {
        async fn is_session_member(&self, session_id: i64, user_id: i64) -> AppResult<bool> {
            Ok(self
                .members
                .lock()
                .unwrap()
                .get(&(session_id, user_id))
                .copied()
                .unwrap_or(false))
        }

        async fn get_session_message_access(
            &self,
            session_id: i64,
            sender_id: i64,
        ) -> AppResult<Option<SessionMessageAccess>> {
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

        async fn list_session_messages(
            &self,
            session_id: i64,
            before_message_id: Option<i64>,
            limit: i64,
        ) -> AppResult<Vec<HistoryMessage>> {
            Ok(self
                .history_messages
                .lock()
                .unwrap()
                .get(&session_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|message| {
                    before_message_id
                        .map(|before_message_id| message.message_id < before_message_id)
                        .unwrap_or(true)
                })
                .take(limit as usize)
                .collect())
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
            SessionMessageAccess {
                session_id: 12,
                recipient_user_ids: vec![2],
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

        assert_eq!(result.recipient_user_ids, vec![2]);
        assert_eq!(result.message.message_id, 1);
        assert_eq!(result.message.content, "hello");
        assert_eq!(result.message.sender_username, "alice");
    }

    #[tokio::test]
    async fn send_text_message_returns_all_group_recipients() {
        let repo = FakeMessageRepository::default();
        repo.access.lock().unwrap().insert(
            (12, 1),
            SessionMessageAccess {
                session_id: 12,
                recipient_user_ids: vec![2, 3],
            },
        );

        let service = MessageService::new(repo);
        let result = service
            .send_text_message(
                &current_user(),
                SendMessageRequest {
                    session_id: 12,
                    content: "hello group".to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(result.recipient_user_ids, vec![2, 3]);
        assert_eq!(result.message.content, "hello group");
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

    #[tokio::test]
    async fn list_history_messages_returns_page_for_session_member() {
        let repo = FakeMessageRepository::default();
        repo.members.lock().unwrap().insert((12, 1), true);
        repo.history_messages.lock().unwrap().insert(
            12,
            vec![
                history_message(5, "latest"),
                history_message(4, "middle"),
                history_message(3, "older"),
            ],
        );

        let service = MessageService::new(repo);
        let page = service
            .list_history_messages(
                &current_user(),
                HistoryMessagesQuery {
                    session_id: 12,
                    limit: 2,
                    before_message_id: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(page.messages.len(), 2);
        assert_eq!(page.messages[0].message_id, 5);
        assert_eq!(page.messages[1].message_id, 4);
        assert!(page.has_more);
        assert_eq!(page.next_before_message_id, Some(4));
    }

    #[tokio::test]
    async fn list_history_messages_applies_before_message_cursor() {
        let repo = FakeMessageRepository::default();
        repo.members.lock().unwrap().insert((12, 1), true);
        repo.history_messages.lock().unwrap().insert(
            12,
            vec![
                history_message(5, "latest"),
                history_message(4, "middle"),
                history_message(3, "older"),
            ],
        );

        let service = MessageService::new(repo);
        let page = service
            .list_history_messages(
                &current_user(),
                HistoryMessagesQuery {
                    session_id: 12,
                    limit: 2,
                    before_message_id: Some(5),
                },
            )
            .await
            .unwrap();

        assert_eq!(
            page.messages
                .iter()
                .map(|message| message.message_id)
                .collect::<Vec<_>>(),
            vec![4, 3]
        );
        assert!(!page.has_more);
        assert_eq!(page.next_before_message_id, None);
    }

    #[tokio::test]
    async fn list_history_messages_rejects_non_member() {
        let service = MessageService::new(FakeMessageRepository::default());
        let error = service
            .list_history_messages(
                &current_user(),
                HistoryMessagesQuery {
                    session_id: 12,
                    limit: 20,
                    before_message_id: None,
                },
            )
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn list_history_messages_rejects_invalid_limit() {
        let service = MessageService::new(FakeMessageRepository::default());
        let error = service
            .list_history_messages(
                &current_user(),
                HistoryMessagesQuery {
                    session_id: 12,
                    limit: 51,
                    before_message_id: None,
                },
            )
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::BAD_REQUEST);
    }

    fn history_message(message_id: i64, content: &str) -> HistoryMessage {
        HistoryMessage {
            message_id,
            session_id: 12,
            sender_id: 2,
            sender_username: "bob".to_string(),
            message_type: "text".to_string(),
            content: content.to_string(),
            created_at: "2026-05-03 12:00:00+00".to_string(),
        }
    }
}
