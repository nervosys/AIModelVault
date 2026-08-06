//! Federation endpoints — the server half of [`crate::federation`].
//!
//! `FederationManager` has always been a complete HTTP *client* pointed at
//! these three paths; until now nothing served them, so a node could only sync
//! against a peer that did not exist. These close that loop.
//!
//! Authentication is a shared key in `X-API-Key`, checked against the keys of
//! enabled peers — deliberately not the JWT used by the rest of the API. A
//! peer is a machine holding a long-lived pre-shared secret, not a user with a
//! session, and issuing it a login token would mean granting a peer the full
//! model API to fetch weights.

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;

use crate::federation_transport as transport;
use crate::formats::{ModelFormat, ModelMetadata};

use super::error::ApiError;
use super::server::AppState;

/// Authenticate an inbound federation request.
///
/// Two failures are deliberately reported the same way. A request when
/// federation is off, and a request with a bad key, both return 401 with an
/// identical body: telling an unauthenticated caller whether federation is
/// configured is free reconnaissance.
fn require_peer_auth(headers: &HeaderMap, state: &AppState) -> Result<(), ApiError> {
    let settings = &state.vault_config.federation;

    let unauthorized = || ApiError::unauthorized("Invalid or missing federation key");

    if !settings.enabled {
        return Err(unauthorized());
    }

    let presented = headers
        .get(transport::API_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(unauthorized)?;

    if presented.is_empty() {
        return Err(unauthorized());
    }

    let accepted = transport::accepted_keys(settings).map_err(|e| {
        // A key that cannot be resolved is an operator problem, not a caller
        // problem, and the message may name a KMS path — log it, do not
        // return it.
        eprintln!("federation: failed to resolve peer keys: {e}");
        ApiError::internal("Federation key configuration error")
    })?;

    if accepted.is_empty() || !transport::key_is_accepted(presented, &accepted) {
        return Err(unauthorized());
    }

    Ok(())
}

/// Reject names that could escape the vault namespace.
fn validate_name(name: &str) -> Result<(), ApiError> {
    if name.is_empty() || name.len() > 128 {
        return Err(ApiError::bad_request("Name must be 1-128 characters"));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(ApiError::bad_request(
            "Name must contain only ASCII alphanumerics, hyphens, underscores, or dots",
        ));
    }
    // `..` is alphanumeric-clean but still traversal once joined to a path.
    if name.contains("..") {
        return Err(ApiError::bad_request("Name must not contain '..'"));
    }
    Ok(())
}

/// GET /api/v1/federation/manifest
pub async fn manifest(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_peer_auth(&headers, &state)?;

    let manager = state
        .federation
        .as_ref()
        .ok_or_else(|| ApiError::internal("Federation manager unavailable"))?;

    let vault = state.vault.read().await;
    let models: Vec<(String, Vec<crate::version::ModelVersion>)> = vault
        .list_models()
        .into_iter()
        .map(|name| {
            let versions = vault.list_versions(&name).into_iter().cloned().collect();
            (name, versions)
        })
        .collect();
    drop(vault);

    let manifest = manager.generate_manifest(models).await;
    serde_json::to_value(&manifest)
        .map(Json)
        .map_err(|e| ApiError::internal(format!("Failed to serialize manifest: {e}")))
}

/// GET /api/v1/federation/models/:name/versions/:checkpoint_id
pub async fn get_version(
    State(state): State<Arc<AppState>>,
    Path((name, checkpoint_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    require_peer_auth(&headers, &state)?;
    validate_name(&name)?;
    validate_name(&checkpoint_id)?;

    let vault = state.vault.read().await;

    // Peers address versions by checkpoint id, not by number: version numbers
    // are per-vault sequences and mean different things on different nodes.
    let version = vault
        .list_versions(&name)
        .into_iter()
        .find(|v| transport::federation_checkpoint_id(v) == checkpoint_id)
        .map(|v| v.version)
        .ok_or_else(|| ApiError::not_found("Version not found"))?;

    let data = vault.get_model(&name, Some(version)).map_err(|e| {
        // Vault errors interpolate paths and model names; do not hand those to
        // a peer.
        eprintln!("federation: failed to read {name}/{checkpoint_id}: {e}");
        ApiError::internal("Failed to read model")
    })?;
    drop(vault);

    let body = transport::seal_for_transit(&state.vault_config.federation, &data).map_err(|e| {
        eprintln!("federation: failed to seal {name}/{checkpoint_id}: {e}");
        ApiError::internal("Failed to seal model for transit")
    })?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/octet-stream")],
        body,
    )
        .into_response())
}

/// PUT /api/v1/federation/models/:name/versions/:checkpoint_id
pub async fn put_version(
    State(state): State<Arc<AppState>>,
    Path((name, checkpoint_id)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    require_peer_auth(&headers, &state)?;
    validate_name(&name)?;
    validate_name(&checkpoint_id)?;

    if body.is_empty() {
        return Err(ApiError::bad_request("Empty body"));
    }

    let data = transport::open_from_transit(&state.vault_config.federation, body.to_vec())
        .map_err(|e| ApiError::bad_request(format!("Rejected transfer: {e}")))?;

    let mut vault = state.vault.write().await;

    // Idempotent: a peer retrying a push must not create a duplicate version.
    if vault
        .list_versions(&name)
        .iter()
        .any(|v| transport::federation_checkpoint_id(v) == checkpoint_id)
    {
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "already_present",
                "checkpoint_id": checkpoint_id,
            })),
        ));
    }

    // The sync manifest carries no format field, so the sender may declare one
    // in a header. `from_stored` maps an unrecognised value to a custom format
    // rather than failing -- a peer should not be able to reject a transfer by
    // naming a format this build does not know.
    let declared_format = headers
        .get("X-Model-Format")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .unwrap_or("unknown");

    let mut metadata = ModelMetadata::new(name.clone(), ModelFormat::from_stored(declared_format))
        .with_description(format!("Received from federation peer ({checkpoint_id})"));
    // Preserve the sender's checkpoint id as this version's federation
    // identity, so the next sync recognises it as already present.
    metadata.custom_fields = transport::origin_metadata(&checkpoint_id);

    let version = vault
        .store_model(&name, data, metadata, None)
        .map_err(|e| {
            eprintln!("federation: failed to store {name}/{checkpoint_id}: {e}");
            ApiError::internal("Failed to store model")
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "stored",
            "version": version.version,
            "checkpoint_id": version.checkpoint_id,
        })),
    ))
}
