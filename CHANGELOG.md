# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Fuzz testing targets** — 3 `cargo-fuzz` targets in `fuzz/`: `fuzz_crypto_roundtrip` (AES-256-GCM encrypt/decrypt roundtrip), `fuzz_format_detection` (ModelFormat::from_extension with arbitrary input), `fuzz_model_metadata` (ModelMetadata builder with fuzzed strings)
- **Code coverage baseline** — 92.82% line coverage (12,094/13,029 lines) measured with cargo-llvm-cov (full features); 87.35% function coverage; 8 modules at 100% coverage
- **Performance baselines** — updated `docs/PERFORMANCE.md` with measured crypto benchmark results (AES-256-GCM, Argon2id, gzip/LZMA compression), vault benchmark results (store/retrieve, format detection, SHA-256, model card serialization), and per-module coverage table
- **Coverage improvements** — 53 new tests for low-coverage modules: `federation.rs` (VectorClock, delta computation, FederationManager lifecycle), `telemetry.rs` (event serialization, client enable/disable, tracking), `compliance.rs` (serialization, severity variants, checker toggle); total lib tests 447 → 505, full-feature tests 1,667
- **Vault benchmark fix** — fixed TempDir lifetime bug in `vault_bench.rs` (replaced `_` with `_tmp` to prevent premature directory cleanup)
- **Python bindings: VaultBuilder export** — registered `PyVaultBuilder` in the PyO3 module init and added `VaultBuilder` to `__init__.py` exports
- **Python bindings documentation** — new `docs/PYTHON_BINDINGS.md` with complete API reference for all 8 PyO3 classes, installation guide, quick start, and feature matrix
- **Python bindings: parse_format tests** — 25 Rust-side unit tests in `src/python.rs` covering all 23+ format aliases and case-insensitive parsing
- **Python test suite expansion** — added compression roundtrip tests, package init tests, vault property/error tests, and compression level tests

### Changed

- **Python package version** — bumped from 1.1.0 to 1.2.0 in both `pyproject.toml` and `__init__.py`
- **Documentation polish** — updated 17 stale references across 10 files: test count 1,609→1,667, lib tests 447→505, coverage ~90%→92.82%, tarpaulin→cargo-llvm-cov

## [1.2.0]

### Added

- **Domain-specific error types** — introduced `CryptoError`, `StorageError`, and `ConversionError` enums in `src/error.rs` with typed variants and `From` conversions into the top-level `VaultError`. All three types are re-exported from the crate root.
- **REST API endpoints for model cards** — `GET /api/v1/models/{name}/card` generates a model card from vault metadata; `POST /api/v1/models/{name}/card` creates/overwrites a custom model card from JSON
- **REST API endpoint for compliance checks** — `GET /api/v1/compliance` runs FIPS 140-3, CVE, MITRE ATT&CK, and CMMC 2.0 checks and returns results as JSON
- **REST API endpoints for RAG** — `POST /api/v1/rag/search` searches the RAG document store; `POST /api/v1/rag/documents` adds a document with metadata
- **GraphQL routing** — wired existing `async-graphql` schema into the Axum router at `/graphql` (GET for Playground, POST for queries/mutations), gated behind `#[cfg(feature = "graphql")]`

### Changed

- **Removed `async-graphql-axum` dependency** — replaced with a manual bridge handler to avoid axum 0.7 / 0.8 version conflict; the `graphql` feature now only requires `async-graphql`
- **Fixed deprecated `TimeoutLayer::new`** — migrated to `TimeoutLayer::with_status_code(REQUEST_TIMEOUT, ...)` per tower-http 0.6.7+
- **Added `timeout` feature to tower-http** in Cargo.toml (was missing, caused compilation failure with `api` feature)
- **Removed unused `ConnectInfo` import** from `src/api/server.rs`
- **Version bump** — 1.1.0 → 1.2.0

### Changed

- **Real SafeTensors ↔ PyTorch converters** — replaced shim/plan converters with real pure-Rust implementations
  - SafeTensors → PyTorch: generates valid ZIP archives with pickle v2 bytecode and tensor data files
  - PyTorch → SafeTensors: parses ZIP archives, extracts tensor metadata from pickle bytecode, produces SafeTensors binary output
  - Full roundtrip conversion support with dtype mapping (F32↔FloatStorage, F16↔HalfStorage, BF16↔BFloat16Storage, etc.)
- **Telemetry changed to opt-in** — disabled by default for privacy
  - `TelemetryConfig::default()` now sets `enabled: false`
  - Unified environment variable handling: both `AIM_TELEMETRY_ENABLED=false` and `AIM_TELEMETRY_DISABLED=1` are respected in all code paths
  - Updated module documentation to reflect opt-in model
  - CLI `telemetry status` now shows both env var options
- **CI/CD hardening**
  - Added `permissions` and `concurrency` blocks to all GitHub Actions workflows
  - Release workflow now generates SHA-256 checksums for all binary artifacts
  - Release binaries properly renamed (e.g., `aim-linux-amd64`, `aim-darwin-arm64`)
  - Removed automatic crates.io publishing from release workflow
  - Consolidated Docker workflow: removed redundant API image job, added per-variant features
  - Fixed duplicate Alpine target in Docker workflow matrix
  - Updated `dependency-review-action` from v3 to v4
  - Added `--locked` flag to cargo install commands in CI
  - Added cargo cache to coverage job
- **deny.toml**: Rewrote for cargo-deny 0.19 schema — removed deprecated fields (`vulnerability`, `unmaintained`, `yanked`, `notice`, `unlicensed`, `copyleft`, `allow-osi-fsf-free`, `default`, `deny`), added `version = 2` to `[licenses]`, added `CC0-1.0`, `CDLA-Permissive-2.0`, `OpenSSL`, `Zlib`, `MPL-2.0` to license allow list
- **Updated qdrant-client** from 1.7 to 1.13 — migrated to builder-pattern API (`CreateCollectionBuilder`, `UpsertPointsBuilder`, `SearchPointsBuilder`, `DeletePointsBuilder`)
- **Replaced deprecated `serde_yaml`** (0.9) with maintained `serde_yml` (0.0.12) — drop-in replacement across all source and test files
- **Updated `zip` crate** from 0.6 to 4 — migrated `FileOptions` → `SimpleFileOptions` API in conversion.rs and utils.rs
- **Updated `bytes`** 1.10.1 → 1.11.1 (fixes RUSTSEC-2026-0007)
- **Updated `time`** 0.3.44 → 0.3.47 (fixes RUSTSEC-2026-0009)
- **Removed unused lancedb dependency** — v0.4 depends on arrow-arith v51 which is incompatible with Rust 1.93+
- **README overhaul** — updated test counts (331 → 1,580), fixed architecture diagram, added Architecture v2 features (API, GraphQL, federation, blockchain, GPU, streaming, VaultBuilder), fixed broken demo script paths, removed stale "NEW" labels, fixed AIMV_PATH_UPDATE link
- **AGENTS.md** — updated project layout, added `vector-db` feature, added telemetry env vars
- **Removed unused `futures` dependency** — confirmed zero usage in src/, not in any feature gate
- **Consolidated 12 coverage test files** into single `coverage_tests.rs` — reduced test binaries from 27 to 16, preserving all 1,609 tests
- **Expanded Makefile `examples` target** — now runs all 10 examples (was 2)
- **Fixed OpenAPI spec** — aligned `.well-known/openapi.yaml` with actual API routes: corrected model store path (`POST /api/v1/models/{name}`), version download path, version delete endpoint, added undocumented routes (health, audit, metrics, events, openapi.json), removed unimplemented routes (model cards, compliance, RAG, GraphQL)
- **Fixed Helm chart health probes** — corrected probe paths from `/health` to `/api/v1/health`, updated image tag to 1.1.0, added `startupProbe` for slow cold starts
- **Rewrote docs/PROJECT_STRUCTURE.md** — updated entire file to reflect current codebase: added 15+ missing src/ modules (crypto/gpu.rs, streaming.rs, cli/, api/, rag/, model_card.rs, blockchain.rs, federation.rs, telemetry.rs, traits.rs, version_sqlite.rs), updated tests/ from 6 to 14 files, examples/ from 5 to 10, docs/ with all new files, fixed license from MIT to AGPL-3.0, added deploy/, website/, .well-known/ directories
- **Updated reports/TEST_COVERAGE.md** — corrected test count from 119 to 1,609, updated test binary count to 16, added all missing test file entries, expanded coverage matrix with 8 new categories (model cards, CLI, VaultBuilder, blockchain, federation, telemetry, format conversion, RAG)
- **Updated all dependencies** — ran `cargo update` (141 packages updated within semver-compatible ranges), all 1,609 tests pass
- **Fixed format count references** — corrected "22 formats" to "23+ formats" across reports/TEST_COVERAGE.md, reports/COMPREHENSIVE_TEST_REPORT.md, reports/UTILITIES_IMPLEMENTATION_COMPLETE.md
- **Fixed MSRV references** — corrected "Rust 1.70+" to "Rust 1.75+" in docs/PROJECT_SUMMARY.md, updated Dockerfile example in docs/SECURITY_HARDENING.md to `rust:1.85-slim-bookworm`
- **Updated website version** — changed version badge from v1.0.0 to v1.1.0 in Header.tsx and page.tsx
- **Fixed stale test count in MIGRATION.md** — corrected "330+ tests" to "1,609+ tests"
- **Fixed stale test count in ROADMAP.md** — corrected "227 tests" to "1,609 tests"
- **Fixed website test count** — corrected "331+" to "1,609" in homepage stats
- **Fixed DEVELOPMENT.md MSRV** — corrected "Rust 1.70" to "Rust 1.75"
- **Fixed remaining "22+" → "23+" format count references** — docs/EXECUTIVE_SUMMARY.md, docs/api/formats.rst, reports/COMPREHENSIVE_TEST_REPORT.md, reports/PRODUCTION_READY.md, reports/PROJECT_COMPLETE.md, website Python docs
- **Root directory cleanup** — moved 5 Python coverage scripts (`analyze_cov.py`, `analyze_coverage.py`, `parse_coverage.py`, `parse_extra.py`, `parse_uncovered.py`) to `scripts/`, deleted tarpaulin artifacts, added `tarpaulin-report.json`, `tarpaulin_stderr.log`, `.cache/` to `.gitignore`
- **README overhaul (round 2)** — removed duplicate Project Structure section, removed duplicate Documentation section with garbled emoji headings, consolidated documentation table with 7 new entries (Architecture, Providers & Formats, Version Control, Cloud Storage, Model Cards, XDG, Roadmap, Changelog), fixed last `(22+)` → `(23+)` format count, updated architecture tree with new `scripts/` directory
- **Fixed all remaining "22+" → "23+" format count references** — ROADMAP.md, examples/huggingface_demo.rs, docs/EXECUTIVE_SUMMARY.md, docs/TOP_10_FEATURES.md, docs/guide/formats.rst, docs/archived/LAUNCH_READINESS.md, docs/archived/LAUNCH_READY.md, reports/COMPREHENSIVE_TEST_REPORT.md, reports/FEATURES_DEMO.md, reports/PRODUCTION_READY.md, reports/PROJECT_COMPLETE.md, reports/TESTING_COMPLETE.md, reports/UTILITIES_IMPLEMENTATION_COMPLETE.md

### Fixed

- Fixed all clippy warnings (29 warnings → 0)
  - Replaced `field_reassign_with_default` patterns with struct init syntax across src/ and tests/
  - Replaced `vec_init_then_push` with pre-initialized `vec![]` literals
  - Fixed `unused_must_use` on `cache_results()` calls
  - Fixed `unnecessary_get_then_check` in traits.rs
  - Fixed `unwrap_on_ok` / `expect_on_ok` in error.rs test
  - Removed unused imports (`EventSubscriber`, `VersionRepo`, `super`)
  - Fixed constant assertions (`assert!(X > 0)` → `assert_ne!(X, 0)`)
  - Suppressed deprecated `assert_cmd::Command::cargo_bin` warning
- Fixed 26 broken internal links across 8 docs/ files
  - Added `../` prefix for root-level files referenced from docs/ (README.md, LICENSE, SECURITY.md, CONTRIBUTING.md, DEVELOPMENT.md, FORMATS.md)
  - Removed redundant `docs/` prefix for same-directory references (QUICKSTART.md, CLI.md, UTILITIES.md)
  - Fixed reports/ directory references (FEATURES_DEMO.md, PRODUCTION_READY.md)
  - Removed links to non-existent files (COMPLIANCE.md, CRYPTO.md, API.md)
  - Fixed incorrect license references (MIT → AGPL-3.0-or-later)
  - Replaced manual `div_ceil` with standard library method in crypto/streaming.rs
  - Used `keys()` iterator instead of destructuring in conversion.rs
  - Added `#[allow(clippy::too_many_arguments)]` where appropriate

### Added

- **Next.js documentation website** (`website/`)
- **docs/FEATURE_FLAGS.md** — comprehensive documentation of all Cargo feature flags with build recipes
- **docs/PERFORMANCE.md** — benchmark baseline for encryption, hashing, compression, model card serialization
- **docs/GPU_ACCELERATION.md** — user guide for OpenCL GPU-accelerated encryption
- **docs/archived/** — moved stale launch readiness docs out of main docs/
- **ROADMAP: Future Improvements** section — documented error type granularity, API expansion, GraphQL routing as v1.2.0+ items
  - 21 documentation pages covering all features
  - Responsive layout with sidebar navigation and mobile menu
  - Light/dark theme with CSS custom properties
  - Reusable components: CodeBlock, Callout, FeatureCard
  - Static generation — all 25 routes prerendered
- Updated README badges and stats (331 tests, v1.0.0, Rust 1.75+)
- Updated ROADMAP version header to v1.0.0

## [1.0.0] - 2026-02-10

### Changed

- **Version bump to 1.0.0** — first production-stable release
  - Cargo.toml: `0.1.0` → `1.0.0`
  - pyproject.toml: `0.1.0` → `1.0.0`, classifier `Alpha` → `Production/Stable`
  - CLI version: `0.1.0` → `1.0.0`
  - OpenAPI spec: `0.5.0` → `1.0.0`

### Added

- **Multi-stage Dockerfile** with Alpine (default, ~12 MB) and Debian variants
  - Static musl binary via `x86_64-unknown-linux-musl` target
  - Non-root user, tini init, XDG volume mounts
  - Configurable `FEATURES` build arg (e.g., `--build-arg FEATURES=api`)
  - `.dockerignore` for minimal build context
- **Kubernetes Helm chart** (`deploy/helm/ai-model-vault/`)
  - Deployment with hardened security context (non-root, read-only FS, drop all caps)
  - Service (ClusterIP), Secret (auto-generated JWT), ServiceAccount
  - PersistentVolumeClaims for data, config, and cache
  - Optional Ingress with TLS support
  - HorizontalPodAutoscaler
  - Values: image, replicas, API config, persistence, resources, probes, autoscaling
- **Docker CI/CD workflow** (`.github/workflows/docker.yml`)
  - Builds and pushes Alpine, Debian, and API images to GHCR on tag push
  - Docker Buildx with GitHub Actions cache
  - OCI metadata labels via `docker/metadata-action`
- **Comprehensive migration guide** (`docs/MIGRATION.md`)
  - Covers Rust crate, Python package, CLI, REST API, Docker, and Kubernetes
  - Breaking changes summary, data migration notes, environment variables
- **Publication readiness metadata**
  - Cargo.toml: added `readme`, `homepage`, `documentation`, `rust-version` fields
  - pyproject.toml: added `[project.urls]` section (Homepage, Docs, Repo, Issues, Changelog)
  - Keywords trimmed to 5 for crates.io compliance

## [0.5.0] - 2026-02-10

### Added

- **REST API server** (`src/api/`, ~1200 lines, behind `api` feature flag)
  - Axum 0.7 HTTP server with 14 RESTful endpoints
  - JWT authentication (`jsonwebtoken` 9.3) with Bearer token auth
  - Endpoints: health, auth/token, models (list/get/store), versions (list/get/delete), lineage, conversions (list/convert), stats, audit
  - Multipart file upload for model storage
  - Base64-encoded conversion API for format conversion over HTTP
  - CORS support via `tower-http` with `--cors-permissive` flag
  - Request body size limits (default 512 MiB)
  - HTTP request tracing via `tower-http::trace`
- **OpenAPI 3.1 specification** at `/api/v1/openapi.json`
  - Complete API documentation with schemas, parameters, and security definitions
- **Embedded web dashboard** served at `/`
  - Single-page HTML/JS/CSS application (no build step required)
  - Model inventory browser with version drill-down
  - Storage statistics (models, versions, size, files)
  - Audit log viewer, conversion registry browser
  - Passphrase-based login with JWT session management
- **CLI `serve` command** (`aim serve`)
  - Flags: `--host`, `--port`, `--jwt-secret`, `--token-expiry`, `--cors-permissive`, `--no-dashboard`
  - Environment variables: `AIM_HOST`, `AIM_PORT`, `AIM_JWT_SECRET`
- **15 API tests** (3 auth unit + 12 integration via tower `oneshot`)
- Dependencies: axum 0.7, tower 0.5, tower-http 0.6, jsonwebtoken 9.3, utoipa 5, base64 0.22, hyper 1.4

## [0.4.0] - 2026-02-10

### Added

- **Format conversion pipeline** (`src/conversion.rs`, ~1350 lines)
  - `Converter` trait with `convert()`, `validate()`, `name()`, `source_format()`, `target_format()`
  - `ConversionPipeline` with BFS multi-step path finding and `with_builtins()` factory
  - `ConversionOptions`: quantization, opset_version, tolerance, preserve_metadata, extra params
  - `ConversionResult`: output data, conversion path, input/output sizes, optional validation report
  - `ConversionProgress` with step tracking and Display impl
  - `ValidationReport` and `ValidationCheck` structures
- **10 built-in format converters**
  - Pure Rust: SafeTensors↔Raw roundtrip, GGUF header parser, ONNX metadata extractor
  - Shim converters (JSON conversion plans): SafeTensors↔PyTorch, PyTorch→ONNX, ONNX→TensorRT, ONNX→CoreML, SafeTensors→GGUF
- **Magic-bytes validation** for SafeTensors, GGUF, PyTorch (ZIP/pickle), ONNX (protobuf), TFLite
- **CLI commands**
  - `aim convert` with `--opset`, `--validate`, `--plan-only` flags
  - `aim list-conversions` to show all registered converters and multi-step paths
- **53 conversion tests** (22 unit + 31 integration)

## [0.3.0] - 2026-02-10

### Added

- **Native Python bindings via PyO3** (`src/python.rs`, ~640 lines)
  - `Vault`: create, unlock, lock, store_model, get_model, list_models, list_versions, get_lineage, delete_version, get_stats, change_passphrase
  - `VaultConfig`: XDG-compliant configuration with optional custom vault directory
  - `ModelFormat`: 23+ format detection with name/extension properties
  - `ModelMetadata`: builder-style constructor (name, format, description, framework, task, architecture, parameters)
  - `ModelVersion`: read-only version snapshot (version, checkpoint_id, timestamp, format, size, checksum)
  - `ModelCard`: create, set_training_data, add_metric, add_metadata, serialization (JSON/YAML/Markdown), deserialization
  - `sha256_hex()`: FIPS-compliant SHA-256 hex digest
  - `version()`: native library version string
- `python` feature flag in Cargo.toml gating PyO3 dependency
- maturin build backend in `pyproject.toml` (replaced setuptools)
- Native import with graceful fallback in `__init__.py` (`_NATIVE` flag)
- **Streaming API** for large models
  - `Vault.store_model_streamed()`: ingest from any iterable of `bytes` chunks
  - `Vault.get_model_streamed()`: retrieve as `ModelStream` iterator (default 8 MiB chunks)
  - `ModelStream`: Python iterator with `total_size`, `remaining` properties
  - Rust `ModelStream` + `Vault::store_model_streamed()` / `Vault::get_model_chunked()`
- **Sphinx documentation** (`docs/`)
  - API reference: Vault, VaultConfig, ModelFormat, ModelMetadata, ModelVersion, ModelCard, utilities
  - User guides: vault lifecycle, format detection, model cards, version control
  - Quick start and installation guides (uv-based)

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

- **Model Format Support (23+ formats)**
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
- PyPI publication as `aimodelvault`

### Planned for v0.4.0

- Real model format conversion pipeline
- PyTorch ↔ ONNX, SafeTensors ↔ PyTorch, GGUF ↔ SafeTensors

---

[0.2.0]: https://github.com/nervosys/AIModelVault/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/nervosys/AIModelVault/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/nervosys/AIModelVault/releases/tag/v0.1.0
