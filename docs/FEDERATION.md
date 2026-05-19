# Federation

Multi-peer vault synchronization with vector clocks for conflict detection.

## Concepts

- **Peer** — another `aim` node identified by a stable ID and reachable endpoint.
- **Vector clock** — per-peer monotonic counter attached to every mutation.
- **Conflict** — concurrent writes to the same model from different peers.

## Operations

Federation operations are exposed through the library (`src/federation.rs`) and the REST API:

- Peer registration / discovery
- Push / pull deltas
- Conflict listing and resolution (manual or last-writer-wins)

## When to use it

Use federation when multiple workstations or CI runners need to share a single logical vault without a central server. For centralized storage, prefer the cloud backends (S3, Azure Blob, GCS) — see [CLOUD_STORAGE.md](CLOUD_STORAGE.md).

See [src/federation.rs](../src/federation.rs).
