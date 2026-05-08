use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    app::AppState,
    auth::types::CurrentUser,
    common::{
        error::AppError,
        response::{ApiResponse, ok},
    },
};

use super::dto::{
    CreatePrivateSessionRequest, CreatePrivateSessionResponse, MarkSessionReadResponse,
};

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

pub async fn mark_session_read(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(session_id): Path<i64>,
) -> Result<Json<ApiResponse<MarkSessionReadResponse>>, AppError> {
    let read_state = state
        .session_service
        .mark_session_read(&current_user, session_id)
        .await?;

    Ok(ok("session marked as read", read_state))
}
