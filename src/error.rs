//! Error types for AI Model Vault
//!
//! The top-level [`VaultError`] enum covers all failure modes.  Domain-specific
//! sub-error types ([`CryptoError`], [`StorageError`], [`ConversionError`])
//! carry richer context and convert into `VaultError` via `From`.

use std::io;
use thiserror::Error;

/// Result type alias for AI Model Vault operations
pub type Result<T> = std::result::Result<T, VaultError>;

// ── Domain-specific error types ─────────────────────────────────────────────

/// Errors originating from cryptographic operations.
#[derive(Error, Debug)]
pub enum CryptoError {
    /// Key derivation failure (Argon2id / PBKDF2)
    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),

    /// Encryption failure
    #[error("Encryption failed: {0}")]
    Encryption(String),

    /// Decryption failure (wrong key, corrupted ciphertext, …)
    #[error("Decryption failed: {0}")]
    Decryption(String),

    /// Data integrity check mismatch (HMAC / SHA-256)
    #[error("Integrity check failed: {0}")]
    Integrity(String),

    /// Generic / uncategorised crypto error
    #[error("Cryptographic error: {0}")]
    Other(String),
}

/// Errors originating from storage and I/O operations.
#[derive(Error, Debug)]
pub enum StorageError {
    /// Underlying I/O error
    #[error(transparent)]
    Io(#[from] io::Error),

    /// Serialization / deserialization failure
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Compression / decompression failure
    #[error("Compression error: {0}")]
    Compression(String),

    /// Database backend error (SQLite, Sled, …)
    #[error("Database error: {0}")]
    Database(String),

    /// Generic storage error
    #[error("Storage error: {0}")]
    Other(String),
}

/// Errors originating from model format conversion.
#[derive(Error, Debug)]
pub enum ConversionError {
    /// Requested conversion path is not supported
    #[error("Unsupported conversion: {0}")]
    Unsupported(String),

    /// Validation of the converted output failed
    #[error("Validation failed: {0}")]
    Validation(String),

    /// Generic conversion error
    #[error("Conversion error: {0}")]
    Other(String),
}

// ── From impls: domain → VaultError ─────────────────────────────────────────

impl From<CryptoError> for VaultError {
    fn from(err: CryptoError) -> Self {
        match err {
            CryptoError::Integrity(msg) => VaultError::IntegrityError(msg),
            other => VaultError::CryptoError(other.to_string()),
        }
    }
}

impl From<StorageError> for VaultError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::Io(e) => VaultError::IoError(e),
            StorageError::Serialization(msg) => VaultError::SerializationError(msg),
            StorageError::Compression(msg) => VaultError::CompressionError(msg),
            StorageError::Database(msg) => VaultError::StorageError(msg),
            StorageError::Other(msg) => VaultError::StorageError(msg),
        }
    }
}

impl From<ConversionError> for VaultError {
    fn from(err: ConversionError) -> Self {
        match err {
            ConversionError::Unsupported(msg) => VaultError::UnsupportedFormat(msg),
            other => VaultError::ConversionError(other.to_string()),
        }
    }
}

// ── Top-level error ─────────────────────────────────────────────────────────

/// AI Model Vault error types
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

impl From<serde_yaml_ng::Error> for VaultError {
    fn from(err: serde_yaml_ng::Error) -> Self {
        VaultError::SerializationError(err.to_string())
    }
}

impl From<zip::result::ZipError> for VaultError {
    fn from(err: zip::result::ZipError) -> Self {
        VaultError::IoError(io::Error::other(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zip_error_conversion() {
        // Covers lines 94, 95 — From<ZipError>
        let zip_err = zip::result::ZipError::FileNotFound;
        let vault_err: VaultError = zip_err.into();
        match vault_err {
            VaultError::IoError(_) => {} // expected
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_serde_yml_error_conversion() {
        let yaml_err = serde_yaml_ng::from_str::<serde_yaml_ng::Value>("\t").unwrap_err();
        let vault_err: VaultError = yaml_err.into();
        match vault_err {
            VaultError::SerializationError(_) => {}
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_serde_json_error_conversion() {
        // Covers L81-83 — From<serde_json::Error>
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let vault_err: VaultError = json_err.into();
        match vault_err {
            VaultError::SerializationError(msg) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_error_display_messages() {
        // Covers L7 (type alias used implicitly) + various Display branches
        let errors: Vec<VaultError> = vec![
            VaultError::CryptoError("crypto fail".into()),
            VaultError::AuthenticationFailed,
            VaultError::IntegrityError("integrity fail".into()),
            VaultError::VersionError("version fail".into()),
            VaultError::ModelNotFound("model1".into()),
            VaultError::VersionNotFound(3, "model1".into()),
            VaultError::ConversionError("conv fail".into()),
            VaultError::UnsupportedFormat("xyz".into()),
            VaultError::ConfigError("config fail".into()),
            VaultError::SerializationError("serde fail".into()),
            VaultError::CompressionError("comp fail".into()),
            VaultError::SecurityViolation("sec fail".into()),
            VaultError::ComplianceViolation("cc fail".into()),
            VaultError::AuditError("audit fail".into()),
            VaultError::InvalidInput("bad input".into()),
            VaultError::StorageError("store fail".into()),
        ];

        let expected_substrings = [
            "crypto fail",
            "invalid passphrase",
            "integrity fail",
            "version fail",
            "model1",
            "Version 3 not found for model model1",
            "conv fail",
            "xyz",
            "config fail",
            "serde fail",
            "comp fail",
            "sec fail",
            "cc fail",
            "audit fail",
            "bad input",
            "store fail",
        ];

        for (err, expected) in errors.iter().zip(expected_substrings.iter()) {
            let msg = format!("{}", err);
            assert!(
                msg.contains(expected),
                "Error '{}' should contain '{}'",
                msg,
                expected
            );
        }
    }

    #[test]
    fn test_result_type_alias() {
        // Covers L7 — the Result<T> type alias usage
        let ok_result: super::Result<i32> = Ok(42);
        assert_eq!(ok_result.ok(), Some(42));

        let err_result: super::Result<i32> = Err(VaultError::CryptoError("test".into()));
        assert!(err_result.is_err());
    }

    #[test]
    fn test_io_error_conversion() {
        // Covers L87 — From<io::Error>
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let vault_err: VaultError = io_err.into();
        match vault_err {
            VaultError::IoError(e) => {
                assert!(e.to_string().contains("file not found"));
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_error_debug_format() {
        // L93 — ensure Debug trait works (thiserror derives it)
        let err = VaultError::StorageError("db fail".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("StorageError"));
    }

    // ── Domain-specific error conversion tests ──────────────────────────

    #[test]
    fn test_crypto_error_into_vault_error() {
        let cases: Vec<(CryptoError, &str)> = vec![
            (CryptoError::KeyDerivation("bad salt".into()), "bad salt"),
            (CryptoError::Encryption("aes fail".into()), "aes fail"),
            (CryptoError::Decryption("wrong key".into()), "wrong key"),
            (CryptoError::Other("misc".into()), "misc"),
        ];
        for (crypto_err, expected) in cases {
            let vault_err: VaultError = crypto_err.into();
            match &vault_err {
                VaultError::CryptoError(msg) => assert!(msg.contains(expected)),
                _ => panic!("Expected CryptoError, got {:?}", vault_err),
            }
        }

        // Integrity maps to IntegrityError
        let integrity = CryptoError::Integrity("hash mismatch".into());
        let vault_err: VaultError = integrity.into();
        assert!(matches!(vault_err, VaultError::IntegrityError(_)));
    }

    #[test]
    fn test_storage_error_into_vault_error() {
        let io_err = StorageError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "gone"));
        assert!(matches!(VaultError::from(io_err), VaultError::IoError(_)));

        let ser = StorageError::Serialization("bad json".into());
        assert!(matches!(
            VaultError::from(ser),
            VaultError::SerializationError(_)
        ));

        let comp = StorageError::Compression("zlib".into());
        assert!(matches!(
            VaultError::from(comp),
            VaultError::CompressionError(_)
        ));

        let db = StorageError::Database("sqlite locked".into());
        assert!(matches!(VaultError::from(db), VaultError::StorageError(_)));

        let other = StorageError::Other("unknown".into());
        assert!(matches!(
            VaultError::from(other),
            VaultError::StorageError(_)
        ));
    }

    #[test]
    fn test_conversion_error_into_vault_error() {
        let unsup = ConversionError::Unsupported("onnx→gguf".into());
        assert!(matches!(
            VaultError::from(unsup),
            VaultError::UnsupportedFormat(_)
        ));

        let val = ConversionError::Validation("shape mismatch".into());
        assert!(matches!(
            VaultError::from(val),
            VaultError::ConversionError(_)
        ));

        let other = ConversionError::Other("misc".into());
        assert!(matches!(
            VaultError::from(other),
            VaultError::ConversionError(_)
        ));
    }
}
