use async_trait::async_trait;

use crate::{
    auth::types::CurrentUser,
    common::error::{AppError, AppResult},
};

use super::{dto::ConversationItem, repo::ConversationRepository};

#[async_trait]
pub trait ConversationUseCase: Send + Sync {
    async fn list_conversations(
        &self,
        current_user: &CurrentUser,
    ) -> AppResult<Vec<ConversationItem>>;
}

pub struct ConversationService<R> {
    repo: R,
}

impl<R> ConversationService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R> ConversationUseCase for ConversationService<R>
where
    R: ConversationRepository,
{
    async fn list_conversations(
        &self,
        current_user: &CurrentUser,
    ) -> AppResult<Vec<ConversationItem>> {
        let conversations = self.repo.list_for_user(current_user.user_id).await?;

        Ok(conversations
            .into_iter()
            .map(|conversation| ConversationItem {
                session_id: conversation.session_id,
                session_type: conversation.session_type,
                session_name: conversation.session_name,
                last_message: conversation.last_message,
                last_message_time: conversation.last_message_time,
                unread_count: conversation.unread_count,
            })
            .collect())
    }
}

#[derive(Default)]
pub struct UnavailableConversationService;

#[async_trait]
impl ConversationUseCase for UnavailableConversationService {
    async fn list_conversations(
        &self,
        _current_user: &CurrentUser,
    ) -> AppResult<Vec<ConversationItem>> {
        Err(AppError::DbNotConfigured)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use async_trait::async_trait;

    use super::*;
    use crate::conversation::{model::ConversationSummary, repo::ConversationRepository};

    #[derive(Default)]
    struct FakeConversationRepository {
        conversations: Mutex<HashMap<i64, Vec<ConversationSummary>>>,
    }

    #[async_trait]
    impl ConversationRepository for FakeConversationRepository {
        async fn list_for_user(&self, user_id: i64) -> AppResult<Vec<ConversationSummary>> {
            Ok(self
                .conversations
                .lock()
                .unwrap()
                .get(&user_id)
                .cloned()
                .unwrap_or_default())
        }
    }

    fn current_user() -> CurrentUser {
        CurrentUser {
            user_id: 1,
            username: "alice".to_string(),
        }
    }

    #[tokio::test]
    async fn list_conversations_returns_items_for_current_user() {
        let repo = FakeConversationRepository::default();
        repo.conversations.lock().unwrap().insert(
            1,
            vec![
                ConversationSummary {
                    session_id: 12,
                    session_type: "private".to_string(),
                    session_name: "bob".to_string(),
                    last_message: Some("hello".to_string()),
                    last_message_time: Some("2026-05-03 12:00:00+00".to_string()),
                    unread_count: 2,
                },
                ConversationSummary {
                    session_id: 7,
                    session_type: "group".to_string(),
                    session_name: "team".to_string(),
                    last_message: None,
                    last_message_time: None,
                    unread_count: 0,
                },
            ],
        );

        let service = ConversationService::new(repo);
        let conversations = service.list_conversations(&current_user()).await.unwrap();

        assert_eq!(conversations.len(), 2);
        assert_eq!(conversations[0].session_id, 12);
        assert_eq!(conversations[0].session_name, "bob");
        assert_eq!(conversations[0].unread_count, 2);
        assert_eq!(conversations[1].last_message, None);
    }
}
