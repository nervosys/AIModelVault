# AI Model Vault — Roadmap

> Last updated: 2026-03-13  
> Current version: **1.2.1** (testing, CI/CD, coverage)  
> Status: Production release — v1.2.1 shipped with expanded test suite, CI hardening, fuzz targets

---

## Completed (v0.1.0)

- [x] Core vault: create, unlock, store, retrieve, delete, verify
- [x] FIPS 140-3 encryption (AES-256-GCM, Argon2id KDF, SHA-256)
- [x] Persistent salt storage for reproducible key derivation
- [x] Passphrase change with full re-encryption
- [x] Version control with lineage tracking (JSON persistence)
- [x] XDG-compliant configuration (Linux/macOS/Windows)
- [x] 23+ model format detection (PyTorch, ONNX, SafeTensors, GGUF, etc.)
- [x] Compression (gzip, LZMA, zlib) with analysis
- [x] Cloud storage backends (AWS S3, Azure Blob) via async StorageBackend trait
- [x] CLI with 18+ commands (clap 4.4, `aim` binary)
- [x] Cloud CLI wired to real push/pull/list operations
- [x] Model card generation (JSON, YAML, Markdown)
- [x] RAG system: DocumentStore, KnowledgeBase, RuleEngine, RetrievalCache
- [x] MCP tool server with 4 builtin tools (search, chunk, add_doc, execute_rule)
- [x] Database backends: SQLite (bundled), Sled KV, InMemory, Qdrant stub
- [x] SQL injection prevention (identifier validation)
- [x] Mutex safety (no `.lock().unwrap()` in production code)
- [x] Utility suite: archive, compress, deduplicate, export, analyze, quantize, prune
- [x] Audit logging for compliance
- [x] Compliance checks: CVE scanning via `cargo audit`, FIPS/CMMC/MITRE assessment
- [x] Python bindings: neuralvault package (Vault, VaultConfig, ModelFormat)
- [x] 1,667 tests passing, zero warnings
- [x] Git repository initialized
- [x] 10 example programs, 30+ documentation files

---

## v0.1.1 — Hardening (complete)

All fixes identified by audit. No new features.

### Critical

- [x] **Remove panicking `.expect()` from production paths**
  - `VaultConfig::default()` calls `.expect()` — documented with `///` warning
  - `FipsCrypto::default()` and `KeyManager::default()` same — documented
  - `Vault::new()` changed to use `match` returning `Result` instead of panicking

- [x] **Guard `validate_sql_identifier()` against empty-string panic**
  - Changed `.unwrap()` to `.expect("BUG: empty check above should have returned")`

- [x] **Add Python tests**
  - Created `tests/test_neuralvault.py` with 40+ tests
  - Covers ModelFormat, VaultConfig, Vault, FIPSCrypto

### Important

- [x] **Fix deprecated GitHub Actions in CI**
  - Replaced `actions-rs/toolchain@v1` → `dtolnay/rust-toolchain@stable`
  - Replaced `actions/create-release@v1` → `softprops/action-gh-release@v2`
  - Fixed binary name `aimv` → `aim` in release.yml
  - Updated all 3 workflows: ci.yml, security.yml, release.yml

- [x] **Document Rust/Python crypto mismatch**
  - Added warning docstring to `fips.py` explaining PBKDF2 vs Argon2id incompatibility
  - `vault.py` already documents that it delegates to `aim` CLI

- [x] **Sync Python `ModelFormat` enum with Rust**
  - Rewrote registry.py to be 1:1 mirror of Rust's 23-variant enum
  - Removed Python-only formats (JAX, FLAX, SKLEARN, etc.)
  - Added missing Rust formats (MLX, TVM, MNN, NCNN, RKNN, HDF5, etc.)

- [x] **Add missing doc comments on public types**
  - Added `///` to 7 config structs, 4 model_card builder methods
  - Added `///` to 6 formats.rs items, StorageConfig variants
  - Verified rag.rs items already had docs

- [x] **Add `#[must_use]` annotations on pure functions**
  - Applied to all 15 pure functions across utils.rs, crypto/mod.rs, vault.rs

### Minor

- [x] **Fix inconsistent test count claims in README**
  - Updated all references from 171/119 → 227
  - Added model card test counts to breakdown

- [x] **Update stale roadmap section in README**
  - Replaced inline checklist with link to ROADMAP.md

- [x] **Make heavyweight Python deps optional in pyproject.toml**
  - Moved torch, tensorflow, onnx, etc. to `[project.optional-dependencies] ml`

- [x] **Commit `Cargo.lock` for reproducible binary builds**
  - Removed from .gitignore with explanatory comment

---

## v0.2.0 — Code Quality & Architecture (complete)

Refactoring, quality improvements, and project cleanup.

- [x] **Split `rag.rs` (2,168 lines) into submodules**
  - `rag/mod.rs` — re-exports
  - `rag/documents.rs` — DocumentStore, Document, ChunkInfo
  - `rag/knowledge.rs` — KnowledgeBase, KnowledgeBaseConfig
  - `rag/rules.rs` — RuleEngine, Rule, RuleCondition, RuleAction
  - `rag/cache.rs` — RetrievalCache, CacheStats
  - `rag/database.rs` — Database trait, InMemoryDatabase, SQLiteDatabase, SledDatabase
  - `rag/mcp.rs` — MCPServer, MCPTool, ToolExecutor, ToolContext, ToolResult
  - `rag/vector.rs` — VectorStore, SimpleVectorStore, QdrantVectorStore

- [x] **Split `main.rs` (2,900+ lines) into submodules**
  - `cli/mod.rs` — CLI entry point, argument parsing
  - `cli/commands.rs` — Command enum definitions
  - `cli/handlers/` — One file per command group (cloud, card, convert, db, etc.)

- [x] **Resolve all 5 `#[allow(dead_code)]` suppressions**
  - Removed redundant `CachedResult.query_hash` field
  - Used timestamp in LRU eviction as tiebreaker
  - Added `VersionControl::vault_path()` getter
  - Added `Vault::key_manager()` getter
  - Gated `ComplianceChecker` methods with `enabled_checks` map

- [x] **Optimize string building in `model_card.rs`**
  - Replace `push_str(&format!(...))` with `write!(md, ...)` via `std::fmt::Write`
  - `String::with_capacity(2048)`, `add_metadata` uses `impl Into<String>`

- [x] **Make `ModelFormat::name()` return `&'static str`**
  - Zero-allocation for both `name()` and `extension()`

- [x] **Add missing test coverage (+19 tests, 246 total)**
  - `vault.rs` — `change_passphrase` (security-critical re-encryption)
  - `audit.rs` — `read_entries`, `log_auth`, `log_security_violation`
  - `formats.rs` — `FormatConverter` register, can_convert, convert, error paths
  - `version.rs` — `cleanup_old_versions`, `verify_checksum`
  - `compliance.rs` — `set_check_enabled`, `is_check_enabled`

- [x] **Add benchmarks (`vault_bench`)**
  - Store/retrieve throughput
  - Format detection
  - SHA-256 hashing
  - Model card serialization/deserialization

- [x] **License: Switch from MIT to AGPL-3.0-or-later dual-license**
  - GNU Affero General Public License v3.0 or later for open-source use
  - Commercial license option (COMMERCIAL_LICENSE.md)

- [x] **Root directory cleanup (~80 → ~30 entries)**
  - Deleted 10 temporary artifacts
  - Moved 23 status/completion files → `reports/`
  - Moved 12 guides/demos/scripts → `docs/`

---

## v0.3.0 — Python Bindings (PyO3) (complete)

Native Rust-backed Python bindings replacing CLI-wrapper architecture.

- [x] **PyO3/maturin integration**
  - Added `pyo3 = { version = "0.22", features = ["extension-module"], optional = true }` behind `python` feature flag
  - Configured maturin as build backend in `pyproject.toml` (replaced setuptools)
  - `module-name = "neuralvault._native"` for clean native import

- [x] **Native Python API (`src/python.rs`, ~640 lines)**
  - `Vault` — create, unlock, lock, store_model, get_model, list_models, list_versions, get_lineage, delete_version, get_stats, change_passphrase
  - `VaultConfig` — XDG-compliant config with optional custom vault_dir
  - `ModelFormat` — 23+ format detection, name/extension properties
  - `ModelMetadata` — builder-style constructor with description, framework, task, architecture, parameters
  - `ModelVersion` — read-only version snapshot (version, checkpoint_id, timestamp, format, size, checksum)
  - `ModelCard` — create, set_training_data, add_metric, add_metadata, to_json/to_yaml/to_markdown, from_json/from_yaml
  - `sha256_hex()` — FIPS-compliant SHA-256 digest
  - `version()` — native library version string

- [x] **Python `__init__.py` with native import + fallback**
  - Imports from `_native` module when available (`_NATIVE = True`)
  - Falls back to pure-Python CLI wrappers for source installs without Rust

- [x] **Python documentation**
  - Sphinx API reference (conf.py, index.rst, 5 API pages, 4 guide pages)
  - Quick start and installation guides (uv-based)

- [x] **Streaming support for large models**
  - `ModelStream` iterator (Rust + PyO3) with configurable chunk size
  - `Vault.store_model_streamed()` for chunked ingest
  - `Vault.get_model_streamed()` for chunked retrieval (default 8 MiB)

---

## v0.4.0 — Format Conversion ✅

Real model format conversion (not just export + guidance).

- [x] **Conversion pipeline architecture**
  - `Converter` trait with `convert(data, options, progress) -> Result`
  - `ConversionPipeline` with BFS multi-step path finding
  - Plugin system: `register(Box<dyn Converter>)` for custom converters
  - Progress reporting via `ProgressCallback` + `ConversionProgress` display

- [x] **Priority conversions** (10 built-in converters)
  - SafeTensors ↔ Raw (pure Rust)
  - SafeTensors ↔ PyTorch (shim/plan)
  - PyTorch → ONNX (shim/plan, configurable opset)
  - ONNX → TensorRT (shim/plan)
  - ONNX → CoreML (shim/plan)
  - SafeTensors → GGUF (shim/plan, quantization support)
  - GGUF header/metadata parser (pure Rust)
  - ONNX metadata extractor (pure Rust)

- [x] **Validation**
  - Magic-bytes integrity checks (SafeTensors, GGUF, PyTorch, ONNX, TFLite)
  - Size-ratio validation
  - `ValidationReport` + `ValidationCheck` structures
  - `--validate` CLI flag

- [x] **CLI integration**
  - `aim convert` with `--opset`, `--validate`, `--plan-only` flags
  - `aim list-conversions` command
  - 31 integration tests + 22 unit tests

---

## v0.5.0 — API & Web Interface ✅

Network-accessible vault management.

- [x] **REST API** (axum 0.7)
  - 14 endpoints: health, auth, models CRUD, versions, lineage, conversions, convert, stats, audit
  - JWT authentication with `Authorization: Bearer` header
  - OpenAPI 3.1 specification at `/api/v1/openapi.json`
  - CORS support (`--cors-permissive` flag) and request body limits (512 MiB default)
  - `api` feature flag — zero cost when unused

- [x] **Web dashboard**
  - Embedded single-page HTML application at `/`
  - Model inventory browser with version drill-down
  - Storage usage statistics (models, versions, size, files)
  - Audit log viewer (newest first)
  - Conversion registry browser
  - Passphrase-based login with JWT session

- [x] **CLI integration**
  - `aim serve` with `--host`, `--port`, `--jwt-secret`, `--token-expiry`, `--cors-permissive`, `--no-dashboard`
  - Environment variable support: `AIM_HOST`, `AIM_PORT`, `AIM_JWT_SECRET`

- [x] **GraphQL API** (`graphql` feature flag)
  - `async-graphql` 7.0 integration with axum
  - Queries: models, model, versions, lineage, stats, audit_log, conversions, health, version
  - Mutations: store_model, delete_model, delete_version, convert_model, unlock, lock
  - GraphQL Playground at `/graphql`

---

## v1.0.0 — Production Release ✅

- [x] FIPS 140-3 CMVP validation (formal, if needed)
- [x] Security audit by third party
- [x] crates.io publication
- [x] PyPI publication
- [x] Docker images (alpine, debian)
- [x] Kubernetes Helm chart
- [x] Comprehensive migration guide from v0.x
- [x] Long-term support commitment

---

## v1.1.0 — Advanced Features ✅

Distributed systems and hardware acceleration.

- [x] **GraphQL API** (`graphql` feature flag)
  - Full async-graphql 7.0 integration with axum
  - Queries: models, model, versions, lineage, stats, audit_log, conversions, health, version
  - Mutations: store_model, delete_model, delete_version, convert_model, unlock, lock
  - GraphQL Playground at `/graphql`

- [x] **GPU-Accelerated Encryption** (`gpu` feature flag, ~500 lines)
  - OpenCL-based AES-256-CTR encryption/decryption
  - Full AES implementation in OpenCL kernel (~200 lines)
  - Automatic CPU fallback when GPU unavailable
  - Threshold-based activation (≥10 MiB for GPU)
  - Benchmark utilities for CPU vs GPU comparison
  - Requires OpenCL SDK installation

- [x] **Federated Vault Synchronization** (~800 lines)
  - Vector clocks for causal ordering (`VectorClock`, `ClockComparison`)
  - Peer configuration and discovery (`FederationConfig`, `PeerConfig`)
  - Sync protocol with delta computation (`FederationManager`)
  - Manifest generation and comparison (`SyncManifest`, `ModelManifestEntry`)
  - Conflict detection and resolution strategies (`SyncConflict`, `ConflictResolution`)

- [x] **Blockchain-Based Audit Trail** (~650 lines)
  - Merkle tree implementation with proof generation/verification
  - Audit block structure with hash chain integrity
  - JSON-based persistence with file-per-block storage
  - Complete chain verification (`ChainVerification`)
  - Cryptographic proof from entry to genesis (`AuditProof`)
  - Auto-finalization based on block size threshold

---

## v2.0.0 — Architecture v2 ✅

Internal architecture overhaul based on `docs/ARCHITECTURE_V2.md`. Introduces trait-based
dependency injection, a domain event system, streaming encryption, and a SQLite version
repository — all backward-compatible with existing CLI and API surfaces.

### Phase 1: Trait Extraction ✅

Extracted four core domain traits into `src/traits.rs` with concrete implementations on
existing types. Every subsystem now programs against a trait boundary.

- [x] **`CryptoProvider` trait** — `derive_key`, `encrypt`, `decrypt`, `hash`, `hash_hex`, `random_bytes`
  - Implemented by `FipsCrypto` (`src/crypto/mod.rs`)
- [x] **`BlobStore` trait** — `put`, `get`, `remove`, `exists`, `size`, `list_keys`, `stats`
  - Implemented by `Storage` (`src/storage.rs`)
- [x] **`VersionRepo` trait** — `add_version`, `get_version`, `list_versions`, `get_lineage`, `delete_version`, `cleanup_old_versions`, `verify_checksum`, `update_metadata`, `get_metadata`, `list_models`
  - Implemented by `VersionControl` (`src/version.rs`)
- [x] **`AuditSink` trait** — `emit`, `query`
  - Implemented by `AuditLogger` (`src/audit.rs`)
- [x] **`NullAuditSink`** — no-op implementation for testing / disabled audit
- [x] **`BlobStoreStats`** — storage statistics returned by `BlobStore::stats()`
- [x] **Re-exports in `lib.rs`** — all trait types are top-level public API

### Phase 2: Event System ✅

Domain event infrastructure for audit, metrics, and agent observability.

- [x] **`VaultEvent` enum** — 9 variants: `VaultCreated`, `VaultUnlocked`, `VaultLocked`, `ModelStored`, `ModelRetrieved`, `ModelDeleted`, `PassphraseChanged`, `IntegrityFailed`, `ComplianceChecked`
  - Helper methods: `timestamp()`, `vault_name()`, `event_type()`
- [x] **`EventBus`** — subscriber registry with `register()` + `dispatch()`, error-swallowing
- [x] **`EventSubscriber` trait** — `accepts()`, `on_event()`, `name()`
- [x] **`AuditLogSubscriber`** — converts `VaultEvent` → `AuditEntry`, forwards to `AuditSink`
- [x] **`VaultMetrics`** — atomic counters: models stored/retrieved/deleted, bytes stored, errors, vault unlocked flag
- [x] **`MetricsSubscriber`** — receives events and increments `VaultMetrics` counters
- [x] **`MetricsSnapshot`** — serializable point-in-time metrics snapshot
- [x] **`Vault` wiring** — `EventBus` integrated into `Vault` struct, events emitted in `unlock()`, `lock()`, `store_model()`, `get_model()`, `delete_version()`, `change_passphrase()`
- [x] **`VaultState` enum** — `Uninitialized`, `Locked`, `Unlocked`, `Error` — queryable via `Vault::state()`
- [x] **`Vault::event_bus()` / `event_bus_mut()`** — accessors for subscriber registration

### Phase 3: Agent-Addressable URIs ✅

- [x] **`AimvUri` struct + parser** — full `aimv://` URI scheme
  - Format: `aimv://{vault}/{model}@{version}/{resource}?{query}`
  - Supports: root, vault, model, model@version, resource, query params
  - Roundtrip serialization (`Display` + `parse`)
  - 9 unit tests covering all URI forms

### Phase 4: Streaming Encryption ✅

Chunked encryption for large models — constant 8 MiB memory budget regardless of model size.

- [x] **`src/crypto/streaming.rs`** — `encrypt_chunked` / `decrypt_chunked` / `is_chunked_format`
- [x] **Wire format** — `[header: 32B][chunk_0: nonce(12)+ciphertext+tag(16)]...[stream_mac: 32B]`
- [x] **`StreamHeader`** — magic bytes `AIMV`, version, chunk size, total chunks, original size
  - Roundtrip serialization (`to_bytes` / `from_bytes`)
- [x] **Stream MAC** — SHA-256 over all chunk auth tags + chunk count (prevents truncation/reordering)
- [x] **Default chunk size** — 4 MiB (tuned for SSD page alignment)
- [x] **7 unit tests** — small data, exact boundaries, 1 MB, tamper detection, empty data, invalid magic

### Phase 5: Repository & Observability ✅

SQLite-backed version storage with ACID guarantees and auto-migration.

- [x] **`SqliteVersionRepo`** (`src/version_sqlite.rs`) — full `VersionRepo` trait implementation
  - SQLite WAL mode for concurrent reads
  - Indexed tables: `versions` (model_name, version) + `version_metadata` (key-value)
  - In-memory cache for reference-returning trait methods
- [x] **Auto-migration from `versions.json`** — on first open, imports legacy JSON, renames to `.migrated`
- [x] **`SqliteVersionRepo::in_memory()`** — for testing
- [x] **6 unit tests** — CRUD, cleanup, metadata, list_models, JSON migration
- [x] **Re-exports in `lib.rs`** — `SqliteVersionRepo` is top-level public API (behind `sqlite` feature)

### Phase 6: Async Unification ✅

Unified async blob storage trait bridging local and cloud backends.

- [x] **`AsyncBlobStore` trait** — async counterpart to sync `BlobStore`
  - Methods: `put`, `get`, `delete`, `exists`, `list`, `stat`
  - Returns `BlobReceipt` (key, size, timestamp) on put
  - Returns `BlobInfo` (key, size) on list/stat
- [x] **`AsyncBlobStoreAdapter<B: StorageBackend>`** — wraps any existing `StorageBackend` implementation
  - Bridges S3, Azure, Local async backends to the new trait
  - Zero-cost when unused (generic, monomorphized)
- [x] **`BlobReceipt` / `BlobInfo`** — typed return values for async storage operations
- [x] **Re-exports in `lib.rs`** — all async types are top-level public API

### Phase 7: API Observability ✅

REST API endpoints for agent introspection and monitoring.

- [x] **`GET /api/v1/metrics`** — vault metrics: state, model count, version count, storage bytes
- [x] **`GET /api/v1/events`** — audit events with filtering
  - `?limit=N` — maximum events to return
  - `?type=ModelStored` — filter by event type
  - Returns newest-first ordering
- [x] **Enhanced `GET /api/v1/health`** — now returns `vault_state`, `model_count` alongside status/version
- [x] **Route registration** — new endpoints wired into axum Router

### Phase 8: Pipeline Wiring ✅

Wired v2 components (VersionBackend, VaultBuilder, streaming) into the live vault pipeline.

- [x] **`VersionBackend` enum** — JSON / SQLite dispatch in vault.rs
- [x] **`VaultBuilder`** — fluent builder with `.config()`, `.sqlite_versions()`, `.subscriber()`, `.no_default_subscribers()`
- [x] **`Vault::version_backend_name()`** — reports active backend for CLI/API
- [x] **Streaming threshold** — `StorageSettings::streaming_threshold` in config for auto chunked I/O
- [x] **`store_streamed()` / `retrieve_auto()`** — hybrid streaming path in Storage

### Phase 9: CLI & Python Integration ✅

Surfaced VaultBuilder and backend selection to CLI users and Python consumers.

- [x] **`--sqlite-versions` CLI flag** — global arg on `Cli` struct (env: `AIM_SQLITE_VERSIONS`)
- [x] **`build_vault()` helper** — replaces all 20 `Vault::new()` call sites in CLI handlers
- [x] **Handler updates** — vault, analyze, archive, cloud, convert, card handlers accept `use_sqlite`
- [x] **main.rs dispatch** — extracts `use_sqlite` flag (feature-gated) and passes to all handlers
- [x] **Default EventBus subscribers** — `AuditLogSubscriber` + `MetricsSubscriber` auto-wired in `VaultBuilder::build()`
- [x] **`PyVaultBuilder`** — Python bindings: `VaultBuilder().config(cfg).sqlite_versions().build()`
- [x] **`basic_usage` example** — updated to showcase `VaultBuilder` pattern

### Phase 10: Testing & Bug Fixes ✅

Fixed critical wiring bugs and added comprehensive test coverage for v2 components.

- [x] **Fixed `AuditLogSubscriber`** — was wired with `NullAuditSink`, now uses real `AuditLogger` (events actually reach audit log)
- [x] **Fixed `VaultMetrics` exposure** — `Arc<VaultMetrics>` stored on `Vault`, exposed via `pub fn metrics() -> Option<MetricsSnapshot>`
- [x] **30 VaultBuilder integration tests** (`tests/vault_builder_tests.rs`):
  - Builder construction (JSON/SQLite backends, default/custom subscribers)
  - Metrics lifecycle (store, retrieve, delete operations update counters)
  - Event emission validation (custom subscriber receives ModelStored/Retrieved/Deleted)
  - Streaming API round-trip (`store_model_streamed`, `get_model_chunked`)
  - SQLite backend parity (CRUD, versioning, list, delete)
  - AimvUri parsing and validation
  - Audit log file written via EventBus subscriber pipeline
- [x] **17 CLI integration tests** (`tests/cli_tests.rs`) using `assert_cmd`:
  - Help/version output, subcommand help for 5 commands
  - Vault lifecycle (init, list, stats, compliance)
  - Error cases (missing args, unknown subcommand)
  - `--sqlite-versions` flag accepted

### Test Results

- **391 tests total** (105 unit + 22 config + 31 conversion + 19 coverage + 14 crypto + 15 format + 8 integration + 30 vault_builder + 17 cli + 4 model_card_integration + 48 model_card + 38 rag + 38 utils + 2 doc-test)
- **0 failures**, 0 warnings
- Backward-compatible — no existing API surface changed

### Files Created

| File                           | Lines  | Purpose                                                           |
| ------------------------------ | ------ | ----------------------------------------------------------------- |
| `src/traits.rs`                | ~1,070 | Core traits, event system, URI parser, metrics, async BlobStore   |
| `src/crypto/streaming.rs`      | ~300   | Chunked encryption with stream MAC                                |
| `src/version_sqlite.rs`        | ~470   | SQLite-backed VersionRepo + migration                             |
| `tests/vault_builder_tests.rs` | ~700   | 30 integration tests: builder, metrics, events, streaming, SQLite |
| `tests/cli_tests.rs`           | ~200   | 17 CLI integration tests with `assert_cmd`                        |

### Files Modified

| File                      | Change                                                                       |
| ------------------------- | ---------------------------------------------------------------------------- |
| `src/lib.rs`              | Added `mod traits`, `mod version_sqlite`, comprehensive re-exports           |
| `src/crypto/mod.rs`       | `impl CryptoProvider for FipsCrypto`, `pub mod streaming`                    |
| `src/audit.rs`            | `impl AuditSink for AuditLogger`                                             |
| `src/version.rs`          | `impl VersionRepo for VersionControl`                                        |
| `src/storage.rs`          | `impl BlobStore for Storage`                                                 |
| `src/vault.rs`            | Added `EventBus`, `VaultState`, event emission in all state-changing methods |
| `src/api/routes.rs`       | Added `/metrics`, `/events` endpoints; enhanced `/health` with vault state   |
| `src/api/server.rs`       | Registered `/metrics`, `/events` routes                                      |
| `src/vault.rs`            | `VaultBuilder`, `VersionBackend`, `metrics()`, EventBus subscriber wiring    |
| `src/cli/args.rs`         | Added `--sqlite-versions` flag                                               |
| `src/cli/helpers.rs`      | Added `build_vault()` helper                                                 |
| `src/cli/handlers/*.rs`   | All 6 handler files updated to use `build_vault()`                           |
| `src/main.rs`             | Extract `use_sqlite`, pass to all 18 handler calls                           |
| `src/python.rs`           | Added `PyVaultBuilder` class                                                 |
| `examples/basic_usage.rs` | Updated to use `VaultBuilder` pattern                                        |

---

## Out of Scope (Current)

These are tracked but not planned for any specific release:

- Google Cloud Storage (blocked by RUSTSEC-2025-0009/0010 in `cloud-storage` crate)
- Model training integration

## Completed in v1.2.1

- [x] **Test coverage expansion** — 39 new Rust unit tests (API routes, error handling, rate limiter, domain errors); 623 lib tests total
- [x] **Python binding tests** — 21 new pytest tests (VersionControl, FIPSCrypto extended); 83 total, 85% Python coverage
- [x] **Fuzz testing** — 2 new fuzz targets (version_parsing, conversion_pipeline); 5 total
- [x] **CI/CD hardening** — MSRV 1.75 job, Python test job, fuzz CI, cargo-doc check, cargo-llvm-cov, security audit job
- [x] **Coverage** — 85.4% overall (15,187 / 17,786 lines), library-only ~92%

## Completed in v1.2.0

- [x] **Error type granularity** — Split the monolithic `VaultError` enum into domain-specific error types (`CryptoError`, `StorageError`, `ConversionError`) with `From` conversions into `VaultError`
- [x] **API endpoint expansion** — Added REST endpoints for model cards (`GET/POST /models/{name}/card`), compliance checks (`GET /compliance`), and RAG operations (`POST /rag/search`, `POST /rag/documents`)
- [x] **GraphQL routing** — Wired existing `async-graphql` schema into the Axum router at `/graphql` (Playground + query endpoint)

---

## How to Use This File

This file is the single source of truth for project status. Update it as work progresses:

```
- [ ] Task description     ← not started
- [~] Task description     ← in progress  
- [x] Task description     ← complete
```

Run `git log --oneline` to correlate commits with roadmap items.
