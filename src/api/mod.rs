//! REST API server for AI Model Vault
//!
//! Provides a network-accessible interface for vault management with:
//! - JWT-based authentication
//! - RESTful model/version CRUD
//! - Format conversion endpoints
//! - Audit log access
//! - OpenAPI specification
//! - Embedded web dashboard
//!
//! Enable with the `api` feature flag.

pub mod auth;
pub mod dashboard;
pub mod error;
#[cfg(feature = "graphql")]
pub mod graphql;
pub mod openapi;
pub mod routes;
pub mod server;

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// API server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Host address to bind to (default: "127.0.0.1").
    pub host: String,
    /// Port to listen on (default: 8080).
    pub port: u16,
    /// JWT secret key for token signing. Should be a strong random secret.
    pub jwt_secret: String,
    /// JWT token expiry in seconds (default: 3600 = 1 hour).
    pub token_expiry_secs: u64,
    /// Enable CORS for all origins (default: false).
    pub cors_permissive: bool,
    /// Maximum request body size in bytes (default: 512 MiB).
    pub max_body_size: usize,
    /// Enable the embedded web dashboard (default: true).
    pub enable_dashboard: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8080,
            jwt_secret: String::new(), // Must be set before serving
            token_expiry_secs: 3600,
            cors_permissive: false,
            max_body_size: 512 * 1024 * 1024,
            enable_dashboard: true,
        }
    }
}

impl Drop for ApiConfig {
    fn drop(&mut self) {
        self.jwt_secret.zeroize();
    }
}
