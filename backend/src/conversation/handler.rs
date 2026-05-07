use axum::{Json, extract::State};

use crate::{
    app::AppState,
    auth::types::CurrentUser,
    common::{
        error::AppError,
        response::{ApiResponse, ok},
    },
};

use super::dto::ConversationItem;

pub async fn list_conversations(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<ApiResponse<Vec<ConversationItem>>>, AppError> {
    let conversations = state
        .conversation_service
        .list_conversations(&current_user)
        .await?;

    Ok(ok("conversations fetched", conversations))
}
