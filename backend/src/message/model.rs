#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateSessionAccess {
    pub session_id: i64,
    pub recipient_user_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredMessage {
    pub message_id: i64,
    pub session_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryMessage {
    pub message_id: i64,
    pub session_id: i64,
    pub sender_id: i64,
    pub sender_username: String,
    pub message_type: String,
    pub content: String,
    pub created_at: String,
}
