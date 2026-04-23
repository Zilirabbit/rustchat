use sqlx::{
    PgPool, Row,
    migrate::Migrator,
    postgres::{PgPoolOptions, PgRow},
};
use std::time::Duration;

use crate::common::{config::DatabaseConfig, error::AppResult};

pub type DbPool = PgPool;

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn new_pool(config: &DatabaseConfig) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout_secs))
        .connect(&config.url)
        .await
}

pub async fn run_migrations(pool: &DbPool) -> AppResult<()> {
    Ok(MIGRATOR.run(pool).await?)
}

pub async fn connect(config: &DatabaseConfig) -> AppResult<DbPool> {
    let pool = new_pool(config).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

pub async fn ping_db(pool: &DbPool) -> Result<i64, sqlx::Error> {
    let row: PgRow = sqlx::query("SELECT 1::bigint as value")
        .fetch_one(pool)
        .await?;
    let value: i64 = row.try_get("value")?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires TEST_DATABASE_URL to point at a reachable PostgreSQL instance"]
    async fn connect_runs_migrations_and_ping_succeeds() {
        let url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set for database integration tests");

        let config = DatabaseConfig {
            url,
            max_connections: 5,
            min_connections: 1,
            acquire_timeout_secs: 5,
        };

        let pool = connect(&config).await.unwrap();
        let value = ping_db(&pool).await.unwrap();

        assert_eq!(value, 1);
    }
}
