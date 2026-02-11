# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-02-10

### Added
- **Native Python bindings via PyO3** (`src/python.rs`, ~640 lines)
  - `Vault`: create, unlock, lock, store_model, get_model, list_models, list_versions, get_lineage, delete_version, get_stats, change_passphrase
  - `VaultConfig`: XDG-compliant configuration with optional custom vault directory
  - `ModelFormat`: 22+ format detection with name/extension properties
  - `ModelMetadata`: builder-style constructor (name, format, description, framework, task, architecture, parameters)
  - `ModelVersion`: read-only version snapshot (version, checkpoint_id, timestamp, format, size, checksum)
  - `ModelCard`: create, set_training_data, add_metric, add_metadata, serialization (JSON/YAML/Markdown), deserialization
  - `sha256_hex()`: FIPS-compliant SHA-256 hex digest
  - `version()`: native library version string
- `python` feature flag in Cargo.toml gating PyO3 dependency
- maturin build backend in `pyproject.toml` (replaced setuptools)
- Native import with graceful fallback in `__init__.py` (`_NATIVE` flag)

### Changed
- Python package now uses native Rust FFI instead of CLI subprocess wrappers when built with maturin
- `pyproject.toml`: build system switched from setuptools to maturin ≥1.7

## [0.2.0] - 2026-02-10

### Changed
- **License**: Switched from MIT to AGPL-3.0-or-later with commercial dual-license option + CLA
- **Architecture**: Split `rag.rs` (2,168 lines) into 7 submodules with backward-compatible re-exports
- **Architecture**: Split `main.rs` (2,931 lines) into 87-line dispatcher + `cli/` module tree (11 files)
- **Performance**: `ModelFormat::name()` and `extension()` return `&'static str` (zero allocation)
- **Performance**: `model_card.rs` uses `write!()` instead of `format!()+push_str()`, `String::with_capacity(2048)`

### Added
- `COMMERCIAL_LICENSE.md` for proprietary/commercial licensing inquiries
- `Vault::key_manager()` getter (resolves dead_code suppression)
- `VersionControl::vault_path()` getter (resolves dead_code suppression)
- `ComplianceChecker` gated methods with `enabled_checks` map
- 19 new tests (246 total): `change_passphrase`, `audit` logging, `FormatConverter`, `cleanup_old_versions`, `verify_checksum`, compliance check toggling
- `vault_bench` benchmarks: store/retrieve, format detection, SHA-256, model card ser/de

### Fixed
- Resolved all 5 `#[allow(dead_code)]` annotations in production code
- Removed redundant `CachedResult.query_hash` field; used timestamp in LRU eviction as tiebreaker

### Removed
- 10 temporary artifacts from root (test outputs, status files)
- Moved 23 status/completion files to `reports/`
- Moved 12 guides/demo scripts to `docs/`

## [0.1.1] - 2026-02-07

### Fixed
- **Critical**: Replaced panicking `.expect()` in `Vault::new()` with `match` returning `Result`
- **Critical**: Guarded `validate_sql_identifier()` against empty-string panic
- Deprecated `actions-rs/toolchain@v1` → `dtolnay/rust-toolchain@stable` in CI
- Deprecated `actions/create-release@v1` → `softprops/action-gh-release@v2`
- Fixed binary name `aimv` → `aim` in release.yml
- Made heavyweight Python deps optional in `pyproject.toml` (`[project.optional-dependencies] ml`)

### Added
- 40+ Python tests for ModelFormat, VaultConfig, Vault, FIPSCrypto
- `#[must_use]` annotations on all 15 pure functions
- `///` doc comments on 17+ public types and builder methods
- Warning docstring to `fips.py` documenting PBKDF2 vs Argon2id incompatibility

### Changed
- Synced Python `ModelFormat` enum 1:1 with Rust's 23-variant enum
- Committed `Cargo.lock` for reproducible binary builds
- Updated test count references from 171/119 → 227

## [0.1.0] - 2025-11-03

### Added
- **Core Vault System**
  - FIPS 140-3 compliant encryption using AES-256-GCM
  - Argon2id key derivation function (64MB memory, 3 iterations)
  - XDG Base Directory compliance for cross-platform support
  - Version control system with complete checkpoint history
  - Secure key storage with memory zeroization
  - Comprehensive audit logging for compliance

- **Model Format Support (22+ formats)**
  - PyTorch (.pt, .pth, .bin)
  - TensorFlow (.pb, .keras, .h5)
  - ONNX (.onnx)
  - Safetensors (.safetensors)
  - GGUF (.gguf) - Quantized LLMs
  - TensorRT (.plan)
  - TFLite (.tflite)
  - MLX (.npz) - Apple Silicon
  - Core ML (.mlmodel, .mlpackage)
  - And 13+ more formats
  - Automatic format detection
  - Metadata management

- **Compression**
  - Gzip (fast, moderate compression)
  - LZMA (slow, high compression)
  - Zlib (balanced)
  - Configurable compression levels (Fast/Balanced/Maximum)
  - Compression analysis and recommendations

- **Model Utilities (8 Components)**
  - ModelArchive: TAR/ZIP archiving for model backup
  - CompressionAnalyzer: Compression ratio analysis
  - RetrievalOptimizer: LRU cache for fast model access
  - QuantizationInfo: Track 10 quantization schemes
  - PruningInfo: Pruning metadata and sparsity calculation
  - ModelAnalyzer: Size and parameter analysis
  - ModelExporter: Export with JSON metadata
  - ModelDeduplicator: SHA-256 duplicate detection

- **Cloud Storage Support** ⭐ NEW
  - **StorageBackend trait**: Pluggable storage architecture
  - **AWS S3 backend**: Full S3 support with multipart uploads
  - **Azure Blob Storage backend**: Azure cloud storage integration
  - **Google Cloud Storage backend**: GCS support
  - **Async operations**: Non-blocking cloud uploads/downloads
  - **Multiple authentication methods**: IAM roles, access keys, service accounts
  - **Optional features**: Build only what you need (s3, azure, gcs, cloud)
  - **Complete documentation**: 600+ line cloud storage guide
- CLI interface with full command set
  - Core commands: `init`, `store`, `get`, `list`, `versions`, `lineage`, `delete`, `stats`, `compliance`
  - **Utility commands**: `archive`, `extract`, `analyze`, `deduplicate`, `export`, `cache`
- **Model Utilities Module** with comprehensive AI model operations:
  - **ModelArchive**: TAR/ZIP archiving for multiple models
  - **CompressionAnalyzer**: Compression ratio analysis and format-specific estimates
  - **RetrievalOptimizer**: LRU cache for fast model retrieval
  - **QuantizationInfo**: Quantization metadata tracking (10 schemes: FP32, FP16, INT8, Q4_0, etc.)
  - **PruningInfo**: Pruning information and sparsity calculation
  - **ModelAnalyzer**: Model analysis with human-readable size/parameter formatting
  - **ModelExporter**: Export models with JSON metadata
  - **ModelDeduplicator**: SHA-256 based duplicate detection and similarity scoring
- **RAG & AI Agent Integration** ⭐ NEW
  - Document store with vector embeddings
  - Knowledge base with text chunking
  - Rule engine for business logic
  - Retrieval cache with LRU eviction
  - Model Context Protocol (MCP) tools
  - Database abstraction layer
  - 23 comprehensive RAG tests

- **CLI Interface (15 Commands)**
  - Core: init, unlock, store, get, list, versions, lineage, delete, stats
  - Utilities: archive, extract, analyze, deduplicate, export, cache
  - Compliance: compliance check
  - Interactive help system
  - User-friendly error messages

- **Comprehensive Test Suite (148 tests)**
  - 37 library unit tests
  - 22 configuration and error tests
  - 14 cryptography tests
  - 15 format detection tests
  - 8 integration tests
  - 38 utilities tests
  - 23 RAG tests
  - 100% passing rate

- **Example Programs (4 demos)**
  - `basic_usage.rs`: Core vault operations
  - `security_demo.rs`: Security features
  - `utilities_demo.rs`: Model utilities showcase
  - `rag_demo.rs`: RAG pipeline demonstration

- **Complete Documentation (5,000+ lines)**
  - Quick start guide (5-minute tutorial)
  - CLI reference (all 15 commands)
  - Utilities guide (600+ lines)
  - RAG guide (600+ lines)
  - MCP tools guide (500+ lines)
  - Cloud storage guide (600+ lines)
  - HDF5 support guide
  - Security policy
  - Development guide
  - Test coverage report

### Security
- **FIPS 140-3** approved cryptographic algorithms
- **Authenticated encryption** with AES-256-GCM (128-bit auth tags)
- **Secure key derivation** with Argon2id (64MB memory, 3 iterations)
- **SHA-256 integrity** verification for all stored models
- **Memory zeroization** for sensitive data (keys, passphrases)
- **Audit logging** for all security-relevant operations
- **CMMC 2.0 Level 2** compliance (17 controls implemented)
- **MITRE ATT&CK** framework alignment (T1552, T1486, T1078, T1005)
- **CVE scanning** with automated vulnerability checks

### Changed
- Made HDF5 support optional (requires system library installation)
- Separated HDF5 into `hdf5-support` feature flag
- Updated build to work without HDF5 by default
- Optimized compression for large model files

### Fixed
- HDF5 build dependency issue (now truly optional)
- Build failures on systems without HDF5 installed
- Generic array deprecation warnings
- Cross-platform path handling improvements

### Documentation
- Added comprehensive HDF5 support guide
- Created launch readiness checklist
- Updated README with HDF5 installation instructions
- Expanded cloud storage documentation
- Added troubleshooting guides

## Future Releases

### Planned for v0.3.0
- Native Python bindings (PyO3/maturin)
- Direct Python API without subprocess
- PyPI publication as `neuralvault`

### Planned for v0.4.0
- Real model format conversion pipeline
- PyTorch ↔ ONNX, SafeTensors ↔ PyTorch, GGUF ↔ SafeTensors

---

[0.2.0]: https://github.com/nervosys/aimodelvault/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/nervosys/aimodelvault/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/nervosys/aimodelvault/releases/tag/v0.1.0
