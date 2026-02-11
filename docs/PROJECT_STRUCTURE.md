# AI Model Vault - Project Structure

This document describes the organization of the AI Model Vault project.

## Root Directory

```
AIModelVault/
├── src/                    # Source code (Rust)
├── tests/                  # Integration and unit tests
├── examples/               # Example programs
├── docs/                   # Documentation
├── reports/                # Development reports and test outputs
├── benches/                # Performance benchmarks
├── .github/                # GitHub Actions workflows
├── .vscode/                # VS Code configuration and instructions
├── target/                 # Build artifacts (gitignored)
│
├── README.md               # Main project documentation
├── EXECUTIVE_SUMMARY.md    # Project status and launch readiness
├── CHANGELOG.md            # Version history and changes
├── LICENSE                 # MIT License
├── SECURITY.md             # Security policy
├── CONTRIBUTING.md         # Contribution guidelines
├── DEVELOPMENT.md          # Developer guide
├── TEST_COVERAGE.md        # Test documentation
├── FORMATS.md              # Supported model formats
│
├── Cargo.toml              # Rust dependencies and configuration
├── Cargo.lock              # Locked dependency versions
├── pyproject.toml          # Python bindings configuration (future)
├── deny.toml               # Dependency security policy
├── Makefile                # Build automation (Unix)
│
├── Demo Scripts
│   ├── demo.ps1            # PowerShell interactive demo
│   ├── demo.sh             # Bash interactive demo
│   └── DEMO_GUIDE.md       # Demo script documentation
│
└── Build Scripts
    ├── build.ps1           # Windows build script
    ├── build.sh            # Unix build script
    ├── validate.ps1        # Windows validation
    ├── validate.sh         # Unix validation
    └── test_cli.ps1        # CLI testing script
```

## Source Code (`src/`)

```
src/
├── lib.rs                  # Library entry point
├── main.rs                 # CLI application entry
├── error.rs                # Error types
├── config.rs               # Configuration management
├── vault.rs                # Core vault logic
├── storage.rs              # Storage abstraction
├── version.rs              # Version control
├── formats.rs              # Format detection
├── utils.rs                # Model utilities
├── rag.rs                  # RAG and rule systems
├── audit.rs                # Audit logging
├── compliance.rs           # Compliance checking
│
├── crypto/                 # Cryptography module
│   ├── mod.rs              # Crypto operations (FIPS)
│   └── compression.rs      # Compression algorithms
│
├── storage/                # Storage backends
│   ├── local.rs            # Local filesystem
│   ├── s3.rs               # AWS S3 (optional)
│   ├── azure.rs            # Azure Blob (optional)
│   └── gcs.rs              # Google Cloud (optional)
│
└── neuralvault/            # Python bindings (future)
    └── ...
```

## Tests (`tests/`)

```
tests/
├── integration_tests.rs    # End-to-end workflow tests
├── crypto_tests.rs         # Cryptography tests
├── format_tests.rs         # Format detection tests
├── config_error_tests.rs   # Configuration tests
├── utils_tests.rs          # Utilities tests
└── rag_tests.rs            # RAG system tests
```

## Examples (`examples/`)

```
examples/
├── basic_usage.rs          # Core vault operations
├── security_demo.rs        # Security features
├── utilities_demo.rs       # Model utilities showcase
├── rag_demo.rs             # RAG pipeline demo
└── mcp_tools_demo.rs       # MCP tools demo
```

## Documentation (`docs/`)

### User Guides
- **QUICKSTART.md** - 5-minute tutorial
- **CLI.md** - Command-line reference
- **UTILITIES.md** - Model utilities guide
- **RAG.md** - RAG systems guide
- **MCP_TOOLS.md** - MCP tools documentation
- **MCP_QUICKREF.md** - MCP quick reference
- **CLOUD_STORAGE.md** - Cloud storage guide
- **HDF5_SUPPORT.md** - HDF5 installation guide

### Technical Docs
- **ARCHITECTURE.md** - System architecture
- **UTILITIES_QUICKREF.md** - Utilities quick reference
- **UTILITIES_SUMMARY.md** - Utilities overview
- **RAG_QUICKREF.md** - RAG quick reference

### Project Info
- **EXAMPLES_GUIDE.md** - Examples walkthrough
- **IMPLEMENTATION.md** - Implementation details
- **PROJECT_SUMMARY.md** - Project overview
- **TOP_10_FEATURES.md** - Feature highlights
- **LAUNCH_READINESS.md** - Launch checklist
- **LAUNCH_READY.md** - Launch approval

## Reports (`reports/`)

### Completion Reports
- **PROJECT_COMPLETE.md** - Overall completion status
- **CLI_UTILITIES_COMPLETE.md** - CLI integration
- **CLOUD_STORAGE_COMPLETE.md** - Cloud storage
- **MCP_IMPLEMENTATION_COMPLETE.md** - MCP tools
- **RAG_IMPLEMENTATION_COMPLETE.md** - RAG system
- **UTILITIES_IMPLEMENTATION_COMPLETE.md** - Utilities
- **TESTING_COMPLETE.md** - Testing validation
- **COMPREHENSIVE_TEST_REPORT.md** - Detailed test results

### Test Outputs
- **basic_usage_output.txt** - Basic example output
- **security_demo_output.txt** - Security demo output
- **utilities_demo_output.txt** - Utilities demo output
- **test_results.txt** - Full test suite results

## Build Artifacts (`target/`)

Generated by Cargo during compilation (not committed to git):
- `debug/` - Development builds
- `release/` - Optimized release builds
- `doc/` - Generated API documentation

## GitHub Actions (`.github/`)

```
.github/workflows/
├── ci.yml              # Continuous integration
├── security.yml        # Security scanning
└── release.yml         # Release automation
```

## VS Code Configuration (`.vscode/`)

```
.vscode/
├── settings.json       # Editor settings
├── tasks.json          # Build tasks
├── launch.json         # Debug configurations
└── instructions.md     # AI assistant instructions
```

## Key Files

| File                       | Purpose                                     |
| -------------------------- | ------------------------------------------- |
| **README.md**              | Main project documentation                  |
| **PRODUCTION_READY.md**    | 🟢 Production readiness status (2025-01-04)  |
| **SECURITY_STATUS.md**     | Current security posture and audit results  |
| **VULNERABILITY_FIXES.md** | Detailed vulnerability resolution report    |
| **EXECUTIVE_SUMMARY.md**   | Project status and metrics                  |
| **PROJECT_STRUCTURE.md**   | This file - project organization guide      |
| **CHANGELOG.md**           | Version history                             |
| **Cargo.toml**             | Rust dependencies and project configuration |
| **LICENSE**                | MIT license text                            |
| **SECURITY.md**            | Security policy and reporting               |
| **CONTRIBUTING.md**        | Contribution guidelines                     |
| **DEVELOPMENT.md**         | Developer setup and workflow                |
| **TEST_COVERAGE.md**       | Testing documentation                       |
| **FORMATS.md**             | Supported AI model formats                  |

## Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Generate documentation
cargo doc --open

# Run example
cargo run --example basic_usage

# Install CLI tool
cargo install --path .
```

## Feature Flags

```bash
# Default features (safetensors, ndarray)
cargo build

# With cloud storage
cargo build --features cloud

# With specific cloud provider
cargo build --features s3
cargo build --features azure
cargo build --features gcs

# With HDF5 support (requires system library)
cargo build --features hdf5-support

# Everything except HDF5
cargo build --features full,cloud
```

## Getting Started

1. **Read** `README.md` for overview
2. **Follow** `docs/QUICKSTART.md` for 5-minute setup
3. **Explore** `examples/` for working code
4. **Reference** `docs/CLI.md` for commands
5. **Contribute** via `CONTRIBUTING.md` guidelines

## Documentation Navigation

- **New Users** → README.md → docs/QUICKSTART.md
- **CLI Users** → docs/CLI.md → docs/UTILITIES.md
- **Developers** → DEVELOPMENT.md → src/ → tests/
- **Security** → SECURITY.md → docs/ARCHITECTURE.md
- **RAG/AI** → docs/RAG.md → docs/MCP_TOOLS.md
- **Cloud** → docs/CLOUD_STORAGE.md → docs/HDF5_SUPPORT.md

---

**Last Updated**: November 4, 2025  
**Project Structure Version**: 1.0
