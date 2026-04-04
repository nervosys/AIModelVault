//! Secrets manager integration — derive vault passphrases from external KMS.
//!
//! Provides a uniform interface for retrieving encryption keys from:
//! - Environment variables (default / CI)
//! - AWS Secrets Manager / KMS
//! - Azure Key Vault
//! - HashiCorp Vault
//!
//! Each backend is a thin wrapper that returns a `SecureKey` ready for use
//! with the vault's crypto layer.

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::error::{Result, VaultError};

// ── Types ────────────────────────────────────────────────────────────────────

/// Supported KMS backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KmsBackend {
    /// Read from environment variable.
    Env,
    /// AWS Secrets Manager.
    AwsSecretsManager,
    /// Azure Key Vault.
    AzureKeyVault,
    /// HashiCorp Vault.
    HashicorpVault,
}

impl std::fmt::Display for KmsBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KmsBackend::Env => write!(f, "env"),
            KmsBackend::AwsSecretsManager => write!(f, "aws-secrets-manager"),
            KmsBackend::AzureKeyVault => write!(f, "azure-key-vault"),
            KmsBackend::HashicorpVault => write!(f, "hashicorp-vault"),
        }
    }
}

impl std::str::FromStr for KmsBackend {
    type Err = VaultError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "env" | "environment" => Ok(KmsBackend::Env),
            "aws" | "aws-secrets-manager" => Ok(KmsBackend::AwsSecretsManager),
            "azure" | "azure-key-vault" => Ok(KmsBackend::AzureKeyVault),
            "hashicorp" | "hashicorp-vault" | "hcv" => Ok(KmsBackend::HashicorpVault),
            _ => Err(VaultError::InvalidInput(format!(
                "Unknown KMS backend: {s}"
            ))),
        }
    }
}

/// Parameters for fetching a secret.
#[derive(Debug, Clone)]
pub struct KmsRequest {
    pub backend: KmsBackend,
    /// Secret name / ARN / Key Vault URI / Vault path.
    pub secret_id: String,
    /// Optional region or endpoint override.
    pub endpoint: Option<String>,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Fetch a passphrase from the configured KMS backend.
///
/// The returned value is wrapped in `Zeroizing` so it is wiped from memory
/// when dropped.
pub fn fetch_secret(req: &KmsRequest) -> Result<Zeroizing<String>> {
    match req.backend {
        KmsBackend::Env => fetch_from_env(&req.secret_id),
        KmsBackend::AwsSecretsManager => fetch_aws_stub(&req.secret_id),
        KmsBackend::AzureKeyVault => fetch_azure_stub(&req.secret_id),
        KmsBackend::HashicorpVault => fetch_hcv_stub(&req.secret_id),
    }
}

/// List available backends (useful for CLI help text).
pub fn available_backends() -> Vec<KmsBackend> {
    vec![
        KmsBackend::Env,
        KmsBackend::AwsSecretsManager,
        KmsBackend::AzureKeyVault,
        KmsBackend::HashicorpVault,
    ]
}

// ── Backend implementations ──────────────────────────────────────────────────

fn fetch_from_env(var_name: &str) -> Result<Zeroizing<String>> {
    std::env::var(var_name).map(Zeroizing::new).map_err(|_| {
        VaultError::ConfigError(format!("Environment variable '{}' not set", var_name))
    })
}

/// Stub for AWS Secrets Manager — placeholder for actual SDK integration.
fn fetch_aws_stub(secret_id: &str) -> Result<Zeroizing<String>> {
    Err(VaultError::ConfigError(format!(
        "AWS Secrets Manager support not compiled in (secret: {}). \
         Enable the 's3' feature and provide AWS credentials.",
        secret_id
    )))
}

/// Stub for Azure Key Vault.
fn fetch_azure_stub(secret_id: &str) -> Result<Zeroizing<String>> {
    Err(VaultError::ConfigError(format!(
        "Azure Key Vault support not compiled in (secret: {}). \
         Enable the 'azure' feature and provide Azure credentials.",
        secret_id
    )))
}

/// Stub for HashiCorp Vault.
fn fetch_hcv_stub(secret_id: &str) -> Result<Zeroizing<String>> {
    Err(VaultError::ConfigError(format!(
        "HashiCorp Vault support not compiled in (secret: {}). \
         Set VAULT_ADDR and VAULT_TOKEN environment variables.",
        secret_id
    )))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_parse() {
        assert_eq!("env".parse::<KmsBackend>().unwrap(), KmsBackend::Env);
        assert_eq!(
            "aws".parse::<KmsBackend>().unwrap(),
            KmsBackend::AwsSecretsManager
        );
        assert_eq!(
            "azure-key-vault".parse::<KmsBackend>().unwrap(),
            KmsBackend::AzureKeyVault
        );
        assert_eq!(
            "hashicorp".parse::<KmsBackend>().unwrap(),
            KmsBackend::HashicorpVault
        );
        assert!("unknown".parse::<KmsBackend>().is_err());
    }

    #[test]
    fn test_fetch_from_env() {
        std::env::set_var("AIM_TEST_SECRET_KMS", "super-secret-passphrase");
        let req = KmsRequest {
            backend: KmsBackend::Env,
            secret_id: "AIM_TEST_SECRET_KMS".into(),
            endpoint: None,
        };
        let secret = fetch_secret(&req).unwrap();
        assert_eq!(&*secret, "super-secret-passphrase");
        std::env::remove_var("AIM_TEST_SECRET_KMS");
    }

    #[test]
    fn test_env_missing() {
        let req = KmsRequest {
            backend: KmsBackend::Env,
            secret_id: "AIM_DEFINITELY_NOT_SET_42".into(),
            endpoint: None,
        };
        assert!(fetch_secret(&req).is_err());
    }

    #[test]
    fn test_stubs_return_errors() {
        let aws = KmsRequest {
            backend: KmsBackend::AwsSecretsManager,
            secret_id: "arn:aws:secretsmanager:us-east-1:123:secret:test".into(),
            endpoint: None,
        };
        assert!(fetch_secret(&aws).is_err());

        let az = KmsRequest {
            backend: KmsBackend::AzureKeyVault,
            secret_id: "https://myvault.vault.azure.net/secrets/test".into(),
            endpoint: None,
        };
        assert!(fetch_secret(&az).is_err());

        let hcv = KmsRequest {
            backend: KmsBackend::HashicorpVault,
            secret_id: "secret/data/myapp".into(),
            endpoint: None,
        };
        assert!(fetch_secret(&hcv).is_err());
    }

    #[test]
    fn test_available_backends() {
        let backends = available_backends();
        assert_eq!(backends.len(), 4);
    }
}
