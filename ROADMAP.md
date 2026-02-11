# AI Model Vault — Roadmap

> Last updated: 2026-02-10  
> Current version: **0.3.0**  
> Status: Native PyO3 Python bindings complete — AGPL-3.0 dual-license

---

## Completed (v0.1.0)

- [x] Core vault: create, unlock, store, retrieve, delete, verify
- [x] FIPS 140-3 encryption (AES-256-GCM, Argon2id KDF, SHA-256)
- [x] Persistent salt storage for reproducible key derivation
- [x] Passphrase change with full re-encryption
- [x] Version control with lineage tracking (JSON persistence)
- [x] XDG-compliant configuration (Linux/macOS/Windows)
- [x] 22+ model format detection (PyTorch, ONNX, SafeTensors, GGUF, etc.)
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
- [x] 227 tests passing, zero warnings
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
  - `ModelFormat` — 22+ format detection, name/extension properties
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

## v0.4.0 — Format Conversion

Real model format conversion (not just export + guidance).

- [ ] **Conversion pipeline architecture**
  - `FormatConverter` trait with `convert(input, output_format) -> Result`
  - Plugin system for format-specific converters
  - Progress reporting for large models

- [ ] **Priority conversions**
  - PyTorch ↔ ONNX (via `torch.onnx.export` / `onnxruntime`)
  - SafeTensors ↔ PyTorch (via safetensors crate)
  - GGUF ↔ SafeTensors (quantized LLM workflows)
  - ONNX → TensorRT (optimization)
  - ONNX → CoreML (Apple deployment)

- [ ] **Validation**
  - Output model integrity checks
  - Numerical accuracy comparison (tolerance-based)
  - Metadata preservation across formats

---

## v0.5.0 — API & Web Interface

Network-accessible vault management.

- [ ] **REST API**
  - `actix-web` or `axum` HTTP server
  - JWT authentication
  - OpenAPI/Swagger documentation
  - Rate limiting and CORS

- [ ] **Web dashboard**
  - Model inventory browser
  - Version history visualization
  - Storage usage analytics
  - Audit log viewer

- [ ] **GraphQL API** (optional)
  - `async-graphql` integration
  - Subscription support for real-time updates

---

## v1.0.0 — Production Release

- [ ] FIPS 140-3 CMVP validation (formal, if needed)
- [ ] Security audit by third party
- [ ] crates.io publication
- [ ] PyPI publication
- [ ] Docker images (alpine, debian)
- [ ] Kubernetes Helm chart
- [ ] Comprehensive migration guide from v0.x
- [ ] Long-term support commitment

---

## Out of Scope (Current)

These are tracked but not planned for any specific release:

- Google Cloud Storage (blocked by RUSTSEC-2025-0009/0010 in `cloud-storage` crate)
- GPU-accelerated encryption
- Model training integration
- Federated vault synchronization
- Blockchain-based audit trail

---

## How to Use This File

This file is the single source of truth for project status. Update it as work progresses:

```
- [ ] Task description     ← not started
- [~] Task description     ← in progress  
- [x] Task description     ← complete
```

Run `git log --oneline` to correlate commits with roadmap items.
