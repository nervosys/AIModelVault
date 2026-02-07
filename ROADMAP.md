# AI Model Vault — Roadmap

> Last updated: 2026-02-07  
> Current version: **0.1.0**  
> Status: Initial release — functional, tested, not yet hardened for production

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

## v0.1.1 — Hardening (next)

Priority fixes identified by audit. No new features.

### Critical

- [ ] **Remove panicking `.expect()` from production paths**
  - `VaultConfig::default()` calls `.expect()` — panics if home dir unavailable
  - `FipsCrypto::default()` and `KeyManager::default()` same pattern
  - `Vault::new()` uses `config.unwrap_or_else(|| VaultConfig::new().expect(...))` 
  - Fix: return `Result` or use fallback values instead of panicking

- [ ] **Guard `validate_sql_identifier()` against empty-string panic**
  - `name.chars().next().unwrap()` is after an empty check, but fragile
  - Fix: use `.unwrap_or_default()` or restructure to be panic-proof

- [ ] **Add Python tests**
  - Zero test coverage for `src/neuralvault/` despite pytest in pyproject.toml
  - Write tests for Vault, VaultConfig, ModelFormat

### Important

- [ ] **Fix deprecated GitHub Actions in CI**
  - Replace `actions-rs/toolchain@v1` → `dtolnay/rust-toolchain@stable`
  - Replace `actions/create-release@v1` → `softprops/action-gh-release`
  - Affects: `.github/workflows/ci.yml`, `security.yml`, `release.yml`

- [ ] **Document Rust/Python crypto mismatch**
  - Python `FIPSCrypto` uses PBKDF2; Rust uses Argon2id — incompatible
  - Either unify or document clearly that the Python package delegates to `aim` CLI

- [ ] **Sync Python `ModelFormat` enum with Rust**
  - Python has formats Rust doesn't (JAX, FLAX, SKLEARN, etc.)
  - Rust has formats Python doesn't (MLX, TVM, MNN, HDF5, etc.)
  - Reconcile to single source of truth

- [ ] **Add missing doc comments on public types**
  - `VaultStats`, `StorageStats`, `StorageConfig` variants
  - `CompressionReport`, `CacheStats`, `ModelAnalysis`, `QuantizationSavings`
  - Builder methods on `ModelMetadata` and `ModelCard`

- [ ] **Add `#[must_use]` annotations on pure functions**
  - `CompressionAnalyzer::compression_ratio()`, `estimate_ratio()`
  - `QuantizationInfo::estimate_size()`, `memory_savings()`
  - `ModelAnalyzer::format_size()`, `format_parameters()`
  - `ModelDeduplicator::calculate_hash()`, `similarity_score()`
  - `FipsCrypto::hash_sha256()`, `hash_sha256_hex()`, `generate_random()`
  - `Vault::is_unlocked()`, `list_models()`

### Minor

- [ ] **Fix inconsistent test count claims in README**
  - Badge says 171, project structure says 119, actual count is 227
  - Update all references to actual count

- [ ] **Update stale roadmap section in README**
  - Lists cloud storage, CLI utilities, conversion as "planned" — already done
  - Point to this ROADMAP.md instead

- [ ] **Make heavyweight Python deps optional in pyproject.toml**
  - `torch>=2.0.0` and `tensorflow>=2.13.0` are multi-GB base deps
  - Move to `[project.optional-dependencies]` groups

- [ ] **Commit `Cargo.lock` for reproducible binary builds**
  - Currently gitignored — should be committed for binary crates

---

## v0.2.0 — Code Quality & Architecture

Refactoring and quality improvements.

- [ ] **Split `rag.rs` (2,168 lines) into submodules**
  - `rag/mod.rs` — re-exports
  - `rag/documents.rs` — DocumentStore, Document, ChunkInfo
  - `rag/knowledge.rs` — KnowledgeBase, KnowledgeBaseConfig
  - `rag/rules.rs` — RuleEngine, Rule, RuleCondition, RuleAction
  - `rag/cache.rs` — RetrievalCache, CacheStats
  - `rag/database.rs` — Database trait, InMemoryDatabase, SQLiteDatabase, SledDatabase
  - `rag/mcp.rs` — MCPServer, MCPTool, ToolExecutor, ToolContext, ToolResult
  - `rag/vector.rs` — VectorStore, SimpleVectorStore, QdrantVectorStore

- [ ] **Split `main.rs` (2,900+ lines) into submodules**
  - `cli/mod.rs` — CLI entry point, argument parsing
  - `cli/commands.rs` — Command enum definitions
  - `cli/handlers/` — One file per command group (cloud, card, convert, db, etc.)

- [ ] **Resolve `#[allow(dead_code)]` suppressions**
  - `Vault.key_manager` — integrate into vault operations or remove
  - `VersionControl.vault_path` — wire into path resolution or remove
  - `ComplianceChecker.enabled_checks` — use to conditionally run checks

- [ ] **Optimize string building in `model_card.rs`**
  - Replace `push_str(&format!(...))` with `write!(md, ...)` via `std::fmt::Write`
  - Eliminates intermediate allocations in `to_markdown()`

- [ ] **Make `ModelFormat::name()` return `&'static str`**
  - Currently returns `String` via `.to_string()` on every call
  - Same for `extension()` — allocates unnecessarily

- [ ] **Add missing test coverage**
  - `audit.rs` — zero tests (inline or external)
  - `vault.rs` — external integration tests for error paths
  - `compliance.rs` — external tests for CVE scanning
  - CLI integration tests (exercise `aim` binary end-to-end)
  - Cloud backend tests (mock-based, no real credentials needed)

- [ ] **Add benchmarks**
  - Storage read/write throughput
  - Compression ratio vs. speed comparison
  - Archive creation/extraction
  - Version control operations at scale

---

## v0.3.0 — Python Bindings (PyO3)

Replace CLI-wrapper Python bindings with native FFI.

- [ ] **PyO3/maturin integration**
  - Add `[lib] crate-type = ["cdylib"]` target
  - Expose `Vault`, `FipsCrypto`, `Storage`, `VersionControl` to Python
  - Publish to PyPI as `neuralvault`

- [ ] **Native Python API**
  - `vault.store()` / `vault.retrieve()` without subprocess
  - Direct access to encryption/decryption
  - Streaming support for large models
  - Async support via `pyo3-asyncio`

- [ ] **Python documentation**
  - Sphinx/MkDocs API reference
  - Jupyter notebook tutorials
  - PyPI README with usage examples

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
