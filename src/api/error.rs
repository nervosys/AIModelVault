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
            VaultError::ModelNotFound(_)
            | VaultError::VersionNotFound(_, _)
            | VaultError::NotFound(_) => ApiError::not_found(err.to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_request() {
        let err = ApiError::bad_request("oops");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.message, "oops");
    }

    #[test]
    fn test_not_found() {
        let err = ApiError::not_found("missing");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.message, "missing");
    }

    #[test]
    fn test_unauthorized() {
        let err = ApiError::unauthorized("denied");
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert_eq!(err.message, "denied");
    }

    #[test]
    fn test_internal() {
        let err = ApiError::internal("boom");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.message, "boom");
    }

    #[test]
    fn test_conflict() {
        let err = ApiError::conflict("exists");
        assert_eq!(err.status, StatusCode::CONFLICT);
        assert_eq!(err.message, "exists");
    }

    #[test]
    fn test_rate_limited() {
        let err = ApiError::rate_limited("slow down");
        assert_eq!(err.status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(err.message, "slow down");
    }

    #[test]
    fn test_into_response() {
        let err = ApiError::bad_request("test error");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_from_vault_error_model_not_found() {
        let err: ApiError = VaultError::ModelNotFound("m1".into()).into();
        assert_eq!(err.status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_from_vault_error_version_not_found() {
        let err: ApiError = VaultError::VersionNotFound(1, "m1".into()).into();
        assert_eq!(err.status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_from_vault_error_auth_failed() {
        let err: ApiError = VaultError::AuthenticationFailed.into();
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
    }

    /// Every not-found category must reach 404. The wildcard arm below maps
    /// anything unlisted to 500, so a new variant added without touching that
    /// match would silently report a missing resource as a server fault.
    #[test]
    fn test_all_not_found_variants_map_to_404() {
        for err in [
            VaultError::ModelNotFound("m".into()),
            VaultError::VersionNotFound(3, "m".into()),
            VaultError::NotFound("profile 'p'".into()),
        ] {
            let text = err.to_string();
            let api: ApiError = err.into();
            assert_eq!(api.status, StatusCode::NOT_FOUND, "{text} must be 404");
        }
    }

    #[test]
    fn test_from_vault_error_security_violation() {
        let err: ApiError = VaultError::SecurityViolation("bad".into()).into();
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_from_vault_error_invalid_input() {
        let err: ApiError = VaultError::InvalidInput("bad input".into()).into();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_from_vault_error_unsupported_format() {
        let err: ApiError = VaultError::UnsupportedFormat("xyz".into()).into();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_from_vault_error_internal() {
        let err: ApiError = VaultError::CryptoError("crypto fail".into()).into();
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.message, "An internal error occurred");
    }

    #[test]
    fn test_api_error_body_serialization() {
        let body = ApiErrorBody {
            error: "not found".into(),
            code: 404,
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("not found"));
        assert!(json.contains("404"));
    }
}
