# Model Signing & Verification

HMAC-SHA256 model signing with detached `.sig` JSON files for tamper detection and provenance tracking.

## Quick Start

```bash
# Sign a vault model (auto-generates key on first use)
aim sign my-model

# Sign with identity
aim sign my-model --identity "ML Team <ml@company.com>"

# Verify a signature
aim verify my-model --signature my-model.sig

# Sign a file on disk
aim sign my-model --file ./model.safetensors
```

## CLI Reference

### sign

```
aim sign <NAME> [OPTIONS]

Arguments:
  <NAME>              Model name in vault

Options:
  -v, --version <V>   Model version (default: latest)
  -k, --key <KEY>     Path to signing key JSON file
  -i, --identity <ID> Signer identity (name/email)
  --file <PATH>       Sign a file on disk instead of vault model
```

### verify

```
aim verify <NAME> --signature <SIG> [OPTIONS]

Arguments:
  <NAME>              Model name in vault

Options:
  --signature <SIG>   Path to .sig file
  -k, --key <KEY>     Path to signing key for verification
  --file <PATH>       Verify a file on disk
  -v, --version <V>   Model version
```

## How It Works

1. **Key Generation** — A signing keypair (`SigningKeyPair`) is auto-generated on first use and saved to `<config_dir>/signing_key.json`
2. **Signing** — HMAC-SHA256 is computed over the file content using the secret seed
3. **Detached Signature** — A `.sig` JSON file is created containing signature, public key, file hash, signer identity, and timestamp
4. **Verification** — The signature is validated against the file hash and the stored public key

## Signature File Format

```json
{
  "signature": "hex-encoded HMAC-SHA256",
  "public_key": "hex-encoded 32-byte key",
  "file_sha256": "hex-encoded SHA-256 of model file",
  "signer": "ML Team <ml@company.com>",
  "signed_at": "2026-04-04T12:00:00Z",
  "version": 1,
  "metadata": {}
}
```

## Rust API

```rust
use ai_model_vault::signing::{ModelSigner, SigningKeyPair};

// Generate keypair
let keypair = ModelSigner::generate_keypair(Some("ML Team"))?;
ModelSigner::save_keypair(&keypair, "signing_key.json")?;

// Sign a file
let signature = ModelSigner::sign(&keypair, Path::new("model.safetensors"), HashMap::new())?;
ModelSigner::save_signature(&signature, Path::new("model.sig"))?;

// Verify
let loaded_sig = ModelSigner::load_signature(Path::new("model.sig"))?;
let result = ModelSigner::verify(&loaded_sig, Path::new("model.safetensors"), None)?;
assert!(result.valid);
```
