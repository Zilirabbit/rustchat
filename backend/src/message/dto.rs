use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SendMessageRequest {
    pub session_id: i64,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ChatMessagePayload {
    pub message_id: i64,
    pub session_id: i64,
    pub sender_id: i64,
    pub sender_username: String,
    pub content: String,
    pub created_at: String,
}
