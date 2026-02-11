//! Axum HTTP server for AI Model Vault.
//!
//! Start with [`serve`] or build a router with [`create_router`].

use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

use crate::config::VaultConfig;
use crate::error::{Result, VaultError};
use crate::vault::Vault;

use super::routes;
use super::ApiConfig;

/// Shared application state.
pub struct AppState {
    /// Thread-safe vault handle.
    pub vault: RwLock<Vault>,
    /// API configuration.
    pub config: ApiConfig,
}

/// Build the axum [`Router`] with all API routes.
pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = if state.config.cors_permissive {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let api = Router::new()
        .route("/health", get(routes::health))
        .route("/auth/token", post(routes::auth_token))
        .route("/models", get(routes::list_models))
        .route(
            "/models/{name}",
            get(routes::get_model).post(routes::store_model),
        )
        .route("/models/{name}/versions", get(routes::list_versions))
        .route(
            "/models/{name}/versions/{version}",
            get(routes::get_version).delete(routes::delete_version),
        )
        .route(
            "/models/{name}/lineage/{version}",
            get(routes::get_lineage),
        )
        .route("/conversions", get(routes::list_conversions))
        .route("/convert", post(routes::convert))
        .route("/stats", get(routes::stats))
        .route("/audit", get(routes::audit_log))
        .route("/openapi.json", get(routes::openapi_json))
        .with_state(state.clone());

    let dashboard = if state.config.enable_dashboard {
        Router::new().route("/", get(routes::dashboard_index))
    } else {
        Router::new()
    };

    dashboard
        .nest("/api/v1", api)
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(state.config.max_body_size))
        .layer(TraceLayer::new_for_http())
}

/// Start the API server.
///
/// This is a blocking call that runs until the process is terminated.
pub async fn serve(vault_config: VaultConfig, api_config: ApiConfig) -> Result<()> {
    if api_config.jwt_secret.is_empty() {
        return Err(VaultError::ConfigError(
            "JWT secret must not be empty. Set --jwt-secret or AIM_JWT_SECRET.".into(),
        ));
    }

    let vault = Vault::new(Some(vault_config))?;
    let state = Arc::new(AppState {
        vault: RwLock::new(vault),
        config: api_config.clone(),
    });

    let router = create_router(state);
    let addr: SocketAddr = format!("{}:{}", api_config.host, api_config.port)
        .parse()
        .map_err(|e| VaultError::ConfigError(format!("Invalid bind address: {e}")))?;

    println!("AI Model Vault API v{}", env!("CARGO_PKG_VERSION"));
    println!("  Listening on http://{}", addr);
    println!("  Dashboard:   http://{}/", addr);
    println!("  OpenAPI:     http://{}/api/v1/openapi.json", addr);
    println!();

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| VaultError::IoError(e))?;

    axum::serve(listener, router)
        .await
        .map_err(|e| VaultError::IoError(std::io::Error::other(e.to_string())))?;

    Ok(())
}
