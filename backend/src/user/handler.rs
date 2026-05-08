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

use super::{
    dto::{AuthResponse, LoginRequest, RegisterRequest, SearchUsersQuery, UserSearchItem},
    model::UserProfile,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, AppError> {
    let user = state.user_service.register(payload).await?;
    Ok(ok("user registered", user))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let auth_response = state.user_service.login(payload).await?;
    Ok(ok("login succeeded", auth_response))
}

pub async fn me(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<ApiResponse<UserProfile>>, AppError> {
    let user = state
        .user_service
        .get_user_by_id(current_user.user_id)
        .await?;
    Ok(ok("current user fetched", user))
}

pub async fn search_users(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<SearchUsersQuery>,
) -> Result<Json<ApiResponse<Vec<UserSearchItem>>>, AppError> {
    let users = state
        .user_service
        .search_users(&current_user, query)
        .await?;
    Ok(ok("users searched", users))
}
