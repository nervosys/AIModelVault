//! Error types for NeuralVault

use std::io;
use thiserror::Error;

/// Result type alias for NeuralVault operations
pub type Result<T> = std::result::Result<T, VaultError>;

/// NeuralVault error types
#[derive(Error, Debug)]
pub enum VaultError {
    /// Cryptographic operation failed
    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    /// Invalid passphrase or authentication failure
    #[error("Authentication failed: invalid passphrase or corrupted data")]
    AuthenticationFailed,

    /// Data integrity check failed
    #[error("Integrity check failed: {0}")]
    IntegrityError(String),

    /// Version control error
    #[error("Version control error: {0}")]
    VersionError(String),

    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Version not found
    #[error("Version {0} not found for model {1}")]
    VersionNotFound(u32, String),

    /// Format conversion error
    #[error("Format conversion error: {0}")]
    ConversionError(String),

    /// Unsupported model format
    #[error("Unsupported model format: {0}")]
    UnsupportedFormat(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Compression error
    #[error("Compression error: {0}")]
    CompressionError(String),

    /// Security policy violation
    #[error("Security policy violation: {0}")]
    SecurityViolation(String),

    /// Compliance violation
    #[error("Compliance violation: {0}")]
    ComplianceViolation(String),

    /// Audit log error
    #[error("Audit log error: {0}")]
    AuditError(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Storage/database error
    #[error("Storage error: {0}")]
    StorageError(String),
}

impl From<serde_json::Error> for VaultError {
    fn from(err: serde_json::Error) -> Self {
        VaultError::SerializationError(err.to_string())
    }
}

impl From<serde_yaml::Error> for VaultError {
    fn from(err: serde_yaml::Error) -> Self {
        VaultError::SerializationError(err.to_string())
    }
}

impl From<zip::result::ZipError> for VaultError {
    fn from(err: zip::result::ZipError) -> Self {
        VaultError::IoError(io::Error::other(err.to_string()))
    }
}
