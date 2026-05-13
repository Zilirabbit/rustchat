use axum::{Router, middleware, routing::{get, post}};

use crate::{app::AppState, middleware::auth};

use super::handler;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/files/init", post(handler::init_upload))
        .route("/api/files/{upload_id}/chunk", post(handler::upload_chunk))
        .route("/api/files/{upload_id}/complete", post(handler::complete_upload))
        .route("/api/files/{file_id}/download", get(handler::download_file))
        .route_layer(middleware::from_fn_with_state(state, auth::require_auth))
}
