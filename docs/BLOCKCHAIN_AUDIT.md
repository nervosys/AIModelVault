# Blockchain Audit Trail

A Merkle-chained, append-only block store for audit entries.

> **Status: library primitive, not a wired feature.**
>
> Nothing in the vault, CLI, or REST API writes to this store. There is no
> `aim audit` command. If you want a tamper-evident chain, you construct
> `BlockchainAudit` yourself and feed it entries.
>
> For the audit trail that *is* wired up and running by default, see
> [`src/audit.rs`](https://github.com/nervosys/AIModelVault/blob/master/src/audit.rs)
> — `Vault` writes to it on unlock, store, delete, and integrity failure when
> `security.audit_log` is enabled. That log is append-only and `0600`, but has
> no Merkle proofs.

An earlier revision of this page documented `aim audit`, `aim audit --verify`,
and `aim audit --export`, and stated that "every mutating operation is
recorded as a block." None of that was true.

---

## What it provides

| Type              | Purpose                                                  |
| ----------------- | -------------------------------------------------------- |
| `MerkleTree`      | Builds a root over a set of serialised entries            |
| `MerkleProof`     | Inclusion proof for a single entry against a root         |
| `AuditBlock`      | One block: entries, Merkle root, chain link, block hash   |
| `BlockchainAudit` | The chain — append, verify, produce and check proofs      |
| `ChainVerification` / `BlockVerification` | Verification results with issue lists |

## `AuditBlock` fields

| Field         | Description                                       |
| ------------- | ------------------------------------------------- |
| `index`       | Block height                                      |
| `timestamp`   | `DateTime<Utc>` of block creation                 |
| `prev_hash`   | Hash of the previous block                        |
| `merkle_root` | Merkle root over this block's entries             |
| `entries`     | `Vec<BlockEntry>`                                 |
| `signature`   | Optional base64 signature over the block          |
| `nonce`       | Proof-of-work nonce, if enabled                   |
| `hash`        | SHA-256 over index, timestamp, prev_hash, merkle_root, nonce |

Per-operation detail lives one level down: each `BlockEntry` wraps an
`audit: AuditEntry` (timestamp, event type, description, model name, version,
success flag, optional metadata) alongside its own `hash` and
`index_in_block`. There is no `principal`, `operation`, or `payload` field —
an earlier revision of this table listed those, and they do not exist.

## Usage

```rust
use ai_model_vault::audit::AuditEntry;
use ai_model_vault::{BlockchainAudit, Result};

fn record(chain_dir: &std::path::Path, entry: AuditEntry) -> Result<()> {
    // Blocks are sealed automatically every `block_size` entries.
    let mut chain = BlockchainAudit::new(chain_dir, 128)?;

    chain.add_entry(entry)?;
    chain.finalize_block()?; // seal early if you need a boundary now

    let result = chain.verify_chain();
    assert!(result.valid, "{:?}", result.issues);

    // Prove one entry belongs to the chain without shipping the whole log.
    let proof = chain.generate_proof(0, 0)?;
    assert!(BlockchainAudit::verify_proof(&proof).valid);
    Ok(())
}
```

`add_entry` returns the entry hash and seals a block automatically once
`block_size` entries have accumulated. `height`, `latest`, `get_block`, and
`search` round out the read side.

## What verification checks

`AuditBlock::verify` reports an issue for each of: block hash mismatch,
previous-hash mismatch, non-sequential index, timestamp earlier than the
predecessor, a non-genesis block with no predecessor, and Merkle root
mismatch against a tree rebuilt from the entries.

Note that `hash` covers the block header only. Entry tampering is caught
through `merkle_root`, not through the block hash directly.

---

See [src/blockchain.rs](https://github.com/nervosys/AIModelVault/blob/master/src/blockchain.rs).
