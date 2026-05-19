//! Vault backup scheduling.
//!
//! Defines backup schedules, manages backup metadata, and supports
//! automatic rotation with configurable retention.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::{Result, VaultError};

// ── Types ────────────────────────────────────────────────────────────────────

/// Backup frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackupFrequency {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

impl std::fmt::Display for BackupFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hourly => write!(f, "hourly"),
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
            Self::Monthly => write!(f, "monthly"),
        }
    }
}

impl std::str::FromStr for BackupFrequency {
    type Err = VaultError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hourly" | "1h" => Ok(Self::Hourly),
            "daily" | "1d" => Ok(Self::Daily),
            "weekly" | "1w" => Ok(Self::Weekly),
            "monthly" | "1m" => Ok(Self::Monthly),
            _ => Err(VaultError::InvalidInput(format!(
                "Unknown backup frequency: {s}"
            ))),
        }
    }
}

/// A backup schedule configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    /// Schedule name.
    pub name: String,
    /// How often to run.
    pub frequency: BackupFrequency,
    /// Maximum number of backups to retain.
    pub max_backups: usize,
    /// Output directory for backup archives.
    pub output_dir: PathBuf,
    /// Whether the schedule is enabled.
    pub enabled: bool,
    /// When the schedule was created.
    pub created_at: String,
}

/// Metadata for a completed backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    /// Backup archive path.
    pub path: PathBuf,
    /// When the backup was created.
    pub timestamp: String,
    /// Size of the backup in bytes.
    pub size_bytes: u64,
    /// Schedule that triggered it.
    pub schedule_name: String,
}

/// Report from running a backup.
#[derive(Debug, Clone, Serialize)]
pub struct BackupReport {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub models_included: usize,
    pub rotated: usize,
}

// ── Store ────────────────────────────────────────────────────────────────────

const SCHEDULES_FILE: &str = "backup_schedules.json";
const HISTORY_FILE: &str = "backup_history.json";

/// Manages backup schedules and history.
#[derive(Debug)]
pub struct BackupManager {
    base_path: PathBuf,
    schedules: BTreeMap<String, BackupSchedule>,
    history: Vec<BackupRecord>,
}

impl BackupManager {
    /// Open or create backup manager.
    pub fn new(vault_path: &Path) -> Result<Self> {
        let base_path = vault_path.to_path_buf();

        let sched_path = base_path.join(SCHEDULES_FILE);
        let schedules: BTreeMap<String, BackupSchedule> = if sched_path.exists() {
            let data = std::fs::read_to_string(&sched_path)
                .map_err(|e| VaultError::StorageError(format!("read backup schedules: {e}")))?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            BTreeMap::new()
        };

        let hist_path = base_path.join(HISTORY_FILE);
        let history: Vec<BackupRecord> = if hist_path.exists() {
            let data = std::fs::read_to_string(&hist_path)
                .map_err(|e| VaultError::StorageError(format!("read backup history: {e}")))?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(Self {
            base_path,
            schedules,
            history,
        })
    }

    fn save_schedules(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.schedules)
            .map_err(|e| VaultError::StorageError(format!("serialize schedules: {e}")))?;
        std::fs::write(self.base_path.join(SCHEDULES_FILE), data)
            .map_err(|e| VaultError::StorageError(format!("write schedules: {e}")))
    }

    fn save_history(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.history)
            .map_err(|e| VaultError::StorageError(format!("serialize history: {e}")))?;
        std::fs::write(self.base_path.join(HISTORY_FILE), data)
            .map_err(|e| VaultError::StorageError(format!("write history: {e}")))
    }

    /// Add or update a backup schedule.
    pub fn set_schedule(&mut self, schedule: BackupSchedule) -> Result<()> {
        self.schedules.insert(schedule.name.clone(), schedule);
        self.save_schedules()
    }

    /// Remove a schedule.
    pub fn remove_schedule(&mut self, name: &str) -> Result<bool> {
        let removed = self.schedules.remove(name).is_some();
        if removed {
            self.save_schedules()?;
        }
        Ok(removed)
    }

    /// Get a schedule by name.
    pub fn get_schedule(&self, name: &str) -> Option<&BackupSchedule> {
        self.schedules.get(name)
    }

    /// List all schedules.
    pub fn list_schedules(&self) -> Vec<&BackupSchedule> {
        self.schedules.values().collect()
    }

    /// Record a completed backup.
    pub fn record_backup(&mut self, record: BackupRecord) -> Result<()> {
        self.history.push(record);
        self.save_history()
    }

    /// Get backup history, optionally filtered by schedule.
    pub fn get_history(&self, schedule: Option<&str>) -> Vec<&BackupRecord> {
        self.history
            .iter()
            .filter(|r| schedule.is_none_or(|s| r.schedule_name == s))
            .collect()
    }

    /// Get backups that should be rotated (oldest first, exceeding max_backups).
    pub fn rotatable_backups(&self, schedule_name: &str) -> Vec<&BackupRecord> {
        let schedule = match self.schedules.get(schedule_name) {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut schedule_backups: Vec<&BackupRecord> = self
            .history
            .iter()
            .filter(|r| r.schedule_name == schedule_name)
            .collect();

        // Sort by timestamp ascending (oldest first)
        schedule_backups.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        if schedule_backups.len() > schedule.max_backups {
            let excess = schedule_backups.len() - schedule.max_backups;
            schedule_backups[..excess].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Total backup count.
    pub fn backup_count(&self) -> usize {
        self.history.len()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_schedule(name: &str, max: usize) -> BackupSchedule {
        BackupSchedule {
            name: name.into(),
            frequency: BackupFrequency::Daily,
            max_backups: max,
            output_dir: PathBuf::from("/tmp/backups"),
            enabled: true,
            created_at: "2025-01-01T00:00:00Z".into(),
        }
    }

    fn make_record(schedule: &str, ts: &str) -> BackupRecord {
        BackupRecord {
            path: PathBuf::from(format!("/tmp/backups/{ts}.tar.gz")),
            timestamp: ts.into(),
            size_bytes: 1024,
            schedule_name: schedule.into(),
        }
    }

    #[test]
    fn test_schedule_crud() {
        let dir = tempdir().unwrap();
        let mut mgr = BackupManager::new(dir.path()).unwrap();

        mgr.set_schedule(make_schedule("nightly", 7)).unwrap();
        assert_eq!(mgr.list_schedules().len(), 1);
        assert!(mgr.get_schedule("nightly").is_some());

        mgr.remove_schedule("nightly").unwrap();
        assert!(mgr.list_schedules().is_empty());
    }

    #[test]
    fn test_backup_history() {
        let dir = tempdir().unwrap();
        let mut mgr = BackupManager::new(dir.path()).unwrap();

        mgr.record_backup(make_record("nightly", "2025-01-01"))
            .unwrap();
        mgr.record_backup(make_record("nightly", "2025-01-02"))
            .unwrap();
        mgr.record_backup(make_record("weekly", "2025-01-01"))
            .unwrap();

        assert_eq!(mgr.backup_count(), 3);
        assert_eq!(mgr.get_history(Some("nightly")).len(), 2);
        assert_eq!(mgr.get_history(Some("weekly")).len(), 1);
        assert_eq!(mgr.get_history(None).len(), 3);
    }

    #[test]
    fn test_rotatable_backups() {
        let dir = tempdir().unwrap();
        let mut mgr = BackupManager::new(dir.path()).unwrap();

        mgr.set_schedule(make_schedule("nightly", 2)).unwrap();

        mgr.record_backup(make_record("nightly", "2025-01-01"))
            .unwrap();
        mgr.record_backup(make_record("nightly", "2025-01-02"))
            .unwrap();
        mgr.record_backup(make_record("nightly", "2025-01-03"))
            .unwrap();

        let rot = mgr.rotatable_backups("nightly");
        assert_eq!(rot.len(), 1);
        assert_eq!(rot[0].timestamp, "2025-01-01");
    }

    #[test]
    fn test_frequency_roundtrip() {
        for freq in [
            BackupFrequency::Hourly,
            BackupFrequency::Daily,
            BackupFrequency::Weekly,
            BackupFrequency::Monthly,
        ] {
            let s = freq.to_string();
            let parsed: BackupFrequency = s.parse().unwrap();
            assert_eq!(parsed, freq);
        }
    }

    #[test]
    fn test_frequency_aliases() {
        assert_eq!(
            "1h".parse::<BackupFrequency>().unwrap(),
            BackupFrequency::Hourly
        );
        assert_eq!(
            "1d".parse::<BackupFrequency>().unwrap(),
            BackupFrequency::Daily
        );
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();

        {
            let mut mgr = BackupManager::new(dir.path()).unwrap();
            mgr.set_schedule(make_schedule("nightly", 7)).unwrap();
            mgr.record_backup(make_record("nightly", "2025-01-01"))
                .unwrap();
        }

        let mgr = BackupManager::new(dir.path()).unwrap();
        assert_eq!(mgr.list_schedules().len(), 1);
        assert_eq!(mgr.backup_count(), 1);
    }
}
