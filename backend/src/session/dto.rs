use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CreatePrivateSessionRequest {
    pub target_user_id: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CreatePrivateSessionResponse {
    pub session_id: i64,
    pub session_type: &'static str,
    pub peer_user_id: i64,
    pub created_at: String,
    pub created: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MarkSessionReadResponse {
    pub session_id: i64,
    pub last_read_message_id: Option<i64>,
    pub last_read_at: String,
}
