#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationSummary {
    pub session_id: i64,
    pub session_type: String,
    pub session_name: String,
    pub last_message: Option<String>,
    pub last_message_time: Option<String>,
    pub unread_count: i64,
}
