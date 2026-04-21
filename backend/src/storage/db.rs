use sqlx::{
    PgPool, Row,
    postgres::{PgPoolOptions, PgRow},
};
use std::time::Duration;

pub async fn new_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
}

pub async fn ping_db(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: PgRow = sqlx::query("SELECT 1::bigint as value")
        .fetch_one(pool)
        .await?;
    let value: i64 = row.try_get("value")?;
    Ok(value)
}
