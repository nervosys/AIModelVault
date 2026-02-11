//! XDG Base Directory Specification compliant configuration
//!
//! Cross-platform support for Linux, macOS, and Windows following XDG standards.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::crypto::compression::{CompressionAlgorithm, CompressionLevel};
use crate::error::{Result, VaultError};

/// AI Model Vault (AIMV) Configuration
///
/// Directory structure:
/// - Config: ~/.config/ai/models/ (or platform equivalent)
/// - Data: ~/.local/share/ai/models/ (or platform equivalent)
/// - Cache: ~/.cache/ai/models/ (or platform equivalent)
/// - Backends: ~/.config/ai/backends/ (cloud storage configs)
/// - Utilities: ~/.config/ai/utilities/ (utility configs)
/// - Databases: ~/.config/ai/databases/ (knowledge bases, training data)
///
/// Compliance:
/// - XDG Base Directory Specification
/// - CMMC AC.3.014: Separate duties of individuals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Version of configuration format
    pub version: String,

    /// Vault settings
    pub vault: VaultSettings,

    /// Cryptographic settings
    pub crypto: CryptoSettings,

    /// Compression settings
    pub compression: CompressionSettings,

    /// Storage settings
    pub storage: StorageSettings,

    /// Security settings
    pub security: SecuritySettings,

    /// Compliance settings
    pub compliance: ComplianceSettings,

    /// Telemetry settings
    #[serde(default)]
    pub telemetry: TelemetrySettings,

    /// Directory paths (not serialized, computed at runtime)
    #[serde(skip)]
    pub dirs: DirectoryPaths,
}

/// Default vault selection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSettings {
    pub default_vault: String,
}

/// Cryptographic algorithm and key derivation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoSettings {
    pub algorithm: String,
    pub kdf: String,
}

/// Compression algorithm and level settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionSettings {
    pub algorithm: String,
    pub level: u8,
}

/// Storage backend behavior settings (versioning, cleanup).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
    pub max_versions: u32,
    pub auto_cleanup: bool,
    pub checkpoint_format: String,
}

/// Security policy settings (passphrase, session timeout, audit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub require_passphrase: bool,
    pub session_timeout_seconds: u64,
    pub audit_log: bool,
}

/// Compliance and regulatory settings (FIPS mode, CVE scanning).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSettings {
    pub fips_mode: bool,
    pub cve_scanning: bool,
    pub audit_retention_days: u32,
}

/// Telemetry and analytics settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySettings {
    /// Whether telemetry is enabled (default: true)
    pub enabled: bool,
    /// Anonymous device ID (auto-generated)
    #[serde(default = "default_device_id")]
    pub device_id: String,
}

fn default_device_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

impl Default for TelemetrySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            device_id: default_device_id(),
        }
    }
}

/// XDG-compliant directory paths for config, data, cache, and logs.
#[derive(Debug, Clone, Default)]
pub struct DirectoryPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub vault_dir: PathBuf,
    pub log_dir: PathBuf,
    pub backends_dir: PathBuf,
    pub utilities_dir: PathBuf,
    pub databases_dir: PathBuf,
}

impl VaultConfig {
    /// Create new configuration with defaults
    pub fn new() -> Result<Self> {
        let dirs = Self::get_project_dirs()?;
        Self::ensure_directories(&dirs)?;

        let config_file = dirs.config_dir.join("config.yaml");

        if config_file.exists() {
            Self::load_from_file(&config_file, dirs)
        } else {
            let config = Self::default_with_dirs(dirs);
            config.save()?;
            Ok(config)
        }
    }

    /// Create configuration with custom directory paths
    pub fn with_dirs(dirs: DirectoryPaths) -> Result<Self> {
        Self::ensure_directories(&dirs)?;
        Ok(Self::default_with_dirs(dirs))
    }

    /// Get XDG project directories for AI Model Vault (AIMV)
    ///
    /// Uses shorter, organized paths:
    /// - ~/.config/ai/models/
    /// - ~/.local/share/ai/models/
    /// - ~/.cache/ai/models/
    /// - ~/.config/ai/backends/
    /// - ~/.config/ai/utilities/
    /// - ~/.config/ai/databases/
    fn get_project_dirs() -> Result<DirectoryPaths> {
        use directories::BaseDirs;

        let base_dirs = BaseDirs::new().ok_or_else(|| {
            VaultError::ConfigError("Failed to determine base directories".to_string())
        })?;

        // Use shorter paths under ~/.config/ai/, ~/.local/share/ai/, etc.
        let config_base = base_dirs.config_dir().join("ai");
        let data_base = base_dirs.data_dir().join("ai");
        let cache_base = base_dirs.cache_dir().join("ai");

        let config_dir = config_base.join("models");
        let data_dir = data_base.join("models");
        let cache_dir = cache_base.join("models");
        let vault_dir = data_dir.join("vaults");
        let log_dir = data_dir.join("logs");
        let backends_dir = config_base.join("backends");
        let utilities_dir = config_base.join("utilities");
        let databases_dir = config_base.join("databases");

        Ok(DirectoryPaths {
            config_dir,
            data_dir,
            cache_dir,
            vault_dir,
            log_dir,
            backends_dir,
            utilities_dir,
            databases_dir,
        })
    }

    /// Ensure all required directories exist with secure permissions
    fn ensure_directories(dirs: &DirectoryPaths) -> Result<()> {
        for dir in [
            &dirs.config_dir,
            &dirs.data_dir,
            &dirs.cache_dir,
            &dirs.vault_dir,
            &dirs.log_dir,
            &dirs.backends_dir,
            &dirs.utilities_dir,
            &dirs.databases_dir,
        ] {
            if !dir.exists() {
                fs::create_dir_all(dir)?;

                // Set restrictive permissions on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = std::fs::Permissions::from_mode(0o700);
                    fs::set_permissions(dir, perms)?;
                }
            }
        }
        Ok(())
    }

    /// Load configuration from file
    fn load_from_file(path: &PathBuf, dirs: DirectoryPaths) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let mut config: VaultConfig = serde_yaml::from_str(&contents)?;
        config.dirs = dirs;
        Ok(config)
    }

    /// Create default configuration with directories
    fn default_with_dirs(dirs: DirectoryPaths) -> Self {
        Self {
            version: "1.0".to_string(),
            vault: VaultSettings {
                default_vault: "default".to_string(),
            },
            crypto: CryptoSettings {
                algorithm: "aes-256-gcm".to_string(),
                kdf: "argon2id".to_string(),
            },
            compression: CompressionSettings {
                algorithm: "gzip".to_string(),
                level: 6,
            },
            storage: StorageSettings {
                max_versions: 10,
                auto_cleanup: true,
                checkpoint_format: "v{version}_{timestamp}".to_string(),
            },
            security: SecuritySettings {
                require_passphrase: true,
                session_timeout_seconds: 3600,
                audit_log: true,
            },
            compliance: ComplianceSettings {
                fips_mode: true,
                cve_scanning: true,
                audit_retention_days: 90,
            },
            telemetry: TelemetrySettings::default(),
            dirs,
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_file = self.dirs.config_dir.join("config.yaml");
        let contents = serde_yaml::to_string(self)?;
        fs::write(&config_file, contents)?;

        // Set secure permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            fs::set_permissions(&config_file, perms)?;
        }

        Ok(())
    }

    /// Get path to specific vault
    pub fn get_vault_path(&self, vault_name: Option<&str>) -> PathBuf {
        let name = vault_name.unwrap_or(&self.vault.default_vault);
        self.dirs.vault_dir.join(name)
    }

    /// Get audit log path
    pub fn get_audit_log_path(&self) -> PathBuf {
        self.dirs.log_dir.join("audit.log")
    }

    /// Get compression algorithm
    pub fn get_compression_algorithm(&self) -> CompressionAlgorithm {
        match self.compression.algorithm.as_str() {
            "gzip" => CompressionAlgorithm::Gzip,
            "lzma" => CompressionAlgorithm::Lzma,
            "none" => CompressionAlgorithm::None,
            _ => CompressionAlgorithm::Gzip,
        }
    }

    /// Get compression level
    pub fn get_compression_level(&self) -> CompressionLevel {
        match self.compression.level {
            0 => CompressionLevel::None,
            1 => CompressionLevel::Fast,
            9 => CompressionLevel::Maximum,
            _ => CompressionLevel::Balanced,
        }
    }
}

/// Note: `VaultConfig::default()` panics if the home directory cannot be determined.
/// Prefer `VaultConfig::new()` which returns `Result` for fallible creation.
impl Default for VaultConfig {
    fn default() -> Self {
        Self::new().expect("Failed to create default configuration: home directory unavailable")
    }
}
