use serde::{Deserialize, Serialize};

use super::model::UserProfile;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SearchUsersQuery {
    pub keyword: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserProfile,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UserSearchItem {
    pub user_id: i64,
    pub username: String,
}
