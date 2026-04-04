//! Webhook / notification system — fire HTTP callbacks on vault events.
//!
//! Implements the `EventSubscriber` trait so it slots into the existing
//! `EventBus`.  Webhook targets are persisted in `webhooks.json`.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, VaultError};
use crate::traits::{EventSubscriber, VaultEvent};

// ── Types ────────────────────────────────────────────────────────────────────

/// A registered webhook target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookTarget {
    /// Unique identifier.
    pub id: String,
    /// Destination URL (HTTPS required in production).
    pub url: String,
    /// Optional shared secret for HMAC-SHA256 payload signing.
    pub secret: Option<String>,
    /// Events this hook should fire for (empty = all).
    pub events: Vec<String>,
    /// Whether this webhook is currently enabled.
    pub enabled: bool,
}

/// Webhook delivery payload.
#[derive(Debug, Clone, Serialize)]
pub struct WebhookPayload {
    pub event: String,
    pub timestamp: String,
    pub details: HashMap<String, String>,
}

/// Persisted webhook configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub targets: Vec<WebhookTarget>,
}

// ── Store ────────────────────────────────────────────────────────────────────

/// Manages webhook registrations alongside a vault.
pub struct WebhookStore {
    path: PathBuf,
    config: WebhookConfig,
}

impl WebhookStore {
    const FILE_NAME: &'static str = "webhooks.json";

    pub fn new(vault_path: &Path) -> Result<Self> {
        let path = vault_path.join(Self::FILE_NAME);
        let config = if path.exists() {
            let data = fs::read_to_string(&path)?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            WebhookConfig::default()
        };
        Ok(Self { path, config })
    }

    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.path, json)?;
        crate::permissions::restrict_file(&self.path)?;
        Ok(())
    }

    /// Register a new webhook target.
    pub fn add(&mut self, target: WebhookTarget) -> Result<()> {
        if self.config.targets.iter().any(|t| t.id == target.id) {
            return Err(VaultError::InvalidInput(format!(
                "Webhook '{}' already exists",
                target.id
            )));
        }
        self.config.targets.push(target);
        self.save()
    }

    /// Remove a webhook by ID.
    pub fn remove(&mut self, id: &str) -> Result<bool> {
        let before = self.config.targets.len();
        self.config.targets.retain(|t| t.id != id);
        if self.config.targets.len() < before {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// List all registered webhooks.
    pub fn list(&self) -> &[WebhookTarget] {
        &self.config.targets
    }

    /// Get targets that should fire for a given event name.
    pub fn targets_for_event(&self, event: &str) -> Vec<&WebhookTarget> {
        self.config
            .targets
            .iter()
            .filter(|t| t.enabled && (t.events.is_empty() || t.events.iter().any(|e| e == event)))
            .collect()
    }

    /// Fire a payload to matching targets (blocking HTTP POST).
    ///
    /// Delivery failures are logged but do not propagate errors to callers.
    pub fn fire(&self, event: &str, details: HashMap<String, String>) {
        let payload = WebhookPayload {
            event: event.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            details,
        };

        let body = match serde_json::to_string(&payload) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("webhook: failed to serialise payload: {e}");
                return;
            }
        };

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build();

        let client = match client {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("webhook: failed to build HTTP client: {e}");
                return;
            }
        };

        for target in self.targets_for_event(event) {
            let mut req = client
                .post(&target.url)
                .header("Content-Type", "application/json")
                .header("X-AIM-Event", event);

            // HMAC-SHA256 signature
            if let Some(ref secret) = target.secret {
                use sha2::Digest;
                let mut mac = sha2::Sha256::new();
                mac.update(secret.as_bytes());
                mac.update(body.as_bytes());
                let sig = hex::encode(mac.finalize());
                req = req.header("X-AIM-Signature", sig);
            }

            match req.body(body.clone()).send() {
                Ok(resp) => {
                    tracing::debug!("webhook {}: status {}", target.id, resp.status());
                }
                Err(e) => {
                    tracing::warn!("webhook {}: delivery failed: {e}", target.id);
                }
            }
        }
    }
}

// ── EventSubscriber integration ──────────────────────────────────────────────

impl EventSubscriber for WebhookStore {
    fn on_event(&self, event: &VaultEvent) -> Result<()> {
        let (name, details) = event_to_webhook(event);
        self.fire(&name, details);
        Ok(())
    }

    fn name(&self) -> &str {
        "WebhookStore"
    }
}

fn event_to_webhook(event: &VaultEvent) -> (String, HashMap<String, String>) {
    let mut details = HashMap::new();
    let name = match event {
        VaultEvent::VaultCreated { vault, .. } => {
            details.insert("vault".into(), vault.clone());
            "vault.created"
        }
        VaultEvent::VaultUnlocked { vault, .. } => {
            details.insert("vault".into(), vault.clone());
            "vault.unlocked"
        }
        VaultEvent::VaultLocked { vault, .. } => {
            details.insert("vault".into(), vault.clone());
            "vault.locked"
        }
        VaultEvent::ModelStored {
            vault,
            model,
            version,
            format,
            size,
            checksum,
            ..
        } => {
            details.insert("vault".into(), vault.clone());
            details.insert("model".into(), model.clone());
            details.insert("version".into(), version.to_string());
            details.insert("format".into(), format.clone());
            details.insert("size".into(), size.to_string());
            details.insert("checksum".into(), checksum.clone());
            "model.stored"
        }
        VaultEvent::ModelRetrieved {
            vault,
            model,
            version,
            ..
        } => {
            details.insert("vault".into(), vault.clone());
            details.insert("model".into(), model.clone());
            details.insert("version".into(), version.to_string());
            "model.retrieved"
        }
        VaultEvent::ModelDeleted {
            vault,
            model,
            version,
            ..
        } => {
            details.insert("vault".into(), vault.clone());
            details.insert("model".into(), model.clone());
            details.insert("version".into(), version.to_string());
            "model.deleted"
        }
        VaultEvent::PassphraseChanged {
            vault,
            files_reencrypted,
            ..
        } => {
            details.insert("vault".into(), vault.clone());
            details.insert("files_reencrypted".into(), files_reencrypted.to_string());
            "vault.passphrase_changed"
        }
        VaultEvent::IntegrityFailed {
            vault,
            model,
            version,
            expected,
            actual,
            ..
        } => {
            details.insert("vault".into(), vault.clone());
            details.insert("model".into(), model.clone());
            details.insert("version".into(), version.to_string());
            details.insert("expected".into(), expected.clone());
            details.insert("actual".into(), actual.clone());
            "integrity.failed"
        }
        VaultEvent::ComplianceChecked { vault, passed, .. } => {
            details.insert("vault".into(), vault.clone());
            details.insert("passed".into(), passed.to_string());
            "compliance.checked"
        }
    };
    (name.to_string(), details)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_store_crud() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = WebhookStore::new(dir.path()).unwrap();

        let target = WebhookTarget {
            id: "test-1".into(),
            url: "https://example.com/hook".into(),
            secret: None,
            events: vec!["model.stored".into()],
            enabled: true,
        };

        store.add(target).unwrap();
        assert_eq!(store.list().len(), 1);

        // Duplicate should fail
        let dup = WebhookTarget {
            id: "test-1".into(),
            url: "https://example.com/other".into(),
            secret: None,
            events: vec![],
            enabled: true,
        };
        assert!(store.add(dup).is_err());

        assert!(store.remove("test-1").unwrap());
        assert_eq!(store.list().len(), 0);
    }

    #[test]
    fn test_targets_for_event() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = WebhookStore::new(dir.path()).unwrap();

        store
            .add(WebhookTarget {
                id: "all".into(),
                url: "https://example.com/all".into(),
                secret: None,
                events: vec![],
                enabled: true,
            })
            .unwrap();

        store
            .add(WebhookTarget {
                id: "store-only".into(),
                url: "https://example.com/store".into(),
                secret: None,
                events: vec!["model.stored".into()],
                enabled: true,
            })
            .unwrap();

        store
            .add(WebhookTarget {
                id: "disabled".into(),
                url: "https://example.com/dis".into(),
                secret: None,
                events: vec![],
                enabled: false,
            })
            .unwrap();

        let targets = store.targets_for_event("model.stored");
        assert_eq!(targets.len(), 2); // "all" + "store-only"

        let targets = store.targets_for_event("model.deleted");
        assert_eq!(targets.len(), 1); // only "all"
    }

    #[test]
    fn test_event_to_webhook_mapping() {
        let event = VaultEvent::ModelStored {
            vault: "test-vault".into(),
            model: "llama".into(),
            version: 3,
            format: "gguf".into(),
            size: 1024,
            checksum: "abc123".into(),
            timestamp: chrono::Utc::now(),
        };
        let (name, details) = event_to_webhook(&event);
        assert_eq!(name, "model.stored");
        assert_eq!(details.get("model").unwrap(), "llama");
    }
}
