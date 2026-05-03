use axum::{Json, extract::State};

use crate::{
    app::AppState,
    auth::types::CurrentUser,
    common::{
        error::AppError,
        response::{ApiResponse, ok},
    },
};

use super::dto::{CreatePrivateSessionRequest, CreatePrivateSessionResponse};

pub async fn create_private_session(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<CreatePrivateSessionRequest>,
) -> Result<Json<ApiResponse<CreatePrivateSessionResponse>>, AppError> {
    let session = state
        .session_service
        .create_private_session(&current_user, payload)
        .await?;

    Ok(ok("private session ready", session))
}
