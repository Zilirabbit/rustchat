use std::sync::Arc;

use crate::{
    auth::jwt::JwtService,
    common::{config::AppConfig, error::AppResult},
    connection::manager::ConnectionManager,
    message::{
        repo::PostgresMessageRepository,
        service::{MessageService, MessageUseCase, UnavailableMessageService},
    },
    session::{
        repo::PostgresSessionRepository,
        service::{SessionService, SessionUseCase, UnavailableSessionService},
    },
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
    pub session_service: Arc<dyn SessionUseCase>,
    pub message_service: Arc<dyn MessageUseCase>,
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
        let (user_service, session_service, message_service): (
            Arc<dyn UserUseCase>,
            Arc<dyn SessionUseCase>,
            Arc<dyn MessageUseCase>,
        ) = match storage.as_ref() {
            Some(storage) => {
                let context = storage.repository_context();
                (
                    Arc::new(UserService::new(
                        PostgresUserRepository::new(context.clone()),
                        jwt.clone(),
                    )),
                    Arc::new(SessionService::new(PostgresSessionRepository::new(
                        context.clone(),
                    ))),
                    Arc::new(MessageService::new(PostgresMessageRepository::new(context))),
                )
            }
            None => (
                Arc::new(UnavailableUserService),
                Arc::new(UnavailableSessionService),
                Arc::new(UnavailableMessageService),
            ),
        };

        Ok(Self {
            storage,
            auth: AuthState { jwt },
            connections: ConnectionManager::new(),
            user_service,
            session_service,
            message_service,
        })
    }

    #[cfg(test)]
    pub fn new(
        storage: Option<Storage>,
        jwt: JwtService,
        user_service: Arc<dyn UserUseCase>,
    ) -> Self {
        Self::new_with_services(
            storage,
            jwt,
            user_service,
            Arc::new(UnavailableSessionService),
            Arc::new(UnavailableMessageService),
        )
    }

    #[cfg(test)]
    pub fn new_with_services(
        storage: Option<Storage>,
        jwt: JwtService,
        user_service: Arc<dyn UserUseCase>,
        session_service: Arc<dyn SessionUseCase>,
        message_service: Arc<dyn MessageUseCase>,
    ) -> Self {
        Self {
            storage,
            auth: AuthState { jwt },
            connections: ConnectionManager::new(),
            user_service,
            session_service,
            message_service,
        }
    }
}
