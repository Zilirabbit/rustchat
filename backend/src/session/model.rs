#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateSession {
    pub session_id: i64,
    pub created_by: i64,
    pub peer_user_id: i64,
    pub created_at: String,
}
