# AGENTS.md — AI Agent Discovery Guide

> Machine-readable project context for AI agents, LLM assistants, and automated tools.

## Project Identity

| Key            | Value                                    |
| -------------- | ---------------------------------------- |
| **Name**       | AI Model Vault                           |
| **Binary**     | `aim`                                    |
| **Crate**      | `ai-model-vault`                         |
| **Version**    | 1.1.0                                    |
| **Language**   | Rust (edition 2021, MSRV 1.75)           |
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

## Project Layout

```
src/
├── lib.rs          # Library root (pub modules)
├── main.rs         # CLI entry point
├── cli/            # CLI subcommand handlers
├── vault.rs        # Core vault logic
├── crypto.rs       # AES-256-GCM, Argon2id
├── storage.rs      # Storage backends
├── version.rs      # Version control
├── formats.rs      # 23+ format detection
├── conversion.rs   # Format conversion pipeline
├── audit.rs        # Security audit logging
├── compliance.rs   # FIPS/CMMC/MITRE checks
├── model_card.rs   # Model Cards
├── rag.rs          # RAG system
├── utils.rs        # Utilities
├── blockchain.rs   # Blockchain audit trail
├── federation.rs   # Federated vault sync
├── api.rs          # REST/GraphQL API
└── config.rs       # XDG-compliant configuration
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
