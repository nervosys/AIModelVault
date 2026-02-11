//! REST API route handlers.

use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::conversion::{ConversionOptions, ConversionPipeline};
use crate::formats::{ModelFormat, ModelMetadata};

use super::auth;
use super::dashboard;
use super::error::ApiError;
use super::openapi;
use super::server::AppState;

// ── Health ───────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// GET /api/v1/health
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

// ── Auth ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AuthRequest {
    passphrase: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_in: u64,
}

/// POST /api/v1/auth/token
///
/// Unlocks the vault with the given passphrase and returns a JWT.
pub async fn auth_token(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Attempt unlock
    {
        let mut vault = state.vault.write().await;
        vault
            .unlock(body.passphrase.into_bytes())
            .map_err(|_| ApiError::unauthorized("Invalid passphrase"))?;
    }

    let token = auth::create_token(&state.config.jwt_secret, state.config.token_expiry_secs)
        .map_err(|e| ApiError::internal(format!("Token creation failed: {e}")))?;

    Ok(Json(AuthResponse {
        token,
        expires_in: state.config.token_expiry_secs,
    }))
}

// ── Models ───────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub version_count: usize,
}

/// GET /api/v1/models
pub async fn list_models(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<ModelInfo>>, ApiError> {
    require_auth(&headers, &state)?;
    let vault = state.vault.read().await;
    let models: Vec<ModelInfo> = vault
        .list_models()
        .into_iter()
        .map(|name| {
            let version_count = vault.list_versions(&name).len();
            ModelInfo {
                name,
                version_count,
            }
        })
        .collect();
    Ok(Json(models))
}

/// GET /api/v1/models/:name
pub async fn get_model(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    require_auth(&headers, &state)?;
    let vault = state.vault.read().await;
    let data = vault.get_model(&name, None).map_err(ApiError::from)?;
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/octet-stream")],
        data,
    )
        .into_response())
}

/// POST /api/v1/models/:name  (multipart: file + format + description?)
pub async fn store_model(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    require_auth(&headers, &state)?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut format_str: Option<String> = None;
    let mut description: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::bad_request(format!("Multipart error: {e}")))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "file" => {
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| ApiError::bad_request(format!("Read error: {e}")))?
                        .to_vec(),
                );
            }
            "format" => {
                format_str = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| ApiError::bad_request(e.to_string()))?,
                );
            }
            "description" => {
                description = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| ApiError::bad_request(e.to_string()))?,
                );
            }
            _ => {} // ignore unknown fields
        }
    }

    let data = file_data.ok_or_else(|| ApiError::bad_request("Missing 'file' field"))?;
    let fmt = format_str.ok_or_else(|| ApiError::bad_request("Missing 'format' field"))?;
    let format = ModelFormat::from_extension(&fmt);

    let mut metadata = ModelMetadata::new(name.clone(), format);
    if let Some(desc) = description {
        metadata = metadata.with_description(desc);
    }

    let mut vault = state.vault.write().await;
    let version = vault
        .store_model(&name, data, metadata, None)
        .map_err(ApiError::from)?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "model": name,
            "version": version.version,
            "checkpoint_id": version.checkpoint_id,
            "size_bytes": version.size_bytes,
            "checksum": version.checksum_sha256,
        })),
    ))
}

// ── Versions ─────────────────────────────────────────────────────────────────

/// GET /api/v1/models/:name/versions
pub async fn list_versions(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    require_auth(&headers, &state)?;
    let vault = state.vault.read().await;
    let versions = vault.list_versions(&name);
    if versions.is_empty() {
        return Err(ApiError::not_found(format!("Model '{}' not found", name)));
    }
    let vs: Vec<serde_json::Value> = versions
        .iter()
        .map(|v| {
            serde_json::json!({
                "version": v.version,
                "checkpoint_id": v.checkpoint_id,
                "timestamp": v.timestamp.to_rfc3339(),
                "format": v.format,
                "size_bytes": v.size_bytes,
                "compressed_size_bytes": v.compressed_size_bytes,
                "checksum_sha256": v.checksum_sha256,
                "parent_version": v.parent_version,
            })
        })
        .collect();
    Ok(Json(vs))
}

/// GET /api/v1/models/:name/versions/:version
pub async fn get_version(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, u32)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    require_auth(&headers, &state)?;
    let vault = state.vault.read().await;
    let data = vault
        .get_model(&name, Some(version))
        .map_err(ApiError::from)?;
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/octet-stream")],
        data,
    )
        .into_response())
}

/// DELETE /api/v1/models/:name/versions/:version
pub async fn delete_version(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, u32)>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_auth(&headers, &state)?;
    let mut vault = state.vault.write().await;
    let deleted = vault
        .delete_version(&name, version)
        .map_err(ApiError::from)?;
    if deleted {
        Ok(Json(serde_json::json!({ "deleted": true, "model": name, "version": version })))
    } else {
        Err(ApiError::not_found(format!(
            "Version {} not found for model '{}'",
            version, name
        )))
    }
}

/// GET /api/v1/models/:name/lineage/:version
pub async fn get_lineage(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, u32)>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    require_auth(&headers, &state)?;
    let vault = state.vault.read().await;
    let lineage = vault.get_lineage(&name, version);
    let vs: Vec<serde_json::Value> = lineage
        .iter()
        .map(|v| {
            serde_json::json!({
                "version": v.version,
                "checkpoint_id": v.checkpoint_id,
                "timestamp": v.timestamp.to_rfc3339(),
                "format": v.format,
                "size_bytes": v.size_bytes,
            })
        })
        .collect();
    Ok(Json(vs))
}

// ── Conversions ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ConversionInfo {
    pub name: String,
    pub source: String,
    pub target: String,
}

/// GET /api/v1/conversions
pub async fn list_conversions() -> Json<Vec<ConversionInfo>> {
    let pipeline = ConversionPipeline::with_builtins();
    let conversions: Vec<ConversionInfo> = pipeline
        .supported_conversions()
        .into_iter()
        .map(|(src, tgt, converter_name)| ConversionInfo {
            name: converter_name.to_string(),
            source: src.to_string(),
            target: tgt.to_string(),
        })
        .collect();
    Json(conversions)
}

#[derive(Deserialize)]
pub struct ConvertRequest {
    data_base64: String,
    source_format: String,
    target_format: String,
    quantization: Option<String>,
    opset_version: Option<u32>,
    #[serde(default)]
    validate: bool,
}

#[derive(Serialize)]
pub struct ConvertResponse {
    pub data_base64: String,
    pub source_format: String,
    pub target_format: String,
    pub conversion_path: Vec<String>,
    pub input_size: u64,
    pub output_size: u64,
    pub validation: Option<serde_json::Value>,
}

/// POST /api/v1/convert
pub async fn convert(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, ApiError> {
    require_auth(&headers, &state)?;

    let data = B64
        .decode(&body.data_base64)
        .map_err(|e| ApiError::bad_request(format!("Invalid base64: {e}")))?;

    let src = parse_format(&body.source_format)?;
    let tgt = parse_format(&body.target_format)?;

    let mut opts = ConversionOptions::default();
    opts.quantization = body.quantization;
    opts.opset_version = body.opset_version;
    opts.validate = body.validate;

    let pipeline = ConversionPipeline::with_builtins();
    let result = pipeline
        .convert(&data, &src, &tgt, &opts, None)
        .map_err(ApiError::from)?;

    let validation = result.validation.map(|r| {
        serde_json::json!({
            "passed": r.passed,
            "checks": r.checks.iter().map(|c| serde_json::json!({
                "name": c.name,
                "passed": c.passed,
                "message": c.message,
            })).collect::<Vec<_>>()
        })
    });

    Ok(Json(ConvertResponse {
        data_base64: B64.encode(&result.data),
        source_format: result.source_format.to_string(),
        target_format: result.target_format.to_string(),
        conversion_path: result
            .conversion_path
            .iter()
            .map(|f| f.to_string())
            .collect(),
        input_size: result.input_size,
        output_size: result.output_size,
        validation,
    }))
}

// ── Stats ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct StatsResponse {
    pub model_count: usize,
    pub total_versions: usize,
    pub total_size_bytes: u64,
    pub file_count: usize,
}

/// GET /api/v1/stats
pub async fn stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<StatsResponse>, ApiError> {
    require_auth(&headers, &state)?;
    let vault = state.vault.read().await;
    let s = vault.get_stats().map_err(ApiError::from)?;
    Ok(Json(StatsResponse {
        model_count: s.model_count,
        total_versions: s.total_versions,
        total_size_bytes: s.total_size_bytes,
        file_count: s.file_count,
    }))
}

// ── Audit ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AuditQuery {
    limit: Option<usize>,
}

/// GET /api/v1/audit
pub async fn audit_log(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<AuditQuery>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    require_auth(&headers, &state)?;

    // Read the audit log file from the vault config path
    let vault = state.vault.read().await;
    let audit_path = vault.get_config().get_audit_log_path();

    if !audit_path.exists() {
        return Ok(Json(vec![]));
    }

    let contents =
        std::fs::read_to_string(&audit_path).map_err(|e| ApiError::internal(e.to_string()))?;

    let entries: Vec<serde_json::Value> = contents
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    let limited = if let Some(n) = q.limit {
        entries.into_iter().take(n).collect()
    } else {
        entries
    };

    Ok(Json(limited))
}

// ── OpenAPI & Dashboard ──────────────────────────────────────────────────────

/// GET /api/v1/openapi.json
pub async fn openapi_json() -> Json<serde_json::Value> {
    Json(openapi::openapi_spec())
}

/// GET /  — serves the embedded web dashboard
pub async fn dashboard_index() -> Html<&'static str> {
    Html(dashboard::dashboard_html())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Extract and verify the Bearer JWT from the Authorization header.
fn require_auth(headers: &HeaderMap, state: &AppState) -> Result<(), ApiError> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("Missing Authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::unauthorized("Invalid Authorization format (expected Bearer)"))?;

    auth::verify_token(token, &state.config.jwt_secret)
        .map_err(|e| ApiError::unauthorized(format!("Invalid token: {e}")))?;

    Ok(())
}

/// Parse a format string into a ModelFormat.
fn parse_format(s: &str) -> Result<ModelFormat, ApiError> {
    let f = match s.to_lowercase().as_str() {
        "safetensors" => ModelFormat::Safetensors,
        "gguf" => ModelFormat::GGUF,
        "pytorch" | "pt" | "pth" => ModelFormat::PyTorch,
        "onnx" => ModelFormat::ONNX,
        "tensorrt" | "trt" => ModelFormat::TensorRT,
        "coreml" | "mlmodel" => ModelFormat::CoreML,
        "tflite" => ModelFormat::TFLite,
        "tensorflow" | "tf" | "pb" => ModelFormat::TensorFlow,
        "keras" => ModelFormat::Keras,
        "openvino" => ModelFormat::OpenVINO,
        "mlx" => ModelFormat::MLX,
        "hdf5" | "h5" => ModelFormat::HDF5,
        "numpy" | "npy" | "npz" => ModelFormat::NumPy,
        "pickle" | "pkl" => ModelFormat::Pickle,
        "mxnet" | "params" => ModelFormat::MXNet,
        "caffe" | "caffemodel" => ModelFormat::Caffe,
        "ncnn" | "param" => ModelFormat::NCNN,
        "mnn" => ModelFormat::MNN,
        "rknn" => ModelFormat::RKNN,
        "darknet" | "weights" => ModelFormat::Darknet,
        other => ModelFormat::Custom(other.to_string()),
    };
    Ok(f)
}
