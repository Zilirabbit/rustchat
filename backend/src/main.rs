mod app;
mod common;
mod connection;
mod message;
mod router;
mod session;
mod storage;
mod user;

use std::net::SocketAddr;

use app::AppState;
use common::config::AppConfig;
use router::create_router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();
    let state = AppState::build(config.clone()).await?;

    if state.db.is_some() {
        tracing::info!("database pool initialized");
    } else {
        tracing::warn!("DATABASE_URL is not set; starting without database connectivity");
    }

    let app = create_router(state);

    let addr: SocketAddr = format!("{}:{}", config.app_host, config.app_port).parse()?;

    tracing::info!("server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
