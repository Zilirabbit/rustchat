use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::common::response::ApiResponse;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("database error")]
    Db(#[from] sqlx::Error),

    #[error("database migration error")]
    DbMigration(#[from] sqlx::migrate::MigrateError),

    #[error("{0}")]
    Config(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Conflict(String),

    #[error("database is not configured")]
    DbNotConfigured,
}

impl AppError {
    pub fn internal(error: impl Into<anyhow::Error>) -> Self {
        Self::Internal(error.into())
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Db(_) | AppError::DbMigration(_) | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::DbNotConfigured => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.to_string();

        let body = Json(ApiResponse::<()> {
            code: status.as_u16(),
            message,
            data: None,
        });

        (status, body).into_response()
    }
}
