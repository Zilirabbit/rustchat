use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::Response,
};
use serde::Deserialize;

use crate::{
    app::AppState,
    auth::types::CurrentUser,
    common::error::AppError,
    common::response::{ApiResponse, ok},
    connection::protocol::ServerEvent,
    message::dto::ChatMessagePayload,
};

use super::model::{InitUploadRequest, InitUploadResponse};

#[derive(Debug, Deserialize)]
pub struct ChunkQuery {
    pub index: u32,
}

#[derive(Debug, Deserialize)]
pub struct CompleteBody {
    pub file_hash: String,
}

pub async fn init_upload(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(body): Json<InitUploadRequest>,
) -> Result<Json<ApiResponse<InitUploadResponse>>, AppError> {
    let file_service = state
        .file_service
        .as_ref()
        .ok_or(AppError::DbNotConfigured)?;

    let upload_id = file_service
        .init_upload(
            current_user.user_id,
            body.session_id,
            &body.file_name,
            body.file_size,
            &body.file_type,
            body.total_chunks,
        )
        .await?;

    Ok(ok("upload initialized", InitUploadResponse { upload_id }))
}

pub async fn upload_chunk(
    State(state): State<AppState>,
    _current_user: CurrentUser,
    Path(upload_id): Path<String>,
    Query(query): Query<ChunkQuery>,
    body: axum::body::Bytes,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let file_service = state
        .file_service
        .as_ref()
        .ok_or(AppError::DbNotConfigured)?;

    if body.is_empty() {
        return Err(AppError::BadRequest(
            "chunk data cannot be empty".to_string(),
        ));
    }

    file_service
        .save_chunk(&upload_id, query.index, &body)
        .await?;

    Ok(ok("chunk received", ()))
}

pub async fn complete_upload(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(upload_id): Path<String>,
    Json(body): Json<CompleteBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let file_service = state
        .file_service
        .as_ref()
        .ok_or(AppError::DbNotConfigured)?;

    let result = file_service
        .complete_upload(current_user.user_id, &upload_id, &body.file_hash)
        .await?;

    // Parse file metadata from content JSON
    let file_meta: serde_json::Value = serde_json::from_str(&result.file_content)
        .map_err(|e| AppError::internal(anyhow::anyhow!("invalid file content: {}", e)))?;

    let file_id = file_meta["file_id"].as_i64().unwrap_or(result.file.file_id);
    let file_name = file_meta["file_name"]
        .as_str()
        .unwrap_or("file")
        .to_string();
    let file_size = file_meta["file_size"].as_i64().unwrap_or(0);
    let file_type = file_meta["file_type"]
        .as_str()
        .unwrap_or("application/octet-stream")
        .to_string();

    let payload = ChatMessagePayload {
        message_id: result.message_id,
        session_id: result.file.session_id,
        sender_id: result.file.sender_id,
        sender_username: current_user.username.clone(),
        message_type: "file".to_string(),
        content: file_name.clone(),
        created_at: result.file.created_at.clone(),
        file_id: Some(file_id),
        file_name: Some(file_name),
        file_size: Some(file_size),
        file_type: Some(file_type),
    };

    // Send MessageSent to sender
    state
        .connections
        .send_to_user(
            current_user.user_id,
            &ServerEvent::MessageSent {
                message: payload.clone(),
                client_message_id: None,
            },
        )
        .await;

    // Send ReceiveMessage to recipients
    for recipient_user_id in &result.recipient_user_ids {
        state
            .connections
            .send_to_user(
                *recipient_user_id,
                &ServerEvent::ReceiveMessage {
                    message: payload.clone(),
                },
            )
            .await;
    }

    Ok(ok(
        "file uploaded successfully",
        serde_json::json!({
            "file_id": result.file.file_id,
            "message_id": result.message_id,
            "file_name": result.file.file_name,
            "file_size": result.file.file_size,
            "file_type": result.file.file_type,
        }),
    ))
}

pub async fn download_file(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(file_id): Path<i64>,
) -> Result<Response, AppError> {
    let file_service = state
        .file_service
        .as_ref()
        .ok_or(AppError::DbNotConfigured)?;

    let file = file_service
        .verify_file_access(current_user.user_id, file_id)
        .await?;

    let full_path = file_service.upload_dir().join(&file.storage_path);

    let data = tokio::fs::read(&full_path)
        .await
        .map_err(|_| AppError::NotFound("file not found on storage".to_string()))?;

    let content_type = if file.file_type.is_empty() {
        "application/octet-stream".to_string()
    } else {
        file.file_type.clone()
    };

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file.file_name),
        )
        .header(header::CONTENT_LENGTH, data.len().to_string())
        .body(axum::body::Body::from(data))
        .map_err(|e| AppError::internal(anyhow::anyhow!("failed to build response: {}", e)))?;

    Ok(response)
}
