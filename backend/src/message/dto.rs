use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SendMessageRequest {
    pub session_id: i64,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct HistoryMessagesQuery {
    pub session_id: i64,
    pub limit: i64,
    pub before_message_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, serde::Deserialize)]
pub struct ChatMessagePayload {
    pub message_id: i64,
    pub session_id: i64,
    pub sender_id: i64,
    pub sender_username: String,
    #[serde(default = "default_message_type")]
    pub message_type: String,
    pub content: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
}

fn default_message_type() -> String {
    "text".to_string()
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MessageListItem {
    pub message_id: i64,
    pub session_id: i64,
    pub sender_id: i64,
    pub sender_username: String,
    pub message_type: String,
    pub content: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MessageListPage {
    pub session_id: i64,
    pub limit: i64,
    pub before_message_id: Option<i64>,
    pub next_before_message_id: Option<i64>,
    pub has_more: bool,
    pub messages: Vec<MessageListItem>,
}
