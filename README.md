# AI Model Vault

> Universal cross-platform encrypted vault for AI/ML model storage, versioning, conversion, and lifecycle management — **agent-first by design**, military-grade security, 23+ formats, 29 production features.

[![Rust](https://img.shields.io/badge/rust-1.89%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE)
[![Security](https://img.shields.io/badge/security-FIPS%20140--3-green.svg)](SECURITY.md)
[![CMMC](https://img.shields.io/badge/CMMC-2.0%20Level%202-green.svg)](docs/SECURITY_HARDENING.md)
[![Tests](https://img.shields.io/badge/tests-2%2C131%2B%20passing-brightgreen.svg)](reports/)
[![Coverage](https://img.shields.io/badge/coverage-85.4%25-brightgreen.svg)](docs/PERFORMANCE.md)
[![Version](https://img.shields.io/badge/version-2.0.0-blue.svg)](CHANGELOG.md)
[![Clippy](https://img.shields.io/badge/clippy-clean-brightgreen.svg)](validate.ps1)
[![Agent-ready](https://img.shields.io/badge/agent--ready-AGENTS.md-blueviolet.svg)](AGENTS.md)

A production-ready, FIPS 140-3 compliant secure vault for storing and managing AI models. Every capability is exposed through **three parallel surfaces — CLI, REST/GraphQL, and MCP** — with a single source of truth (`aim introspect`) and self-describing manifests in [`.well-known/`](.well-known/). Built for autonomous agents, scriptable for CI, friendly for humans.

---

## For AI Agents — Read This First

If you are an LLM agent, IDE assistant, or automation pipeline, **start here** instead of scanning the rest of this README.

### One-line bootstrap

```bash
aim introspect --format json          # entire CLI schema, machine-readable
```

### Discovery surface (all in [.well-known/](.well-known/))

| File                                                 | Purpose                                                           |
| ---------------------------------------------------- | ----------------------------------------------------------------- |
| [`agents.json`](.well-known/agents.json)             | Capability catalog (29 features), taxonomy, interface inventory   |
| [`mcp-manifest.json`](.well-known/mcp-manifest.json) | **86 MCP tools** with full JSON Schema inputs, resources, prompts |
| [`openapi.yaml`](.well-known/openapi.yaml)           | OpenAPI 3.1 — **53 REST endpoints** across 20 tag groups          |
| [`ontology.jsonld`](.well-known/ontology.jsonld)     | JSON-LD ontology — every concept, class, and relationship         |
| [`ai-plugin.json`](.well-known/ai-plugin.json)       | OpenAI-compatible plugin manifest cross-linking the above         |
| [`AGENTS.md`](AGENTS.md)                             | Canonical project context — features, CLI cheat sheet, layout     |

### Canonical agent integration pattern

```bash
# 1. Discover — get every command, flag, type
aim introspect --format jsonld > schema.jsonld

# 2. Speak any surface
aim <subcommand> --format json        # local CLI, JSON out
curl  http://host:8080/api/v1/...     # REST (see openapi.yaml)
# or call MCP tools from mcp-manifest.json over your MCP client
```

### Stability contract for agents

- **JSON output:** every read-style subcommand accepts `--format json`. Output schema versioned alongside the crate.
- **Exit codes:** `0` success · `1` user error · `2` not found · `3` integrity / verification failure · `4` permission denied. Non-zero ⇒ failure, always.
- **Idempotent reads:** `list`, `get`, `search`, `versions`, `lineage`, `stats`, `compliance`, `introspect`, `*/show`, `*/list` are side-effect free.
- **Destructive ops gated:** `delete`, `policy apply`, `gc`, `vault-import` accept `--dry-run` (where applicable) or require an explicit name argument.
- **Self-describing errors:** error JSON includes `code`, `message`, and `hint`; never just a string.
- **URIs:** Vault resources are addressable via the [`aimv://`](docs/UTILITIES.md) scheme — agents can pass `aimv://vault/model@version` between tools.
- **No surprise network:** the CLI never phones home except `aim pull` (explicit), `aim cloud` (explicit), and opt-in telemetry (off by default; honors `DO_NOT_TRACK=1`).

### Three-surface coverage matrix

Every one of the 29 features in [AGENTS.md](AGENTS.md) is reachable from **all three** of: CLI subcommand, REST endpoint, and MCP tool. See the parity table in [agents.json](.well-known/agents.json) for the precise mapping.

---

## Table of Contents

| For Agents                                                       | For Humans                                          | Operations                                     |
| ---------------------------------------------------------------- | --------------------------------------------------- | ---------------------------------------------- |
| [AGENTS.md](AGENTS.md) — canonical context                       | [Quick Start](#quick-start)                         | [Security & Compliance](#security--compliance) |
| [`.well-known/`](.well-known/) — discovery manifests             | [Installation](#installation)                       | [Build & Validate](#build--validate)           |
| [`aim introspect`](#for-ai-agents--read-this-first) — CLI schema | [CLI Reference](docs/CLI.md)                        | [Architecture](#architecture)                  |
| [MCP tools](docs/MCP_TOOLS.md) — 86 tools                        | [Rust API Quickstart](#rust-library-api-quickstart) | [Performance](docs/PERFORMANCE.md)             |
| [OpenAPI 3.1](.well-known/openapi.yaml) — 53 endpoints           | [Demos](#interactive-demos)                         | [Contributing](CONTRIBUTING.md)                |

---

## Why AI Model Vault?

- **Agent-first** — three coequal surfaces (CLI / REST+GraphQL / MCP), one schema, self-describing via `introspect` and `.well-known/`
- **Secure by default** — AES-256-GCM with Argon2id KDF, FIPS 140-3 / CMMC 2.0 L2 / MITRE ATT&CK aligned
- **Format-agnostic** — auto-detect 23+ formats; convert natively between SafeTensors, PyTorch, and raw. Conversions that need a Python toolchain (→ ONNX, → TensorRT, → Core ML, → GGUF) return a runnable plan rather than a silently wrong file
- **Provenance built-in** — SHA-256 checksums, HMAC signatures, blockchain audit trail, license & pickle scanning
- **Operational** — version control, retention policies, garbage collection, multi-vault, profiles, plugins, scheduled backups
- **Integrated** — REST + GraphQL APIs, 86 MCP tools, Python bindings, Ollama / LM Studio interop, HuggingFace / Ollama / URL pull
- **Quality** — 2,088 Rust + 84 Python tests, 0 clippy warnings, fuzz targets, property-based tests, criterion benchmarks

---

## Quick Start

### Install

```bash
# From source
git clone https://github.com/nervosys/AIModelVault.git
cd AIModelVault
cargo build --release --features full
# Binary at target/release/aim (~17 MB, LTO + stripped)
```

```bash
# Or via cargo
cargo install ai-model-vault --features full
```

### 30-second walkthrough

```bash
# 1. Initialize an encrypted vault
aim init

# 2. Store a model (auto-detects format)
aim store llama-7b ./model.safetensors \
  --description "Fine-tuned Llama 7B" --framework pytorch --task text-generation

# 3. Pull from HuggingFace, Ollama, or a URL
aim pull hf:mistralai/Mistral-7B-v0.1 --store --name mistral-7b
aim pull ollama:llama3 --store --name llama3

# 4. Convert SafeTensors → GGUF Q4_K_M for edge deployment
aim convert llama-7b --to-format gguf --quantization q4_k_m --validate

# 5. Sign, scan, and tag
aim sign llama-7b --identity "trainer@company.com"
aim scan llama-7b
aim tag add llama-7b production fine-tuned

# 6. Check security & compliance
aim compliance --verbose

# 7. Browse the vault interactively
aim browse
```

---

## Feature Matrix

All features below are fully implemented, tested, and exposed via both CLI and library API unless noted.

### Storage & Encryption

| Feature                 | CLI           | Notes                                                      |
| ----------------------- | ------------- | ---------------------------------------------------------- |
| AES-256-GCM encryption  | (default)     | Argon2id KDF (64 MB / 3 iterations / 32-byte salt)         |
| Streaming encryption    | (auto)        | Constant 8 MiB memory for multi-GB models                  |
| KMS integration         | `$aimodelvault_PASSPHRASE` | `env://`, `file://`, `azure-kv://`, `vault://`, `aws-sm://` (`--features s3`) |
| 23+ model formats       | (auto-detect) | See [Supported Formats](#supported-model-formats)          |
| Cloud storage           | `aim cloud`   | AWS S3, Azure Blob, GCS                                    |

### Version Control & Lineage

| Feature                 | CLI                     | Notes                                           |
| ----------------------- | ----------------------- | ----------------------------------------------- |
| Sequential versioning   | `aim versions`          | Unique checkpoint IDs per version               |
| Parent lineage          | `aim lineage`           | Parent-child genealogy with branching           |
| Cross-model lineage DAG | `aim lineage-graph`     | Ancestors / descendants of derived models       |
| Instant rollback        | `aim get -v N`          | Time-travel to any historical checkpoint        |
| Retention policies      | `aim policy`            | Max versions / age / minimum keep, with dry-run |
| SQLite version backend  | `AIM_SQLITE_VERSIONS=1` | ACID-compliant, auto-migrates from JSON         |

### Conversion & Quantization

| Feature                  | CLI                | Notes                                                 |
| ------------------------ | ------------------ | ----------------------------------------------------- |
| Format conversion (10×)  | `aim convert`      | Native: PyTorch ↔ SafeTensors, ↔ raw. Plan-only (needs Python): → ONNX/TensorRT/Core ML/GGUF |
| GGUF quantization        | `--quantization …` | Q4_0, Q4_K_M, Q5_K_M, Q8_0, F16, F32                  |
| Quantization profiles    | `aim quantize`     | Per-model method selection, size estimation           |
| ONNX → TensorRT/OpenVINO | `aim convert`      | Edge & GPU deployment paths                           |

### Safety, Signing & Validation

| Feature              | CLI                 | Notes                                                |
| -------------------- | ------------------- | ---------------------------------------------------- |
| HMAC-SHA256 signing  | `aim sign / verify` | Detached `.sig` files for provenance                 |
| Pickle scanner       | `aim scan`          | Detects `REDUCE`, `GLOBAL`, `os.system`, `eval`, …   |
| License scanner      | `aim license-scan`  | Model cards, `config.json`, GGUF meta, LICENSE; SPDX |
| Integrity validation | `aim validate`      | SHA-256 integrity probe per version                  |
| Tensor-level diff    | `aim diff`          | SafeTensors / GGUF / generic binary fallback         |

### Provenance, Audit & Compliance

| Feature            | CLI              | Notes                                             |
| ------------------ | ---------------- | ------------------------------------------------- |
| Audit log          | (automatic)      | Every operation; structured, append-only          |
| Blockchain audit   | —                | Merkle-tree-proofed append-only chain             |
| Model cards        | (via API)        | Google / HuggingFace standard, JSON/YAML/Markdown |
| Compliance check   | `aim compliance` | FIPS 140-3, CMMC 2.0 L2, MITRE ATT&CK             |
| Benchmark metadata | `aim benchmark`  | MMLU, HellaSwag, etc., per model version          |
| Evaluation harness | `aim eval`       | Record, compare, query across suites and metrics  |

### Discovery, Operations & Lifecycle

| Feature               | CLI                      | Notes                                    |
| --------------------- | ------------------------ | ---------------------------------------- |
| Tags & search         | `aim tag` / `aim search` | Labels + key-value annotations           |
| Garbage collection    | `aim gc`                 | Orphan blobs, temp files; `--dry-run`    |
| Vault export / import | `aim vault-export`       | Portable `.tar.gz` bundles               |
| Multi-vault registry  | `aim vaults`             | Register, switch active vault            |
| Backup scheduling     | `aim backup`             | Daily / weekly / monthly / custom        |
| Config profiles       | `aim profile`            | Named overrides, activate / deactivate   |
| Plugin system         | `aim plugin`             | Discover, install JSON-manifest plugins  |
| TUI dashboard         | `aim browse`             | Terminal UI vault browser                |
| Webhooks              | `aim webhook`            | HTTP notifications via `EventSubscriber` |
| Access control (RBAC) | `aim acl`                | Reader / Writer / Admin per principal    |

### Integration & APIs

| Feature              | Surface               | Notes                                              |
| -------------------- | --------------------- | -------------------------------------------------- |
| REST API             | `aim serve`           | Axum + JWT + 41 endpoints, OpenAPI 3.1             |
| GraphQL API          | `aim serve --graphql` | `async-graphql` with playground                    |
| MCP tools            | library               | 4 built-in tools + custom registration             |
| Python bindings      | `pip install` (PyO3)  | `--features python`                                |
| Engine interop       | `aim register`        | Ollama (`ollama create`) + LM Studio               |
| Model download       | `aim pull`            | HuggingFace, Ollama, URLs (+ SHA-256 verification) |
| Federation           | library               | Vector-clock peer sync                             |
| RAG / Knowledge base | `aim database`        | SQLite / Sled / Qdrant backends                    |
| `aimv://` URI scheme | library               | Agent-addressable vault resources                  |
| Agent introspection  | `aim introspect`      | JSON / YAML / JSON-LD CLI schema                   |

> Full machine-readable surface (29 features, all CLI subcommands, ontology, OpenAPI, MCP manifest) is in [`.well-known/`](.well-known/) and [`AGENTS.md`](AGENTS.md).

---

## Supported Model Formats

| Category    | Formats                                                                                                                |
| ----------- | ---------------------------------------------------------------------------------------------------------------------- |
| **LLM**     | SafeTensors, GGUF, PyTorch (.pt/.pth/.bin), TensorRT (.plan), ONNX, MLX (.npz), CoreML (.mlmodel), TorchScript, TFLite |
| **General** | TensorFlow (.pb), Keras (.h5/.keras), OpenVINO (.xml+.bin), TVM (.so), NCNN (.param+.bin), MNN (.mnn), RKNN (.rknn)    |
| **Legacy**  | Caffe (.caffemodel), MXNet (.params), Darknet (.weights)                                                               |
| **Data**    | HDF5 (.h5/.hdf5), Pickle (.pkl), NumPy (.npy/.npz)                                                                     |

**Conversion paths**

```
PyTorch     → SafeTensors, ONNX, TorchScript, CoreML, MLX
SafeTensors → GGUF (q4_0, q4_k_m, q5_k_m, q8_0, f16, f32)
ONNX        → TensorRT, OpenVINO, TFLite
TensorFlow  → TFLite
```

See [docs/PROVIDERS_FORMATS.md](docs/PROVIDERS_FORMATS.md) and [FORMATS.md](FORMATS.md) for full details.

---

## Installation

### From source (recommended for now)

```bash
git clone https://github.com/nervosys/AIModelVault.git
cd AIModelVault

# Default build (Safetensors + ndarray + SQLite)
cargo build --release

# Full feature set
cargo build --release --features full,graphql

# Or use the helpers
./build.sh release           # Linux/macOS
.\build.ps1 release          # Windows
```

The release binary lives at `target/release/aim` (~17 MB, LTO + stripped).

### Cargo feature flags

| Feature        | Description                          |
| -------------- | ------------------------------------ |
| `default`      | SafeTensors + ndarray + SQLite       |
| `full`         | All non-system features              |
| `sqlite`       | SQLite RAG backend                   |
| `kv-store`     | Sled KV backend                      |
| `vector-db`    | Qdrant vector database               |
| `s3`           | AWS S3 cloud storage                 |
| `azure`        | Azure Blob storage                   |
| `cloud`        | All cloud backends                   |
| `api`          | REST API (Axum + JWT)                |
| `graphql`      | GraphQL API                          |
| `python`       | Python bindings (PyO3)               |

### Optional system dependencies

- **HashiCorp Vault / AWS / Azure** — only if you use the corresponding KMS / cloud features.

---

## Rust Library API Quickstart

```rust
use ai_model_vault::{Vault, VaultConfig};
use ai_model_vault::formats::{ModelFormat, ModelMetadata};

let mut vault = Vault::new(None)?;
vault.unlock(b"your-secure-passphrase".to_vec())?;

// Store
let data = std::fs::read("model.safetensors")?;
let metadata = ModelMetadata::new("llama-7b".into(), ModelFormat::Safetensors)
    .with_description("Fine-tuned Llama 7B".into())
    .with_framework("PyTorch".into())
    .with_task("text-generation".into())
    .with_parameters(7_000_000_000);
let version = vault.store_model("llama-7b", data, metadata, None)?;

// Retrieve specific version
let v2 = vault.get_model("llama-7b", Some(2))?;

// List history
for v in vault.list_versions("llama-7b") {
    println!("v{}: {} bytes", v.version, v.original_size);
}
```

### Trait-based dependency injection (advanced)

```rust
use ai_model_vault::{VaultBuilder, AuditLogSubscriber, MetricsSubscriber};

let vault = VaultBuilder::new()
    .config(VaultConfig::default())
    .sqlite_versions(true)
    .subscriber(Box::new(AuditLogSubscriber::default()))
    .subscriber(Box::new(MetricsSubscriber::default()))
    .build()?;
```

`CryptoProvider`, `BlobStore`, `VersionRepo`, `AuditSink`, and `EventSubscriber` are all swappable traits. See [docs/ARCHITECTURE_V2.md](docs/ARCHITECTURE_V2.md).

### MCP / RAG tools

```rust
use ai_model_vault::rag::*;

let mut server = MCPServer::new();
server.register_builtin_tools()?;

let ctx = ToolContext::new()
    .with_knowledge_base("research_kb".into())
    .with_data("user_id".into(), "researcher_1".into());

let result = server.execute_tool("search_documents", &ctx, /* args */ ..)?;
```

Built-in tools: `search_documents`, `add_document`, `chunk_text`, `execute_rule`. Custom tools via `MCPServer::register_tool(tool, executor_fn)`.

---

## Cloud Storage

```bash
# Push, list, pull
aim cloud push  llama-7b --provider s3 --bucket my-models
aim cloud list  --provider s3 --bucket my-models
aim cloud pull  llama-7b --provider s3 --bucket my-models --remote-path llama-7b/safetensors/v1.vault
```

| Provider             | Status                                       |
| -------------------- | -------------------------------------------- |
| AWS S3               | ✅ `--features s3`                            |
| Azure Blob           | ✅ `--features azure`                         |
| Google Cloud Storage | ⚠️ Temporarily disabled (rebuild in progress) |

Models are AES-256-GCM encrypted **before** upload; the cloud only ever sees ciphertext. Credentials come from standard env vars (`AWS_*`, `AZURE_STORAGE_*`, `GOOGLE_APPLICATION_CREDENTIALS`).

Full guide: [docs/CLOUD_STORAGE.md](docs/CLOUD_STORAGE.md) · CLI: [docs/CLOUD_CLI.md](docs/CLOUD_CLI.md).

---

## Security & Compliance

| Layer            | Implementation                                           |
| ---------------- | -------------------------------------------------------- |
| Symmetric crypto | AES-256-GCM (12-byte nonce, 16-byte auth tag)            |
| Key derivation   | Argon2id (64 MB memory, 3 iterations, 32-byte salt)      |
| Integrity        | SHA-256 checksums on every operation                     |
| Memory hygiene   | `zeroize` on key material                                |
| Audit trail      | Append-only with optional Merkle-tree blockchain proofs  |
| Permissions      | `0700` directories / `0600` files (Unix), ACLs (Windows) |
| Signing          | HMAC-SHA256 with detached `.sig`                         |
| Scanning         | Pickle opcode scanner + license/SPDX scanner             |
| Access control   | Per-principal RBAC (Reader / Writer / Admin)             |

### Standards

| Standard         | Status                                        |
| ---------------- | --------------------------------------------- |
| **FIPS 140-3**   | Compliant — AES-256-GCM, SHA-256, Argon2id    |
| **CMMC 2.0 L2**  | 17 controls implemented (AC, AU, IA, SC)      |
| **MITRE ATT&CK** | Mitigates T1552, T1486, T1078, T1005          |
| **OWASP Top 10** | Reviewed; no known issues in first-party code |

### Dependency security

Current status of [`cargo audit`](https://github.com/rustsec/rustsec) on `master`:

- ✅ `rustls-webpki` 0.103.13 in the primary `reqwest` / `hyper-rustls` path (RUSTSEC-2026-0098/0099/0104 patched)
- ⚠️ A handful of advisories remain in **transitive** dependencies (`aws-smithy-http-client` 1.1.12 → old `rustls` 0.21; sled, hdf5, azure SDK unmaintained helpers). All are documented and tracked in [`deny.toml`](deny.toml) with justification; `cargo deny check` passes.

These will clear automatically once AWS SDK upgrades to a Smithy client that uses `hyper-rustls` ≥ 0.27. No first-party code is affected.

Reporting vulnerabilities: **security@nervosys.ai** — do **not** open public issues. See [SECURITY.md](SECURITY.md).

---

## Build & Validate

```bash
# Full validation pipeline (fmt + clippy + build + test + doc)
.\validate.ps1          # Windows
./validate.sh           # Linux/macOS

# Individually
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --features full,graphql
cargo doc --no-deps --all-features
```

Current `master` status:

- ✅ `cargo fmt` clean
- ✅ `cargo clippy` — 0 warnings
- ✅ `cargo build --features full,graphql` — clean
- ✅ `cargo test` — **2,026+ tests passing** across 18 suites
- ✅ `cargo doc` — no warnings
- ✅ `cargo deny check` — pass

### Quality engineering

- 51 cross-module integration tests
- 11 property-based test strategies (proptest)
- 8 fuzz targets (pickle scanner, diff engine, model card parser, …)
- Criterion benchmarks with CI regression tracking (`benches/`)

---

## Interactive Demos

```bash
# Quick 2-minute tour
.\docs\demo.ps1 -Quick           # Windows
./docs/demo.sh   --quick          # Linux/macOS

# Specific feature demos
.\docs\demo.ps1 -HuggingFace
.\docs\demo.ps1 -Security
```

### Cargo examples

```bash
cargo run --example basic_usage             # End-to-end vault flow
cargo run --example version_control_demo    # Versioning, lineage, rollback
cargo run --example providers_formats_demo  # 23+ formats walkthrough
cargo run --example signing_demo            # HMAC signing & verification
cargo run --example scanning_demo           # Pickle safety scanning
cargo run --example diff_demo               # Tensor-level diffing
cargo run --example download_demo           # HF / Ollama / URL pull
cargo run --example interop_demo            # Ollama + LM Studio registration
cargo run --example benchmark_demo          # Benchmark metadata
cargo run --example license_scan_demo       # License detection
cargo run --example model_card_demo         # Model cards (Google/HF)
cargo run --example mcp_tools_demo          # MCP tool usage
cargo run --example rag_demo                # RAG with knowledge base
cargo run --example security_demo           # Compliance + audit
cargo run --example utilities_demo          # Archive / analyze / dedupe
cargo run --example xdg_demo                # XDG paths
cargo run --example api_demo                # REST + GraphQL
cargo run --example huggingface_demo        # HF integration
```

Full demo guide: [docs/DEMO_GUIDE.md](docs/DEMO_GUIDE.md).

### Environment variables

| Variable                                                     | Purpose                            |
| ------------------------------------------------------------ | ---------------------------------- |
| `aimodelvault_PASSPHRASE`                                    | Vault passphrase (CI / automation) — literal value or KMS URI, see [docs/KMS.md](docs/KMS.md) |
| `aimodelvault_VAULT`                                         | Default vault name                 |
| `aimodelvault_CONFIG`                                        | Config directory override          |
| `aimodelvault_HOME`                                          | Relocates all config/data/cache directories under one root |
| `AIM_SQLITE_VERSIONS`                                        | Use SQLite version backend         |
| `AIM_TELEMETRY_DISABLED=1` / `DO_NOT_TRACK=1`                | Disable anonymous telemetry        |
| `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` / `AWS_REGION` | AWS S3 credentials                 |
| `AZURE_STORAGE_ACCOUNT` / `AZURE_STORAGE_SAS_TOKEN`          | Azure: account + SAS. Or Entra ID via `AZURE_TENANT_ID` / `AZURE_CLIENT_ID` / `AZURE_CLIENT_SECRET`. Shared keys (`AZURE_STORAGE_KEY`) are not supported |
| `GOOGLE_APPLICATION_CREDENTIALS` / `GCP_PROJECT`             | GCS credentials                    |

---

## Architecture

```text
src/
├── lib.rs / main.rs           # Library root + CLI entry
├── cli/                       # CLI dispatcher + per-command handlers
├── crypto/                    # AES-256-GCM, Argon2id, streaming
├── rag/                       # 7 RAG submodules (docs, KB, MCP, rules…)
├── vault.rs                   # Core vault logic + VaultBuilder
├── traits.rs                  # CryptoProvider, BlobStore, EventBus, URI parser
├── storage.rs                 # Local + S3/Azure/GCS backends
├── version.rs / version_sqlite.rs  # Version control (JSON + SQLite backends)
├── formats.rs                 # 23+ format detection
├── conversion.rs              # 10 format converters
├── model_card.rs              # Google / HuggingFace model cards
├── api.rs                     # REST (Axum) + GraphQL (async-graphql)
├── blockchain.rs              # Append-only audit chain with Merkle proofs
├── federation.rs              # Vector-clock peer sync
├── compliance.rs / audit.rs   # FIPS / CMMC / MITRE checks + audit log
├── download.rs                # HuggingFace / Ollama / URL pull (+ SHA-256)
├── signing.rs                 # HMAC-SHA256 signing
├── scanning.rs                # Pickle opcode scanner
├── diff.rs                    # Tensor-level diffing
├── interop.rs                 # Ollama + LM Studio
├── benchmark.rs / evaluation.rs  # Benchmark + eval metadata
├── license_scan.rs            # License detection + SPDX
├── tags.rs                    # Tags + key-value annotations
├── vault_bundle.rs            # Export / import bundles
├── gc.rs                      # Garbage collection
├── tui.rs                     # Terminal UI dashboard
├── webhooks.rs                # HTTP notification system
├── access_control.rs          # RBAC
├── kms.rs                     # AWS / Azure / HashiCorp / env
├── validation.rs              # Integrity probes
├── policies.rs                # Retention policies
├── lineage_graph.rs           # Cross-model DAG
├── plugins.rs                 # Plugin discovery + install
├── profiles.rs                # Config profiles
├── quantization.rs            # Quantization profile store
├── scheduler.rs               # Backup scheduling
├── multi_vault.rs             # Multi-vault registry
├── telemetry.rs               # Anonymous opt-in usage
├── config.rs                  # XDG-compliant config
└── python.rs                  # PyO3 bindings
```

Deep dives: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) · [docs/ARCHITECTURE_V2.md](docs/ARCHITECTURE_V2.md).

---

## Documentation

| Topic                                   | Document                                                                                                           |
| --------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| CLI reference                           | [docs/CLI.md](docs/CLI.md)                                                                                         |
| Cloud storage                           | [docs/CLOUD_STORAGE.md](docs/CLOUD_STORAGE.md) · [docs/CLOUD_CLI.md](docs/CLOUD_CLI.md)                            |
| RAG & MCP                               | [docs/RAG.md](docs/RAG.md) · [docs/MCP_TOOLS.md](docs/MCP_TOOLS.md) · [docs/MCP_QUICKREF.md](docs/MCP_QUICKREF.md) |
| Model cards                             | [docs/MODEL_CARDS.md](docs/MODEL_CARDS.md) · [docs/MODEL_CARDS_QUICKREF.md](docs/MODEL_CARDS_QUICKREF.md)          |
| Version control                         | [docs/VERSION_CONTROL.md](docs/VERSION_CONTROL.md)                                                                 |
| Model download                          | [docs/MODEL_DOWNLOAD.md](docs/MODEL_DOWNLOAD.md)                                                                   |
| Model signing                           | [docs/MODEL_SIGNING.md](docs/MODEL_SIGNING.md)                                                                     |
| Safety scanning                         | [docs/SAFETY_SCANNING.md](docs/SAFETY_SCANNING.md)                                                                 |
| Model diffing                           | [docs/MODEL_DIFFING.md](docs/MODEL_DIFFING.md)                                                                     |
| License scanning                        | [docs/LICENSE_SCANNING.md](docs/LICENSE_SCANNING.md)                                                               |
| Engine interop (Ollama, LM Studio)      | [docs/ENGINE_INTEROP.md](docs/ENGINE_INTEROP.md)                                                                   |
| Quantization                            | [docs/QUANTIZATION.md](docs/QUANTIZATION.md)                                                                       |
| Evaluation harness                      | [docs/EVALUATION.md](docs/EVALUATION.md)                                                                           |
| Backup scheduling                       | [docs/BACKUP_SCHEDULING.md](docs/BACKUP_SCHEDULING.md)                                                             |
| Multi-vault                             | [docs/MULTI_VAULT.md](docs/MULTI_VAULT.md)                                                                         |
| Python bindings                         | [docs/PYTHON_BINDINGS.md](docs/PYTHON_BINDINGS.md)                                                                 |
| Security hardening                      | [docs/SECURITY_HARDENING.md](docs/SECURITY_HARDENING.md) · [docs/SECURITY_AUDIT.md](docs/SECURITY_AUDIT.md)        |
| XDG compliance                          | [docs/XDG_COMPLIANCE.md](docs/XDG_COMPLIANCE.md) · [docs/XDG_QUICKREF.md](docs/XDG_QUICKREF.md)                    |
| Architecture                            | [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) · [docs/ARCHITECTURE_V2.md](docs/ARCHITECTURE_V2.md)                  |
| Performance benchmarks                  | [docs/PERFORMANCE.md](docs/PERFORMANCE.md) · [docs/BENCHMARKS.md](docs/BENCHMARKS.md)                              |
| Agent discovery (JSON-LD, MCP, OpenAPI) | [AGENTS.md](AGENTS.md) · [`.well-known/`](.well-known/)                                                            |
| Roadmap                                 | [ROADMAP.md](ROADMAP.md)                                                                                           |
| Changelog                               | [CHANGELOG.md](CHANGELOG.md)                                                                                       |

---

## Contributing

Pull requests welcome. Please:

1. Read [CONTRIBUTING.md](CONTRIBUTING.md).
2. Sign the [CLA](CLA.md) — required for all PRs.
3. Run `./validate.ps1` (or `./validate.sh`) before submitting. PRs must pass fmt, clippy, tests, and docs.

---

## License

Dual-licensed:

- **AGPL-3.0-or-later** — free for open-source use. Any modified version or network-facing service must release its source under the AGPL. See [LICENSE](LICENSE).
- **Commercial License** — for proprietary, SaaS, or closed-source use without AGPL obligations. See [COMMERCIAL_LICENSE.md](COMMERCIAL_LICENSE.md) or email **licensing@nervosys.ai**.

---

## Support

- 📖 [Documentation site](https://aimodelvault.nervosys.ai) · [Local website/](website/)
- 💬 [GitHub Discussions](https://github.com/nervosys/AIModelVault/discussions)
- 🐛 [Issue tracker](https://github.com/nervosys/AIModelVault/issues)
- 📧 General: dev@nervosys.ai · Security: security@nervosys.ai · Licensing: licensing@nervosys.ai

---

**Built with 🦀 Rust for maximum security, performance, and reliability.**
