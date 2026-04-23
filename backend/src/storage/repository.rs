#![allow(dead_code)]

use sqlx::PgPool;

#[derive(Clone)]
pub struct RepositoryContext {
    pool: PgPool,
}

impl RepositoryContext {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

pub trait Repository {
    fn context(&self) -> &RepositoryContext;

    fn pool(&self) -> &PgPool {
        self.context().pool()
    }
}
