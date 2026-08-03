# Federation

Multi-peer vault synchronization with vector clocks for conflict detection.

## Concepts

- **Peer** — another `aim` node identified by a stable ID and reachable endpoint.
- **Vector clock** — per-peer monotonic counter attached to every mutation.
- **Conflict** — concurrent writes to the same model from different peers.

## Operations

> **Status: library only.** `FederationManager` and friends are public API of
> the crate, but nothing drives them — there is no `aim` subcommand and no
> REST endpoint. An earlier revision of this page claimed REST exposure;
> `src/api/` contains no reference to the module.

Available on `FederationManager` (`src/federation.rs`):

- Peer registration / discovery
- Push / pull deltas
- Conflict listing and resolution (manual or last-writer-wins)

The README's capability table lists Federation as `library` for this reason.

## When to use it

Use federation when multiple workstations or CI runners need to share a single logical vault without a central server. For centralized storage, prefer the cloud backends (S3, Azure Blob, GCS) — see [CLOUD_STORAGE.md](CLOUD_STORAGE.md).

See [src/federation.rs](https://github.com/nervosys/AIModelVault/blob/master/src/federation.rs).
