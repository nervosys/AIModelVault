//! Audit logging for compliance and security monitoring
//!
//! Compliance:
//! - CMMC AU.3.046: Create and retain audit logs
//! - CMMC AU.3.049: Protect audit information
//! - NIST SP 800-53 AU family

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::Result;

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditEventType {
    /// Vault created
    VaultCreated,
    /// Vault opened
    VaultOpened,
    /// Model stored
    ModelStored,
    /// Model retrieved
    ModelRetrieved,
    /// Model deleted
    ModelDeleted,
    /// Version deleted
    VersionDeleted,
    /// Authentication succeeded
    AuthSuccess,
    /// Authentication failed
    AuthFailure,
    /// Configuration changed
    ConfigChanged,
    /// Security violation detected
    SecurityViolation,
    /// Integrity check failed
    IntegrityFailure,
    /// Key derived
    KeyDerived,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp of event
    pub timestamp: DateTime<Utc>,

    /// Event type
    pub event_type: AuditEventType,

    /// Event description
    pub description: String,

    /// Associated model name (if applicable)
    pub model_name: Option<String>,

    /// Associated version (if applicable)
    pub version: Option<u32>,

    /// Success status
    pub success: bool,

    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Audit logger
pub struct AuditLogger {
    log_file: PathBuf,
}

impl AuditLogger {
    /// Create new audit logger
    pub fn new(log_path: &Path) -> Result<Self> {
        // Ensure log directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self {
            log_file: log_path.to_path_buf(),
        })
    }

    /// Log an audit event
    pub fn log(&self, entry: AuditEntry) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)?;

        let json = serde_json::to_string(&entry)?;
        writeln!(file, "{}", json)?;

        // Set restrictive permissions on first write
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.log_file, perms)?;
        }

        Ok(())
    }

    /// Log a model storage event
    pub fn log_model_stored(&self, model_name: &str, version: u32, success: bool) -> Result<()> {
        self.log(AuditEntry {
            timestamp: Utc::now(),
            event_type: AuditEventType::ModelStored,
            description: format!("Model '{}' version {} stored", model_name, version),
            model_name: Some(model_name.to_string()),
            version: Some(version),
            success,
            metadata: None,
        })
    }

    /// Log a model retrieval event
    pub fn log_model_retrieved(&self, model_name: &str, version: u32, success: bool) -> Result<()> {
        self.log(AuditEntry {
            timestamp: Utc::now(),
            event_type: AuditEventType::ModelRetrieved,
            description: format!("Model '{}' version {} retrieved", model_name, version),
            model_name: Some(model_name.to_string()),
            version: Some(version),
            success,
            metadata: None,
        })
    }

    /// Log an authentication event
    pub fn log_auth(&self, success: bool, reason: Option<&str>) -> Result<()> {
        self.log(AuditEntry {
            timestamp: Utc::now(),
            event_type: if success {
                AuditEventType::AuthSuccess
            } else {
                AuditEventType::AuthFailure
            },
            description: if success {
                "Authentication successful".to_string()
            } else {
                format!("Authentication failed: {}", reason.unwrap_or("unknown"))
            },
            model_name: None,
            version: None,
            success,
            metadata: None,
        })
    }

    /// Log a security violation
    pub fn log_security_violation(&self, description: &str) -> Result<()> {
        self.log(AuditEntry {
            timestamp: Utc::now(),
            event_type: AuditEventType::SecurityViolation,
            description: description.to_string(),
            model_name: None,
            version: None,
            success: false,
            metadata: None,
        })
    }

    /// Read audit log entries
    pub fn read_entries(&self, limit: Option<usize>) -> Result<Vec<AuditEntry>> {
        if !self.log_file.exists() {
            return Ok(Vec::new());
        }

        let contents = std::fs::read_to_string(&self.log_file)?;
        let mut entries: Vec<AuditEntry> = contents
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        if let Some(n) = limit {
            entries.truncate(n);
        }

        Ok(entries)
    }
}
