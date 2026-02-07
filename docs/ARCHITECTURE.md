# aimodelvault Architecture

## System Architecture Diagram

```mermaid
flowchart TB
    subgraph User["User Interface"]
        CLI[CLI Application]
        API[Rust API]
    end
    
    subgraph Vault["Vault Core"]
        VaultLogic[Vault Logic]
        Config[Configuration]
        VersionCtrl[Version Control]
    end
    
    subgraph Security["Security Layer"]
        Crypto[FIPS Crypto Module]
        Audit[Audit Logger]
        Compliance[Compliance Checker]
    end
    
    subgraph Storage["Storage Backend"]
        EncStorage[Encrypted Storage]
        Compression[Compression]
        FileSystem[File System]
    end
    
    subgraph Formats["Model Formats"]
        PyTorch[PyTorch]
        TensorFlow[TensorFlow]
        ONNX[ONNX]
        Safetensors[Safetensors]
        Other[Other Formats]
    end
    
    CLI --> VaultLogic
    API --> VaultLogic
    
    VaultLogic --> Config
    VaultLogic --> VersionCtrl
    VaultLogic --> Crypto
    VaultLogic --> Audit
    
    Crypto --> EncStorage
    EncStorage --> Compression
    Compression --> FileSystem
    
    VaultLogic --> Formats
    Compliance --> Audit
    
    style Crypto fill:#f9f,stroke:#333,stroke-width:2px
    style Audit fill:#ff9,stroke:#333,stroke-width:2px
    style VaultLogic fill:#9ff,stroke:#333,stroke-width:2px
```

## Data Flow - Store Model

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Vault
    participant Crypto
    participant Storage
    participant FS as File System
    
    User->>CLI: aimv store model.pt
    CLI->>Vault: store_model()
    
    Note over Vault: Validate input
    Vault->>Crypto: derive_key(passphrase)
    Crypto-->>Vault: encryption_key
    
    Note over Vault: Read model data
    Vault->>Crypto: compress(data)
    Crypto-->>Vault: compressed_data
    
    Vault->>Crypto: encrypt(compressed_data, key)
    Crypto-->>Vault: encrypted_data
    
    Note over Vault: Generate checksum
    Vault->>Storage: save_encrypted_file()
    Storage->>FS: write file
    
    Note over Vault: Update version control
    Vault->>Storage: update_versions_json()
    Storage->>FS: write metadata
    
    Note over Vault: Log audit event
    Vault->>CLI: return version info
    CLI->>User: ✓ Model stored
```

## Data Flow - Retrieve Model

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Vault
    participant Crypto
    participant Storage
    participant FS as File System
    
    User->>CLI: aimv get model
    CLI->>Vault: get_model()
    
    Note over Vault: Verify authentication
    Vault->>Crypto: derive_key(passphrase)
    Crypto-->>Vault: encryption_key
    
    Vault->>Storage: get_version_metadata()
    Storage->>FS: read versions.json
    FS-->>Storage: metadata
    Storage-->>Vault: version_info
    
    Vault->>Storage: read_encrypted_file()
    Storage->>FS: read file
    FS-->>Storage: encrypted_data
    Storage-->>Vault: encrypted_data
    
    Vault->>Crypto: decrypt(encrypted_data, key)
    Crypto-->>Vault: compressed_data
    
    Vault->>Crypto: decompress(compressed_data)
    Crypto-->>Vault: original_data
    
    Note over Vault: Verify checksum
    Note over Vault: Log audit event
    
    Vault->>CLI: return model_data
    CLI->>User: ✓ Model retrieved
```

## Cryptographic Architecture

```mermaid
flowchart LR
    subgraph Input
        Pass[Passphrase]
        Data[Model Data]
    end
    
    subgraph KDF["Key Derivation"]
        Argon2[Argon2id<br/>64MB, 3 iterations]
        Salt[Random Salt<br/>32 bytes]
    end
    
    subgraph Encryption
        AES[AES-256-GCM]
        Nonce[Random Nonce<br/>12 bytes]
        Tag[Auth Tag<br/>16 bytes]
    end
    
    subgraph Output
        Encrypted[Encrypted Data]
        Metadata[Metadata + Checksum]
    end
    
    Pass --> Argon2
    Salt --> Argon2
    Argon2 --> Key[256-bit Key]
    
    Data --> Compress[Compression]
    Compress --> AES
    Key --> AES
    Nonce --> AES
    
    AES --> Encrypted
    AES --> Tag
    Tag --> Metadata
    
    style Argon2 fill:#f96,stroke:#333,stroke-width:2px
    style AES fill:#96f,stroke:#333,stroke-width:2px
    style Key fill:#ff9,stroke:#333,stroke-width:2px
```

## Compliance Framework

```mermaid
mindmap
  root((aimodelvault<br/>Compliance))
    FIPS_140_3
      AES-256-GCM
      Argon2id
      SHA-256
      Approved RNG
    CMMC_2_0
      Level 2
        AC: Access Control
        AU: Audit
        IA: Authentication
        SC: Crypto Protection
    MITRE_ATTACK
      T1552: Credentials
      T1486: Encryption
      T1078: Accounts
      T1005: Local Data
    CVE
      cargo-audit
      Dependency Scanning
      Vulnerability Tracking
```

## Version Control Structure

```mermaid
gitGraph
    commit id: "v1: Initial model"
    commit id: "v2: Fine-tuned"
    branch experiment
    commit id: "v3: Experimental arch"
    checkout main
    commit id: "v4: Production update"
    checkout experiment
    commit id: "v5: Advanced features"
    checkout main
    merge experiment tag: "v6: Merged"
    commit id: "v7: Optimized"
```

## Directory Structure (XDG Compliant)

```mermaid
graph TD
    Home[~/ Home Directory]
    
    Home --> Config[.config/aimodelvault/]
    Home --> Data[.local/share/aimodelvault/]
    Home --> Cache[.cache/aimodelvault/]
    
    Config --> ConfigYAML[config.yaml]
    
    Data --> Vaults[vaults/]
    Data --> Logs[logs/]
    
    Vaults --> Default[default/]
    Default --> Models[models/]
    Default --> Versions[versions.json]
    
    Models --> Model1[model_name/]
    Model1 --> V1[v1_timestamp.nvault]
    Model1 --> V2[v2_timestamp.nvault]
    
    Logs --> Audit[audit.log]
    
    style ConfigYAML fill:#9f9
    style V1 fill:#f99
    style V2 fill:#f99
    style Audit fill:#ff9
```

## Security Layers

```mermaid
flowchart TD
    subgraph L1["Layer 1: Application"]
        CLI[CLI Interface]
        API[Public API]
    end
    
    subgraph L2["Layer 2: Validation"]
        Input[Input Validation]
        Auth[Authentication]
    end
    
    subgraph L3["Layer 3: Business Logic"]
        Vault[Vault Operations]
        Version[Version Control]
    end
    
    subgraph L4["Layer 4: Security"]
        Encrypt[Encryption]
        Audit[Audit Logging]
    end
    
    subgraph L5["Layer 5: Storage"]
        FS[File System]
        Perms[Permissions 700/600]
    end
    
    CLI --> Input
    API --> Input
    Input --> Auth
    Auth --> Vault
    Vault --> Version
    Version --> Encrypt
    Encrypt --> Audit
    Audit --> FS
    FS --> Perms
    
    style Auth fill:#f96,stroke:#333,stroke-width:2px
    style Encrypt fill:#f96,stroke:#333,stroke-width:2px
    style Audit fill:#ff9,stroke:#333,stroke-width:2px
```

## Module Dependencies

```mermaid
graph LR
    Main[main.rs] --> Vault
    Main --> CLI[clap CLI]
    
    Vault[vault.rs] --> Config[config.rs]
    Vault --> Storage[storage.rs]
    Vault --> Version[version.rs]
    Vault --> Crypto
    Vault --> Audit[audit.rs]
    
    Storage --> Crypto[crypto/mod.rs]
    Storage --> FS[File System]
    
    Crypto --> AES[aes-gcm]
    Crypto --> Argon[argon2]
    Crypto --> Compress[compression.rs]
    
    Compress --> Gzip[flate2]
    Compress --> LZMA[lzma-rs]
    
    Audit --> Log[tracing]
    
    Config --> XDG[directories]
    
    style Vault fill:#9ff,stroke:#333,stroke-width:2px
    style Crypto fill:#f9f,stroke:#333,stroke-width:2px
```
