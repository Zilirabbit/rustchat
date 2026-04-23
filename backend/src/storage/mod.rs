pub mod db;
pub mod repository;

use crate::common::{config::DatabaseConfig, error::AppResult};

use self::{db::DbPool, repository::RepositoryContext};

#[derive(Clone)]
pub struct Storage {
    pool: DbPool,
}

impl Storage {
    pub async fn connect(config: &DatabaseConfig) -> AppResult<Self> {
        let pool = db::connect(config).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    #[allow(dead_code)]
    pub fn repository_context(&self) -> RepositoryContext {
        RepositoryContext::new(self.pool.clone())
    }

    pub async fn ping(&self) -> AppResult<i64> {
        Ok(db::ping_db(self.pool()).await?)
    }
}
