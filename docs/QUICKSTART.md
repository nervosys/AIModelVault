# aimodelvault Quick Start Guide

> **Current version: 1.2.1** — See [CHANGELOG](https://github.com/nervosys/AIModelVault/blob/master/CHANGELOG.md) for details.

## What's New in v1.2

- **Domain Events** — `VaultEvent` enum with `EventBus` subscriber system for audit, metrics, and agent observability
- **Vault Metrics** — `VaultMetrics` atomic counters (models stored/retrieved/deleted, bytes, errors); `MetricsSnapshot` for point-in-time reporting
- **Observability API** — `GET /api/v1/metrics` and `GET /api/v1/events` endpoints; enhanced `GET /api/v1/health` with vault state
- **VaultBuilder** — Fluent builder pattern: `.config()`, `.sqlite_versions()`, `.subscriber()`, `.no_default_subscribers()`
- **SQLite Version Backend** — ACID-compliant version storage with WAL mode; auto-migration from JSON; enable with `--sqlite-versions` or `AIM_SQLITE_VERSIONS=1`
- **Streaming Encryption** — Chunked AES-256-GCM for large models with constant 8 MiB memory budget
- **Agent-Addressable URIs** — `aimv://vault/model@version/resource?query` scheme
- **Domain Error Types** — `CryptoError`, `StorageError`, `ConversionError` with `From` into `VaultError`

## Installation

### From Source

```bash
git clone https://github.com/nervosys/AIModelVault.git
cd AIModelVault
cargo build --release
cargo install --path .
```

### From crates.io

```bash
cargo install aimodelvault
```

## Basic Usage

### 1. Initialize a Vault

```bash
# Initialize default vault
aim init

# Initialize named vault
aim init --name my-models
```

### 2. Store a Model

```bash
# Store a PyTorch model
aim store my-gpt2 \
  --path ./model.pt \
  --format pytorch \
  --description "Fine-tuned GPT-2 model" \
  --framework "PyTorch 2.0" \
  --task "text-generation"

# You'll be prompted for a passphrase
```

### 3. List Models

```bash
# List all models
aim list

# Show versions of a specific model
aim versions my-gpt2
```

### 4. Retrieve a Model

```bash
# Get latest version
aim get my-gpt2 --output ./retrieved_model.pt

# Get specific version
aim get my-gpt2 --version 2 --output ./model_v2.pt
```

### 5. View Version History

```bash
# Show lineage of a version
aim lineage my-gpt2 --version 3
```

### 6. Delete a Version

```bash
# Delete specific version
aim delete my-gpt2 --version 1

# Force delete without confirmation
aim delete my-gpt2 --version 1 --force
```

### 7. View Statistics

```bash
aim stats
```

### 8. Run Compliance Checks

```bash
aim compliance
```

## Python API Usage

```python
from aimodelvault import Vault

# Initialize vault
vault = Vault()

# Store a model
vault.store_model(
    "my-bert",
    model_path="./bert_model.pt",
    format="pytorch",
    metadata={
        "task": "sentiment-analysis",
        "framework": "transformers",
        "accuracy": "94.5%"
    }
)

# Retrieve model
model_data = vault.get_model("my-bert", version="latest")

# List versions
versions = vault.list_versions("my-bert")
for v in versions:
    print(f"v{v.version}: {v.timestamp} - {v.size_bytes} bytes")
```

## Configuration

aimodelvault follows XDG Base Directory specification:

- **Config**: `~/.config/aimodelvault/config.yaml`
- **Data**: `~/.local/share/aimodelvault/vaults/`
- **Logs**: `~/.local/share/aimodelvault/logs/`
- **Cache**: `~/.cache/aimodelvault/`

### Configuration File

Edit `~/.config/aimodelvault/config.yaml`:

```yaml
version: "1.0"

vault:
  data_dir: ~/.local/share/aimodelvault/vaults
  default_vault: default

crypto:
  algorithm: aes-256-gcm
  kdf: argon2id

compression:
  algorithm: gzip
  level: 6

storage:
  max_versions: 10
  auto_cleanup: true

security:
  require_passphrase: true
  session_timeout: 3600
  audit_log: true

compliance:
  fips_mode: true
  cve_scanning: true
  audit_retention_days: 90
```

## Security Best Practices

1. **Use strong passphrases**: Minimum 20 characters, mix of letters, numbers, symbols
2. **Never share passphrases**: Each vault should have a unique passphrase
3. **Backup your vault**: Regularly backup `~/.local/share/aimodelvault/`
4. **Review audit logs**: Check `~/.local/share/aimodelvault/logs/audit.log`
5. **Keep software updated**: Run `cargo install aimodelvault` regularly
6. **Secure your system**: Use full-disk encryption and secure boot

## Advanced Features

### Model Format Conversion

```bash
# Store in one format, retrieve in another (future feature)
aim store model.pt --format pytorch
aim get model --output model.onnx --target-format onnx
```

### Version Branching

```python
# Create a branch from version 2
vault.store_model(
    "my-model",
    model_data=new_data,
    parent_version=2  # Branch from v2
)
```

### Compression Options

```yaml
compression:
  algorithm: lzma  # Options: gzip, lzma, zlib
  level: 9  # 0-9, higher = better compression
```

## Troubleshooting

### Forgotten Passphrase

Unfortunately, there is no way to recover a vault if you forget the passphrase. This is by design for security. Always:
- Keep passphrases in a secure password manager
- Consider backing up the passphrase securely
- Test recovery procedures

### Permission Errors

```bash
# Fix permissions
chmod 700 ~/.local/share/aimodelvault/vaults
chmod 600 ~/.config/aimodelvault/config.yaml
```

### Corrupted Vault

```bash
# Check integrity against a signature (--key is required for a real check)
aim verify my-model --signature my-model.sig --key signing_key.json

# Restore from backup
cp -r /backup/aimodelvault ~/.local/share/
```

## Support

- Documentation: https://aimodelvault.nervosys.ai/docs
- Issues: https://github.com/nervosys/AIModelVault/issues
- Security: security@nervosys.ai

## Next Steps

- Read the [CLI Reference](CLI.md)
- Explore [Security Features](https://github.com/nervosys/AIModelVault/blob/master/SECURITY.md)
- View [API Documentation](https://docs.rs/aimodelvault)
