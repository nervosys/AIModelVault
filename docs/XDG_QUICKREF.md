# XDG Compliance Quick Reference

## What is XDG?

**XDG Base Directory Specification** - A standard for organizing application files on Unix-like systems, now adopted cross-platform.

## Quick Facts

✓ **100% XDG Compliant** - All 9 compliance checks passed  
✓ **Cross-Platform** - Works on Linux, macOS, Windows  
✓ **Configurable** - Override via environment variables  
✓ **Secure** - 0700 permissions on Unix, ACLs on Windows  

## Directory Structure

### Linux
```
~/.config/aimodelvault/           # Configuration
~/.local/share/aimodelvault/      # Models & data
~/.cache/aimodelvault/            # Cache & temp files
```

### macOS
```
~/Library/Application Support/ai.nervosys.aimodelvault/  # Config & data
~/Library/Caches/ai.nervosys.aimodelvault/               # Cache
```

### Windows
```
%APPDATA%\nervosys\aimodelvault\config\     # Configuration
%APPDATA%\nervosys\aimodelvault\data\       # Models & data
%LOCALAPPDATA%\nervosys\aimodelvault\cache\ # Cache
```

## Environment Variables

Override default locations:

```bash
# Linux/macOS
export XDG_CONFIG_HOME=/custom/config
export XDG_DATA_HOME=/custom/data
export XDG_CACHE_HOME=/custom/cache

# Windows (PowerShell)
$env:XDG_CONFIG_HOME = "C:\custom\config"
$env:XDG_DATA_HOME = "C:\custom\data"
$env:XDG_CACHE_HOME = "C:\custom\cache"
```

## What Goes Where

| Directory  | Contains                         | Safe to Delete?      |
| ---------- | -------------------------------- | -------------------- |
| **CONFIG** | config.yaml, preferences         | ❌ No - settings lost |
| **DATA**   | Encrypted models, metadata, logs | ❌ No - models lost   |
| **CACHE**  | Decompressed models, temp files  | ✅ Yes - regenerated  |

## Common Use Cases

### Development Environment
```bash
export XDG_DATA_HOME=$HOME/dev/aimodelvault_dev
```

### Network Storage
```bash
export XDG_DATA_HOME=/mnt/nfs/models
# Config stays local, models on network
```

### Testing
```bash
export XDG_CONFIG_HOME=/tmp/test_config
export XDG_DATA_HOME=/tmp/test_data
export XDG_CACHE_HOME=/tmp/test_cache
```

### Backup Strategy
```bash
# Backup config (small, fast)
tar czf config.tar.gz ~/.config/aimodelvault/

# Backup models (large, important)
tar czf models.tar.gz ~/.local/share/aimodelvault/

# Skip cache (not needed)
```

## See It in Action

```bash
# Run XDG compliance demo
cargo run --example xdg_demo --release

# Shows:
# ✓ Current XDG environment
# ✓ Directory paths (platform-specific)
# ✓ Directory creation & permissions
# ✓ File organization
# ✓ Compliance checklist (9/9 passed)
```

## Benefits

1. **User Isolation** - No conflicts between users
2. **Backup Friendly** - Easy to backup config vs data separately
3. **Portable** - Works across all platforms
4. **Configurable** - Override via env vars
5. **Standards Compliant** - Follows FHS and XDG specs
6. **Secure** - Owner-only permissions (Unix)

## More Information

- 📖 [docs/XDG_COMPLIANCE.md](XDG_COMPLIANCE.md) - Complete guide
- 🔍 [examples/xdg_demo.rs](https://github.com/nervosys/AIModelVault/blob/master/examples/xdg_demo.rs) - Source code
- 📋 [XDG Spec](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) - Official specification
