use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UserProfile {
    pub user_id: i64,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            user_id: user.id,
            username: user.username,
            avatar_url: user.avatar_url,
        }
    }
}

impl From<&User> for UserProfile {
    fn from(user: &User) -> Self {
        Self {
            user_id: user.id,
            username: user.username.clone(),
            avatar_url: user.avatar_url.clone(),
        }
    }
}
