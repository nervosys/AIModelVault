# Migration Guide — v0.x → v1.0.0

This document covers upgrading from any v0.x release of AI Model Vault to v1.0.0.

---

## Table of Contents

1. [Overview](#overview)
2. [Breaking Changes](#breaking-changes)
3. [Rust Crate Migration](#rust-crate-migration)
4. [Python Package Migration](#python-package-migration)
5. [CLI Migration](#cli-migration)
6. [API Migration](#api-migration)
7. [Docker Deployment (New)](#docker-deployment-new)
8. [Kubernetes Deployment (New)](#kubernetes-deployment-new)
9. [Configuration Changes](#configuration-changes)
10. [Data Migration](#data-migration)

---

## Overview

v1.0.0 is the first production-stable release of AI Model Vault. It encompasses
all features from v0.1.0 through v0.5.0 with hardened security, comprehensive
testing (1,831+ tests), and deployment-ready packaging.

| Version    | Highlights                                                   |
| ---------- | ------------------------------------------------------------ |
| v0.1.0     | Core vault, encryption, model cards, XDG compliance          |
| v0.1.1     | Hardening pass                                               |
| v0.2.0     | Code quality & architecture refactor                         |
| v0.3.0     | PyO3 Python bindings, Sphinx docs, streaming API             |
| v0.4.0     | Format conversion pipeline (10 converters, BFS path-finding) |
| v0.5.0     | REST API (14 endpoints), JWT auth, embedded web dashboard    |
| **v1.0.0** | **Production release — Docker, Helm, publication readiness** |

---

## Breaking Changes

v1.0.0 introduces **no breaking API changes** relative to v0.5.0. The public
Rust API, Python bindings, CLI commands, and REST endpoints are all
backward-compatible. The changes are additive:

- Docker and Helm deployment support (new)
- Version string bumps across all surfaces
- Classifier updated from "Alpha" to "Production/Stable"
- OpenAPI spec version now `1.0.0`

If you are upgrading from a version earlier than v0.5.0, review the relevant
section below for each intermediate release.

---

## Rust Crate Migration

### From v0.4.x or v0.5.x

Update your `Cargo.toml`:

```toml
[dependencies]
ai-model-vault = "1.0.0"
```

No source code changes required. All public types, traits, and functions
retain their existing signatures.

### From v0.3.x

The format conversion API was added in v0.4.0. If you were using the crate
purely for vault operations, no changes are needed. If you want conversion:

```rust
use ai_model_vault::conversion::{ConversionRegistry, ModelFormat};

let registry = ConversionRegistry::new();
let path = registry.find_conversion_path(ModelFormat::ONNX, ModelFormat::SafeTensors);
```

### From v0.2.x or earlier

The Python bindings module (`python` feature) was added in v0.3.0.
Enable it if needed:

```toml
[dependencies]
ai-model-vault = { version = "1.0.0", features = ["python"] }
```

---

## Python Package Migration

### From any v0.x

Update the package:

```bash
pip install --upgrade aimodelvault==1.0.0
```

Or with optional ML dependencies:

```bash
pip install "aimodelvault[ml]==1.0.0"
```

The Python API is unchanged. All functions in `aimodelvault._native` retain
their existing signatures.

---

## CLI Migration

### From any v0.x

Replace the `aim` binary. If installed via cargo:

```bash
cargo install ai-model-vault --version 1.0.0
```

All existing commands work identically. Verify:

```bash
aim --version
# aim 1.0.0
```

### New: Docker-based CLI

```bash
docker run --rm -v $(pwd)/vault:/data ghcr.io/nervosys/ai-model-vault:1.0.0 \
  aim store my-model --format safetensors --file model.safetensors
```

---

## API Migration

### From v0.5.x

The REST API is fully backward-compatible. The only change is the OpenAPI
spec version field (`0.5.0` → `1.0.0`). All 14 endpoints retain their
existing request/response schemas:

| Endpoint                        | Method | Status    |
| ------------------------------- | ------ | --------- |
| `/health`                       | GET    | Unchanged |
| `/auth/token`                   | POST   | Unchanged |
| `/models`                       | GET    | Unchanged |
| `/models`                       | POST   | Unchanged |
| `/models/{name}`                | GET    | Unchanged |
| `/models/{name}`                | DELETE | Unchanged |
| `/models/{name}/versions`       | POST   | Unchanged |
| `/models/{name}/versions`       | GET    | Unchanged |
| `/models/{name}/versions/{ver}` | GET    | Unchanged |
| `/models/{name}/versions/{ver}` | DELETE | Unchanged |
| `/models/{name}/lineage/{ver}`  | GET    | Unchanged |
| `/conversions`                  | GET    | Unchanged |
| `/convert`                      | POST   | Unchanged |
| `/stats`                        | GET    | Unchanged |

### From pre-v0.5.0

The REST API did not exist before v0.5.0. Enable it with:

```bash
cargo build --features api
aim serve --host 0.0.0.0 --port 8080
```

---

## Docker Deployment (New)

> **Removed in 4.5.0.** The Dockerfile, published images, and Helm chart no
> longer exist. This section is kept as a record of what v1.0.0 shipped.

v1.0.0 introduced first-class Docker support:

```bash
# Build locally
docker build -t aim:latest .
docker build --build-arg FEATURES=api -t aim:api .

# Pull from registry (when published)
docker pull ghcr.io/nervosys/ai-model-vault:1.0.0
```

Alpine (default) and Debian variants are available:

```bash
docker build --target alpine -t aim:alpine .
docker build --target debian -t aim:debian .
```

### Volumes

| Mount Point | Purpose                         |
| ----------- | ------------------------------- |
| `/data`     | Vault data (XDG_DATA_HOME)      |
| `/config`   | Configuration (XDG_CONFIG_HOME) |
| `/cache`    | Cache files (XDG_CACHE_HOME)    |

---

## Kubernetes Deployment (New)

> **Removed in 4.5.0.** The chart no longer exists; this records v1.0.0.

v1.0.0 provided a Helm chart at `deploy/helm/ai-model-vault/`:

```bash
# Install
helm install aim deploy/helm/ai-model-vault/ \
  --set api.jwtSecret=your-secret-key

# With ingress
helm install aim deploy/helm/ai-model-vault/ \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=aim.example.com \
  --set ingress.hosts[0].paths[0].path=/ \
  --set ingress.hosts[0].paths[0].pathType=Prefix

# Upgrade
helm upgrade aim deploy/helm/ai-model-vault/
```

The chart includes:
- Deployment with security context (non-root, read-only FS, drop all caps)
- Service (ClusterIP)
- Secret (auto-generated JWT secret)
- PersistentVolumeClaims (data, config, cache)
- Optional Ingress
- Optional HorizontalPodAutoscaler
- ServiceAccount

---

## Configuration Changes

### Environment Variables

All existing environment variables are unchanged:

| Variable          | Since  | Purpose            |
| ----------------- | ------ | ------------------ |
| `AIM_HOST`        | v0.5.0 | API listen address |
| `AIM_PORT`        | v0.5.0 | API listen port    |
| `AIM_JWT_SECRET`  | v0.5.0 | JWT signing key    |
| `XDG_DATA_HOME`   | v0.1.0 | Data directory     |
| `XDG_CONFIG_HOME` | v0.1.0 | Config directory   |
| `XDG_CACHE_HOME`  | v0.1.0 | Cache directory    |

New in v1.0.0:

| Variable           | Purpose                                       |
| ------------------ | --------------------------------------------- |
| `AIM_TOKEN_EXPIRY` | JWT token lifetime in seconds (default: 3600) |

---

## Data Migration

### Vault Data

Vault files created by any v0.x release are fully compatible with v1.0.0.
No data migration is required. The on-disk format has not changed.

### Database (SQLite)

If using the `sqlite` feature, the database schema is unchanged from v0.5.0.
No migration SQL is needed.

### Verification

After upgrading, verify your vault is accessible:

```bash
aim list
aim versions my-model   # per-version detail
aim analyze my-model    # format, size, tensor summary
aim --version
```

---

## Support

- **Issues**: https://github.com/nervosys/AIModelVault/issues
- **Discussions**: https://github.com/nervosys/AIModelVault/discussions
- **Security**: See [SECURITY.md](https://github.com/nervosys/AIModelVault/blob/master/SECURITY.md)
