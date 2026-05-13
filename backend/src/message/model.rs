#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMessageAccess {
    pub session_id: i64,
    pub recipient_user_ids: Vec<i64>,
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
    pub file_id: Option<i64>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub file_type: Option<String>,
}
