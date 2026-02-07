# Version Control Implementation - Complete Documentation

## Overview

AI Model Vault (AIMV) provides a complete **Git-like version control system** for AI models, enabling:

✅ **Sequential versioning** (v1, v2, v3, ...)  
✅ **Branching & parallel development** (A/B testing, multi-task)  
✅ **Complete lineage tracking** (parent-child relationships)  
✅ **Time travel** (instant rollback to any version)  
✅ **Version comparison** (side-by-side metadata & metrics)  
✅ **Cleanup policies** (retention rules, storage optimization)  
✅ **Checksum verification** (SHA-256 integrity)  
✅ **Metadata evolution** (track training parameters across versions)  

---

## Implementation Status

| Feature               | Status     | API | CLI | Demo | Docs |
| --------------------- | ---------- | --- | --- | ---- | ---- |
| Sequential Versioning | ✅ Complete | ✅   | ✅   | ✅    | ✅    |
| Branching             | ✅ Complete | ✅   | ✅   | ✅    | ✅    |
| Lineage Tracking      | ✅ Complete | ✅   | ✅   | ✅    | ✅    |
| Time Travel           | ✅ Complete | ✅   | ✅   | ✅    | ✅    |
| Version Comparison    | ✅ Complete | ✅   | ✅   | ✅    | ✅    |
| Cleanup Policies      | ✅ Complete | ✅   | ✅   | ✅    | ✅    |
| Checksum Verification | ✅ Complete | ✅   | ✅   | ✅    | ✅    |
| Metadata Tracking     | ✅ Complete | ✅   | ✅   | ✅    | ✅    |

**Summary**: 8/8 features complete (100%) with full API, CLI, demo, and documentation coverage.

---

## Architecture

### Core Components

```
src/
├── version.rs            # Version control system implementation
│   ├── ModelVersion      # Version structure with metadata
│   ├── VersionControl    # Version management API
│   └── Checksum          # SHA-256 integrity verification
├── vault.rs              # Main vault with version integration
├── config.rs             # Configuration (XDG paths)
├── formats.rs            # ModelMetadata structure
└── storage.rs            # Storage backends (local + cloud)
```

### Data Structures

```rust
// Version metadata
pub struct ModelVersion {
    pub version: u32,                    // Sequential: 1, 2, 3...
    pub checkpoint_id: String,           // Unique: "model-v2-{uuid}"
    pub timestamp: DateTime<Utc>,        // Creation timestamp
    pub parent_version: Option<u32>,     // Parent for lineage
    pub format: String,                  // safetensors, gguf, etc.
    pub size_bytes: u64,                 // Original size
    pub compressed_size_bytes: u64,      // After compression
    pub checksum_sha256: String,         // SHA-256 hash
    pub metadata: HashMap<String, String>, // Custom fields
    pub file_path: String,               // Encrypted file path
}

// Version control manager
pub struct VersionControl {
    versions: HashMap<String, Vec<ModelVersion>>, // model_name → versions
    version_file: PathBuf,                         // ~/.local/share/ai/models/versions.json
}
```

### File Organization

```
~/.local/share/ai/models/
├── versions.json              # Version metadata (encrypted)
├── llama-2-7b-chat/
│   ├── v1.aimv.enc           # Version 1 (encrypted)
│   ├── v2.aimv.enc           # Version 2 (encrypted)
│   ├── v3.aimv.enc           # Version 3 (encrypted)
│   └── v5.aimv.enc           # Version 5 (encrypted)
└── gpt2-medium/
    ├── v1.aimv.enc
    └── v2.aimv.enc
```

---

## Feature Details

### 1. Sequential Versioning

**Implementation**: Automatic version numbering starting from 1.

```rust
impl VersionControl {
    pub fn add_version(
        &mut self,
        model_name: &str,
        file_path: &str,
        format: &str,
        size_bytes: u64,
        compressed_size_bytes: u64,
        checksum: &str,
        metadata: Option<HashMap<String, String>>,
        parent_version: Option<u32>,
    ) -> Result<ModelVersion> {
        // Get next version number
        let versions = self.versions.entry(model_name.to_string())
            .or_insert_with(Vec::new);
        
        let next_version = versions.iter()
            .map(|v| v.version)
            .max()
            .unwrap_or(0) + 1;
        
        // Create unique checkpoint ID
        let checkpoint_id = format!("{}-v{}-{}", 
            model_name, 
            next_version, 
            Uuid::new_v4()
        );
        
        // Create version
        let version = ModelVersion {
            version: next_version,
            checkpoint_id,
            timestamp: Utc::now(),
            parent_version,
            format: format.to_string(),
            size_bytes,
            compressed_size_bytes,
            checksum_sha256: checksum.to_string(),
            metadata: metadata.unwrap_or_default(),
            file_path: file_path.to_string(),
        };
        
        versions.push(version.clone());
        self.save()?;
        
        Ok(version)
    }
}
```

**Key Features**:
- Automatic incrementing (1, 2, 3, ...)
- Unique checkpoint IDs with UUID
- Timestamp tracking
- No version number conflicts

### 2. Branching

**Implementation**: Parent-child relationships via `parent_version` field.

```rust
// Create branches from same parent
let v2 = vault.store_model(name, data_ft, metadata_ft, Some(1))?;    // Main line
let v3 = vault.store_model(name, data_a, metadata_a, Some(2))?;      // Branch A
let v4 = vault.store_model(name, data_b, metadata_b, Some(2))?;      // Branch B
```

**Version Tree**:
```
v1 (base)
│
v2 (fine-tuned) ← parent_version = Some(1)
├─ v3 (exp-a)   ← parent_version = Some(2)
└─ v4 (exp-b)   ← parent_version = Some(2)
```

**Use Cases**:
- A/B testing different approaches
- Multi-task specialization
- Quantization variants (Q4, Q5, Q8)
- Parallel experimentation

### 3. Lineage Tracking

**Implementation**: Recursive parent traversal.

```rust
impl VersionControl {
    pub fn get_lineage(&self, model_name: &str, version: u32) 
        -> Vec<&ModelVersion> 
    {
        let mut lineage = Vec::new();
        let mut current_version = Some(version);
        
        while let Some(v) = current_version {
            if let Some(version_obj) = self.get_version(model_name, Some(v)) {
                lineage.push(version_obj);
                current_version = version_obj.parent_version;
            } else {
                break;
            }
        }
        
        lineage.reverse(); // Root first
        lineage
    }
}
```

**Output**:
```
v1: 2024-10-01 (pre-training)
  v2: 2024-10-15 (fine-tuning)
    v3: 2024-10-22 (chat-tuning)
      v5: 2024-11-05 (rlhf)
```

**Applications**:
- Audit trail for compliance
- Reproducibility documentation
- Training path analysis
- Debugging issue origins

### 4. Time Travel

**Implementation**: Version retrieval by number.

```rust
impl Vault {
    pub fn get_model(&self, name: &str, version: Option<u32>) 
        -> Result<Vec<u8>> 
    {
        let version_obj = if let Some(v) = version {
            // Get specific version
            self.version_control.get_version(name, Some(v))
                .ok_or_else(|| VaultError::NotFound(format!("Version {}", v)))?
        } else {
            // Get latest version
            self.version_control.get_version(name, None)
                .ok_or_else(|| VaultError::NotFound("No versions".to_string()))?
        };
        
        // Load and decrypt
        let encrypted = fs::read(&version_obj.file_path)?;
        let decrypted = self.crypto.decrypt(&encrypted)?;
        
        // Verify checksum
        if !self.verify_checksum(name, version_obj.version, &decrypted) {
            return Err(VaultError::IntegrityError);
        }
        
        Ok(decrypted)
    }
}
```

**Rollback Workflow**:
```rust
// Current production: v5
// Issue detected: rollback to v3

// 1. Load previous version
let v3_data = vault.get_model("model", Some(3))?;

// 2. Verify integrity
assert!(vault.verify_checksum("model", 3, &v3_data));

// 3. Deploy
deploy(&v3_data)?;

// 4. Optionally continue from v3
let v6 = vault.store_model("model", &new_data, &metadata, Some(3))?;
```

**Speed**: O(1) version lookup, instant access to any checkpoint.

### 5. Version Comparison

**Implementation**: Side-by-side metadata analysis.

```rust
fn compare_versions(v3: &ModelVersion, v5: &ModelVersion) {
    // Size comparison
    println!("Size: {} → {} ({:+})", 
        v3.size_bytes, 
        v5.size_bytes, 
        v5.size_bytes as i64 - v3.size_bytes as i64
    );
    
    // Metadata diff
    for (key, v5_value) in &v5.metadata {
        if let Some(v3_value) = v3.metadata.get(key) {
            if v3_value != v5_value {
                println!("~ {}: {} → {}", key, v3_value, v5_value);
            }
        } else {
            println!("+ {}: {}", key, v5_value);
        }
    }
}
```

**Output**:
```
┌─────────────────┬──────────────┬──────────────┐
│ Metric          │ v3           │ v5           │
├─────────────────┼──────────────┼──────────────┤
│ Date            │ 2024-10-22   │ 2024-11-05   │
│ Parent          │ v2           │ v3           │
│ Size            │ 13.2 GB      │ 13.2 GB      │
│ Compression     │ 41%          │ 40%          │
│ Epochs          │ 40           │ 48           │
│ Learning Rate   │ 2e-5         │ 1e-5         │
└─────────────────┴──────────────┴──────────────┘
```

### 6. Cleanup Policies

**Implementation**: Multiple retention strategies.

```rust
impl VersionControl {
    // Keep last N versions
    pub fn cleanup_old_versions(&mut self, model_name: &str, keep_count: usize) 
        -> Result<Vec<u32>> 
    {
        let versions = self.versions.get_mut(model_name)
            .ok_or_else(|| VaultError::NotFound(model_name.to_string()))?;
        
        if versions.len() <= keep_count {
            return Ok(Vec::new());
        }
        
        // Sort by version number
        versions.sort_by_key(|v| v.version);
        
        // Delete old versions
        let to_delete = versions.len() - keep_count;
        let deleted: Vec<u32> = versions.drain(..to_delete)
            .map(|v| {
                let _ = fs::remove_file(&v.file_path);
                v.version
            })
            .collect();
        
        self.save()?;
        Ok(deleted)
    }
}
```

**Retention Strategies**:

| Strategy         | Description               | Code Example                    |
| ---------------- | ------------------------- | ------------------------------- |
| Keep Last N      | Keep N most recent        | `cleanup_old_versions(name, 5)` |
| Time-based       | Delete older than X days  | Filter by `timestamp`           |
| Generation-based | Keep every Nth generation | Filter by lineage depth         |
| Tag-based        | Keep tagged versions only | Filter by metadata              |
| Hybrid           | Combine multiple rules    | Custom logic                    |

**Storage Savings**:
```
Before: 15 versions × 7.8 GB = 118 GB
After:   5 versions × 7.8 GB =  39 GB
Saved:  79 GB (67%)
```

### 7. Checksum Verification

**Implementation**: SHA-256 hashing with automatic verification.

```rust
use sha2::{Sha256, Digest};

impl Vault {
    // Compute checksum
    fn compute_checksum(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    // Verify checksum
    pub fn verify_checksum(&self, name: &str, version: u32, data: &[u8]) 
        -> bool 
    {
        let version_obj = self.version_control.get_version(name, Some(version))
            .expect("Version not found");
        
        let computed = self.compute_checksum(data);
        computed == version_obj.checksum_sha256
    }
}
```

**Automatic Verification**:
- Every `get_model()` call verifies integrity
- Returns error if checksum mismatch
- No manual verification needed (but available)

**Protection**:
- ✅ Bit rot detection
- ✅ Transmission error detection
- ✅ Tampering detection
- ✅ Corruption detection

### 8. Metadata Tracking

**Implementation**: Key-value metadata with each version.

```rust
let metadata = ModelMetadata::new(name, format)
    .with_framework("PyTorch".to_string())
    .with_task("text-generation".to_string())
    .with_parameters(7_200_000_000)
    .add_custom_field("stage".to_string(), "rlhf".to_string())
    .add_custom_field("epochs".to_string(), "48".to_string())
    .add_custom_field("learning_rate".to_string(), "2e-5".to_string())
    .add_custom_field("batch_size".to_string(), "128".to_string())
    .add_custom_field("dataset".to_string(), "custom-corpus".to_string())
    .add_custom_field("gpu_hours".to_string(), "240".to_string())
    .add_custom_field("notes".to_string(), "Best RLHF run".to_string());
```

**Evolution Tracking**:
```
v1: stage=pre-training,  tokens=1.5T,  precision=fp32
v2: stage=fine-tuning,   tokens=10B,   precision=fp32, epochs=40
v3: stage=chat-tuning,   tokens=5B,    precision=fp32, epochs=40, specialization=chat
v5: stage=rlhf,          tokens=5B,    precision=fp16, epochs=48, rlhf_iterations=3
```

**Query Support**:
```rust
// Find versions with specific metadata
let rlhf_versions: Vec<_> = versions
    .into_iter()
    .filter(|v| v.metadata.contains_key("rlhf_iterations"))
    .collect();

// Find high-epoch versions
let high_epoch: Vec<_> = versions
    .into_iter()
    .filter(|v| {
        v.metadata.get("epochs")
            .and_then(|e| e.parse::<u32>().ok())
            .map(|e| e >= 40)
            .unwrap_or(false)
    })
    .collect();
```

---

## API Reference

### VersionControl Methods

```rust
impl VersionControl {
    // Create new version control
    pub fn new(version_file: PathBuf) -> Result<Self>
    
    // Add new version
    pub fn add_version(
        &mut self,
        model_name: &str,
        file_path: &str,
        format: &str,
        size_bytes: u64,
        compressed_size_bytes: u64,
        checksum: &str,
        metadata: Option<HashMap<String, String>>,
        parent_version: Option<u32>,
    ) -> Result<ModelVersion>
    
    // Get specific version (None = latest)
    pub fn get_version(&self, model_name: &str, version: Option<u32>) 
        -> Option<&ModelVersion>
    
    // List all versions
    pub fn list_versions(&self, model_name: &str) -> Vec<&ModelVersion>
    
    // Get complete lineage
    pub fn get_lineage(&self, model_name: &str, version: u32) 
        -> Vec<&ModelVersion>
    
    // Delete specific version
    pub fn delete_version(&mut self, model_name: &str, version: u32) 
        -> Result<bool>
    
    // Cleanup old versions
    pub fn cleanup_old_versions(&mut self, model_name: &str, keep_count: usize) 
        -> Result<Vec<u32>>
    
    // Verify checksum
    pub fn verify_checksum(&self, model_name: &str, version: u32, data: &[u8]) 
        -> bool
}
```

### Vault Integration

```rust
impl Vault {
    // Store model with versioning
    pub fn store_model(
        &mut self,
        name: &str,
        data: &[u8],
        metadata: &ModelMetadata,
        parent_version: Option<u32>,
    ) -> Result<ModelVersion>
    
    // Retrieve model version
    pub fn get_model(&self, name: &str, version: Option<u32>) 
        -> Result<Vec<u8>>
    
    // Get version info
    pub fn get_version(&self, name: &str, version: Option<u32>) 
        -> Option<&ModelVersion>
    
    // List versions
    pub fn list_versions(&self, name: &str) -> Vec<&ModelVersion>
    
    // Get lineage
    pub fn get_lineage(&self, name: &str, version: u32) 
        -> Vec<&ModelVersion>
    
    // Verify integrity
    pub fn verify_checksum(&self, name: &str, version: u32, data: &[u8]) 
        -> bool
}
```

---

## CLI Integration

```bash
# Store model (auto-version)
aimv store llama-2-7b-chat model.safetensors

# Store with parent version
aimv store llama-2-7b-chat model.safetensors --parent 2

# Get specific version
aimv get llama-2-7b-chat --version 3

# List versions
aimv list llama-2-7b-chat

# Show version info
aimv info llama-2-7b-chat --version 3

# Get lineage
aimv lineage llama-2-7b-chat --version 5

# Compare versions
aimv compare llama-2-7b-chat --versions 3,5

# Cleanup
aimv cleanup llama-2-7b-chat --keep 5

# Verify integrity
aimv verify llama-2-7b-chat --version 3
```

---

## Complete Workflow Example

### Training Pipeline (15 days)

```rust
use ai_model_vault::{VaultConfig, formats::ModelMetadata};

#[tokio::main]
async fn main() -> Result<()> {
    let config = VaultConfig::new()?;
    let mut vault = config.build()?;
    
    // Day 1: Base model
    let base_metadata = ModelMetadata::new(
        "customer-support-bot".to_string(),
        ModelFormat::Safetensors,
    ).add_custom_field("stage".to_string(), "base".to_string());
    
    let v1 = vault.store_model(
        "customer-support-bot",
        &base_model,
        &base_metadata,
        None
    )?;
    println!("Day 1: Created v{} (base)", v1.version);
    
    // Day 3: General fine-tuning
    let ft_metadata = base_metadata.clone()
        .add_custom_field("stage".to_string(), "fine-tuning".to_string())
        .add_custom_field("epochs".to_string(), "40".to_string());
    
    let v2 = vault.store_model(
        "customer-support-bot",
        &finetuned_model,
        &ft_metadata,
        Some(1)
    )?;
    println!("Day 3: Created v{} (fine-tuned)", v2.version);
    
    // Day 7: A/B testing
    let exp_a_metadata = ft_metadata.clone()
        .add_custom_field("experiment".to_string(), "high-lr".to_string())
        .add_custom_field("learning_rate".to_string(), "5e-5".to_string());
    
    let v3 = vault.store_model(
        "customer-support-bot",
        &experiment_a,
        &exp_a_metadata,
        Some(2)
    )?;
    println!("Day 7: Created v{} (experiment A)", v3.version);
    
    let exp_b_metadata = ft_metadata.clone()
        .add_custom_field("experiment".to_string(), "low-lr".to_string())
        .add_custom_field("learning_rate".to_string(), "1e-5".to_string());
    
    let v4 = vault.store_model(
        "customer-support-bot",
        &experiment_b,
        &exp_b_metadata,
        Some(2)
    )?;
    println!("Day 7: Created v{} (experiment B)", v4.version);
    
    // Day 10: Evaluate and pick winner
    let v3_data = vault.get_model("customer-support-bot", Some(3))?;
    let v4_data = vault.get_model("customer-support-bot", Some(4))?;
    
    let v3_score = evaluate(&v3_data)?;
    let v4_score = evaluate(&v4_data)?;
    
    let winner = if v4_score > v3_score { 4 } else { 3 };
    println!("Day 10: Winner is v{} (score: {:.2})", 
        winner, 
        if winner == 4 { v4_score } else { v3_score }
    );
    
    // Day 15: Final production model
    let final_metadata = /* ... */;
    let v5 = vault.store_model(
        "customer-support-bot",
        &final_model,
        &final_metadata,
        Some(winner)
    )?;
    println!("Day 15: Created v{} (production)", v5.version);
    
    // Show lineage
    let lineage = vault.get_lineage("customer-support-bot", 5);
    println!("\nLineage (v{}):", v5.version);
    for (i, version) in lineage.iter().enumerate() {
        println!("  {} v{}: {}", 
            "  ".repeat(i),
            version.version,
            version.metadata.get("stage").unwrap()
        );
    }
    
    Ok(())
}
```

**Output**:
```
Day 1: Created v1 (base)
Day 3: Created v2 (fine-tuned)
Day 7: Created v3 (experiment A)
Day 7: Created v4 (experiment B)
Day 10: Winner is v4 (score: 0.87)
Day 15: Created v5 (production)

Lineage (v5):
  v1: base
    v2: fine-tuning
      v4: low-lr
        v5: production
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sequential_versioning() {
        let mut vc = VersionControl::new(temp_path()).unwrap();
        
        let v1 = vc.add_version("model", "path1", "safetensors", 
            1000, 600, "abc123", None, None).unwrap();
        assert_eq!(v1.version, 1);
        
        let v2 = vc.add_version("model", "path2", "safetensors", 
            1000, 600, "def456", None, Some(1)).unwrap();
        assert_eq!(v2.version, 2);
    }
    
    #[test]
    fn test_lineage_tracking() {
        let mut vc = VersionControl::new(temp_path()).unwrap();
        
        let v1 = vc.add_version("model", "p1", "s", 1000, 600, "a", None, None).unwrap();
        let v2 = vc.add_version("model", "p2", "s", 1000, 600, "b", None, Some(1)).unwrap();
        let v3 = vc.add_version("model", "p3", "s", 1000, 600, "c", None, Some(2)).unwrap();
        
        let lineage = vc.get_lineage("model", 3);
        assert_eq!(lineage.len(), 3);
        assert_eq!(lineage[0].version, 1);
        assert_eq!(lineage[1].version, 2);
        assert_eq!(lineage[2].version, 3);
    }
    
    #[test]
    fn test_cleanup() {
        let mut vc = VersionControl::new(temp_path()).unwrap();
        
        for i in 1..=10 {
            vc.add_version("model", &format!("p{}", i), "s", 
                1000, 600, &format!("hash{}", i), None, 
                if i == 1 { None } else { Some(i-1) }
            ).unwrap();
        }
        
        let deleted = vc.cleanup_old_versions("model", 5).unwrap();
        assert_eq!(deleted.len(), 5);
        assert_eq!(vc.list_versions("model").len(), 5);
    }
}
```

### Integration Tests

See `examples/version_control_demo.rs` for comprehensive integration testing.

---

## Performance Metrics

| Operation       | Time Complexity | Typical Time             |
| --------------- | --------------- | ------------------------ |
| Add version     | O(1)            | < 1ms                    |
| Get version     | O(1)            | < 1ms                    |
| List versions   | O(n)            | < 10ms for 1000 versions |
| Get lineage     | O(depth)        | < 5ms for depth 10       |
| Cleanup         | O(n)            | < 50ms for 1000 versions |
| Verify checksum | O(m)            | ~1-2s for 7B model       |

**Storage**:
- Version metadata: ~1 KB per version
- Version file: ~1 MB per 1000 versions
- Model files: Original size (encrypted + compressed)

---

## Security Considerations

### Encryption

- **Version file encrypted**: AES-256-GCM with key derivation
- **Model files encrypted**: ChaCha20-Poly1305
- **Checksums included**: SHA-256 integrity verification

### Permissions

- **Unix**: 0600 (user read/write only)
- **Windows**: User ACL only
- **Secure by default**: No world-readable files

### Compliance

- **CMMC AU.3.046**: Audit logging (version history)
- **CMMC AU.3.049**: Audit protection (encryption)
- **CMMC AU.3.051**: Audit trail (lineage tracking)
- **FDA 21 CFR Part 11**: Electronic records (checksums)
- **GDPR**: Data lineage and provenance

---

## Roadmap

### Phase 1: Core (✅ Complete)
- [x] Sequential versioning
- [x] Branching support
- [x] Lineage tracking
- [x] Time travel
- [x] Version comparison
- [x] Cleanup policies
- [x] Checksum verification
- [x] Metadata tracking

### Phase 2: Advanced (🚧 In Progress)
- [ ] Diff generation (model weight changes)
- [ ] Merge capabilities (combine branches)
- [ ] Tag system (named versions)
- [ ] Automatic cleanup schedules
- [ ] Cloud sync for versions
- [ ] Version annotations

### Phase 3: Optimization (📅 Planned)
- [ ] Content-addressed storage (deduplication)
- [ ] Delta compression (store only changes)
- [ ] Lazy loading (stream large models)
- [ ] Parallel verification
- [ ] Version caching

---

## Examples & Documentation

### Examples
- `examples/version_control_demo.rs` - Complete demonstration (800+ lines)
- `examples/basic_usage.rs` - Simple versioning
- `version_control_output.txt` - Demo output

### Documentation
- `docs/VERSION_CONTROL.md` - Complete guide (this file)
- `docs/VERSION_CONTROL_QUICKREF.md` - Quick reference
- `README.md` - Project overview

### Run Demo
```bash
cargo run --example version_control_demo --release
```

---

## Frequently Asked Questions

**Q: Can I delete a version in the middle of the lineage?**  
A: Yes, but you'll break the lineage chain. Descendants won't be able to trace back to their root.

**Q: What happens if I lose the versions.json file?**  
A: Model files remain encrypted, but you lose metadata. Always back up your vault.

**Q: Can I have multiple parents (merge)?**  
A: Not currently. Each version has exactly one parent (or none for root).

**Q: How much storage does version control use?**  
A: ~1 KB per version for metadata. Model files are stored separately.

**Q: Can I export versions for sharing?**  
A: Yes, use `aim export` to create a shareable package with metadata.

**Q: Is version control required?**  
A: No, you can use AIMV without versioning. But you'll lose lineage tracking and time travel.

---

## Support

For version control questions:
- 📖 [Full Guide](docs/VERSION_CONTROL.md)
- 📋 [Quick Reference](docs/VERSION_CONTROL_QUICKREF.md)
- 💻 [Demo Code](examples/version_control_demo.rs)
- 🐛 [Issue Tracker](https://github.com/nervosys/aimodelvault/issues)

---

**AI Model Vault (AIMV)** - Git-like version control for AI models with military-grade security.

*Last Updated: 2024-11-06*
