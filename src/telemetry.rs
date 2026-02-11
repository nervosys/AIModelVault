//! Telemetry and analytics module for AI Model Vault.
//!
//! Collects anonymous usage data to help improve the product.
//! **Enabled by default** — users can opt out via:
//! - Config file: `telemetry.enabled = false`
//! - Environment variable: `AIM_TELEMETRY_ENABLED=false`
//! - CLI flag: `--no-telemetry`
//!
//! ## Data Collected
//!
//! - **Events**: Commands run, features used, errors encountered
//! - **Environment**: OS, architecture, version, feature flags
//! - **Performance**: Operation durations (aggregated)
//! - **Anonymous ID**: Random UUID generated on first run (not linked to user identity)
//!
//! ## Data NOT Collected
//!
//! - Model contents or file data
//! - Passphrases or encryption keys
//! - File paths or model names
//! - Personal information
//! - IP addresses (anonymized by backend)

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::error::Result;

/// Global telemetry instance
static TELEMETRY: OnceLock<Arc<TelemetryClient>> = OnceLock::new();

/// Whether telemetry has been explicitly disabled
static TELEMETRY_DISABLED: AtomicBool = AtomicBool::new(false);

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled (default: true)
    pub enabled: bool,

    /// Anonymous device ID (auto-generated UUID)
    #[serde(default = "generate_device_id")]
    pub device_id: String,

    /// Telemetry endpoint URL
    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    /// Batch size before sending (reduces network calls)
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Flush interval in seconds
    #[serde(default = "default_flush_interval")]
    pub flush_interval_secs: u64,
}

fn generate_device_id() -> String {
    Uuid::new_v4().to_string()
}

fn default_endpoint() -> String {
    "https://telemetry.nervosys.ai/v1/events".to_string()
}

fn default_batch_size() -> usize {
    25
}

fn default_flush_interval() -> u64 {
    300 // 5 minutes
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true, // Enabled by default
            device_id: generate_device_id(),
            endpoint: default_endpoint(),
            batch_size: default_batch_size(),
            flush_interval_secs: default_flush_interval(),
        }
    }
}

/// Telemetry event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TelemetryEvent {
    /// Application started
    AppStart {
        version: String,
        os: String,
        arch: String,
        features: Vec<String>,
    },

    /// Command executed
    CommandRun {
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        subcommand: Option<String>,
        duration_ms: u64,
        success: bool,
    },

    /// Model operation (store, get, delete)
    ModelOperation {
        operation: String,
        format: String,
        size_bucket: String, // "small", "medium", "large", "xlarge"
        duration_ms: u64,
        success: bool,
    },

    /// Format conversion
    Conversion {
        source_format: String,
        target_format: String,
        duration_ms: u64,
        success: bool,
    },

    /// API endpoint called
    ApiCall {
        endpoint: String,
        method: String,
        status_code: u16,
        duration_ms: u64,
    },

    /// Error occurred
    Error {
        error_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
    },

    /// Feature usage
    FeatureUsed {
        feature: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
}

/// Envelope for telemetry events
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TelemetryEnvelope {
    device_id: String,
    session_id: String,
    timestamp: u64,
    event: TelemetryEvent,
}

/// Telemetry client for collecting and sending analytics data
pub struct TelemetryClient {
    config: TelemetryConfig,
    session_id: String,
    events: parking_lot::Mutex<Vec<TelemetryEnvelope>>,
    enabled: AtomicBool,
}

impl TelemetryClient {
    /// Create a new telemetry client
    pub fn new(config: TelemetryConfig) -> Self {
        let enabled = config.enabled && !Self::is_disabled_by_env();

        Self {
            enabled: AtomicBool::new(enabled),
            session_id: Uuid::new_v4().to_string(),
            events: parking_lot::Mutex::new(Vec::new()),
            config,
        }
    }

    /// Check if telemetry is disabled via environment variable
    fn is_disabled_by_env() -> bool {
        std::env::var("AIM_TELEMETRY_ENABLED")
            .map(|v| v.to_lowercase() == "false" || v == "0")
            .unwrap_or(false)
            || std::env::var("DO_NOT_TRACK")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false)
    }

    /// Disable telemetry
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }

    /// Enable telemetry
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
    }

    /// Check if telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// Track an event
    pub fn track(&self, event: TelemetryEvent) {
        if !self.is_enabled() {
            return;
        }

        let envelope = TelemetryEnvelope {
            device_id: self.config.device_id.clone(),
            session_id: self.session_id.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event,
        };

        let mut events = self.events.lock();
        events.push(envelope);

        // Flush if we've reached batch size
        if events.len() >= self.config.batch_size {
            let batch = std::mem::take(&mut *events);
            drop(events);
            self.send_batch(batch);
        }
    }

    /// Flush all pending events
    pub fn flush(&self) {
        if !self.is_enabled() {
            return;
        }

        let batch = std::mem::take(&mut *self.events.lock());
        if !batch.is_empty() {
            self.send_batch(batch);
        }
    }

    /// Send a batch of events to the telemetry endpoint
    fn send_batch(&self, events: Vec<TelemetryEnvelope>) {
        // Fire and forget - don't block on telemetry
        let _endpoint = self.config.endpoint.clone();

        std::thread::spawn(move || {
            // Use a simple blocking HTTP client
            // In production, this would use reqwest or similar
            if let Ok(body) = serde_json::to_string(&events) {
                // For now, just log to a local file for offline collection
                // This avoids adding network dependencies
                let _ = Self::write_to_local_queue(&body);
            }
        });
    }

    /// Write events to local queue file (for offline/batched collection)
    fn write_to_local_queue(body: &str) -> std::io::Result<()> {
        use std::io::Write;

        let queue_dir = directories::BaseDirs::new()
            .map(|d| d.cache_dir().join("ai").join("telemetry"))
            .unwrap_or_else(|| PathBuf::from(".").join(".cache").join("ai").join("telemetry"));

        fs::create_dir_all(&queue_dir)?;

        let queue_file = queue_dir.join("events.jsonl");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(queue_file)?;

        writeln!(file, "{}", body)?;
        Ok(())
    }

    /// Get the device ID
    pub fn device_id(&self) -> &str {
        &self.config.device_id
    }
}

impl Drop for TelemetryClient {
    fn drop(&mut self) {
        self.flush();
    }
}

// === Global telemetry functions ===

/// Initialize the global telemetry client
pub fn init(config: TelemetryConfig) {
    let _ = TELEMETRY.set(Arc::new(TelemetryClient::new(config)));
}

/// Initialize telemetry with default config, loading from disk if available
pub fn init_default(config_dir: Option<&PathBuf>) -> Result<()> {
    let config = load_or_create_config(config_dir)?;
    init(config);
    Ok(())
}

/// Load telemetry config from disk or create default
fn load_or_create_config(config_dir: Option<&PathBuf>) -> Result<TelemetryConfig> {
    let config_path = config_dir
        .cloned()
        .or_else(|| {
            directories::BaseDirs::new()
                .map(|d| d.config_dir().join("ai").join("models"))
        })
        .map(|d| d.join("telemetry.yaml"));

    if let Some(path) = &config_path {
        if path.exists() {
            let contents = fs::read_to_string(path)?;
            let config: TelemetryConfig = serde_yaml::from_str(&contents)?;
            return Ok(config);
        }
    }

    // Create default config and save it
    let config = TelemetryConfig::default();

    if let Some(path) = config_path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = serde_yaml::to_string(&config)?;
        fs::write(&path, contents)?;
    }

    Ok(config)
}

/// Disable telemetry globally
pub fn disable() {
    TELEMETRY_DISABLED.store(true, Ordering::SeqCst);
    if let Some(client) = TELEMETRY.get() {
        client.disable();
    }
}

/// Check if telemetry is enabled
pub fn is_enabled() -> bool {
    !TELEMETRY_DISABLED.load(Ordering::SeqCst)
        && TELEMETRY.get().map(|c| c.is_enabled()).unwrap_or(false)
}

/// Track an event
pub fn track(event: TelemetryEvent) {
    if let Some(client) = TELEMETRY.get() {
        client.track(event);
    }
}

/// Flush pending events
pub fn flush() {
    if let Some(client) = TELEMETRY.get() {
        client.flush();
    }
}

// === Convenience tracking functions ===

/// Track application start
pub fn track_app_start() {
    let features = collect_enabled_features();

    track(TelemetryEvent::AppStart {
        version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        features,
    });
}

/// Track a command execution
pub fn track_command(command: &str, subcommand: Option<&str>, duration: Duration, success: bool) {
    track(TelemetryEvent::CommandRun {
        command: command.to_string(),
        subcommand: subcommand.map(|s| s.to_string()),
        duration_ms: duration.as_millis() as u64,
        success,
    });
}

/// Track a model operation
pub fn track_model_op(
    operation: &str,
    format: &str,
    size_bytes: u64,
    duration: Duration,
    success: bool,
) {
    let size_bucket = match size_bytes {
        0..=10_000_000 => "small",         // < 10MB
        10_000_001..=100_000_000 => "medium", // 10MB - 100MB
        100_000_001..=1_000_000_000 => "large", // 100MB - 1GB
        _ => "xlarge",                      // > 1GB
    };

    track(TelemetryEvent::ModelOperation {
        operation: operation.to_string(),
        format: format.to_string(),
        size_bucket: size_bucket.to_string(),
        duration_ms: duration.as_millis() as u64,
        success,
    });
}

/// Track format conversion
pub fn track_conversion(
    source_format: &str,
    target_format: &str,
    duration: Duration,
    success: bool,
) {
    track(TelemetryEvent::Conversion {
        source_format: source_format.to_string(),
        target_format: target_format.to_string(),
        duration_ms: duration.as_millis() as u64,
        success,
    });
}

/// Track an API call
pub fn track_api_call(endpoint: &str, method: &str, status_code: u16, duration: Duration) {
    track(TelemetryEvent::ApiCall {
        endpoint: endpoint.to_string(),
        method: method.to_string(),
        status_code,
        duration_ms: duration.as_millis() as u64,
    });
}

/// Track an error
pub fn track_error(error_type: &str, context: Option<&str>) {
    track(TelemetryEvent::Error {
        error_type: error_type.to_string(),
        context: context.map(|s| s.to_string()),
    });
}

/// Track feature usage
pub fn track_feature(feature: &str, detail: Option<&str>) {
    track(TelemetryEvent::FeatureUsed {
        feature: feature.to_string(),
        detail: detail.map(|s| s.to_string()),
    });
}

/// Collect enabled feature flags
fn collect_enabled_features() -> Vec<String> {
    let mut features = Vec::new();

    #[cfg(feature = "api")]
    features.push("api".to_string());

    #[cfg(feature = "python")]
    features.push("python".to_string());

    #[cfg(feature = "cloud")]
    features.push("cloud".to_string());

    features
}

/// Timer guard for automatic duration tracking
pub struct TrackingTimer {
    start: Instant,
    command: String,
    subcommand: Option<String>,
}

impl TrackingTimer {
    pub fn new(command: &str, subcommand: Option<&str>) -> Self {
        Self {
            start: Instant::now(),
            command: command.to_string(),
            subcommand: subcommand.map(|s| s.to_string()),
        }
    }

    pub fn finish(self, success: bool) {
        track_command(
            &self.command,
            self.subcommand.as_deref(),
            self.start.elapsed(),
            success,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(config.enabled);
        assert!(!config.device_id.is_empty());
        assert!(config.endpoint.contains("telemetry"));
    }

    #[test]
    fn test_telemetry_client_disable() {
        let config = TelemetryConfig::default();
        let client = TelemetryClient::new(config);

        assert!(client.is_enabled());
        client.disable();
        assert!(!client.is_enabled());
    }

    #[test]
    fn test_size_bucket() {
        // Test the size bucketing logic
        let check_bucket = |size: u64| -> &'static str {
            match size {
                0..=10_000_000 => "small",
                10_000_001..=100_000_000 => "medium",
                100_000_001..=1_000_000_000 => "large",
                _ => "xlarge",
            }
        };

        assert_eq!(check_bucket(1_000), "small");
        assert_eq!(check_bucket(50_000_000), "medium");
        assert_eq!(check_bucket(500_000_000), "large");
        assert_eq!(check_bucket(2_000_000_000), "xlarge");
    }

    #[test]
    fn test_event_serialization() {
        let event = TelemetryEvent::CommandRun {
            command: "store".to_string(),
            subcommand: None,
            duration_ms: 150,
            success: true,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("command_run"));
        assert!(json.contains("store"));
    }
}
