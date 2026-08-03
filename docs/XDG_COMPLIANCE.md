# XDG Base Directory Specification Compliance

AI Model Vault is **fully compliant** with the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html), ensuring proper organization of configuration files, application data, and cache across all platforms.

## Table of Contents

- [What is XDG?](#what-is-xdg)
- [XDG Directory Structure](#xdg-directory-structure)
- [Platform Support](#platform-support)
- [Environment Variables](#environment-variables)
- [Directory Organization](#directory-organization)
- [Security & Permissions](#security--permissions)
- [Compliance Checklist](#compliance-checklist)
- [Examples](#examples)

## What is XDG?

The **XDG Base Directory Specification** defines standard locations for storing:
- **Configuration files** - User preferences and settings
- **Application data** - User-specific data files (models, databases)
- **Cache files** - Non-essential cached data
- **State files** - Logs, history, and runtime state

### Benefits

✓ **User isolation** - No conflicts between different users  
✓ **Backup friendly** - Easy to backup config separately from cache  
✓ **Portable** - Works across Linux, macOS, Windows  
✓ **Configurable** - Override via environment variables  
✓ **Standards compliant** - Follows FHS (Filesystem Hierarchy Standard)  

## XDG Directory Structure

AI Model Vault uses the following XDG-compliant directories:

### Linux

```
~/.config/aimodelvault/           # XDG_CONFIG_HOME
  └── config.yaml                 # Vault configuration

~/.local/share/aimodelvault/      # XDG_DATA_HOME
  ├── vaults/
  │   └── default/
  │       ├── models/             # Encrypted model files
  │       └── metadata/           # Model metadata
  └── logs/
      └── audit.log               # Security audit log

~/.cache/aimodelvault/            # XDG_CACHE_HOME
  ├── decompressed/               # LRU cache for models
  └── temp/                       # Temporary files
```

### macOS

```
~/Library/Application Support/ai.nervosys.aimodelvault/
  ├── config.yaml
  ├── vaults/
  └── logs/

~/Library/Caches/ai.nervosys.aimodelvault/
  └── decompressed/
```

### Windows

```
%APPDATA%\nervosys\aimodelvault\config\
  └── config.yaml

%APPDATA%\nervosys\aimodelvault\data\
  ├── vaults\
  └── logs\

%LOCALAPPDATA%\nervosys\aimodelvault\cache\
  └── decompressed\
```

## Platform Support

AI Model Vault provides **native XDG-style directory organization** on all platforms:

| Platform    | Config Location                  | Data Location                    | Cache Location              |
| ----------- | -------------------------------- | -------------------------------- | --------------------------- |
| **Linux**   | `~/.config/`                     | `~/.local/share/`                | `~/.cache/`                 |
| **macOS**   | `~/Library/Application Support/` | `~/Library/Application Support/` | `~/Library/Caches/`         |
| **Windows** | `%APPDATA%\...\config\`          | `%APPDATA%\...\data\`            | `%LOCALAPPDATA%\...\cache\` |

While Windows doesn't natively support XDG, we maintain the same **separation of concerns**:
- APPDATA for persistent config and data
- LOCALAPPDATA for cache (can be cleared)

## Environment Variables

AI Model Vault respects XDG environment variables:

### XDG_CONFIG_HOME

**Purpose**: Base directory for user-specific configuration files  
**Default**: `~/.config` (Linux), `~/Library/Application Support` (macOS)  
**Contains**: `config.yaml`, user preferences  

```bash
export XDG_CONFIG_HOME=/custom/config
# AI Model Vault will use: /custom/config/aimodelvault/
```

### XDG_DATA_HOME

**Purpose**: Base directory for user-specific data files  
**Default**: `~/.local/share` (Linux), `~/Library/Application Support` (macOS)  
**Contains**: Encrypted models, metadata, version history  

```bash
export XDG_DATA_HOME=/custom/data
# AI Model Vault will use: /custom/data/aimodelvault/
```

### XDG_CACHE_HOME

**Purpose**: Base directory for user-specific non-essential cached data  
**Default**: `~/.cache` (Linux), `~/Library/Caches` (macOS)  
**Contains**: Decompressed models, LRU cache, temporary files  

```bash
export XDG_CACHE_HOME=/custom/cache
# AI Model Vault will use: /custom/cache/aimodelvault/
```

### XDG_STATE_HOME

**Purpose**: Base directory for user-specific state data  
**Default**: `~/.local/state` (Linux)  
**Contains**: Logs, history, runtime state  

```bash
export XDG_STATE_HOME=/custom/state
# Future: May be used for operation logs
```

## Directory Organization

### Configuration Directory (`XDG_CONFIG_HOME`)

```
~/.config/aimodelvault/
├── config.yaml              # Main configuration file
└── preferences.yaml         # User preferences (future)
```

**What goes here:**
- Vault settings (compression, encryption)
- User preferences
- CLI defaults
- No model data (config only!)

### Data Directory (`XDG_DATA_HOME`)

```
~/.local/share/aimodelvault/
├── vaults/
│   ├── default/             # Default vault
│   │   ├── models/
│   │   │   ├── model_id_v1.bin.enc
│   │   │   ├── model_id_v2.bin.enc
│   │   │   └── model_id_v3.bin.enc
│   │   ├── metadata/
│   │   │   ├── model_id_v1.json
│   │   │   ├── model_id_v2.json
│   │   │   └── model_id_v3.json
│   │   └── index.db         # Model index
│   └── production/          # Production vault (example)
└── logs/
    ├── audit.log            # FIPS 140-3 audit trail
    └── operations.log       # Operation history
```

**What goes here:**
- Encrypted model files
- Model metadata (version, lineage, tags)
- Audit logs (required for compliance)
- Database indexes

### Cache Directory (`XDG_CACHE_HOME`)

```
~/.cache/aimodelvault/
├── decompressed/            # LRU cache of decompressed models
│   ├── model_id_v1.bin
│   └── model_id_v2.bin
├── temp/                    # Temporary workspace
│   └── upload_*.tmp
└── lru_cache.db             # Cache metadata
```

**What goes here:**
- Decompressed models (LRU eviction)
- Temporary files during operations
- Can be safely deleted anytime
- Automatically recreated if missing

## Security & Permissions

AI Model Vault enforces **secure permissions** on all directories:

### Unix/Linux/macOS

All directories created with `0700` permissions (owner-only access):

```bash
drwx------ user user ~/.config/aimodelvault/
drwx------ user user ~/.local/share/aimodelvault/
drwx------ user user ~/.cache/aimodelvault/
```

This ensures:
- ✓ Only the owner can read/write/execute
- ✓ Other users cannot access vault data
- ✓ Compliance with security standards (FIPS 140-3, CMMC)

### Windows

Uses platform-appropriate ACLs:
- User has full control
- System has full control
- Other users: No access

## Compliance Checklist

AI Model Vault is **100% XDG compliant**:

- ✅ **Configuration in XDG_CONFIG_HOME** - All config in `~/.config/aimodelvault/`
- ✅ **Data in XDG_DATA_HOME** - All models in `~/.local/share/aimodelvault/`
- ✅ **Cache in XDG_CACHE_HOME** - All cache in `~/.cache/aimodelvault/`
- ✅ **Respects environment variables** - Honors XDG_* overrides
- ✅ **Falls back to defaults** - Works without XDG_* set
- ✅ **No hardcoded paths** - All paths computed at runtime
- ✅ **Cross-platform** - Works on Linux, macOS, Windows
- ✅ **Secure permissions** - Owner-only (0700 on Unix)
- ✅ **Proper separation** - Config ≠ Data ≠ Cache
- ✅ **Standard compliant** - Follows FHS and XDG specs

## Examples

### Basic Usage (Default XDG Paths)

```rust
use ai_model_vault::{VaultConfig, ModelVault};

// Uses XDG directories automatically
let config = VaultConfig::new()?;
let vault = ModelVault::new(&config.dirs.vault_dir)?;

println!("Config: {}", config.dirs.config_dir.display());
println!("Data:   {}", config.dirs.data_dir.display());
println!("Cache:  {}", config.dirs.cache_dir.display());
```

### Custom XDG Paths

```bash
# Override XDG directories
export XDG_CONFIG_HOME=/mnt/config
export XDG_DATA_HOME=/mnt/data
export XDG_CACHE_HOME=/tmp/cache

# AI Model Vault will automatically use custom paths
cargo run --example xdg_demo
```

### Testing with Temporary Directories

```rust
use std::path::PathBuf;
use ai_model_vault::{VaultConfig, DirectoryPaths};

// Create test directories
let temp_dir = std::env::temp_dir().join("test_vault");
let dirs = DirectoryPaths {
    config_dir: temp_dir.join("config"),
    data_dir: temp_dir.join("data"),
    cache_dir: temp_dir.join("cache"),
    vault_dir: temp_dir.join("data/vaults"),
    log_dir: temp_dir.join("data/logs"),
};

let config = VaultConfig::with_dirs(dirs)?;
// Now vault uses temporary directories
```

### Multi-Environment Setup

```bash
# Development
export XDG_DATA_HOME=$HOME/dev/aimodelvault_dev

# Staging
export XDG_DATA_HOME=$HOME/staging/aimodelvault_staging

# Production
export XDG_DATA_HOME=/mnt/production/aimodelvault

# Each environment has isolated vault data
```

### Backup Strategy

XDG separation makes backups easy:

```bash
# Backup configuration only (small, fast)
tar czf config_backup.tar.gz ~/.config/aimodelvault/

# Backup data and models (large, infrequent)
tar czf data_backup.tar.gz ~/.local/share/aimodelvault/

# Skip cache (not needed in backups)
# ~/.cache/aimodelvault/ - can be regenerated
```

### Network Storage

```bash
# Store models on network drive, config locally
export XDG_DATA_HOME=/mnt/nfs/shared
# XDG_CONFIG_HOME defaults to ~/.config/ (local)

# Now models are shared, config is per-user
```

## Run the Demo

See XDG compliance in action:

```bash
# Build and run XDG demo
cargo run --example xdg_demo

# Shows:
# - Current XDG environment variables
# - Computed directory paths
# - Directory creation and permissions
# - File organization
# - Platform-specific behavior
# - Compliance checklist
```

## References

- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [Filesystem Hierarchy Standard (FHS)](https://refspecs.linuxfoundation.org/FHS_3.0/fhs/index.html)
- [directories crate](https://docs.rs/directories/) - Used for XDG support
- [macOS File System Programming Guide](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/)
- [Windows Known Folders](https://docs.microsoft.com/en-us/windows/win32/shell/known-folders)

## FAQ

### Q: Why use XDG on Windows?

**A:** While Windows doesn't natively support XDG, the **principles** are universal:
- Separate config from data
- Separate persistent data from cache
- User-specific directories

We map XDG concepts to Windows equivalents (APPDATA/LOCALAPPDATA).

### Q: Can I change the directories after creation?

**A:** Yes, via environment variables:

```bash
export XDG_DATA_HOME=/new/location
# Restart AI Model Vault - it will use new location
```

Note: Existing models won't be automatically moved. To migrate the whole vault,
use `aim vault-export archive.tar.gz` against the old location and
`aim vault-import archive.tar.gz` against the new one. For individual models,
`aim export <name> <dir>` then `aim store` into the new vault. (There is no
`aim import` — it takes the `vault-` prefix.)

### Q: What happens if I delete the cache directory?

**A:** Safe to delete! The cache is **non-essential**:
- Will be automatically recreated
- Models are still in `XDG_DATA_HOME` (encrypted)
- May be slower temporarily (no LRU cache)

### Q: How do I find my XDG directories?

**A:** Run the demo:

```bash
cargo run --example xdg_demo
```

Or check manually:

```bash
# Linux
echo $XDG_CONFIG_HOME  # Falls back to ~/.config
echo $XDG_DATA_HOME    # Falls back to ~/.local/share
echo $XDG_CACHE_HOME   # Falls back to ~/.cache
```

### Q: Does this work in Docker containers?

**A:** Yes! XDG directories work in containers:

```dockerfile
# Set custom XDG paths for container
ENV XDG_CONFIG_HOME=/app/config
ENV XDG_DATA_HOME=/app/data
ENV XDG_CACHE_HOME=/tmp/cache

# Mount volumes
VOLUME ["/app/config", "/app/data"]
```

### Q: Is this required for FIPS 140-3 compliance?

**A:** While not strictly required, proper directory separation is a **security best practice**:
- Separate config (less sensitive) from data (highly sensitive)
- Clear audit trail in logs directory
- Cache can be on faster storage (temporary data)

This aligns with CMMC AC.3.014 (Separation of Duties).

## Troubleshooting

### Permission Denied Errors

```bash
# Check directory permissions
ls -la ~/.config/aimodelvault/
ls -la ~/.local/share/aimodelvault/

# Should show: drwx------ (0700)
# If not, fix with:
chmod 700 ~/.config/aimodelvault/
chmod 700 ~/.local/share/aimodelvault/
```

### Directory Not Found

```bash
# Ensure directories exist
mkdir -p ~/.config/aimodelvault
mkdir -p ~/.local/share/aimodelvault
mkdir -p ~/.cache/aimodelvault

# Or let AI Model Vault create them:
cargo run --example xdg_demo
```

### Custom XDG Paths Not Working

```bash
# Verify environment variables are set
env | grep XDG_

# Set them in your shell profile (~/.bashrc, ~/.zshrc)
export XDG_CONFIG_HOME=/custom/config
export XDG_DATA_HOME=/custom/data
export XDG_CACHE_HOME=/custom/cache

# Reload
source ~/.bashrc
```

## Next Steps

- Read [ARCHITECTURE.md](ARCHITECTURE.md) for system design
- See [SECURITY.md](https://github.com/nervosys/AIModelVault/blob/master/SECURITY.md) for security practices
- Check [QUICKSTART.md](QUICKSTART.md) for usage guide
- Run `cargo run --example xdg_demo` to see it in action
