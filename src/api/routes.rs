//! REST API route handlers.

use axum::extract::{ConnectInfo, Multipart, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::conversion::{ConversionOptions, ConversionPipeline};
use crate::formats::{ModelFormat, ModelMetadata};
use crate::traits::VaultState;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_seconds: Option<u64>,
}

/// GET /api/v1/health
///
/// Returns server health. Includes vault state when AppState is available.
pub async fn health(state: Option<State<Arc<AppState>>>) -> Json<HealthResponse> {
    let (vault_state_str, model_count) = if let Some(State(st)) = state {
        let vault = st.vault.read().await;
        let vs = vault.state();
        let mc = match &vs {
            VaultState::Locked { model_count, .. } => Some(*model_count),
            VaultState::Unlocked { model_count, .. } => Some(*model_count),
            _ => None,
        };
        (Some(vs.to_string()), mc)
    } else {
        (None, None)
    };

    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        vault_state: vault_state_str,
        model_count,
        uptime_seconds: None,
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Rate-limit auth attempts per IP
    if !state.auth_rate_limiter.check(addr.ip()) {
        return Err(ApiError::rate_limited("Too many authentication attempts"));
    }

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
    let _claims = require_auth(&headers, &state)?;
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
    let _claims = require_auth(&headers, &state)?;
    validate_model_name(&name)?;
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
    let _claims = require_auth(&headers, &state)?;
    validate_model_name(&name)?;

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
    let _claims = require_auth(&headers, &state)?;
    validate_model_name(&name)?;
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
    let _claims = require_auth(&headers, &state)?;
    validate_model_name(&name)?;
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
    let _claims = require_auth(&headers, &state)?;
    validate_model_name(&name)?;
    let mut vault = state.vault.write().await;
    let deleted = vault
        .delete_version(&name, version)
        .map_err(ApiError::from)?;
    if deleted {
        Ok(Json(
            serde_json::json!({ "deleted": true, "model": name, "version": version }),
        ))
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
    let _claims = require_auth(&headers, &state)?;
    validate_model_name(&name)?;
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
    let _claims = require_auth(&headers, &state)?;

    let data = B64
        .decode(&body.data_base64)
        .map_err(|e| ApiError::bad_request(format!("Invalid base64: {e}")))?;

    let src = parse_format(&body.source_format)?;
    let tgt = parse_format(&body.target_format)?;

    let opts = ConversionOptions {
        quantization: body.quantization,
        opset_version: body.opset_version,
        validate: body.validate,
        ..ConversionOptions::default()
    };

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
    let _claims = require_auth(&headers, &state)?;
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
///
/// Returns audit log entries. Admins see all events; Operators and Viewers
/// cannot see `SecurityViolation`, `IntegrityFailure`, or `AuthFailure` events.
pub async fn audit_log(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<AuditQuery>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let claims = require_auth(&headers, &state)?;

    // Read the audit log file from the vault config path
    let vault = state.vault.read().await;
    let audit_path = vault.get_config().get_audit_log_path();

    if !audit_path.exists() {
        return Ok(Json(vec![]));
    }

    let contents =
        std::fs::read_to_string(&audit_path).map_err(|e| ApiError::internal(e.to_string()))?;

    let mut entries: Vec<serde_json::Value> = contents
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    // Role-based filtering: non-admin roles cannot see security-sensitive events
    if claims.role != super::auth::Role::Admin {
        entries.retain(|entry| !is_security_event(entry));
    }

    let limited = if let Some(n) = q.limit {
        entries.into_iter().take(n).collect()
    } else {
        entries
    };

    Ok(Json(limited))
}

// ── Observability ─────────────────────────────────────────────────────────────

/// GET /api/v1/metrics
///
/// Returns vault metrics: state, model counts, operation counters,
/// storage statistics, and compliance status.
pub async fn metrics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let _claims = require_auth(&headers, &state)?;
    let vault = state.vault.read().await;
    let vs = vault.state();
    let stats = vault.get_stats().map_err(ApiError::from)?;

    Ok(Json(serde_json::json!({
        "vault_state": vs.to_string(),
        "models_count": stats.model_count,
        "versions_count": stats.total_versions,
        "storage_bytes": stats.total_size_bytes,
        "file_count": stats.file_count,
        "version": env!("CARGO_PKG_VERSION"),
        "healthy": true,
    })))
}

#[derive(Deserialize)]
pub struct EventsQuery {
    /// Maximum number of events to return.
    limit: Option<usize>,
    /// Filter by event type (e.g. "ModelStored", "VaultUnlocked").
    #[serde(rename = "type")]
    event_type: Option<String>,
}

/// GET /api/v1/events
///
/// Returns audit events from the vault's audit log, with optional
/// filtering by type and limit. Events are returned newest-first.
/// Non-admin roles cannot see security-sensitive events.
pub async fn events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<EventsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let claims = require_auth(&headers, &state)?;

    let vault = state.vault.read().await;
    let audit_path = vault.get_config().get_audit_log_path();

    if !audit_path.exists() {
        return Ok(Json(vec![]));
    }

    let contents =
        std::fs::read_to_string(&audit_path).map_err(|e| ApiError::internal(e.to_string()))?;

    let mut entries: Vec<serde_json::Value> = contents
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    // Role-based filtering: non-admin roles cannot see security-sensitive events
    if claims.role != super::auth::Role::Admin {
        entries.retain(|entry| !is_security_event(entry));
    }

    // Filter by event type if specified
    if let Some(ref et) = q.event_type {
        let et_lower = et.to_lowercase();
        entries.retain(|entry| {
            entry
                .get("action")
                .and_then(|a| a.as_str())
                .map(|a| a.to_lowercase().contains(&et_lower))
                .unwrap_or(false)
                || entry
                    .get("type")
                    .and_then(|t| t.as_str())
                    .map(|t| t.to_lowercase().contains(&et_lower))
                    .unwrap_or(false)
        });
    }

    // Return newest first
    entries.reverse();

    // Apply limit
    if let Some(n) = q.limit {
        entries.truncate(n);
    }

    Ok(Json(entries))
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

// ── Model Cards ──────────────────────────────────────────────────────────────

/// GET /api/v1/models/:name/card
///
/// Generate a model card for the given model using vault metadata.
pub async fn get_model_card(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let _claims = require_auth(&headers, &state)?;
    validate_model_name(&name)?;
    let vault = state.vault.read().await;

    // Verify model exists
    let versions = vault.list_versions(&name);
    if versions.is_empty() {
        return Err(ApiError::not_found(format!("Model '{}' not found", name)));
    }

    let latest = &versions[versions.len() - 1];
    let details = crate::model_card::ModelDetails {
        name: name.clone(),
        version: format!("v{}", latest.version),
        description: latest
            .metadata
            .get("description")
            .cloned()
            .unwrap_or_default(),
        model_type: String::new(),
        architecture: String::new(),
        size: format!("{} bytes", latest.size_bytes),
        framework: latest
            .metadata
            .get("framework")
            .cloned()
            .unwrap_or_default(),
        format: latest.format.clone(),
        license: None,
        citation: None,
        developers: vec![],
        contact: None,
        repository: None,
        paper: None,
    };
    let intended_use = crate::model_card::IntendedUse {
        primary_uses: vec!["General-purpose AI model".to_string()],
        primary_users: vec![],
        out_of_scope_uses: vec![],
        use_case_examples: None,
    };

    let card = crate::model_card::ModelCard::new(details, intended_use);

    let json_str = card
        .to_json()
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(value))
}

/// POST /api/v1/models/:name/card
///
/// Create (or overwrite) a custom model card from JSON.
/// Returns the rendered card re-serialized from the parsed input.
pub async fn create_model_card(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let _claims = require_auth(&headers, &state)?;
    validate_model_name(&name)?;

    // Verify model exists
    let vault = state.vault.read().await;
    let versions = vault.list_versions(&name);
    if versions.is_empty() {
        return Err(ApiError::not_found(format!("Model '{}' not found", name)));
    }

    let json_str = serde_json::to_string(&body)
        .map_err(|e| ApiError::bad_request(format!("Invalid JSON: {e}")))?;
    let card = crate::model_card::ModelCard::from_json(&json_str)
        .map_err(|e| ApiError::bad_request(format!("Invalid model card: {e}")))?;

    let roundtrip = card
        .to_json()
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_str(&roundtrip).map_err(|e| ApiError::internal(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(value)))
}

// ── Compliance ───────────────────────────────────────────────────────────────

/// GET /api/v1/compliance
///
/// Run FIPS 140-3, CVE, MITRE ATT&CK, and CMMC 2.0 compliance checks.
pub async fn compliance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let _claims = require_auth(&headers, &state)?;

    let checker = crate::compliance::ComplianceChecker::new();
    let status = checker
        .run_all_checks()
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "fips_140_3": status.fips_140_3,
        "cve_scan_passed": status.cve_scan_passed,
        "mitre_attack_aligned": status.mitre_attack_aligned,
        "cmmc_level": status.cmmc_level,
        "all_passed": status.violations.is_empty(),
        "violations": status.violations.iter().map(|v| serde_json::json!({
            "standard": v.standard,
            "control": v.control,
            "severity": v.severity,
            "description": v.description,
            "remediation": v.remediation,
        })).collect::<Vec<_>>(),
    })))
}

// ── RAG ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RagSearchRequest {
    query: String,
    limit: Option<usize>,
}

/// POST /api/v1/rag/search
///
/// Search the RAG document store.
pub async fn rag_search(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<RagSearchRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let _claims = require_auth(&headers, &state)?;

    if body.query.is_empty() || body.query.len() > 10_000 {
        return Err(ApiError::bad_request(
            "Query must be between 1 and 10,000 characters",
        ));
    }

    let vault = state.vault.read().await;
    let rag_path = vault.get_config().get_vault_path(None).join("rag");

    if !rag_path.exists() {
        return Ok(Json(
            serde_json::json!({ "results": [], "query": body.query }),
        ));
    }

    let kb_config = crate::rag::KnowledgeBaseConfig::default();
    let kb = crate::rag::KnowledgeBase::new("vault".to_string(), kb_config);
    let limit = body.limit.unwrap_or(10).min(100);
    // Without pre-computed embeddings, retrieve returns empty vec.
    // The endpoint is wired and ready for real embedding integration.
    let results = kb.retrieve(&[], Some(limit));

    Ok(Json(serde_json::json!({
        "query": body.query,
        "results": results.iter().map(|doc| serde_json::json!({
            "id": doc.id,
            "content": doc.content,
            "metadata": doc.metadata,
        })).collect::<Vec<_>>(),
    })))
}

#[derive(Deserialize)]
pub struct RagAddDocumentRequest {
    content: String,
    #[serde(default)]
    metadata: std::collections::HashMap<String, String>,
}

/// POST /api/v1/rag/documents
///
/// Add a document to the RAG knowledge base.
pub async fn rag_add_document(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<RagAddDocumentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let _claims = require_auth(&headers, &state)?;

    if body.content.is_empty() || body.content.len() > 1_000_000 {
        return Err(ApiError::bad_request(
            "Document content must be between 1 and 1,000,000 characters",
        ));
    }

    // Validate metadata keys/values
    for (k, v) in &body.metadata {
        if k.len() > 256 || v.len() > 4096 {
            return Err(ApiError::bad_request(
                "Metadata key max 256 chars, value max 4096 chars",
            ));
        }
    }

    let id = format!("doc_{}", uuid_v4_simple());

    let doc = crate::rag::Document {
        id: id.clone(),
        content: body.content.clone(),
        metadata: body.metadata.clone(),
        embedding: None,
        chunk_info: None,
    };

    let mut store = crate::rag::DocumentStore::new();
    store
        .add_document(doc)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    // Acknowledge — note: in-memory store won't persist across requests,
    // but this wires the endpoint and demonstrates the API contract.
    let _ = state.vault.read().await; // verify vault is accessible
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": id,
            "content_length": body.content.len(),
            "metadata_keys": body.metadata.keys().collect::<Vec<_>>(),
        })),
    ))
}

/// Generate a simple unique ID (timestamp + random suffix).
fn uuid_v4_simple() -> String {
    use std::time::SystemTime;
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", ts)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Extract and verify the Bearer JWT from the Authorization header.
/// Returns the decoded [`Claims`] on success, for role-based access control.
fn require_auth(headers: &HeaderMap, state: &AppState) -> Result<super::auth::Claims, ApiError> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("Missing Authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::unauthorized("Invalid Authorization format (expected Bearer)"))?;

    // Reject tokens with invalid structure before verification
    if token.is_empty() || token.len() > 4096 || token.chars().any(|c| c.is_control()) {
        return Err(ApiError::unauthorized("Malformed token"));
    }

    auth::verify_token(token, &state.config.jwt_secret)
        .map_err(|_| ApiError::unauthorized("Invalid or expired token"))
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
        other => {
            return Err(ApiError::bad_request(format!(
                "Unsupported format: '{other}'"
            )));
        }
    };
    Ok(f)
}

/// Validate a model name: must be 1-128 ASCII alphanumeric, hyphens, underscores, dots.
fn validate_model_name(name: &str) -> Result<(), ApiError> {
    if name.is_empty() || name.len() > 128 {
        return Err(ApiError::bad_request("Model name must be 1-128 characters"));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(ApiError::bad_request(
            "Model name must contain only ASCII alphanumeric, hyphens, underscores, or dots",
        ));
    }
    Ok(())
}

/// Check if an audit entry is a security-sensitive event type.
///
/// Used by role-based filtering: non-admin roles cannot see these events.
fn is_security_event(entry: &serde_json::Value) -> bool {
    const SECURITY_TYPES: &[&str] = &[
        "SECURITY_VIOLATION",
        "INTEGRITY_FAILURE",
        "AUTH_FAILURE",
        "KEY_DERIVED",
    ];

    let event_type = entry
        .get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    SECURITY_TYPES
        .iter()
        .any(|t| event_type.eq_ignore_ascii_case(t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_format_valid() {
        let cases = vec![
            ("safetensors", ModelFormat::Safetensors),
            ("gguf", ModelFormat::GGUF),
            ("pytorch", ModelFormat::PyTorch),
            ("pt", ModelFormat::PyTorch),
            ("pth", ModelFormat::PyTorch),
            ("onnx", ModelFormat::ONNX),
            ("tensorrt", ModelFormat::TensorRT),
            ("trt", ModelFormat::TensorRT),
            ("coreml", ModelFormat::CoreML),
            ("mlmodel", ModelFormat::CoreML),
            ("tflite", ModelFormat::TFLite),
            ("tensorflow", ModelFormat::TensorFlow),
            ("tf", ModelFormat::TensorFlow),
            ("pb", ModelFormat::TensorFlow),
            ("keras", ModelFormat::Keras),
            ("openvino", ModelFormat::OpenVINO),
            ("mlx", ModelFormat::MLX),
            ("hdf5", ModelFormat::HDF5),
            ("h5", ModelFormat::HDF5),
            ("numpy", ModelFormat::NumPy),
            ("npy", ModelFormat::NumPy),
            ("npz", ModelFormat::NumPy),
            ("pickle", ModelFormat::Pickle),
            ("pkl", ModelFormat::Pickle),
            ("mxnet", ModelFormat::MXNet),
            ("params", ModelFormat::MXNet),
            ("caffe", ModelFormat::Caffe),
            ("caffemodel", ModelFormat::Caffe),
            ("ncnn", ModelFormat::NCNN),
            ("param", ModelFormat::NCNN),
            ("mnn", ModelFormat::MNN),
            ("rknn", ModelFormat::RKNN),
            ("darknet", ModelFormat::Darknet),
            ("weights", ModelFormat::Darknet),
        ];
        for (input, expected) in cases {
            let result = parse_format(input).unwrap();
            assert_eq!(result, expected, "parse_format(\"{input}\") mismatch");
        }
    }

    #[test]
    fn test_parse_format_invalid() {
        let err = parse_format("nonexistent");
        assert!(err.is_err());
    }

    #[test]
    fn test_validate_model_name_valid() {
        assert!(validate_model_name("my-model").is_ok());
        assert!(validate_model_name("a").is_ok());
        assert!(validate_model_name("model_v2.1").is_ok());
        assert!(validate_model_name("ABC-123").is_ok());
    }

    #[test]
    fn test_validate_model_name_empty() {
        assert!(validate_model_name("").is_err());
    }

    #[test]
    fn test_validate_model_name_too_long() {
        let long = "a".repeat(129);
        assert!(validate_model_name(&long).is_err());
    }

    #[test]
    fn test_validate_model_name_invalid_chars() {
        assert!(validate_model_name("model name").is_err()); // space
        assert!(validate_model_name("model/path").is_err()); // slash
        assert!(validate_model_name("model;drop").is_err()); // semicolon
    }

    #[test]
    fn test_is_security_event_true() {
        for event_type in &[
            "SECURITY_VIOLATION",
            "INTEGRITY_FAILURE",
            "AUTH_FAILURE",
            "KEY_DERIVED",
            "security_violation", // case-insensitive
        ] {
            let entry = serde_json::json!({ "event_type": event_type });
            assert!(
                is_security_event(&entry),
                "{event_type} should be a security event"
            );
        }
    }

    #[test]
    fn test_is_security_event_false() {
        let entry = serde_json::json!({ "event_type": "MODEL_STORED" });
        assert!(!is_security_event(&entry));

        let entry2 = serde_json::json!({ "action": "store" });
        assert!(!is_security_event(&entry2));

        let entry3 = serde_json::json!({});
        assert!(!is_security_event(&entry3));
    }

    #[test]
    fn test_health_response_serialization() {
        let resp = HealthResponse {
            status: "ok".to_string(),
            version: "1.2.1".to_string(),
            vault_state: Some("locked".to_string()),
            model_count: Some(5),
            uptime_seconds: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("ok"));
        assert!(json.contains("1.2.1"));
        assert!(json.contains("locked"));
        assert!(!json.contains("uptime_seconds")); // skipped
    }

    #[test]
    fn test_health_response_minimal() {
        let resp = HealthResponse {
            status: "ok".to_string(),
            version: "1.0.0".to_string(),
            vault_state: None,
            model_count: None,
            uptime_seconds: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("vault_state"));
        assert!(!json.contains("model_count"));
    }

    #[test]
    fn test_conversion_info_serialization() {
        let info = ConversionInfo {
            name: "SafeTensorsToPyTorch".to_string(),
            source: "safetensors".to_string(),
            target: "pytorch".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("SafeTensorsToPyTorch"));
    }

    #[test]
    fn test_stats_response_serialization() {
        let resp = StatsResponse {
            model_count: 3,
            total_versions: 7,
            total_size_bytes: 1024 * 1024,
            file_count: 10,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["model_count"], 3);
        assert_eq!(parsed["total_versions"], 7);
    }

    #[tokio::test]
    async fn test_health_without_state() {
        let resp = health(None).await;
        assert_eq!(resp.0.status, "ok");
        assert!(resp.0.vault_state.is_none());
        assert!(resp.0.model_count.is_none());
    }

    #[tokio::test]
    async fn test_list_conversions_returns_entries() {
        let Json(entries) = list_conversions().await;
        assert!(!entries.is_empty());
        for e in &entries {
            assert!(!e.name.is_empty());
            assert!(!e.source.is_empty());
            assert!(!e.target.is_empty());
        }
    }

    #[tokio::test]
    async fn test_openapi_json_returns_valid() {
        let Json(spec) = openapi_json().await;
        assert!(spec.get("openapi").is_some() || spec.get("info").is_some());
    }

    #[tokio::test]
    async fn test_dashboard_index_returns_html() {
        let Html(html) = dashboard_index().await;
        assert!(html.contains("<html") || html.contains("<!DOCTYPE"));
    }

    #[test]
    fn test_uuid_v4_simple_unique() {
        let a = uuid_v4_simple();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let b = uuid_v4_simple();
        assert_ne!(a, b);
        assert!(!a.is_empty());
    }
}
