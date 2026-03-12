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

/// Maximum audit log size before rotation (10 MiB).
const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024;
/// Number of rotated log files to keep.
const MAX_ROTATED_LOGS: u32 = 9;

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
        // Rotate log if it exceeds the size limit
        self.rotate_if_needed()?;

        let mut options = OpenOptions::new();
        options.create(true).append(true);
        crate::permissions::set_create_mode(&mut options);

        let mut file = options.open(&self.log_file)?;
        // Ensure restrictive ACLs on Windows too
        crate::permissions::restrict_file(&self.log_file)?;
        let json = serde_json::to_string(&entry)?;
        writeln!(file, "{}", json)?;

        Ok(())
    }

    /// Rotate the audit log when it exceeds [`MAX_LOG_SIZE`].
    ///
    /// Keeps up to [`MAX_ROTATED_LOGS`] archived copies:
    /// `audit.log` → `audit.log.1` → `audit.log.2` → … → `audit.log.9` (deleted).
    fn rotate_if_needed(&self) -> Result<()> {
        let size = match std::fs::metadata(&self.log_file) {
            Ok(m) => m.len(),
            Err(_) => return Ok(()), // file doesn't exist yet
        };

        if size < MAX_LOG_SIZE {
            return Ok(());
        }

        // Shift existing rotated files: .9 → delete, .8 → .9, … .1 → .2
        for i in (1..MAX_ROTATED_LOGS).rev() {
            let src = self.log_file.with_extension(format!("log.{i}"));
            let dst = self.log_file.with_extension(format!("log.{}", i + 1));
            if src.exists() {
                let _ = std::fs::rename(&src, &dst);
            }
        }

        // Current → .1
        let rotated = self.log_file.with_extension("log.1");
        let _ = std::fs::rename(&self.log_file, &rotated);

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
    pub fn log_auth(&self, success: bool, _reason: Option<&str>) -> Result<()> {
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
                "Authentication failed".to_string()
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

// ── Trait implementation ─────────────────────────────────────

impl crate::traits::AuditSink for AuditLogger {
    fn emit(&self, entry: AuditEntry) -> Result<()> {
        self.log(entry)
    }

    fn query(&self, limit: Option<usize>) -> Result<Vec<AuditEntry>> {
        self.read_entries(limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::AuditSink;

    #[test]
    fn test_audit_logger_emit_and_query() {
        // Covers lines 197, 198 — AuditSink::query() trait impl
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::new(&log_path).unwrap();

        let sink: &dyn AuditSink = &logger;
        sink.emit(AuditEntry {
            timestamp: Utc::now(),
            event_type: AuditEventType::ModelStored,
            description: "Test entry".into(),
            model_name: Some("m1".into()),
            version: Some(1),
            success: true,
            metadata: None,
        })
        .unwrap();

        let entries = sink.query(Some(10)).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].description, "Test entry");
    }

    #[test]
    fn test_audit_logger_query_limit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::new(&log_path).unwrap();

        for i in 0..5 {
            logger
                .log(AuditEntry {
                    timestamp: Utc::now(),
                    event_type: AuditEventType::ModelStored,
                    description: format!("entry {}", i),
                    model_name: None,
                    version: None,
                    success: true,
                    metadata: None,
                })
                .unwrap();
        }

        let entries = logger.read_entries(Some(3)).unwrap();
        assert_eq!(entries.len(), 3);

        let all = logger.read_entries(None).unwrap();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_log_auth_success() {
        // Covers L143 — log_auth with success=true
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::new(&log_path).unwrap();

        logger.log_auth(true, None).unwrap();

        let entries = logger.read_entries(None).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].success);
        assert_eq!(entries[0].description, "Authentication successful");
        assert!(matches!(entries[0].event_type, AuditEventType::AuthSuccess));
    }

    #[test]
    fn test_log_auth_failure() {
        // Covers L148 — log_auth with success=false
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::new(&log_path).unwrap();

        logger.log_auth(false, Some("bad password")).unwrap();

        let entries = logger.read_entries(None).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].success);
        assert!(entries[0].description.contains("Authentication failed"));
        assert!(matches!(entries[0].event_type, AuditEventType::AuthFailure));
    }

    #[test]
    fn test_log_auth_failure_no_reason() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::new(&log_path).unwrap();

        logger.log_auth(false, None).unwrap();

        let entries = logger.read_entries(None).unwrap();
        assert!(entries[0].description.contains("Authentication failed"));
    }

    #[test]
    fn test_log_security_violation() {
        // Covers L158-166, L173
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::new(&log_path).unwrap();

        logger
            .log_security_violation("Unauthorized access attempt")
            .unwrap();

        let entries = logger.read_entries(None).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].success);
        assert_eq!(entries[0].description, "Unauthorized access attempt");
        assert!(matches!(
            entries[0].event_type,
            AuditEventType::SecurityViolation
        ));
    }

    #[test]
    fn test_read_entries_no_file() {
        // Covers the early-return when log_file doesn't exist
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("nonexistent.log");
        let logger = AuditLogger { log_file: log_path };
        let entries = logger.read_entries(None).unwrap();
        assert!(entries.is_empty());
    }
}
