//! Axum HTTP server for AI Model Vault.
//!
//! Start with [`serve`] or build a router with [`create_router`].
//!
//! ## TLS / HTTPS
//!
//! This server binds plain HTTP by default. For production deployments,
//! terminate TLS at a reverse proxy (e.g., nginx, Caddy, AWS ALB) or use
//! `axum-server` with `rustls` for direct TLS termination. Never expose
//! the API over plain HTTP on untrusted networks.

use axum::routing::{delete, get, post, put};
use axum::Router;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
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
    /// Per-IP rate limiter for auth endpoints.
    pub auth_rate_limiter: RateLimiter,
}

/// Simple sliding-window rate limiter keyed by IP address.
pub struct RateLimiter {
    /// Map of IP → (attempt count, window start).
    state: std::sync::Mutex<HashMap<std::net::IpAddr, (u32, Instant)>>,
    /// Maximum attempts per window.
    max_attempts: u32,
    /// Window duration.
    window: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(max_attempts: u32, window: Duration) -> Self {
        Self {
            state: std::sync::Mutex::new(HashMap::new()),
            max_attempts,
            window,
        }
    }

    /// Check if the given IP is allowed. Returns `true` if under the limit.
    pub fn check(&self, ip: std::net::IpAddr) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();

        let entry = state.entry(ip).or_insert((0, now));

        // Reset window if expired
        if now.duration_since(entry.1) >= self.window {
            *entry = (0, now);
        }

        entry.0 += 1;
        entry.0 <= self.max_attempts
    }

    /// Prune expired entries to prevent unbounded memory growth.
    pub fn prune(&self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        state.retain(|_, (_, start)| now.duration_since(*start) < self.window);
    }
}

/// Build the axum [`Router`] with all API routes.
pub fn create_router(state: Arc<AppState>) -> Router {
    // CORS: only permissive when explicitly configured; default is restrictive
    let cors = if state.config.cors_permissive {
        CorsLayer::permissive()
    } else {
        // Restrictive CORS: no cross-origin requests allowed by default.
        // Configure allowed_origins in ApiConfig for specific trusted domains.
        CorsLayer::new()
    };

    let api = Router::new()
        .route("/health", get(routes::health))
        .route("/auth/token", post(routes::auth_token))
        .route("/models", get(routes::list_models))
        .route(
            "/models/:name",
            get(routes::get_model).post(routes::store_model),
        )
        .route(
            "/models/:name/card",
            get(routes::get_model_card).post(routes::create_model_card),
        )
        .route("/models/:name/versions", get(routes::list_versions))
        .route(
            "/models/:name/versions/:version",
            get(routes::get_version).delete(routes::delete_version),
        )
        .route("/models/:name/lineage/:version", get(routes::get_lineage))
        .route("/conversions", get(routes::list_conversions))
        .route("/convert", post(routes::convert))
        .route("/compliance", get(routes::compliance))
        .route("/rag/search", post(routes::rag_search))
        .route("/rag/documents", post(routes::rag_add_document))
        .route("/stats", get(routes::stats))
        .route("/audit", get(routes::audit_log))
        .route("/metrics", get(routes::metrics))
        .route("/events", get(routes::events))
        .route("/openapi.json", get(routes::openapi_json))
        // v1.4.0 endpoints
        .route(
            "/models/:name/tags",
            get(routes::get_tags)
                .post(routes::add_tags)
                .delete(routes::remove_tags),
        )
        .route("/search", post(routes::search_models))
        .route(
            "/acl",
            get(routes::acl_list)
                .post(routes::acl_grant)
                .delete(routes::acl_revoke),
        )
        .route(
            "/webhooks",
            get(routes::webhook_list).post(routes::webhook_add),
        )
        .route("/webhooks/:id", delete(routes::webhook_remove))
        .route("/models/:name/validate", post(routes::validate_model))
        .route("/gc", post(routes::garbage_collect))
        .route("/models/:name/policy", put(routes::policy_set))
        .route("/policies", get(routes::policy_list))
        .route(
            "/profiles",
            get(routes::profile_list).post(routes::profile_create),
        )
        .route("/profiles/:name/activate", post(routes::profile_activate))
        .route(
            "/lineage-graph",
            get(routes::lineage_graph_show).post(routes::lineage_graph_add),
        )
        .route("/plugins", get(routes::plugin_list))
        // v1.5.0 endpoints
        .route(
            "/quantization/profiles",
            get(routes::quant_profile_list).post(routes::quant_profile_set),
        )
        .route("/quantization/estimate", post(routes::quant_estimate))
        .route(
            "/evaluations",
            get(routes::eval_list).post(routes::eval_record),
        )
        .route("/evaluations/suites", get(routes::eval_suites))
        .route(
            "/backups/schedules",
            get(routes::backup_schedule_list).post(routes::backup_schedule_set),
        )
        .route("/backups/history", get(routes::backup_history))
        .route(
            "/vaults",
            get(routes::vault_list).post(routes::vault_register),
        )
        .route("/vaults/:name/activate", post(routes::vault_activate))
        .with_state(state.clone());

    let dashboard = if state.config.enable_dashboard {
        Router::new().route("/", get(routes::dashboard_index))
    } else {
        Router::new()
    };

    #[cfg(feature = "graphql")]
    let graphql_routes = {
        use super::graphql;
        let schema = graphql::build_schema(state.clone());
        Router::new()
            .route(
                "/graphql",
                get(graphql::graphql_playground).post(graphql::graphql_handler),
            )
            .with_state(schema)
    };
    #[cfg(not(feature = "graphql"))]
    let graphql_routes = Router::new();

    dashboard
        .merge(graphql_routes)
        .nest("/api/v1", api)
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(state.config.max_body_size))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(300),
        ))
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
        auth_rate_limiter: RateLimiter::new(5, Duration::from_secs(60)),
    });

    let router = create_router(state.clone()).into_make_service_with_connect_info::<SocketAddr>();

    // Spawn periodic cleanup of expired rate-limiter entries
    let limiter = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(300)).await;
            limiter.auth_rate_limiter.prune();
        }
    });

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
        .map_err(VaultError::IoError)?;

    axum::serve(listener, router)
        .await
        .map_err(|e| VaultError::IoError(std::io::Error::other(e.to_string())))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_under_limit() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));
        let ip: std::net::IpAddr = "127.0.0.1".parse().unwrap();
        assert!(limiter.check(ip));
        assert!(limiter.check(ip));
        assert!(limiter.check(ip));
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        let ip: std::net::IpAddr = "127.0.0.1".parse().unwrap();
        assert!(limiter.check(ip));
        assert!(limiter.check(ip));
        assert!(!limiter.check(ip)); // 3rd attempt blocked
    }

    #[test]
    fn test_rate_limiter_separate_ips() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        let ip1: std::net::IpAddr = "10.0.0.1".parse().unwrap();
        let ip2: std::net::IpAddr = "10.0.0.2".parse().unwrap();
        assert!(limiter.check(ip1));
        assert!(limiter.check(ip2));
        assert!(!limiter.check(ip1)); // ip1 blocked
        assert!(!limiter.check(ip2)); // ip2 blocked
    }

    #[test]
    fn test_rate_limiter_window_reset() {
        let limiter = RateLimiter::new(1, Duration::from_millis(1));
        let ip: std::net::IpAddr = "127.0.0.1".parse().unwrap();
        assert!(limiter.check(ip));
        assert!(!limiter.check(ip));
        std::thread::sleep(Duration::from_millis(5));
        assert!(limiter.check(ip)); // window expired, reset
    }

    #[test]
    fn test_rate_limiter_prune_expired() {
        let limiter = RateLimiter::new(5, Duration::from_millis(1));
        let ip: std::net::IpAddr = "127.0.0.1".parse().unwrap();
        limiter.check(ip);
        std::thread::sleep(Duration::from_millis(5));
        limiter.prune();
        // State should be empty after prune
        let state = limiter.state.lock().unwrap();
        assert!(state.is_empty());
    }

    #[test]
    fn test_rate_limiter_prune_keeps_active() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));
        let ip: std::net::IpAddr = "127.0.0.1".parse().unwrap();
        limiter.check(ip);
        limiter.prune();
        let state = limiter.state.lock().unwrap();
        assert_eq!(state.len(), 1); // still active
    }
}
