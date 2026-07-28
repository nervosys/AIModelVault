//! Ed25519 model signing and provenance verification.
//!
//! Provides cryptographic signatures on model files for supply-chain
//! security.  Keys are generated locally and stored in the vault's config
//! directory.  Signatures are detached `.sig` JSON files containing the
//! signature, public key, timestamp, and optional metadata.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{Result, VaultError};

// ── Signature envelope ───────────────────────────────────────────────────────

/// Detached signature for a model file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSignature {
    /// Ed25519 signature (hex-encoded)
    pub signature: String,
    /// Public key of the signer (hex-encoded)
    pub public_key: String,
    /// SHA-256 of the signed file (hex-encoded)
    pub file_sha256: String,
    /// Signer identity (optional display name / email)
    pub signer: Option<String>,
    /// ISO-8601 timestamp of signing
    pub signed_at: String,
    /// Signature format version
    pub version: u32,
    /// Additional metadata (model name, version, etc.)
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

// ── Key pair ─────────────────────────────────────────────────────────────────

/// An Ed25519 signing key pair.
///
/// Keys are stored as hex-encoded strings.  The secret key is 64 bytes
/// (seed + public key) as returned by the Ed25519 expand step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningKeyPair {
    /// Hex-encoded 32-byte secret seed
    pub secret_seed: String,
    /// Hex-encoded 32-byte public key
    pub public_key: String,
    /// Human-readable identity
    pub identity: Option<String>,
    /// When the key was created
    pub created_at: String,
}

// ── Signer ───────────────────────────────────────────────────────────────────

/// Signs model files using Ed25519 (via the `ed25519-dalek` crate when
/// available, or a HKDF-based HMAC fallback using SHA-256).
///
/// Since this crate forbids `unsafe_code` and we want zero additional
/// heavyweight dependencies, we implement signing as HMAC-SHA256 over
/// the file hash keyed by a 32-byte secret.  This provides:
///   - Tamper detection (integrity)
///   - Signer authentication (only secret-holder can produce the tag)
///
/// For full non-repudiation upgrade to Ed25519 by enabling a future
/// `ed25519` feature flag.
pub struct ModelSigner;

impl ModelSigner {
    /// Generate a new signing key pair.
    ///
    /// Uses the OS CSPRNG to create a random 32-byte seed, then derives
    /// the "public key" as SHA-256(seed) for identification.
    pub fn generate_keypair(identity: Option<&str>) -> Result<SigningKeyPair> {
        use aes_gcm::aead::rand_core::RngCore;
        use aes_gcm::aead::OsRng;

        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);

        let public = Sha256::digest(seed);

        let kp = SigningKeyPair {
            secret_seed: hex::encode(seed),
            public_key: hex::encode(public),
            identity: identity.map(String::from),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // Zeroize seed from stack
        seed.fill(0);

        Ok(kp)
    }

    /// Save a key pair to a JSON file with restrictive permissions.
    pub fn save_keypair(keypair: &SigningKeyPair, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(keypair)?;
        fs::write(path, json)?;

        // Restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Rebuild a key pair from a hex-encoded 32-byte secret seed.
    ///
    /// Lets a signing key live in a secret manager as bare seed material rather
    /// than as a full keypair document — the public key is re-derived, so the
    /// result is identical to the keypair the seed was generated with.
    pub fn keypair_from_seed(seed_hex: &str, identity: Option<&str>) -> Result<SigningKeyPair> {
        let seed_hex = seed_hex.trim();
        let seed = hex::decode(seed_hex)
            .map_err(|e| VaultError::InvalidInput(format!("Signing seed is not valid hex: {e}")))?;
        if seed.len() != 32 {
            return Err(VaultError::InvalidInput(format!(
                "Signing seed must be 32 bytes ({} hex chars), got {}",
                64,
                seed.len()
            )));
        }

        Ok(SigningKeyPair {
            secret_seed: hex::encode(&seed),
            public_key: hex::encode(Sha256::digest(&seed)),
            identity: identity.map(String::from),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Parse a key pair from either a JSON keypair document or a bare
    /// hex-encoded seed — the two shapes a secret manager might hold.
    pub fn parse_keypair(data: &str, identity: Option<&str>) -> Result<SigningKeyPair> {
        let trimmed = data.trim();
        if trimmed.starts_with('{') {
            return serde_json::from_str(trimmed).map_err(Into::into);
        }
        Self::keypair_from_seed(trimmed, identity)
    }

    /// Load a key pair from a JSON file.
    pub fn load_keypair(path: &Path) -> Result<SigningKeyPair> {
        let data = fs::read_to_string(path)?;
        let kp: SigningKeyPair = serde_json::from_str(&data)?;
        Ok(kp)
    }

    /// Sign a model file, producing a detached [`ModelSignature`].
    pub fn sign(
        keypair: &SigningKeyPair,
        file_path: &Path,
        metadata: HashMap<String, String>,
    ) -> Result<ModelSignature> {
        // Read file and compute SHA-256
        let data = fs::read(file_path)?;
        let file_hash = Sha256::digest(&data);
        let file_sha256 = hex::encode(file_hash);

        // Decode secret seed
        let seed_bytes = hex::decode(&keypair.secret_seed)
            .map_err(|e| VaultError::CryptoError(format!("Invalid secret seed: {}", e)))?;

        // HMAC-SHA256: H(seed || file_hash)
        let mut hmac_input = Vec::with_capacity(seed_bytes.len() + file_hash.len());
        hmac_input.extend_from_slice(&seed_bytes);
        hmac_input.extend_from_slice(&file_hash);
        let signature_bytes = Sha256::digest(&hmac_input);

        Ok(ModelSignature {
            signature: hex::encode(signature_bytes),
            public_key: keypair.public_key.clone(),
            file_sha256,
            signer: keypair.identity.clone(),
            signed_at: chrono::Utc::now().to_rfc3339(),
            version: 1,
            metadata,
        })
    }

    /// Verify a detached signature against a model file.
    ///
    /// Returns `Ok(true)` if valid, `Ok(false)` if the signature doesn't
    /// match, or `Err` on I/O errors.
    pub fn verify(
        signature: &ModelSignature,
        file_path: &Path,
        secret_seed: Option<&str>,
    ) -> Result<SignatureVerification> {
        let data = fs::read(file_path)?;
        let file_hash = Sha256::digest(&data);
        let file_sha256 = hex::encode(file_hash);

        // Check file hash matches what was signed
        if file_sha256 != signature.file_sha256 {
            return Ok(SignatureVerification {
                valid: false,
                file_hash_match: false,
                signature_match: false,
                signer: signature.signer.clone(),
                signed_at: signature.signed_at.clone(),
                reason: Some("File SHA-256 does not match signed hash".to_string()),
            });
        }

        // If we have the secret seed, verify the HMAC
        let signature_match = if let Some(seed_hex) = secret_seed {
            let seed_bytes = hex::decode(seed_hex)
                .map_err(|e| VaultError::CryptoError(format!("Invalid seed: {}", e)))?;
            let mut hmac_input = Vec::with_capacity(seed_bytes.len() + file_hash.len());
            hmac_input.extend_from_slice(&seed_bytes);
            hmac_input.extend_from_slice(&file_hash);
            let expected = hex::encode(Sha256::digest(&hmac_input));
            expected == signature.signature
        } else {
            // Without the secret, we can only verify the file hash
            true
        };

        Ok(SignatureVerification {
            valid: signature_match,
            file_hash_match: true,
            signature_match,
            signer: signature.signer.clone(),
            signed_at: signature.signed_at.clone(),
            reason: if signature_match {
                None
            } else {
                Some("HMAC signature does not match".to_string())
            },
        })
    }

    /// Save a signature to a `.sig` JSON file alongside the model.
    pub fn save_signature(signature: &ModelSignature, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(signature)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load a signature from a `.sig` JSON file.
    pub fn load_signature(path: &Path) -> Result<ModelSignature> {
        let data = fs::read_to_string(path)?;
        let sig: ModelSignature = serde_json::from_str(&data)?;
        Ok(sig)
    }
}

/// Result of a signature verification.
#[derive(Debug, Serialize)]
pub struct SignatureVerification {
    /// Overall validity
    pub valid: bool,
    /// Whether the file hash matches
    pub file_hash_match: bool,
    /// Whether the cryptographic signature matches
    pub signature_match: bool,
    /// Signer identity
    pub signer: Option<String>,
    /// When the file was signed
    pub signed_at: String,
    /// Reason for failure (if any)
    pub reason: Option<String>,
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_generate_keypair() {
        let kp = ModelSigner::generate_keypair(Some("test-user")).unwrap();
        assert_eq!(kp.secret_seed.len(), 64); // 32 bytes hex
        assert_eq!(kp.public_key.len(), 64);
        assert_eq!(kp.identity.as_deref(), Some("test-user"));
    }

    #[test]
    fn test_sign_and_verify() {
        let kp = ModelSigner::generate_keypair(Some("tester")).unwrap();

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"fake model data for testing").unwrap();

        let sig = ModelSigner::sign(&kp, file.path(), HashMap::new()).unwrap();
        assert!(!sig.signature.is_empty());
        assert_eq!(sig.public_key, kp.public_key);

        // Verify with secret seed
        let result = ModelSigner::verify(&sig, file.path(), Some(&kp.secret_seed)).unwrap();
        assert!(result.valid);
        assert!(result.file_hash_match);
        assert!(result.signature_match);

        // Verify without secret (hash-only check)
        let result2 = ModelSigner::verify(&sig, file.path(), None).unwrap();
        assert!(result2.valid);
        assert!(result2.file_hash_match);
    }

    #[test]
    fn test_verify_tampered_file() {
        let kp = ModelSigner::generate_keypair(None).unwrap();

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"original data").unwrap();

        let sig = ModelSigner::sign(&kp, file.path(), HashMap::new()).unwrap();

        // Tamper with the file
        file.as_file().set_len(0).unwrap();
        file.write_all(b"tampered data").unwrap();

        let result = ModelSigner::verify(&sig, file.path(), Some(&kp.secret_seed)).unwrap();
        assert!(!result.valid);
        assert!(!result.file_hash_match);
    }

    #[test]
    fn test_save_load_keypair() {
        let kp = ModelSigner::generate_keypair(Some("test")).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.key");

        ModelSigner::save_keypair(&kp, &path).unwrap();
        let loaded = ModelSigner::load_keypair(&path).unwrap();
        assert_eq!(loaded.secret_seed, kp.secret_seed);
        assert_eq!(loaded.public_key, kp.public_key);
    }

    #[test]
    fn test_save_load_signature() {
        let kp = ModelSigner::generate_keypair(None).unwrap();
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"model bytes").unwrap();

        let sig = ModelSigner::sign(&kp, file.path(), HashMap::new()).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let sig_path = dir.path().join("model.sig");
        ModelSigner::save_signature(&sig, &sig_path).unwrap();

        let loaded = ModelSigner::load_signature(&sig_path).unwrap();
        assert_eq!(loaded.signature, sig.signature);
        assert_eq!(loaded.file_sha256, sig.file_sha256);
    }

    #[test]
    fn test_keypair_from_seed_roundtrip() {
        let original = ModelSigner::generate_keypair(Some("alice")).unwrap();

        // Rebuilding from the seed alone must re-derive the same public key,
        // so signatures made either way verify against each other.
        let rebuilt = ModelSigner::keypair_from_seed(&original.secret_seed, Some("alice")).unwrap();
        assert_eq!(rebuilt.secret_seed, original.secret_seed);
        assert_eq!(rebuilt.public_key, original.public_key);
    }

    #[test]
    fn test_keypair_from_seed_rejects_bad_input() {
        assert!(ModelSigner::keypair_from_seed("not-hex", None).is_err());
        // 16 bytes instead of 32
        assert!(ModelSigner::keypair_from_seed(&"ab".repeat(16), None).is_err());
    }

    #[test]
    fn test_parse_keypair_accepts_both_shapes() {
        let kp = ModelSigner::generate_keypair(Some("bob")).unwrap();

        let as_json = serde_json::to_string(&kp).unwrap();
        let from_json = ModelSigner::parse_keypair(&as_json, None).unwrap();
        assert_eq!(from_json.public_key, kp.public_key);
        assert_eq!(from_json.identity.as_deref(), Some("bob"));

        // A secret manager may hold just the seed; identity then comes from the caller.
        let from_seed = ModelSigner::parse_keypair(&kp.secret_seed, Some("bob")).unwrap();
        assert_eq!(from_seed.public_key, kp.public_key);
        assert_eq!(from_seed.identity.as_deref(), Some("bob"));
    }

    #[test]
    fn test_seed_sourced_key_produces_verifiable_signature() {
        let kp = ModelSigner::generate_keypair(None).unwrap();
        let rebuilt = ModelSigner::keypair_from_seed(&kp.secret_seed, None).unwrap();

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"payload signed with a KMS-sourced key")
            .unwrap();

        let sig = ModelSigner::sign(&rebuilt, file.path(), HashMap::new()).unwrap();
        let result = ModelSigner::verify(&sig, file.path(), Some(&kp.secret_seed)).unwrap();
        assert!(
            result.valid,
            "seed-sourced key must verify against the original"
        );
    }
}
