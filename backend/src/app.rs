use std::sync::Arc;

use crate::{
    auth::jwt::JwtService,
    common::{config::AppConfig, error::AppResult},
    connection::manager::ConnectionManager,
    storage::Storage,
    user::{
        repo::PostgresUserRepository,
        service::{UnavailableUserService, UserService, UserUseCase},
    },
};

#[derive(Clone)]
pub struct AppState {
    pub storage: Option<Storage>,
    pub auth: AuthState,
    pub connections: ConnectionManager,
    pub user_service: Arc<dyn UserUseCase>,
}

#[derive(Clone)]
pub struct AuthState {
    pub jwt: JwtService,
}

impl AppState {
    pub async fn build(config: AppConfig) -> AppResult<Self> {
        let storage = match config.database.as_ref() {
            Some(database_config) => Some(Storage::connect(database_config).await?),
            None => None,
        };

        let jwt = JwtService::new(config.jwt.clone());
        let user_service: Arc<dyn UserUseCase> = match storage.as_ref() {
            Some(storage) => Arc::new(UserService::new(
                PostgresUserRepository::new(storage.repository_context()),
                jwt.clone(),
            )),
            None => Arc::new(UnavailableUserService),
        };

        Ok(Self {
            storage,
            auth: AuthState { jwt },
            connections: ConnectionManager::new(),
            user_service,
        })
    }

    #[cfg(test)]
    pub fn new(
        storage: Option<Storage>,
        jwt: JwtService,
        user_service: Arc<dyn UserUseCase>,
    ) -> Self {
        Self {
            storage,
            auth: AuthState { jwt },
            connections: ConnectionManager::new(),
            user_service,
        }
    }
}
