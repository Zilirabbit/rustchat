mod app;
mod auth;
mod common;
mod connection;
mod conversation;
mod message;
mod middleware;
mod router;
mod session;
mod storage;
mod user;

use std::net::SocketAddr;

use app::AppState;
use common::config::AppConfig;
use common::error::AppResult;
use common::logging;
use router::create_router;

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;
    logging::init(&config.log_level);
    let state = AppState::build(config.clone()).await?;

    if state.storage.is_some() {
        tracing::info!("database pool initialized and migrations applied");
    } else {
        tracing::warn!("DATABASE_URL is not set; starting without database connectivity");
    }

    let app = create_router(state);

    let addr: SocketAddr = format!("{}:{}", config.app_host, config.app_port)
        .parse::<SocketAddr>()
        .map_err(|error| anyhow::anyhow!(error))?;

    tracing::info!("server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|error| anyhow::anyhow!(error))?;
    axum::serve(listener, app)
        .await
        .map_err(|error| anyhow::anyhow!(error))?;

    Ok(())
}
