# AGENTS.md — AI Agent Discovery Guide

> Machine-readable project context for AI agents, LLM assistants, and automated tools.
> AI Model Vault is **designed agent-first** — every capability is reachable from CLI, REST/GraphQL, and MCP, all derived from a single introspectable schema.

## Bootstrap in three commands

```bash
# 1. Get the full CLI schema (commands, flags, types, examples)
aim introspect --format json

# 2. List the 86 MCP tools (JSON Schema inputs)
cat .well-known/mcp-manifest.json | jq '.tools[] | {name, description}'

# 3. List the 53 REST endpoints
cat .well-known/openapi.yaml | grep -E '^  /api/v1/'
```

That is the minimum surface needed to plan a task. Everything else in this file is reference material.

## Worked examples for agents

Three runnable Rust examples cover the three canonical integration patterns:

| Example                                                                  | Pattern                  | Shows                                                                                    |
| ------------------------------------------------------------------------ | ------------------------ | ---------------------------------------------------------------------------------------- |
| [`examples/agent_bootstrap.rs`](examples/agent_bootstrap.rs)             | Out-of-process via CLI   | Shell out to `aim introspect`, parse the schema, invoke a subcommand, handle the error envelope. |
| [`examples/agent_mcp_workflow.rs`](examples/agent_mcp_workflow.rs)       | In-process via MCP       | Register vault-backed `MCPTool`s, drive them in an agent loop with JSON parameters.       |
| [`examples/agent_pipeline.rs`](examples/agent_pipeline.rs)               | Direct Rust API pipeline | End-to-end: scan → store → tag → search → sign → verify, emitting an audit envelope.    |

Run any of them with `cargo run --example <name>`.

## Stability contract

| Guarantee         | Detail                                                                                                                                  |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| JSON output       | Every read subcommand accepts `--format json`. Schemas evolve with semver; breaking changes bump the major version of `ai-model-vault`. |
| Exit codes        | `0` success · `1` user error · `2` not found · `3` integrity / verification failure · `4` permission denied                             |
| Idempotent reads  | `list`, `get`, `search`, `versions`, `lineage`, `stats`, `compliance`, `introspect`, every `*/show` and `*/list` are side-effect free   |
| Destructive gates | `delete`, `policy apply`, `gc`, `vault-import` either require explicit names or accept `--dry-run`                                      |
| Error envelope    | Errors emit JSON `{ "code": "...", "message": "...", "hint": "..." }` on stderr; never bare strings                                     |
| No surprise I/O   | The CLI never makes network calls except `aim pull`, `aim cloud *`, and opt-in telemetry (off by default; honors `DO_NOT_TRACK=1`)      |
| URI scheme        | `aimv://vault/model@version` resolves through any of the three surfaces                                                                 |

## Project Identity

| Key            | Value                                    |
| -------------- | ---------------------------------------- |
| **Name**       | AI Model Vault                           |
| **Binary**     | `aim`                                    |
| **Crate**      | `ai-model-vault`                         |
| **Version**    | 1.6.0                                    |
| **Language**   | Rust (edition 2021, MSRV 1.89)           |
| **License**    | AGPL-3.0-or-later                        |
| **Repository** | https://github.com/nervosys/AIModelVault |

## What This Project Does

AI Model Vault is an **encrypted AI/ML model management system**. It provides:

1. **Encrypted Storage** — AES-256-GCM encryption with Argon2id key derivation (FIPS 140-3)
2. **Version Control** — Sequential versioning with parent lineage trees and instant rollback
3. **Format Conversion** — Convert between 23+ model formats (SafeTensors, GGUF, ONNX, PyTorch, TensorRT, CoreML, MLX, etc.)
4. **Compliance** — FIPS 140-3, CMMC 2.0 Level 2, MITRE ATT&CK validation
5. **RAG System** — Document store, knowledge base, rule engine with MCP tool integration
6. **Cloud Storage** — Push/pull to AWS S3, Azure Blob, Google Cloud Storage
7. **Model Cards** — Google/HuggingFace standard model documentation
8. **Federation** — Sync vaults across peers with vector clocks
9. **Blockchain Audit** — Append-only audit trail with Merkle proofs
10. **API Server** — REST (Axum) + GraphQL (async-graphql) with JWT auth
11. **Model Download** — Pull models from HuggingFace Hub, Ollama registry, or URLs with SHA-256 verification
12. **Model Signing** — HMAC-SHA256 signatures with detached `.sig` files for provenance
13. **Pickle Scanning** — Detect dangerous opcodes and patterns in PyTorch/pickle files
14. **Model Diffing** — Compare model versions at the tensor level (SafeTensors, GGUF, generic)
15. **Engine Interop** — Register models with Ollama (`ollama create`) and LM Studio
16. **Benchmark Metadata** — Store and query benchmark results per model version
17. **License Scanning** — Detect licenses from model cards, config.json, GGUF metadata, LICENSE files
18. **Model Tags & Search** — Tag models with labels and annotations, search by name/tags/annotations
19. **Vault Export/Import** — Portable tar.gz vault bundles with selective model export
20. **Garbage Collection** — Orphaned blob detection, temp file cleanup, space reclaim
21. **TUI Dashboard** — Terminal UI browser for vault contents
22. **Webhooks** — HTTP notification system with EventSubscriber integration
23. **Access Control** — Role-based ACL (Reader/Writer/Admin) per principal
24. **KMS Integration** — Fetch secrets from env, AWS Secrets Manager, Azure Key Vault, HashiCorp Vault
25. **Model Validation** — Integrity probes with SHA-256 checksums per model version
26. **Retention Policies** — Configurable max versions/age/minimum with dry-run enforcement
27. **Cross-Model Lineage DAG** — Directed acyclic graph tracking model derivation chains
28. **Plugin System** — Discover, install, uninstall plugins with JSON manifests
29. **Config Profiles** — Named configuration profiles with activate/deactivate switching

## Discovery Files

| File                                                             | Purpose                                                       |
| ---------------------------------------------------------------- | ------------------------------------------------------------- |
| [`.well-known/ai-plugin.json`](.well-known/ai-plugin.json)       | OpenAI-compatible plugin manifest                             |
| [`.well-known/ontology.jsonld`](.well-known/ontology.jsonld)     | JSON-LD ontology — all concepts, entities, relationships      |
| [`.well-known/mcp-manifest.json`](.well-known/mcp-manifest.json) | MCP tool definitions with JSON Schema inputs                  |
| [`.well-known/openapi.yaml`](.well-known/openapi.yaml)           | OpenAPI 3.1 specification for REST/GraphQL API                |
| [`.well-known/agents.json`](.well-known/agents.json)             | Agent discovery metadata — interfaces, capabilities, taxonomy |

## CLI Quick Reference

```bash
# Vault lifecycle
aim init [--name NAME]                    # Create encrypted vault
aim store <NAME> <PATH> [-f FORMAT]       # Store model (auto-detects format)
aim get <NAME> <OUTPUT> [-v VERSION]      # Retrieve & decrypt model
aim list                                  # List all models
aim versions <NAME>                       # List versions
aim lineage <NAME> <VERSION>              # Show ancestry tree
aim delete <NAME> <VERSION>               # Delete version
aim stats                                 # Storage statistics
aim compliance [--verbose]                # FIPS/CMMC/MITRE check

# Format conversion
aim convert <MODEL> --to-format <FMT> [--quantization q4_k_m] [--validate]
aim list-conversions                      # Show conversion paths

# Cloud storage
aim cloud push <MODEL> --provider s3 --bucket <BUCKET>
aim cloud pull <MODEL> --provider s3 --bucket <BUCKET> --remote-path <PATH>
aim cloud list --provider s3 --bucket <BUCKET>

# RAG / Database
aim database init --path <P> --db-type sqlite
aim database store --path <P> --input <FILE>
aim database search --path <P> <QUERY>

# Utilities
aim archive <MODELS>... <OUTPUT> [-f tar|zip]
aim extract <ARCHIVE> [-o DIR]
aim analyze <NAME>
aim deduplicate
aim export <NAME> <OUTPUT>

# API server (requires --features api)
aim serve [--port 8080] [--jwt-secret SECRET]

# Agent discovery (machine-readable CLI schema)
aim introspect [--format json|yaml|jsonld] [--compact]

# Model download
aim pull <SOURCE> [-o DIR] [--sha256 HASH] [--token TOKEN] [--store] [--name NAME]

# Model signing & verification
aim sign <NAME> [--version V] [--key KEY] [--identity ID] [--file PATH]
aim verify <NAME> --signature <SIG> [--key KEY] [--file PATH]

# Safety scanning
aim scan [<NAME>] [--file PATH] [--version V] [--format text|json]

# Model diffing
aim diff <LEFT> <RIGHT> [--format text|json]   # LEFT/RIGHT: file path or name@version

# Engine registration
aim register <NAME> --engine <ollama|lm-studio> [--version V] [--alias NAME] [--system-prompt TEXT]

# Benchmark metadata
aim benchmark add <NAME> --version V --benchmark <BENCH> --score <N> --unit <UNIT>
aim benchmark show <NAME> [--version V] [--format text|json]

# License scanning
aim license-scan <PATH> [--format text|json]

# Model tags & search
aim tag add <MODEL> <TAGS>...                # Add tags to a model
aim tag remove <MODEL> <TAGS>...             # Remove tags from a model
aim tag list <MODEL>                          # List tags on a model
aim tag annotate <MODEL> --key <K> --value <V>  # Add key-value annotation
aim search <QUERY> [--tag TAG] [--format text|json]  # Search models

# Vault export/import
aim vault-export <OUTPUT>                     # Export vault as tar.gz bundle
aim vault-import <ARCHIVE> [TARGET]           # Import vault bundle

# Garbage collection
aim gc [--dry-run]                            # Clean orphaned blobs & temp files

# TUI dashboard
aim browse                                    # Browse vault in terminal UI

# Webhooks
aim webhook add --url <URL> [--secret SECRET]  # Add webhook target
aim webhook remove <ID>                       # Remove webhook target
aim webhook list                              # List webhook targets
aim webhook test <ID>                         # Test webhook delivery

# Access control
aim acl grant <PRINCIPAL> --role <ROLE>        # Grant role (reader/writer/admin)
aim acl revoke <PRINCIPAL>                    # Revoke access
aim acl list                                  # List ACL entries
aim acl check <PRINCIPAL> --role <ROLE>        # Check permission

# Model validation
aim validate <NAME> [--version V]             # Validate model integrity

# Retention policies
aim policy set <MODEL> [--max-versions N] [--max-age-days N] [--keep-minimum N]
aim policy remove <MODEL>                     # Remove retention policy
aim policy list                               # List all policies
aim policy apply <MODEL> [--dry-run]          # Apply policy to model
aim policy apply-all [--dry-run]              # Apply all policies

# Cross-model lineage DAG
aim lineage-graph add --child <C> --parents <P>... --kind <KIND>
aim lineage-graph show                        # Display lineage graph
aim lineage-graph ancestors <MODEL>           # Show ancestors
aim lineage-graph descendants <MODEL>         # Show descendants

# Plugin system
aim plugin discover                           # Scan for plugins
aim plugin install <PATH>                     # Install plugin from manifest
aim plugin uninstall <ID>                     # Uninstall plugin
aim plugin list                               # List installed plugins
aim plugin info <ID>                          # Show plugin details

# Config profiles
aim profile create <NAME> [--description TEXT] [--override KEY=VALUE]...
aim profile remove <NAME>                     # Remove profile
aim profile list                              # List all profiles
aim profile activate <NAME>                   # Activate profile
aim profile deactivate                        # Deactivate current profile
aim profile show                              # Show active profile

# Quantization pipeline
aim quantize set <MODEL> --method <METHOD> [--version V] [--bits N]
aim quantize remove <MODEL> [--version V]     # Remove quantization profile
aim quantize list [MODEL]                     # List quantization profiles
aim quantize estimate <MODEL> --method <METHOD>  # Estimate output size

# Evaluation harness
aim eval record <MODEL> --suite <SUITE> --metric <METRIC> --score <N> [--version V]
aim eval list <MODEL> [--version V] [--suite SUITE] [--format text|json]
aim eval compare <MODEL> --versions <V1,V2,...> [--format text|json]
aim eval suites                               # List known evaluation suites

# Backup scheduling
aim backup schedule <VAULT> --interval <daily|weekly|monthly|custom> [--hour H]
aim backup list                               # List backup schedules
aim backup run [VAULT]                        # Run backup now
aim backup history [VAULT] [--format text|json]  # Show backup history

# Multi-vault management
aim vaults register <NAME> <PATH>             # Register a vault
aim vaults unregister <NAME>                  # Unregister a vault
aim vaults list                               # List all registered vaults
aim vaults activate <NAME>                    # Switch active vault
aim vaults active                             # Show active vault
```

## Supported Model Formats (23+)

| Category       | Formats                                                                                                                |
| -------------- | ---------------------------------------------------------------------------------------------------------------------- |
| **LLM**        | SafeTensors, GGUF, PyTorch (.pt/.pth/.bin), TensorRT (.plan), ONNX, MLX (.npz), CoreML (.mlmodel), TorchScript, TFLite |
| **General DL** | TensorFlow (.pb), Keras (.h5/.keras), OpenVINO (.xml+.bin), TVM (.so), NCNN (.param+.bin), MNN (.mnn), RKNN (.rknn)    |
| **Legacy**     | Caffe (.caffemodel), MXNet (.params), Darknet (.weights)                                                               |
| **Data**       | HDF5 (.h5/.hdf5), Pickle (.pkl), NumPy (.npy/.npz)                                                                     |

## Conversion Paths

```
PyTorch  → SafeTensors, ONNX, TorchScript, CoreML, MLX
SafeTensors → GGUF (with quantization: q4_0, q4_k_m, q5_k_m, q8_0)
ONNX → TensorRT, OpenVINO, TFLite
TensorFlow → TFLite
```

## MCP Tools (Model Context Protocol)

| Tool               | Description                                    |
| ------------------ | ---------------------------------------------- |
| `search_documents` | Vector similarity search in RAG knowledge base |
| `add_document`     | Add document with metadata and embeddings      |
| `chunk_text`       | Split text into overlapping chunks for RAG     |
| `execute_rule`     | Execute business rule from rule engine         |

Custom tools can be registered via `MCPServer::register_tool(tool, executor_fn)`.

## Cargo Features

| Feature        | Description                         |
| -------------- | ----------------------------------- |
| `default`      | SafeTensors, ndarray, SQLite        |
| `full`         | All non-system features             |
| `sqlite`       | SQLite RAG backend                  |
| `kv-store`     | Sled KV backend                     |
| `vector-db`    | Qdrant vector database              |
| `s3`           | AWS S3 cloud storage                |
| `azure`        | Azure Blob storage                  |
| `cloud`        | All cloud backends                  |
| `api`          | REST API (Axum + JWT)               |
| `graphql`      | GraphQL API                         |
| `gpu`          | GPU-accelerated encryption (OpenCL) |
| `python`       | Python bindings (PyO3)              |
| `hdf5-support` | HDF5 format support                 |

## Environment Variables

| Variable                                                     | Purpose                              |
| ------------------------------------------------------------ | ------------------------------------ |
| `aimodelvault_PASSPHRASE`                                    | Vault passphrase (for CI/automation) |
| `aimodelvault_VAULT`                                         | Default vault name                   |
| `aimodelvault_CONFIG`                                        | Custom config path                   |
| `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` / `AWS_REGION` | AWS S3 credentials                   |
| `AZURE_STORAGE_ACCOUNT` / `AZURE_STORAGE_KEY`                | Azure credentials                    |
| `GOOGLE_APPLICATION_CREDENTIALS` / `GCP_PROJECT`             | GCS credentials                      |
| `AIM_TELEMETRY_ENABLED`                                      | Set to `false` to disable telemetry  |
| `AIM_TELEMETRY_DISABLED`                                     | Set to `1` to disable telemetry      |
| `DO_NOT_TRACK`                                               | Set to `1` to disable telemetry      |
| `AIM_SQLITE_VERSIONS`                                        | Use SQLite version backend           |

## Project Layout

```bash
src/
├── lib.rs              # Library root (pub modules)
├── main.rs             # CLI entry point
├── cli/                # CLI subcommand handlers
├── vault.rs            # Core vault logic + VaultBuilder
├── traits.rs           # Core traits, event system, URI parser, metrics
├── crypto/             # AES-256-GCM, Argon2id, streaming encryption
├── storage.rs          # Storage backends
├── version.rs          # Version control (JSON backend)
├── version_sqlite.rs   # Version control (SQLite backend)
├── formats.rs          # 23+ format detection
├── conversion.rs       # Format conversion pipeline (10 converters)
├── audit.rs            # Security audit logging
├── compliance.rs       # FIPS/CMMC/MITRE checks
├── model_card.rs       # Model Cards
├── rag/                # RAG system (7 submodules)
├── utils.rs            # Utilities
├── blockchain.rs       # Blockchain audit trail
├── federation.rs       # Federated vault sync
├── api.rs              # REST/GraphQL API
├── telemetry.rs        # Anonymous telemetry (opt-in)
├── config.rs           # XDG-compliant configuration
├── error.rs            # Error types
├── download.rs         # Model download (HuggingFace, Ollama, URLs)
├── signing.rs          # HMAC-SHA256 model signing & verification
├── scanning.rs         # Pickle safety scanning
├── diff.rs             # Model diffing (tensor-level comparison)
├── interop.rs          # Ollama & LM Studio registration
├── benchmark.rs        # Benchmark metadata storage
├── license_scan.rs     # License detection & SPDX normalization
├── tags.rs             # Model tagging and search
├── vault_bundle.rs     # Vault export/import bundles
├── gc.rs               # Garbage collection
├── tui.rs              # Terminal UI dashboard
├── webhooks.rs         # Webhook notification system
├── access_control.rs   # Role-based access control
├── kms.rs              # External secrets manager integration
├── validation.rs       # Model integrity validation
├── policies.rs         # Retention policy enforcement
├── lineage_graph.rs    # Cross-model lineage DAG
├── plugins.rs          # Plugin system
├── profiles.rs         # Configuration profiles
├── quantization.rs     # Quantization pipeline & profile store
├── evaluation.rs       # Model evaluation harness
├── scheduler.rs        # Vault backup scheduling
├── multi_vault.rs      # Multi-vault registry & switching
└── python.rs           # Python bindings (PyO3)
```

## Security Model

| Layer          | Technology                                         |
| -------------- | -------------------------------------------------- |
| Encryption     | AES-256-GCM (12-byte nonce, 16-byte auth tag)      |
| Key Derivation | Argon2id (64MB memory, 3 iterations, 32-byte salt) |
| Integrity      | SHA-256 checksums on every operation               |
| Memory         | Zeroize (secure memory zeroing)                    |
| Audit          | Append-only blockchain with Merkle proofs          |
| Permissions    | 700 dirs / 600 files (Unix), ACLs (Windows)        |

## Compliance

| Standard         | Status                                     |
| ---------------- | ------------------------------------------ |
| FIPS 140-3       | Compliant (AES-256-GCM, SHA-256, Argon2id) |
| CMMC 2.0 Level 2 | Certified (17 controls: AC, AU, IA, SC)    |
| MITRE ATT&CK     | Mitigated (T1552, T1486, T1078, T1005)     |

## Agent Interaction Patterns

### Bootstrap (agent-first discovery)
```bash
# 1. Get the full CLI schema as JSON (pipe to jq, parse, etc.)
aim introspect --format json

# 2. Compact mode omits descriptions and examples for smaller payloads
aim introspect --format json --compact

# 3. JSON-LD output links to the ontology for semantic interop
aim introspect --format jsonld
```

### Store a model
```bash
aim init
aim store my-llm ./model.safetensors -d "Fine-tuned LLaMA" --framework pytorch --task text-generation
```

### Convert for edge deployment
```bash
aim convert my-llm --to-format gguf --quantization q4_k_m --validate
```

### Check compliance
```bash
aim compliance --verbose
```

### Search RAG knowledge base
```bash
aim database init --path ./kb --db-type sqlite
aim database store --path ./kb --input paper.pdf
aim database search --path ./kb "transformer attention mechanism"
```

### Push to cloud
```bash
aim cloud push my-llm --provider s3 --bucket my-models
```
