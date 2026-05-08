use async_trait::async_trait;

use crate::{
    auth::types::CurrentUser,
    auth::{jwt::JwtService, password::PasswordService},
    common::error::{AppError, AppResult},
};

use super::{
    dto::{AuthResponse, LoginRequest, RegisterRequest, SearchUsersQuery, UserSearchItem},
    model::UserProfile,
    repo::UserRepository,
};

const INVALID_CREDENTIALS_MESSAGE: &str = "invalid username or password";
const MAX_SEARCH_RESULTS: i64 = 20;

#[async_trait]
pub trait UserUseCase: Send + Sync {
    async fn register(&self, request: RegisterRequest) -> AppResult<UserProfile>;
    async fn login(&self, request: LoginRequest) -> AppResult<AuthResponse>;
    async fn get_user_by_id(&self, user_id: i64) -> AppResult<UserProfile>;
    async fn search_users(
        &self,
        current_user: &CurrentUser,
        query: SearchUsersQuery,
    ) -> AppResult<Vec<UserSearchItem>>;
}

pub struct UserService<R> {
    jwt: JwtService,
    password: PasswordService,
    repo: R,
}

impl<R> UserService<R> {
    pub fn new(repo: R, jwt: JwtService) -> Self {
        Self {
            jwt,
            password: PasswordService::new(),
            repo,
        }
    }
}

#[async_trait]
impl<R> UserUseCase for UserService<R>
where
    R: UserRepository,
{
    async fn register(&self, request: RegisterRequest) -> AppResult<UserProfile> {
        let username = validate_registration_username(&request.username)?;
        validate_password_strength(&request.password)?;

        if self.repo.find_by_username(&username).await?.is_some() {
            return Err(AppError::Conflict("username already exists".to_string()));
        }

        let password_hash = self.password.hash_password(&request.password)?;
        let user = self.repo.create_user(&username, &password_hash).await?;

        Ok(user.into())
    }

    async fn login(&self, request: LoginRequest) -> AppResult<AuthResponse> {
        let username = normalize_login_username(&request.username)?;
        let user = self
            .repo
            .find_by_username(&username)
            .await?
            .ok_or_else(|| AppError::Unauthorized(INVALID_CREDENTIALS_MESSAGE.to_string()))?;

        let password_matches = self
            .password
            .verify_password(&request.password, &user.password_hash)?;

        if !password_matches {
            return Err(AppError::Unauthorized(
                INVALID_CREDENTIALS_MESSAGE.to_string(),
            ));
        }

        let profile = UserProfile::from(&user);
        let token = self.jwt.issue_token(user.id, &user.username)?;

        Ok(AuthResponse {
            token,
            user: profile,
        })
    }

    async fn get_user_by_id(&self, user_id: i64) -> AppResult<UserProfile> {
        let user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid or expired token".to_string()))?;

        Ok(user.into())
    }

    async fn search_users(
        &self,
        current_user: &CurrentUser,
        query: SearchUsersQuery,
    ) -> AppResult<Vec<UserSearchItem>> {
        let keyword = validate_search_keyword(&query.keyword)?;
        let users = self
            .repo
            .search_by_username_keyword(&keyword, current_user.user_id, MAX_SEARCH_RESULTS)
            .await?;

        Ok(users
            .into_iter()
            .map(|user| UserSearchItem {
                user_id: user.user_id,
                username: user.username,
            })
            .collect())
    }
}

#[derive(Default)]
pub struct UnavailableUserService;

#[async_trait]
impl UserUseCase for UnavailableUserService {
    async fn register(&self, _request: RegisterRequest) -> AppResult<UserProfile> {
        Err(AppError::DbNotConfigured)
    }

    async fn login(&self, _request: LoginRequest) -> AppResult<AuthResponse> {
        Err(AppError::DbNotConfigured)
    }

    async fn get_user_by_id(&self, _user_id: i64) -> AppResult<UserProfile> {
        Err(AppError::DbNotConfigured)
    }

    async fn search_users(
        &self,
        _current_user: &CurrentUser,
        _query: SearchUsersQuery,
    ) -> AppResult<Vec<UserSearchItem>> {
        Err(AppError::DbNotConfigured)
    }
}

fn validate_registration_username(username: &str) -> AppResult<String> {
    let username = username.trim();
    let username_len = username.chars().count();

    if !(3..=32).contains(&username_len) {
        return Err(AppError::BadRequest(
            "username must be between 3 and 32 characters".to_string(),
        ));
    }

    Ok(username.to_string())
}

fn normalize_login_username(username: &str) -> AppResult<String> {
    let username = username.trim();

    if username.is_empty() {
        return Err(AppError::Unauthorized(
            INVALID_CREDENTIALS_MESSAGE.to_string(),
        ));
    }

    Ok(username.to_string())
}

fn validate_password_strength(password: &str) -> AppResult<()> {
    let password_len = password.chars().count();

    if !(6..=32).contains(&password_len) {
        return Err(AppError::BadRequest(
            "password must be between 6 and 32 characters".to_string(),
        ));
    }

    Ok(())
}

fn validate_search_keyword(keyword: &str) -> AppResult<String> {
    let keyword = keyword.trim();
    let keyword_len = keyword.chars().count();

    if !(1..=32).contains(&keyword_len) {
        return Err(AppError::BadRequest(
            "keyword must be between 1 and 32 characters".to_string(),
        ));
    }

    Ok(keyword.to_string())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use async_trait::async_trait;

    use crate::{
        auth::password::PasswordService,
        common::{config::JwtConfig, error::AppError},
        user::model::{User, UserSearchResult},
    };

    use super::*;

    #[derive(Default)]
    struct FakeUserRepository {
        next_id: Mutex<i64>,
        users: Mutex<HashMap<i64, User>>,
    }

    #[async_trait]
    impl UserRepository for FakeUserRepository {
        async fn create_user(&self, username: &str, password_hash: &str) -> AppResult<User> {
            if self.find_by_username(username).await?.is_some() {
                return Err(AppError::Conflict("username already exists".to_string()));
            }

            let mut next_id = self.next_id.lock().unwrap();
            *next_id += 1;

            let user = User {
                id: *next_id,
                username: username.to_string(),
                password_hash: password_hash.to_string(),
                avatar_url: None,
            };

            self.users.lock().unwrap().insert(user.id, user.clone());
            Ok(user)
        }

        async fn find_by_id(&self, user_id: i64) -> AppResult<Option<User>> {
            Ok(self.users.lock().unwrap().get(&user_id).cloned())
        }

        async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
            Ok(self
                .users
                .lock()
                .unwrap()
                .values()
                .find(|user| user.username.eq_ignore_ascii_case(username))
                .cloned())
        }

        async fn search_by_username_keyword(
            &self,
            keyword: &str,
            exclude_user_id: i64,
            limit: i64,
        ) -> AppResult<Vec<UserSearchResult>> {
            let keyword = keyword.to_lowercase();

            let mut users = self
                .users
                .lock()
                .unwrap()
                .values()
                .filter(|user| user.id != exclude_user_id)
                .filter(|user| user.username.to_lowercase().contains(&keyword))
                .map(|user| UserSearchResult {
                    user_id: user.id,
                    username: user.username.clone(),
                })
                .collect::<Vec<_>>();
            users.sort_by(|left, right| {
                left.username
                    .cmp(&right.username)
                    .then(left.user_id.cmp(&right.user_id))
            });
            users.truncate(limit as usize);

            Ok(users)
        }
    }

    fn jwt_service() -> JwtService {
        JwtService::new(JwtConfig {
            secret: "service-test-secret".to_string(),
            expires_in_secs: 3_600,
            issuer: "rustchat-test".to_string(),
        })
    }

    #[tokio::test]
    async fn register_creates_user() {
        let service = UserService::new(FakeUserRepository::default(), jwt_service());

        let profile = service
            .register(RegisterRequest {
                username: "  alice  ".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(profile.user_id, 1);
        assert_eq!(profile.username, "alice");
    }

    #[tokio::test]
    async fn register_rejects_duplicate_usernames_case_insensitively() {
        let service = UserService::new(FakeUserRepository::default(), jwt_service());

        service
            .register(RegisterRequest {
                username: "Alice".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap();

        let error = service
            .register(RegisterRequest {
                username: "alice".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn login_returns_token_for_valid_credentials() {
        let repository = FakeUserRepository::default();
        let password_hash = PasswordService::new().hash_password("secret123").unwrap();
        repository
            .create_user("alice", &password_hash)
            .await
            .unwrap();

        let service = UserService::new(repository, jwt_service());
        let response = service
            .login(LoginRequest {
                username: "alice".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(response.user.username, "alice");
        assert!(!response.token.is_empty());
    }

    #[tokio::test]
    async fn login_rejects_invalid_password() {
        let repository = FakeUserRepository::default();
        let password_hash = PasswordService::new().hash_password("secret123").unwrap();
        repository
            .create_user("alice", &password_hash)
            .await
            .unwrap();

        let service = UserService::new(repository, jwt_service());
        let error = service
            .login(LoginRequest {
                username: "alice".to_string(),
                password: "wrong-password".to_string(),
            })
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::UNAUTHORIZED);
        assert_eq!(error.to_string(), INVALID_CREDENTIALS_MESSAGE);
    }

    #[tokio::test]
    async fn search_users_returns_matching_public_profiles() {
        let repository = FakeUserRepository::default();
        let password_hash = PasswordService::new().hash_password("secret123").unwrap();
        repository
            .create_user("alice", &password_hash)
            .await
            .unwrap();
        repository.create_user("bob", &password_hash).await.unwrap();
        repository
            .create_user("bobby", &password_hash)
            .await
            .unwrap();

        let service = UserService::new(repository, jwt_service());
        let users = service
            .search_users(
                &CurrentUser {
                    user_id: 1,
                    username: "alice".to_string(),
                },
                SearchUsersQuery {
                    keyword: "bo".to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "bob");
        assert_eq!(users[1].username, "bobby");
    }

    #[tokio::test]
    async fn search_users_rejects_blank_keyword() {
        let service = UserService::new(FakeUserRepository::default(), jwt_service());

        let error = service
            .search_users(
                &CurrentUser {
                    user_id: 1,
                    username: "alice".to_string(),
                },
                SearchUsersQuery {
                    keyword: " ".to_string(),
                },
            )
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::BAD_REQUEST);
    }
}
