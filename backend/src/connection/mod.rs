pub mod handler;
pub mod manager;
pub mod protocol;

use axum::{Router, routing::get};

use self::handler::ws_handler;

pub fn router() -> Router<crate::app::AppState> {
    Router::new().route("/ws", get(ws_handler))
}
