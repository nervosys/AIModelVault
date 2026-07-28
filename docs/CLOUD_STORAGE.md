# Cloud Storage Guide

**AI Model Vault Cloud Storage Integration**

Store and sync your AI models across AWS S3, Azure Blob Storage, and Google Cloud Storage with the same security and encryption as local storage.

---

## 📋 Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [AWS S3](#aws-s3)
- [Azure Blob Storage](#azure-blob-storage)
- [Google Cloud Storage](#google-cloud-storage)
- [API Usage](#api-usage)
- [CLI Usage](#cli-usage)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

---

## 🌟 Overview

### What Cloud Storage Adds

- **Team Collaboration**: Share models across team members
- **Backup & DR**: Automatic off-site backups
- **Scalability**: Store unlimited models without local disk space
- **Multi-Region**: Deploy models close to inference servers
- **Cost Optimization**: Lifecycle policies for old versions

### Security Model

All data is **encrypted before upload**:
1. Model compressed locally
2. Encrypted with AES-256-GCM
3. Uploaded to cloud storage
4. Only encrypted data leaves your machine

**Cloud providers never see your plaintext models.**

---

## 📦 Installation

### Build with Cloud Support

```bash
# AWS S3 only
cargo build --release --features s3

# Azure Blob Storage only
cargo build --release --features azure

# Google Cloud Storage only
cargo build --release --features gcs

# All cloud providers
cargo build --release --features cloud

# Full build (all features)
cargo build --release --features full,cloud
```

### Dependencies

Each cloud provider requires its SDK:

- **S3**: `aws-sdk-s3` (included)
- **Azure**: `azure_storage_blobs` (included)
- **GCS**: `cloud-storage` (included)

---

## ☁️ AWS S3

### Prerequisites

1. AWS account with S3 access
2. IAM user/role with permissions:
   ```json
   {
     "Version": "2012-10-17",
     "Statement": [
       {
         "Effect": "Allow",
         "Action": [
           "s3:PutObject",
           "s3:GetObject",
           "s3:DeleteObject",
           "s3:ListBucket"
         ],
         "Resource": [
           "arn:aws:s3:::your-bucket-name",
           "arn:aws:s3:::your-bucket-name/*"
         ]
       }
     ]
   }
   ```

### Authentication

**Option 1: Environment Variables** (Recommended)
```bash
export AWS_ACCESS_KEY_ID="your-access-key"
export AWS_SECRET_ACCESS_KEY="your-secret-key"
export AWS_DEFAULT_REGION="us-east-1"
```

**Option 2: AWS CLI Config**
```bash
aws configure
# Enter your credentials when prompted
```

**Option 3: IAM Role** (EC2/ECS/Lambda)
- Attach IAM role to instance
- SDK automatically uses role credentials

### Configuration

```rust
use ai_model_vault::storage::{StorageConfig, StorageBackend};

#[tokio::main]
async fn main() -> Result<()> {
    let config = StorageConfig::S3 {
        bucket: "AI Model Vault-models".to_string(),
        region: "us-east-1".to_string(),
        prefix: Some("production".to_string()), // Optional folder
    };

    let backend = config.create_backend().await?;
    
    // Upload encrypted model
    let model_data = std::fs::read("model.bin")?;
    backend.upload("gpt-3/checkpoint-1000", &model_data).await?;
    
    println!("Model uploaded to S3!");
    Ok(())
}
```

### CLI Usage

```bash
# Configure S3 remote
aim remote add s3-prod \
    --provider s3 \
    --bucket AI Model Vault-models \
    --region us-east-1 \
    --prefix production

# Push model to S3
aim push gpt-3-v1 --remote s3-prod

# Pull model from S3
aim pull gpt-3-v1 --remote s3-prod

# Sync all models to S3
aim sync --remote s3-prod --direction push

# List S3 models
aim remote list s3-prod
```

### Cost Optimization

```bash
# Use S3 Intelligent-Tiering
aws s3api put-bucket-intelligent-tiering-configuration \
    --bucket AI Model Vault-models \
    --id archive-old-models \
    --intelligent-tiering-configuration '{
        "Id": "archive-old-models",
        "Status": "Enabled",
        "Tierings": [
            {
                "Days": 90,
                "AccessTier": "ARCHIVE_ACCESS"
            }
        ]
    }'

# Lifecycle policy for old versions
aws s3api put-bucket-lifecycle-configuration \
    --bucket AI Model Vault-models \
    --lifecycle-configuration file://lifecycle.json
```

**lifecycle.json**:
```json
{
  "Rules": [
    {
      "Id": "DeleteOldVersions",
      "Status": "Enabled",
      "NoncurrentVersionExpiration": {
        "NoncurrentDays": 180
      }
    }
  ]
}
```

---

## 🔵 Azure Blob Storage

### Prerequisites

1. Azure account with Storage Account
2. Storage Account access (Account Key or SAS Token)

### Create Storage Account

```bash
# Login to Azure
az login

# Create resource group
az group create --name AI Model Vault-rg --location eastus

# Create storage account
az storage account create \
    --name AI Model Vaultstorage \
    --resource-group AI Model Vault-rg \
    --location eastus \
    --sku Standard_LRS

# Create container
az storage container create \
    --name models \
    --account-name AI Model Vaultstorage
```

### Authentication

**Option 1: Account Key**
```bash
# Get account key
az storage account keys list \
    --account-name AI Model Vaultstorage \
    --query '[0].value' -o tsv

export AZURE_STORAGE_ACCOUNT="AI Model Vaultstorage"
export AZURE_STORAGE_KEY="your-account-key"
```

**Option 2: SAS Token**
```bash
# Generate SAS token (read/write for 1 year)
az storage container generate-sas \
    --account-name AI Model Vaultstorage \
    --name models \
    --permissions rwdl \
    --expiry 2025-12-31 \
    --https-only

export AZURE_STORAGE_ACCOUNT="AI Model Vaultstorage"
export AZURE_STORAGE_SAS_TOKEN="your-sas-token"
```

### Configuration

```rust
use ai_model_vault::storage::{StorageConfig, StorageBackend};

#[tokio::main]
async fn main() -> Result<()> {
    let config = StorageConfig::Azure {
        account: "AI Model Vaultstorage".to_string(),
        container: "models".to_string(),
        prefix: Some("production".to_string()),
    };

    let backend = config.create_backend().await?;
    
    // Upload model
    let model_data = std::fs::read("model.bin")?;
    backend.upload("llama-7b/v1", &model_data).await?;
    
    println!("Model uploaded to Azure!");
    Ok(())
}
```

### CLI Usage

```bash
# Configure Azure remote
aim remote add azure-prod \
    --provider azure \
    --account AI Model Vaultstorage \
    --container models \
    --prefix production

# Push to Azure
aim push llama-7b --remote azure-prod

# Pull from Azure
aim pull llama-7b --remote azure-prod
```

### Access Tiers

```bash
# Move to cool tier (30+ day retention)
az storage blob set-tier \
    --account-name AI Model Vaultstorage \
    --container-name models \
    --name production/llama-7b/v1 \
    --tier Cool

# Move to archive tier (rare access)
az storage blob set-tier \
    --account-name AI Model Vaultstorage \
    --container-name models \
    --name production/old-model/v1 \
    --tier Archive
```

---

## 🟢 Google Cloud Storage

### Prerequisites

1. GCP account with Cloud Storage enabled
2. Service account with Storage Object Admin role

### Setup

```bash
# Install gcloud CLI
# https://cloud.google.com/sdk/docs/install

# Login
gcloud auth login

# Create project (if needed)
gcloud projects create AI Model Vault-project

# Set project
gcloud config set project AI Model Vault-project

# Create bucket
gsutil mb -l us-east1 gs://AI Model Vault-models

# Create service account
gcloud iam service-accounts create AI Model Vault-sa \
    --display-name "AI Model Vault Service Account"

# Grant permissions
gsutil iam ch \
    serviceAccount:AI Model Vault-sa@AI Model Vault-project.iam.gserviceaccount.com:objectAdmin \
    gs://AI Model Vault-models

# Create key file
gcloud iam service-accounts keys create \
    ~/AI Model Vault-key.json \
    --iam-account AI Model Vault-sa@AI Model Vault-project.iam.gserviceaccount.com
```

### Authentication

```bash
export GOOGLE_APPLICATION_CREDENTIALS="$HOME/AI Model Vault-key.json"
export GCP_PROJECT="AI Model Vault-project"
```

### Configuration

```rust
use ai_model_vault::storage::{StorageConfig, StorageBackend};

#[tokio::main]
async fn main() -> Result<()> {
    let config = StorageConfig::Gcs {
        bucket: "AI Model Vault-models".to_string(),
        project: "AI Model Vault-project".to_string(),
        prefix: Some("production".to_string()),
    };

    let backend = config.create_backend().await?;
    
    // Upload model
    let model_data = std::fs::read("model.bin")?;
    backend.upload("bert-base/v2", &model_data).await?;
    
    println!("Model uploaded to GCS!");
    Ok(())
}
```

### CLI Usage

```bash
# Configure GCS remote
aim remote add gcs-prod \
    --provider gcs \
    --bucket AI Model Vault-models \
    --project AI Model Vault-project \
    --prefix production

# Push to GCS
aim push bert-base --remote gcs-prod

# Pull from GCS
aim pull bert-base --remote gcs-prod
```

### Lifecycle Management

```bash
# Create lifecycle policy
cat > lifecycle.json <<EOF
{
  "lifecycle": {
    "rule": [
      {
        "action": {"type": "SetStorageClass", "storageClass": "NEARLINE"},
        "condition": {"age": 30}
      },
      {
        "action": {"type": "SetStorageClass", "storageClass": "COLDLINE"},
        "condition": {"age": 90}
      },
      {
        "action": {"type": "Delete"},
        "condition": {"age": 365}
      }
    ]
  }
}
EOF

gsutil lifecycle set lifecycle.json gs://AI Model Vault-models
```

---

## 🔧 API Usage

### Complete Example

```rust
use ai_model_vault::storage::{StorageConfig, StorageBackend};
use ai_model_vault::{Vault, VaultConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup cloud backend
    let s3_config = StorageConfig::S3 {
        bucket: "my-models".to_string(),
        region: "us-west-2".to_string(),
        prefix: Some("team/production".to_string()),
    };
    
    let backend = s3_config.create_backend().await?;
    
    // 2. Store encrypted model locally first
    let mut vault = Vault::new("my-vault")?;
    vault.init(b"strong-passphrase")?;
    
    let model_data = std::fs::read("gpt-neo-125m.bin")?;
    vault.store(
        "gpt-neo-125m",
        "v1.0",
        &model_data,
        &metadata,
    )?;
    
    // 3. Push to cloud
    let encrypted_data = vault.get_raw("gpt-neo-125m", "v1.0")?;
    backend.upload("gpt-neo-125m/v1.0.enc", &encrypted_data).await?;
    
    println!("✅ Model encrypted and uploaded to S3!");
    
    // 4. Pull from cloud (on different machine)
    let downloaded = backend.download("gpt-neo-125m/v1.0.enc").await?;
    std::fs::write("downloaded.enc", &downloaded)?;
    
    // 5. Decrypt and use
    let decrypted = vault.decrypt_raw(&downloaded, b"strong-passphrase")?;
    
    println!("✅ Model downloaded and decrypted!");
    Ok(())
}
```

### Async Storage Operations

```rust
use ai_model_vault::storage::{StorageBackend, StorageConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let backend = StorageConfig::S3 {
        bucket: "models".to_string(),
        region: "us-east-1".to_string(),
        prefix: None,
    }.create_backend().await?;
    
    // Upload
    let data = b"model data";
    backend.upload("model.bin", data).await?;
    
    // Check exists
    if backend.exists("model.bin").await? {
        println!("Model exists!");
    }
    
    // Get size
    let size = backend.size("model.bin").await?;
    println!("Model size: {} bytes", size);
    
    // Download
    let downloaded = backend.download("model.bin").await?;
    assert_eq!(data, &downloaded[..]);
    
    // List all
    let files = backend.list().await?;
    for file in files {
        println!("- {}", file);
    }
    
    // Delete
    backend.delete("model.bin").await?;
    
    Ok(())
}
```

---

## 💻 CLI Usage

### Remote Management

```bash
# Add remotes
aim remote add s3-backup --provider s3 --bucket backup --region us-east-1
aim remote add azure-prod --provider azure --account prod --container models
aim remote add gcs-archive --provider gcs --bucket archive --project my-proj

# List remotes
aim remote list
# Output:
# s3-backup      s3://backup (us-east-1)
# azure-prod     azure://prod/models
# gcs-archive    gs://archive

# Remove remote
aim remote remove s3-backup
```

### Push/Pull Operations

```bash
# Push single model
aim push gpt-3-v1 --remote s3-backup

# Push with progress
aim push llama-70b --remote azure-prod --progress

# Push all models
aim push --all --remote gcs-archive

# Pull single model
aim pull bert-base --remote s3-backup

# Pull specific version
aim pull gpt-3-v1 --version v2.0 --remote azure-prod

# Pull and overwrite local
aim pull llama-7b --remote gcs-archive --force
```

### Sync Operations

```bash
# Sync to cloud (push only new/changed)
aim sync --remote s3-backup --direction push

# Sync from cloud (pull only new/changed)
aim sync --remote azure-prod --direction pull

# Two-way sync
aim sync --remote gcs-archive --direction both

# Dry-run (show what would be synced)
aim sync --remote s3-backup --dry-run
```

### Cloud Storage Info

```bash
# List files in remote
aim remote list s3-backup --files

# Show remote statistics
aim remote stats azure-prod
# Output:
# Models: 23
# Total size: 147 GB
# Last sync: 2024-03-15 14:30 UTC

# Check model exists in cloud
aim remote check gpt-3-v1 --remote s3-backup
```

---

## ✅ Best Practices

### 1. Security

```bash
# ✅ Always use IAM roles when possible (no keys to manage)
# ✅ Rotate access keys every 90 days
# ✅ Use separate buckets for prod/dev
# ✅ Enable bucket versioning for recovery
# ✅ Enable MFA delete for S3

# S3 bucket versioning
aws s3api put-bucket-versioning \
    --bucket AI Model Vault-models \
    --versioning-configuration Status=Enabled

# S3 MFA delete (requires root account)
aws s3api put-bucket-versioning \
    --bucket AI Model Vault-models \
    --versioning-configuration Status=Enabled,MFADelete=Enabled \
    --mfa "arn:aws:iam::ACCOUNT:mfa/root-account-mfa-device XXXXXX"
```

### 2. Performance

```bash
# ✅ Use regions close to your compute
# ✅ Enable transfer acceleration for S3 (global uploads)
# ✅ Use multipart uploads for models >100MB
# ✅ Compress models before cloud upload

# S3 transfer acceleration
aws s3api put-bucket-accelerate-configuration \
    --bucket AI Model Vault-models \
    --accelerate-configuration Status=Enabled
```

### 3. Cost Management

```bash
# ✅ Use lifecycle policies to move old models to cheaper storage
# ✅ Delete old versions after X days
# ✅ Monitor storage costs with CloudWatch/Azure Monitor/GCP Monitoring

# Cost monitoring example (AWS)
aws ce get-cost-and-usage \
    --time-period Start=2024-03-01,End=2024-03-31 \
    --granularity MONTHLY \
    --metrics "BlendedCost" \
    --filter file://s3-filter.json
```

### 4. Disaster Recovery

```bash
# ✅ Use cross-region replication
# ✅ Regular backup verification
# ✅ Document recovery procedures

# S3 cross-region replication
aws s3api put-bucket-replication \
    --bucket AI Model Vault-models \
    --replication-configuration file://replication.json
```

---

## 🔍 Troubleshooting

### S3 Issues

**Error: "Access Denied"**
```bash
# Check IAM permissions
aws iam get-user-policy --user-name AI Model Vault --policy-name S3Access

# Verify bucket policy
aws s3api get-bucket-policy --bucket AI Model Vault-models

# Test credentials
aws sts get-caller-identity
```

**Error: "No credentials found"**
```bash
# Check environment
echo $AWS_ACCESS_KEY_ID
echo $AWS_SECRET_ACCESS_KEY

# Verify AWS CLI config
cat ~/.aws/credentials
cat ~/.aws/config

# Test with AWS CLI
aws s3 ls s3://AI Model Vault-models
```

### Azure Issues

**Error: "Authentication failed"**
```bash
# Check account key
az storage account keys list --account-name AI Model Vaultstorage

# Verify SAS token expiry
az storage container show \
    --account-name AI Model Vaultstorage \
    --name models

# Test connection
az storage blob list \
    --account-name AI Model Vaultstorage \
    --container-name models
```

### GCS Issues

**Error: "Service account not found"**
```bash
# Check credentials file
cat $GOOGLE_APPLICATION_CREDENTIALS

# List service accounts
gcloud iam service-accounts list

# Test authentication
gcloud auth application-default print-access-token

# Verify bucket access
gsutil ls gs://AI Model Vault-models
```

### Network Issues

```bash
# Test connectivity
curl -I https://s3.amazonaws.com
curl -I https://AI Model Vaultstorage.blob.core.windows.net
curl -I https://storage.googleapis.com

# Use proxy if behind firewall
export HTTP_PROXY=http://proxy:port
export HTTPS_PROXY=http://proxy:port

# Enable SDK debug logging
export RUST_LOG=aws_sdk_s3=debug
export RUST_LOG=azure_storage=debug
export RUST_LOG=cloud_storage=debug
```

---

## 📊 Performance Comparison

| Provider | Upload 1GB | Download 1GB | List 1000 files | Cost (per GB/month) |
| -------- | ---------- | ------------ | --------------- | ------------------- |
| S3       | ~8 sec     | ~6 sec       | ~0.5 sec        | $0.023              |
| Azure    | ~10 sec    | ~7 sec       | ~0.8 sec        | $0.020              |
| GCS      | ~9 sec     | ~6 sec       | ~0.6 sec        | $0.020              |
| Local    | ~2 sec     | ~2 sec       | ~0.1 sec        | -                   |

*Note: Times are approximate and depend on network speed and region.*

---

## 📚 Additional Resources

- [AWS S3 Documentation](https://docs.aws.amazon.com/s3/)
- [Azure Blob Storage Documentation](https://docs.microsoft.com/azure/storage/blobs/)
- [Google Cloud Storage Documentation](https://cloud.google.com/storage/docs)
- [AI Model Vault Security Guide](https://github.com/nervosys/AIModelVault/blob/master/SECURITY.md)

---

## 🚀 What's Next?

- Multi-region replication
- Intelligent caching layer
- P2P model sharing
- Built-in cost analysis
- Auto-compression recommendations

---

**Built with 🦀 Rust for secure, reliable cloud model storage.**
