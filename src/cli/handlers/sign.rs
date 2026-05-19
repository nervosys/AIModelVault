//! CLI handlers for model signing and verification (aim sign, aim verify).

use std::collections::HashMap;
use std::path::PathBuf;

use ai_model_vault::{ModelSigner, Result, VaultConfig, VaultError};

use crate::cli::helpers::{build_vault, prompt_passphrase};

#[allow(clippy::too_many_arguments)]
pub fn handle_sign(
    name: String,
    version: Option<u32>,
    key: Option<PathBuf>,
    identity: Option<String>,
    file: Option<PathBuf>,
    config: VaultConfig,
    use_sqlite: bool,
) -> Result<()> {
    // Determine or generate the signing key
    let key_path = key.unwrap_or_else(|| config.dirs.config_dir.join("signing_key.json"));

    let keypair = if key_path.exists() {
        println!("Loading signing key from: {}", key_path.display());
        ModelSigner::load_keypair(&key_path)?
    } else {
        let id = identity.as_deref();
        println!("Generating new signing key pair...");
        let kp = ModelSigner::generate_keypair(id)?;
        ModelSigner::save_keypair(&kp, &key_path)?;
        println!("Key pair saved to: {}", key_path.display());
        kp
    };

    let metadata = HashMap::new();

    if let Some(file_path) = file {
        // Sign a file on disk
        let sig = ModelSigner::sign(&keypair, &file_path, metadata)?;
        let sig_path = file_path.with_extension("sig");
        ModelSigner::save_signature(&sig, &sig_path)?;
        println!("Signed: {}", file_path.display());
        println!("Signature: {}", sig_path.display());
        println!("SHA-256: {}", sig.file_sha256);
    } else {
        // Sign a model from the vault — we need to export it first
        let passphrase = prompt_passphrase("Enter vault passphrase: ")?;
        let mut vault = build_vault(config, use_sqlite)?;
        vault.unlock(passphrase)?;

        let data = vault.get_model(&name, version)?;

        let temp_dir = tempfile::tempdir().map_err(VaultError::IoError)?;
        let temp_path = temp_dir.path().join(&name);
        std::fs::write(&temp_path, &data)?;

        let sig = ModelSigner::sign(&keypair, &temp_path, metadata)?;
        let sig_dir = vault.get_config().dirs.data_dir.clone();
        let sig_path = sig_dir.join(format!("{}_v{}.sig", name, version.unwrap_or(0)));
        ModelSigner::save_signature(&sig, &sig_path)?;

        println!("Signed model '{}' (v{})", name, version.unwrap_or(0));
        println!("Signature: {}", sig_path.display());
        println!("SHA-256: {}", sig.file_sha256);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn handle_verify(
    name: String,
    _version: Option<u32>,
    signature: PathBuf,
    key: Option<PathBuf>,
    file: Option<PathBuf>,
    config: VaultConfig,
    use_sqlite: bool,
) -> Result<()> {
    let sig = ModelSigner::load_signature(&signature)?;

    let _temp_dir;
    let file_path = if let Some(fp) = file {
        fp
    } else {
        // Export model from vault to temp
        let passphrase = prompt_passphrase("Enter vault passphrase: ")?;
        let mut vault = build_vault(config, use_sqlite)?;
        vault.unlock(passphrase)?;

        let data = vault.get_model(&name, None)?;
        _temp_dir = tempfile::tempdir().map_err(VaultError::IoError)?;
        let temp_path = _temp_dir.path().join(&name);
        std::fs::write(&temp_path, &data)?;
        temp_path
    };

    // Load secret key if provided (for HMAC verification)
    let secret_seed = if let Some(key_path) = key {
        let kp = ModelSigner::load_keypair(&key_path)?;
        Some(kp.secret_seed.clone())
    } else {
        None
    };

    let result = ModelSigner::verify(&sig, &file_path, secret_seed.as_deref())?;

    if result.file_hash_match {
        println!("✓ File hash matches signature");
    } else {
        println!("✗ File hash does NOT match signature");
    }

    if result.signature_match {
        println!("✓ Cryptographic signature valid");
    } else {
        println!("✗ Cryptographic signature INVALID or no key provided");
    }

    if let Some(signer) = &sig.signer {
        println!("Signer: {}", signer);
    }
    println!("Signed at: {}", sig.signed_at);

    if result.valid {
        println!("\n✓ Verification PASSED");
    } else {
        println!("\n✗ Verification FAILED");
        if let Some(reason) = &result.reason {
            println!("Reason: {}", reason);
        }
    }

    Ok(())
}
