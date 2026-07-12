//! Centralized error type that maps cleanly onto HTTP responses,
//! preserving the JSON error shape (`{"error": ...}` / `{"errors": ...}`) used by the Go service.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 400 — generic bad request placed under the `error` key.
    #[error("{0}")]
    BadRequest(String),

    /// 400 — validation failure placed under the `errors` key.
    #[error("{0}")]
    Validation(String),

    /// 401
    #[error("{0}")]
    Unauthorized(String),

    /// 403
    #[error("{0}")]
    Forbidden(String),

    /// 404
    #[error("{0}")]
    NotFound(String),

    /// 409
    #[error("{0}")]
    Conflict(String),

    /// 500
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

/// Convert sqlx errors: a missing row becomes a domain "not found", everything else is internal.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("record not found".to_string()),
            other => AppError::Internal(other.to_string()),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorBody { error: None, errors: Some(msg) },
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorBody { error: Some(msg), errors: None },
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                ErrorBody { error: Some(msg), errors: None },
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                ErrorBody { error: Some(msg), errors: None },
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ErrorBody { error: Some(msg), errors: None },
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                ErrorBody { error: Some(msg), errors: None },
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorBody { error: Some(msg), errors: None },
            ),
        };

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
