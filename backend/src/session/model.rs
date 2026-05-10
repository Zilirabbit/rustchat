#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateSession {
    pub session_id: i64,
    pub created_by: i64,
    pub peer_user_id: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupSession {
    pub session_id: i64,
    pub name: String,
    pub created_by: i64,
    pub member_user_ids: Vec<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMember {
    pub session_id: i64,
    pub user_id: i64,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionReadState {
    pub session_id: i64,
    pub user_id: i64,
    pub last_read_message_id: Option<i64>,
    pub last_read_at: String,
}
