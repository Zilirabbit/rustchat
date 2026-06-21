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
    connection::protocol::ServerEvent,
};

use super::dto::{
    AddGroupMemberRequest, AddGroupMemberResponse, CreateGroupSessionRequest,
    CreateGroupSessionResponse, CreatePrivateSessionRequest, CreatePrivateSessionResponse,
    LeaveGroupSessionResponse, ListGroupMembersResponse, MarkSessionReadResponse,
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

pub async fn create_group_session(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<CreateGroupSessionRequest>,
) -> Result<Json<ApiResponse<CreateGroupSessionResponse>>, AppError> {
    let session = state
        .session_service
        .create_group_session(&current_user, payload)
        .await?;

    notify_group_members_conversation_updated(
        &state,
        &session.member_user_ids,
        current_user.user_id,
        session.session_id,
    )
    .await;

    Ok(ok("group session created", session))
}

pub async fn add_group_member(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(session_id): Path<i64>,
    Json(payload): Json<AddGroupMemberRequest>,
) -> Result<Json<ApiResponse<AddGroupMemberResponse>>, AppError> {
    let member = state
        .session_service
        .add_group_member(&current_user, session_id, payload)
        .await?;

    if member.added {
        notify_conversation_updated(&state, member.user_id, member.session_id).await;
    }

    Ok(ok("group member ready", member))
}

pub async fn list_group_members(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(session_id): Path<i64>,
) -> Result<Json<ApiResponse<ListGroupMembersResponse>>, AppError> {
    let members = state
        .session_service
        .list_group_members(&current_user, session_id)
        .await?;

    Ok(ok("group members ready", members))
}

pub async fn leave_group_session(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(session_id): Path<i64>,
) -> Result<Json<ApiResponse<LeaveGroupSessionResponse>>, AppError> {
    let response = state
        .session_service
        .leave_group_session(&current_user, session_id)
        .await?;

    Ok(ok("group session left", response))
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

async fn notify_group_members_conversation_updated(
    state: &AppState,
    member_user_ids: &[i64],
    actor_user_id: i64,
    session_id: i64,
) {
    for member_user_id in member_user_ids {
        if *member_user_id != actor_user_id {
            notify_conversation_updated(state, *member_user_id, session_id).await;
        }
    }
}

async fn notify_conversation_updated(state: &AppState, user_id: i64, session_id: i64) {
    if !state
        .connections
        .send_to_user(user_id, &ServerEvent::ConversationUpdated { session_id })
        .await
    {
        tracing::debug!(
            user_id,
            session_id,
            "conversation update websocket push skipped"
        );
    }
}
