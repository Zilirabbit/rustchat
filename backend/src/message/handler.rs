use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    app::AppState,
    auth::types::CurrentUser,
    common::{
        error::AppError,
        response::{ApiResponse, ok},
    },
};

use super::dto::{HistoryMessagesQuery, MessageListPage};

pub async fn list_history_messages(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<HistoryMessagesQuery>,
) -> Result<Json<ApiResponse<MessageListPage>>, AppError> {
    let page = state
        .message_service
        .list_history_messages(&current_user, query)
        .await?;

    Ok(ok("history messages fetched", page))
}
