//! Integration tests for the REST API server.
//!
//! These tests start an in-process axum server and exercise the endpoints
//! through a real HTTP client.

#![cfg(feature = "api")]

use ai_model_vault::api::server::{create_router, AppState};
use ai_model_vault::api::ApiConfig;
use ai_model_vault::config::{DirectoryPaths, VaultConfig};
use ai_model_vault::vault::Vault;

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt; // for `oneshot`

/// Helper: create a test AppState backed by a temporary vault.
fn test_state(dir: &tempfile::TempDir) -> Arc<AppState> {
    let dirs = DirectoryPaths {
        config_dir: dir.path().join("config"),
        data_dir: dir.path().join("data"),
        cache_dir: dir.path().join("cache"),
        vault_dir: dir.path().join("data/vaults/default"),
        log_dir: dir.path().join("data/logs"),
        backends_dir: dir.path().join("config/backends"),
        utilities_dir: dir.path().join("config/utilities"),
        databases_dir: dir.path().join("config/databases"),
    };
    let config = VaultConfig::with_dirs(dirs).unwrap();
    let vault = Vault::new(Some(config)).unwrap();

    Arc::new(AppState {
        vault: RwLock::new(vault),
        config: ApiConfig {
            jwt_secret: "test-secret-for-integration-tests".into(),
            token_expiry_secs: 3600,
            cors_permissive: true,
            enable_dashboard: true,
            ..Default::default()
        },
    })
}

/// Helper: authenticate and return a JWT token.
async fn get_token(state: &Arc<AppState>) -> String {
    // First unlock the vault
    {
        let mut vault = state.vault.write().await;
        vault
            .unlock(b"integration-test-passphrase".to_vec())
            .unwrap();
    }

    let token = ai_model_vault::api::auth::create_token(
        &state.config.jwt_secret,
        state.config.token_expiry_secs,
    )
    .unwrap();
    token
}

// ── Health ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_health_endpoint() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);
    let app = create_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

// ── Auth ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_auth_token_success() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);
    let app = create_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&serde_json::json!({
                        "passphrase": "my-vault-password"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["token"].is_string());
    assert!(json["expires_in"].as_u64().unwrap() > 0);
}

// ── Models (unauthorized) ────────────────────────────────────────────────────

#[tokio::test]
async fn test_models_unauthorized() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);
    let app = create_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/models")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ── Models (authorized, empty vault) ─────────────────────────────────────────

#[tokio::test]
async fn test_list_models_empty() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);
    let token = get_token(&state).await;
    let app = create_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/models")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json, serde_json::json!([]));
}

// ── Stats ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_stats_endpoint() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);
    let token = get_token(&state).await;
    let app = create_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/stats")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["model_count"], 0);
}

// ── Conversions ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_conversions_endpoint() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);
    let app = create_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/conversions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(json.len() >= 10);
}

// ── Convert ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_convert_endpoint() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);
    let token = get_token(&state).await;
    let app = create_router(state);

    let data = b"raw tensor data for api test 0123456789";
    let payload = serde_json::json!({
        "data_base64": B64.encode(data),
        "source_format": "raw",
        "target_format": "safetensors",
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/convert")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["data_base64"].is_string());
    assert!(json["output_size"].as_u64().unwrap() > 0);
}

// ── OpenAPI ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_openapi_endpoint() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);
    let app = create_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["openapi"], "3.1.0");
    assert_eq!(json["info"]["title"], "AI Model Vault API");
}

// ── Dashboard ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_dashboard_endpoint() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);
    let app = create_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("AI Model Vault"));
    assert!(html.contains("<!DOCTYPE html>"));
}

// ── Audit ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_audit_endpoint_empty() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);
    let token = get_token(&state).await;
    let app = create_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/audit")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Invalid token ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_invalid_token_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);
    let app = create_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/models")
                .header(header::AUTHORIZATION, "Bearer invalid.token.here")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ── Model not found ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_nonexistent_model() {
    let dir = tempfile::tempdir().unwrap();
    let state = test_state(&dir);
    let token = get_token(&state).await;
    let app = create_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/models/nonexistent/versions")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
