#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateSession {
    pub session_id: i64,
    pub created_by: i64,
    pub peer_user_id: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionReadState {
    pub session_id: i64,
    pub user_id: i64,
    pub last_read_message_id: Option<i64>,
    pub last_read_at: String,
}
