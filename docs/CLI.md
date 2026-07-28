# aimodelvault CLI Reference

Complete command-line interface documentation for aimodelvault.

## Global Options

```
-h, --help       Show help information
-V, --version    Show version information
-v, --vault      Specify vault name (default: "default")
-c, --config     Path to configuration file
```

## Commands

### `init` - Initialize a New Vault

Initialize a new secure vault for storing AI models.

```bash
aim init [OPTIONS]
```

**Options:**
- `--name <NAME>` - Vault name (default: "default")

**Example:**
```bash
aim init --name production-models
```

---

### `store` - Store a Model

Store a model file in the vault with encryption and compression.

```bash
aim store <NAME> <PATH> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Model name/identifier
- `<PATH>` - Path to model file

**Options:**
- `-f, --format <FORMAT>` - Model format (pytorch, tensorflow, onnx, safetensors, etc.)
- `-d, --description <TEXT>` - Model description
- `--framework <NAME>` - Framework name and version
- `--task <TASK>` - ML task (e.g., text-generation, image-classification)

**Examples:**
```bash
# Store a PyTorch model
aim store gpt2-finetuned ./model.pt \
  --format pytorch \
  --description "GPT-2 fine-tuned on custom dataset" \
  --framework "PyTorch 2.1" \
  --task "text-generation"

# Store a Safetensors model
aim store llama-7b ./model.safetensors \
  --format safetensors \
  --description "Llama 7B base model"

# Store ONNX model
aim store resnet50 ./model.onnx \
  --format onnx \
  --task "image-classification"
```

---

### `get` - Retrieve a Model

Retrieve and decrypt a model from the vault.

```bash
aim get <NAME> <OUTPUT> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Model name
- `<OUTPUT>` - Output file path

**Options:**
- `-v, --version <NUMBER>` - Version number (default: latest)

**Examples:**
```bash
# Get latest version
aim get gpt2-finetuned ./retrieved_model.pt

# Get specific version
aim get gpt2-finetuned ./model_v3.pt --version 3
```

---

### `list` - List All Models

Display all models stored in the vault.

```bash
aim list
```

**Example Output:**
```
Models in vault:
  gpt2-finetuned (5 versions)
  bert-base (3 versions)
  resnet50 (2 versions)
```

---

### `versions` - List Model Versions

Show all versions of a specific model.

```bash
aim versions <NAME>
```

**Arguments:**
- `<NAME>` - Model name

**Example:**
```bash
aim versions gpt2-finetuned
```

**Example Output:**
```
Versions of 'gpt2-finetuned':
  v1 - 2024-01-15 10:30:00 UTC - 548,000,000 bytes (PyTorch)
  v2 - 2024-01-16 14:20:00 UTC - 548,000,000 bytes (PyTorch)
  v3 - 2024-01-17 09:15:00 UTC - 548,200,000 bytes (PyTorch)
```

---

### `lineage` - Show Version Lineage

Display the complete lineage/generation history of a model version.

```bash
aim lineage <NAME> <VERSION>
```

**Arguments:**
- `<NAME>` - Model name
- `<VERSION>` - Version number

**Example:**
```bash
aim lineage gpt2-finetuned 5
```

**Example Output:**
```
Lineage for 'gpt2-finetuned' v5:
  v1 - 2024-01-15 10:30:00 UTC - abc123def456
    v2 - 2024-01-16 14:20:00 UTC - def789ghi012
      v3 - 2024-01-17 09:15:00 UTC - ghi345jkl678
        v5 - 2024-01-19 16:45:00 UTC - mno901pqr234
```

---

### `delete` - Delete a Model Version

Delete a specific version of a model.

```bash
aim delete <NAME> <VERSION> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Model name
- `<VERSION>` - Version number to delete

**Options:**
- `-f, --force` - Skip confirmation prompt

**Examples:**
```bash
# Delete with confirmation
aim delete gpt2-finetuned 1

# Force delete without confirmation
aim delete gpt2-finetuned 1 --force
```

**Warning:** Deleted versions cannot be recovered unless you have backups.

---

### `stats` - Show Vault Statistics

Display statistics about the vault.

```bash
aim stats
```

**Example Output:**
```
Vault Statistics:
  Models: 15
  Total versions: 47
  Total size: 25,600,000,000 bytes (24.50 GB)
  Files: 47
```

---

### `compliance` - Run Compliance Checks

Run security and compliance verification checks.

```bash
aim compliance
```

**Example Output:**
```
Running compliance checks...

Compliance Status:
  FIPS 140-3: ✓ PASS
  CVE Scan: ✓ PASS
  MITRE ATT&CK: ✓ PASS
  CMMC Level: 2

✓ No violations detected
```

---

### `change-passphrase` - Change Vault Passphrase

Change the passphrase for the vault (requires re-encryption).

```bash
aim change-passphrase
```

**Note:** This feature re-encrypts all stored models with the new passphrase.

---

## Utility Commands

### `archive` - Archive Models

Archive multiple models into a single TAR or ZIP file for backup or transfer.

```bash
aim archive <MODELS>... <OUTPUT> [OPTIONS]
```

**Arguments:**
- `<MODELS>...` - Model names to archive (space-separated)
- `<OUTPUT>` - Output archive path

**Options:**
- `-f, --format <FORMAT>` - Archive format: `tar` or `zip` (default: tar)
- `-v, --versions <NUMBERS>` - Specific version numbers for each model

**Examples:**
```bash
# Archive models to TAR
aim archive gpt2-finetuned bert-base resnet50 backup.tar

# Archive to ZIP with specific versions
aim archive model1 model2 backup.zip --format zip --versions 3 2

# Quick backup of all critical models
aim archive prod-model-v1 prod-model-v2 prod-backup.tar
```

---

### `extract` - Extract Archive

Extract models from a TAR or ZIP archive.

```bash
aim extract <ARCHIVE> [OPTIONS]
```

**Arguments:**
- `<ARCHIVE>` - Path to archive file (.tar or .zip)

**Options:**
- `-o, --output <DIR>` - Output directory (default: current directory)

**Examples:**
```bash
# Extract to current directory
aim extract backup.tar

# Extract to specific directory
aim extract backup.zip --output ./restored_models
```

---

### `analyze` - Analyze Model

Analyze compression efficiency and model characteristics.

```bash
aim analyze <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Model name

**Options:**
- `-v, --version <NUMBER>` - Version number (default: latest)

**Example:**
```bash
aim analyze gpt2-finetuned
```

**Example Output:**
```
Compression Analysis for 'gpt2-finetuned' v3:
  Original size: 548,000,000 bytes
  Compressed size: 412,500,000 bytes
  Compression ratio: 24.73%

📊 Compression Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Original Size:    522.6 MB
Compressed Size:  393.5 MB
Compression:      24.73%
Space Saved:      129.1 MB
Format:           PyTorch
Efficiency:       Excellent ✓

Model Analysis:
  Size: 522.6 MB
  Format: PyTorch
  Parameters: ~355M
  Framework: PyTorch 2.1
  Task: text-generation
```

---

### `deduplicate` - Find Duplicates

Find duplicate models in the vault using content hashing.

```bash
aim deduplicate [OPTIONS]
```

**Options:**
- `-d, --detailed` - Show detailed similarity scores

**Examples:**
```bash
# Find duplicates
aim deduplicate

# Show detailed similarity analysis
aim deduplicate --detailed
```

**Example Output:**
```
Scanning for duplicate models...

Found 2 duplicate groups:

Group 1 (2 models):
  - gpt2-backup
  - gpt2-copy
    Similarity: 100.00%

Group 2 (3 models):
  - bert-v1
  - bert-v1-backup
  - bert-snapshot

You can save space by removing duplicates.
```

---

### `export` - Export with Metadata

Export a model along with its metadata as a JSON file.

```bash
aim export <NAME> <OUTPUT> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Model name
- `<OUTPUT>` - Output directory

**Options:**
- `-v, --version <NUMBER>` - Version number (default: latest)

**Examples:**
```bash
# Export latest version
aim export gpt2-finetuned ./exports

# Export specific version
aim export bert-base ./models --version 5
```

**Creates:**
- `<OUTPUT>/<NAME>` - Model file
- `<OUTPUT>/<NAME>_metadata.json` - Metadata file

**Metadata Format:**
```json
{
  "name": "gpt2-finetuned",
  "format": "PyTorch",
  "version": "3",
  "framework": "PyTorch 2.1",
  "task": "text-generation",
  "description": "GPT-2 fine-tuned on custom dataset"
}
```

---

### `cache` - Cache Statistics

Show caching statistics (for programmatic usage information).

```bash
aim cache
```

**Note:** The CLI displays usage information. To use caching in your code, see the API documentation and examples.

---

### `convert` - Convert Model Formats

Convert models between different formats (PyTorch, ONNX, Safetensors, GGUF, etc.).

```bash
aim convert <MODEL> --to-format <FORMAT> [OPTIONS]
```

**Arguments:**
- `<MODEL>` - Model name in your vault

**Options:**
- `-t, --to-format <FORMAT>` - Target format (safetensors, onnx, gguf, tflite, coreml, mlx, etc.)
- `-o, --output <PATH>` - Output file path (optional, defaults to model_name.{extension})
- `-v, --version <VERSION>` - Version number (latest if not specified)
- `-q, --quantization <LEVEL>` - Quantization level for GGUF conversion (q4_0, q4_k_m, q8_0, etc.)

**Supported Formats:**
- `safetensors` - Safetensors format
- `gguf` - GGUF (llama.cpp) format
- `pytorch` / `pt` - PyTorch format
- `onnx` - ONNX format
- `tensorrt` / `trt` - TensorRT format
- `tflite` - TensorFlow Lite format
- `coreml` - Core ML format
- `mlx` - Apple MLX format
- `torchscript` - TorchScript format
- `openvino` - OpenVINO format
- `ncnn` - NCNN format
- `mnn` - MNN format

**Examples:**
```bash
# Convert PyTorch to Safetensors
aim convert llama-2-7b --to-format safetensors

# Convert to GGUF with quantization
aim convert gpt2-model --to-format gguf --quantization q4_k_m

# Convert specific version to ONNX
aim convert bert-base --to-format onnx --version 2 --output bert-v2.onnx

# Convert to TensorFlow Lite for mobile
aim convert mobilenet --to-format tflite --output model.tflite

# Convert to Core ML for Apple devices
aim convert resnet50 --to-format coreml --output resnet50.mlmodel
```

**How it Works:**

The convert command provides guidance on converting between formats:

1. **Automatic Detection**: Detects source format from vault metadata
2. **Conversion Paths**: Shows recommended tools and commands for conversion
3. **Guidance Output**: Provides step-by-step instructions with specific tools:
   - **PyTorch → Safetensors**: Uses `safetensors.torch.save_file()`
   - **PyTorch → ONNX**: Uses `torch.onnx.export()`
   - **Safetensors → GGUF**: Uses llama.cpp `convert.py` with quantization
   - **ONNX → TensorRT**: Uses `trtexec` compiler
   - **PyTorch → Core ML**: Uses `coremltools.convert()`
   - **PyTorch → TFLite**: Uses `ai_edge_torch`

**Common Conversion Workflows:**

```bash
# Training → Production (LLM)
aim convert my-llm --to-format safetensors        # PyTorch → Safetensors
aim convert my-llm --to-format gguf -q q4_k_m     # Safetensors → GGUF

# Research → Mobile
aim convert my-model --to-format onnx             # PyTorch → ONNX
aim convert my-model --to-format tflite           # ONNX → TFLite

# Edge Deployment
aim convert vision-model --to-format openvino     # ONNX → OpenVINO
aim convert vision-model --to-format ncnn         # ONNX → NCNN
```

**Note:** Format conversion requires external tools (PyTorch, ONNX Runtime, llama.cpp, etc.). The command provides detailed instructions for each conversion path.

---

### `cloud` - Cloud Storage Operations

Upload, download, and manage models in cloud storage (S3, Azure, GCS).

```bash
aim cloud <SUBCOMMAND>
```

**Subcommands:**

#### `push` - Push Model to Cloud

```bash
aim cloud push <MODEL> --provider <PROVIDER> --bucket <BUCKET> [OPTIONS]
```

**Arguments:**
- `<MODEL>` - Model name in your vault

**Options:**
- `-p, --provider <PROVIDER>` - Cloud provider: s3, azure, or gcs
- `-b, --bucket <BUCKET>` - Bucket/container name
- `-v, --version <VERSION>` - Version number (latest if not specified)

**Examples:**
```bash
# Push to AWS S3
aim cloud push gpt2-finetuned --provider s3 --bucket my-models

# Push specific version to Azure
aim cloud push bert-classifier --provider azure --bucket ml-models --version 3
```

#### `pull` - Pull Model from Cloud

```bash
aim cloud pull <MODEL> --provider <PROVIDER> --bucket <BUCKET> --remote-path <PATH>
```

**Arguments:**
- `<MODEL>` - Model name to save as locally

**Options:**
- `-p, --provider <PROVIDER>` - Cloud provider: s3, azure, or gcs
- `-b, --bucket <BUCKET>` - Bucket/container name
- `-r, --remote-path <PATH>` - Path to model in cloud storage

**Examples:**
```bash
# Pull from S3
aim cloud pull gpt2-finetuned --provider s3 --bucket my-models --remote-path gpt2-finetuned/safetensors/v2.vault

# Pull from Azure
aim cloud pull bert-classifier --provider azure --bucket ml-models --remote-path models/bert/v1.vault
```

#### `list` - List Cloud Models

```bash
aim cloud list --provider <PROVIDER> --bucket <BUCKET> [OPTIONS]
```

**Options:**
- `-p, --provider <PROVIDER>` - Cloud provider: s3, azure, or gcs
- `-b, --bucket <BUCKET>` - Bucket/container name
- `--prefix <PREFIX>` - Filter by path prefix (optional)

**Examples:**
```bash
# List all models in S3 bucket
aim cloud list --provider s3 --bucket my-models

# List models with prefix
aim cloud list --provider azure --bucket ml-models --prefix production/
```

#### `config` - Configure Cloud Credentials

```bash
aim cloud config --provider <PROVIDER> [--show]
```

**Options:**
- `-p, --provider <PROVIDER>` - Cloud provider: s3, azure, or gcs
- `--show` - Display current configuration status

**Examples:**
```bash
# Show S3 configuration
aim cloud config --provider s3 --show

# Show Azure configuration
aim cloud config --provider azure --show
```

**Cloud Provider Setup:**

**AWS S3:**
```bash
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_REGION=us-east-1  # optional
```

**Azure Blob Storage:**
```bash
export AZURE_STORAGE_ACCOUNT=your_account_name
export AZURE_STORAGE_SAS_TOKEN=your_sas_token   # or Entra ID: AZURE_TENANT_ID / AZURE_CLIENT_ID / AZURE_CLIENT_SECRET
```

**Google Cloud Storage:**
```
⚠️  Temporarily disabled due to security vulnerabilities
Use S3 or Azure instead
```

**See Also:** [Cloud CLI Guide](CLOUD_CLI.md) for detailed cloud storage documentation.

---

## Environment Variables

- `aimodelvault_CONFIG` - Config directory override (`config.yaml`, profiles, plugins)
- `aimodelvault_HOME` - Relocates all config/data/cache directories under one root
- `aimodelvault_VAULT` - Default vault name
- `aimodelvault_PASSPHRASE` - Passphrase for unattended use: a literal value or a KMS URI (`env://`, `file://`, `aws-sm://`, `azure-kv://`, `vault://`). Prefer a KMS URI or `file://` over a literal on shared hosts — see [KMS.md](KMS.md)
- `AWS_ACCESS_KEY_ID` - AWS access key for S3
- `AWS_SECRET_ACCESS_KEY` - AWS secret key for S3
- `AWS_REGION` - AWS region (default: us-east-1)
- `AZURE_STORAGE_ACCOUNT` - Azure storage account name
- `AZURE_STORAGE_SAS_TOKEN` - Azure SAS token (shared account keys are not supported; use a SAS or Entra ID)

---

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Authentication failed
- `3` - Model not found
- `4` - Permission denied
- `5` - Integrity check failed

---

## Configuration File

Location: `~/.config/aimodelvault/config.yaml`

See [QUICKSTART.md](QUICKSTART.md#configuration) for configuration details.

---

## Security Considerations

1. **Passphrase Entry**: Always enter passphrases interactively (prompts hide input)
2. **Audit Logs**: Review `~/.local/share/aimodelvault/logs/audit.log` regularly
3. **Permissions**: Ensure proper file permissions (700 for directories, 600 for files)
4. **Backups**: Backup your vault data regularly to a secure location

---

## Examples

### Complete Workflow

```bash
# 1. Initialize vault
aim init --name ml-models

# 2. Store a model
aim store my-transformer ./model.pt \
  --format pytorch \
  --description "Custom transformer model" \
  --task "text-classification"

# 3. List models
aim list

# 4. View versions
aim versions my-transformer

# 5. Retrieve model
aim get my-transformer ./output.pt

# 6. Analyze compression and characteristics
aim analyze my-transformer

# 7. Check compliance
aim compliance

# 8. View statistics
aim stats
```

### Batch Operations

```bash
# Store multiple models
for model in models/*.pt; do
  name=$(basename "$model" .pt)
  aim store "$name" "$model" --format pytorch
done

# Export all latest versions
for model in $(aim list | grep -oE '^\s+\S+'); do
  aim get "$model" "./exports/${model}.pt"
done

# Create backup archive
aim archive model1 model2 model3 backup.tar

# Find and remove duplicates
aim deduplicate --detailed
```

### Utility Workflows

```bash
# Backup workflow
aim archive prod-model test-model backup-$(date +%Y%m%d).tar
aim extract backup-20241028.tar --output ./restored

# Analysis workflow
aim analyze my-model
aim deduplicate
aim stats

# Export workflow
aim export my-model ./exports
# Creates: ./exports/my-model and ./exports/my-model_metadata.json
```

---

## Troubleshooting

### Command Not Found

```bash
# Ensure aimodelvault is in PATH
cargo install --path .
# or
export PATH="$HOME/.cargo/bin:$PATH"
```

### Permission Denied

```bash
# Fix permissions
chmod 700 ~/.local/share/aimodelvault
chmod 600 ~/.config/aimodelvault/config.yaml
```

### Authentication Failures

- Verify you're using the correct passphrase
- Check caps lock is not enabled
- Ensure vault has not been corrupted

---

## See Also

- [Quick Start Guide](QUICKSTART.md)
- [Security Policy](https://github.com/nervosys/AIModelVault/blob/master/SECURITY.md)
- [API Documentation](https://docs.rs/aimodelvault)
