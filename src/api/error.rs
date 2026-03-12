//! API error types and JSON error responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::error::VaultError;

/// JSON error body returned to clients.
#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub error: String,
    pub code: u16,
}

/// API error that converts to an HTTP response.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: msg.into(),
        }
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: msg.into(),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: msg.into(),
        }
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: msg.into(),
        }
    }

    pub fn rate_limited(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            message: msg.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorBody {
            error: self.message,
            code: self.status.as_u16(),
        };
        (self.status, axum::Json(body)).into_response()
    }
}

impl From<VaultError> for ApiError {
    fn from(err: VaultError) -> Self {
        match &err {
            VaultError::ModelNotFound(_) | VaultError::VersionNotFound(_, _) => {
                ApiError::not_found(err.to_string())
            }
            VaultError::AuthenticationFailed => ApiError::unauthorized(err.to_string()),
            VaultError::SecurityViolation(_) => ApiError::unauthorized("Access denied"),
            VaultError::InvalidInput(_) | VaultError::UnsupportedFormat(_) => {
                ApiError::bad_request(err.to_string())
            }
            // Don't leak internal error details to clients
            _ => {
                tracing::error!("Internal error: {err}");
                ApiError::internal("An internal error occurred")
            }
        }
    }
}
