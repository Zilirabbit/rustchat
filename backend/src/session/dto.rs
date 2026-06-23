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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CreateGroupSessionRequest {
    pub name: String,
    #[serde(default)]
    pub member_user_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CreateGroupSessionResponse {
    pub session_id: i64,
    pub session_type: &'static str,
    pub name: String,
    pub created_by: i64,
    pub member_user_ids: Vec<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AddGroupMemberRequest {
    pub user_id: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AddGroupMemberResponse {
    pub session_id: i64,
    pub user_id: i64,
    pub role: String,
    pub joined_at: String,
    pub added: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GroupMemberListItem {
    pub user_id: i64,
    pub username: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ListGroupMembersResponse {
    pub session_id: i64,
    pub members: Vec<GroupMemberListItem>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LeaveGroupSessionResponse {
    pub session_id: i64,
    pub user_id: i64,
    pub left: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RemoveGroupMemberResponse {
    pub session_id: i64,
    pub user_id: i64,
    pub removed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MarkSessionReadResponse {
    pub session_id: i64,
    pub last_read_message_id: Option<i64>,
    pub last_read_at: String,
}
