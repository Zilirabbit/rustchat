use crate::{common::config::AppConfig, storage::db::new_pool};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: Option<PgPool>,
}

impl AppState {
    pub async fn build(config: AppConfig) -> Result<Self, sqlx::Error> {
        let db = match config.database_url.as_deref() {
            Some(database_url) => Some(new_pool(database_url).await?),
            None => None,
        };

        Ok(Self { db })
    }
}
