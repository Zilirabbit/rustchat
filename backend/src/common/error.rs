use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("database error")]
    Db(#[from] sqlx::Error),

    #[error("database is not configured")]
    DbNotConfigured,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Db(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DbNotConfigured => StatusCode::SERVICE_UNAVAILABLE,
        };

        let body = Json(ErrorBody {
            code: status.as_u16(),
            message: self.to_string(),
        });

        (status, body).into_response()
    }
}
