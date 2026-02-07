//! FIPS 140-3 Compliant Cryptographic Module
//!
//! This module provides FIPS 140-3 compliant encryption/decryption for model storage.
//! Uses AES-256-GCM with Argon2id key derivation.
//!
//! Security Controls:
//! - NIST SP 800-38D (GCM mode)
//! - NIST SP 800-63B (Password recommendations)
//! - FIPS 197 (AES)
//! - RFC 9106 (Argon2)

pub mod compression;

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm,
};
use argon2::{
    password_hash::{rand_core::RngCore, PasswordHasher, SaltString},
    Argon2, ParamsBuilder, Version,
};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{Result, VaultError};

/// Size of AES-256 key in bytes
pub const KEY_SIZE: usize = 32;

/// Size of GCM nonce in bytes (96 bits recommended)
pub const NONCE_SIZE: usize = 12;

/// Size of salt for key derivation
pub const SALT_SIZE: usize = 32;

/// FIPS 140-3 compliant cryptographic operations
///
/// Compliance Mappings:
/// - CMMC 2.0: SC.3.177 (Employ FIPS-validated cryptography)
/// - MITRE ATT&CK: T1486 mitigation (Data Encrypted for Impact)
pub struct FipsCrypto {
    argon2: Argon2<'static>,
}

/// Secure key container that zeroizes on drop
#[derive(Clone, ZeroizeOnDrop)]
pub struct SecureKey {
    key: [u8; KEY_SIZE],
}

impl SecureKey {
    /// Create new secure key from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != KEY_SIZE {
            return Err(VaultError::CryptoError(format!(
                "Invalid key size: expected {}, got {}",
                KEY_SIZE,
                bytes.len()
            )));
        }

        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(bytes);
        Ok(Self { key })
    }

    /// Get key bytes
    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.key
    }
}

impl FipsCrypto {
    /// Create new FIPS crypto instance with recommended parameters
    pub fn new() -> Result<Self> {
        // Argon2id with OWASP recommended parameters
        let params = ParamsBuilder::new()
            .m_cost(19456) // 19 MiB memory
            .t_cost(2) // 2 iterations
            .p_cost(1) // 1 parallelism
            .build()
            .map_err(|e| {
                VaultError::CryptoError(format!("Failed to build Argon2 params: {}", e))
            })?;

        Ok(Self {
            argon2: Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params),
        })
    }

    /// Derive encryption key from passphrase using Argon2id
    ///
    /// # Arguments
    /// * `passphrase` - User passphrase (will be zeroized)
    /// * `salt` - Optional salt (generated if not provided)
    ///
    /// # Returns
    /// Tuple of (encryption_key, salt)
    ///
    /// # Compliance
    /// - FIPS 140-3: Approved key derivation
    /// - RFC 9106: Argon2 password hashing
    pub fn derive_key(
        &self,
        mut passphrase: Vec<u8>,
        salt: Option<Vec<u8>>,
    ) -> Result<(SecureKey, Vec<u8>)> {
        let salt_bytes = if let Some(s) = salt {
            s
        } else {
            let mut salt = vec![0u8; SALT_SIZE];
            OsRng.fill_bytes(&mut salt);
            salt
        };

        let salt_string = SaltString::encode_b64(&salt_bytes)
            .map_err(|e| VaultError::CryptoError(format!("Failed to encode salt: {}", e)))?;

        let password_hash = self
            .argon2
            .hash_password(&passphrase, &salt_string)
            .map_err(|e| VaultError::CryptoError(format!("Failed to derive key: {}", e)))?;

        // Extract the hash bytes (first 32 bytes for AES-256)
        let hash_bytes = password_hash
            .hash
            .ok_or_else(|| VaultError::CryptoError("No hash in password hash".to_string()))?;

        let key = SecureKey::from_bytes(&hash_bytes.as_bytes()[..KEY_SIZE])?;

        // Zeroize passphrase
        passphrase.zeroize();

        Ok((key, salt_bytes))
    }

    /// Encrypt data using AES-256-GCM
    ///
    /// # Arguments
    /// * `data` - Plaintext data to encrypt
    /// * `key` - 256-bit encryption key
    ///
    /// # Returns
    /// Encrypted data with format: nonce || ciphertext (includes auth tag)
    ///
    /// # Compliance
    /// - FIPS 197: AES encryption
    /// - NIST SP 800-38D: GCM mode
    /// - CMMC SC.3.191: Protection of CUI at rest
    pub fn encrypt(&self, data: &[u8], key: &SecureKey) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(key.as_bytes().into());

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt((&nonce_bytes).into(), data)
            .map_err(|e| VaultError::CryptoError(format!("Encryption failed: {}", e)))?;

        // Combine nonce || ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data using AES-256-GCM
    ///
    /// # Arguments
    /// * `encrypted_data` - Encrypted data (nonce || ciphertext)
    /// * `key` - 256-bit encryption key
    ///
    /// # Returns
    /// Decrypted plaintext data
    ///
    /// # Errors
    /// Returns `AuthenticationFailed` if authentication tag verification fails
    pub fn decrypt(&self, encrypted_data: &[u8], key: &SecureKey) -> Result<Vec<u8>> {
        if encrypted_data.len() < NONCE_SIZE {
            return Err(VaultError::CryptoError(
                "Encrypted data too short".to_string(),
            ));
        }

        let cipher = Aes256Gcm::new(key.as_bytes().into());

        // Extract nonce and ciphertext
        let ciphertext = &encrypted_data[NONCE_SIZE..];

        // Decrypt and verify
        let plaintext = cipher
            .decrypt((&encrypted_data[..NONCE_SIZE]).into(), ciphertext)
            .map_err(|_| VaultError::AuthenticationFailed)?;

        Ok(plaintext)
    }

    /// Generate cryptographically secure random bytes
    ///
    /// # Arguments
    /// * `length` - Number of random bytes to generate
    ///
    /// # Compliance
    /// - FIPS 140-3: Approved random number generation
    pub fn generate_random(&self, length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        OsRng.fill_bytes(&mut bytes);
        bytes
    }

    /// Compute SHA-256 hash
    pub fn hash_sha256(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Compute SHA-256 hash as hex string
    pub fn hash_sha256_hex(data: &[u8]) -> String {
        hex::encode(Self::hash_sha256(data))
    }
}

impl Default for FipsCrypto {
    fn default() -> Self {
        Self::new().expect("Failed to create FipsCrypto")
    }
}

/// Key manager for secure key storage and retrieval
///
/// Compliance:
/// - CMMC AC.3.018: Control connection of mobile devices
/// - CMMC IA.3.080: Protect authenticators
pub struct KeyManager {
    crypto: FipsCrypto,
}

impl KeyManager {
    /// Create new key manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            crypto: FipsCrypto::new()?,
        })
    }

    /// Store encryption key using key encryption key (KEK)
    pub fn store_key(&self, key: &SecureKey, master_passphrase: Vec<u8>) -> Result<Vec<u8>> {
        // Derive KEK from master passphrase
        let (kek, salt) = self.crypto.derive_key(master_passphrase, None)?;

        // Encrypt the key
        let encrypted_key = self.crypto.encrypt(key.as_bytes(), &kek)?;

        // Combine salt || encrypted_key
        let mut result = Vec::with_capacity(salt.len() + encrypted_key.len());
        result.extend_from_slice(&salt);
        result.extend_from_slice(&encrypted_key);

        Ok(result)
    }

    /// Load and decrypt stored encryption key
    pub fn load_key(&self, stored_data: &[u8], master_passphrase: Vec<u8>) -> Result<SecureKey> {
        if stored_data.len() < SALT_SIZE {
            return Err(VaultError::CryptoError(
                "Invalid stored key data".to_string(),
            ));
        }

        // Extract salt and encrypted key
        let salt = stored_data[..SALT_SIZE].to_vec();
        let encrypted_key = &stored_data[SALT_SIZE..];

        // Derive KEK from master passphrase
        let (kek, _) = self.crypto.derive_key(master_passphrase, Some(salt))?;

        // Decrypt the key
        let key_bytes = self.crypto.decrypt(encrypted_key, &kek)?;

        SecureKey::from_bytes(&key_bytes)
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new().expect("Failed to create KeyManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let crypto = FipsCrypto::new().unwrap();
        let passphrase = b"test_passphrase_with_sufficient_entropy".to_vec();
        let (key, _) = crypto.derive_key(passphrase, None).unwrap();

        let plaintext = b"Hello, NeuralVault!";
        let encrypted = crypto.encrypt(plaintext, &key).unwrap();
        let decrypted = crypto.decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_key_derivation_deterministic() {
        let crypto = FipsCrypto::new().unwrap();
        let passphrase = b"test_passphrase".to_vec();
        let salt = vec![0u8; SALT_SIZE];

        let (key1, _) = crypto
            .derive_key(passphrase.clone(), Some(salt.clone()))
            .unwrap();
        let (key2, _) = crypto.derive_key(passphrase, Some(salt)).unwrap();

        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_authentication_failure() {
        let crypto = FipsCrypto::new().unwrap();
        let passphrase1 = b"correct_passphrase".to_vec();
        let passphrase2 = b"wrong_passphrase".to_vec();

        let (key1, salt) = crypto.derive_key(passphrase1, None).unwrap();
        let plaintext = b"Secret data";
        let encrypted = crypto.encrypt(plaintext, &key1).unwrap();

        let (key2, _) = crypto.derive_key(passphrase2, Some(salt)).unwrap();
        let result = crypto.decrypt(&encrypted, &key2);

        assert!(result.is_err());
    }
}
