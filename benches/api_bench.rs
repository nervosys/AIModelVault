//! API endpoint benchmarks — measures request/response latency for key endpoints.

use ai_model_vault::api::server::{create_router, AppState, RateLimiter};
use ai_model_vault::api::ApiConfig;
use ai_model_vault::config::{DirectoryPaths, VaultConfig};
use ai_model_vault::vault::Vault;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{header, Request, StatusCode};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

fn bench_state(dir: &tempfile::TempDir) -> Arc<AppState> {
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
            host: "127.0.0.1".into(),
            port: 0,
            jwt_secret: "bench-secret-key-for-benchmarks-only".into(),
            token_expiry_secs: 3600,
            cors_permissive: true,
            max_body_size: 512 * 1024 * 1024,
            enable_dashboard: false,
        },
        auth_rate_limiter: RateLimiter::new(100, std::time::Duration::from_secs(60)),
    })
}

fn bench_router(state: Arc<AppState>) -> axum::Router {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    create_router(state).layer(axum::Extension(ConnectInfo(addr)))
}

fn bench_health(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("api_health", |b| {
        b.iter(|| {
            rt.block_on(async {
                let dir = tempfile::tempdir().unwrap();
                let state = bench_state(&dir);
                let app = bench_router(state);

                let resp = app
                    .oneshot(
                        Request::builder()
                            .uri("/api/v1/health")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(black_box(resp.status()), StatusCode::OK);
            });
        });
    });
}

fn bench_auth_token(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("api_auth_token", |b| {
        b.iter(|| {
            rt.block_on(async {
                let dir = tempfile::tempdir().unwrap();
                let state = bench_state(&dir);

                // Unlock vault first
                {
                    let mut vault = state.vault.write().await;
                    vault
                        .unlock(b"bench-passphrase-with-entropy".to_vec())
                        .unwrap();
                }

                let app = bench_router(state);
                let resp = app
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/api/v1/auth/token")
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(
                                r#"{"passphrase":"bench-passphrase-with-entropy"}"#,
                            ))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(black_box(resp.status()), StatusCode::OK);
            });
        });
    });
}

fn bench_list_models(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("api_list_models", |b| {
        b.iter(|| {
            rt.block_on(async {
                let dir = tempfile::tempdir().unwrap();
                let state = bench_state(&dir);

                let mut vault = state.vault.write().await;
                vault
                    .unlock(b"bench-passphrase-with-entropy".to_vec())
                    .unwrap();
                drop(vault);

                let token = ai_model_vault::api::auth::create_token(
                    &state.config.jwt_secret,
                    state.config.token_expiry_secs,
                )
                .unwrap();

                let app = bench_router(state);
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

                assert_eq!(black_box(resp.status()), StatusCode::OK);
            });
        });
    });
}

fn bench_compliance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("api_compliance", |b| {
        b.iter(|| {
            rt.block_on(async {
                let dir = tempfile::tempdir().unwrap();
                let state = bench_state(&dir);

                let mut vault = state.vault.write().await;
                vault
                    .unlock(b"bench-passphrase-with-entropy".to_vec())
                    .unwrap();
                drop(vault);

                let token = ai_model_vault::api::auth::create_token(
                    &state.config.jwt_secret,
                    state.config.token_expiry_secs,
                )
                .unwrap();

                let app = bench_router(state);
                let resp = app
                    .oneshot(
                        Request::builder()
                            .uri("/api/v1/compliance")
                            .header(header::AUTHORIZATION, format!("Bearer {}", token))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(black_box(resp.status()), StatusCode::OK);
            });
        });
    });
}

criterion_group!(
    benches,
    bench_health,
    bench_auth_token,
    bench_list_models,
    bench_compliance,
);
criterion_main!(benches);
