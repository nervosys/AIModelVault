# Blockchain Audit Trail

Append-only audit log with Merkle proofs — every mutating operation is recorded as a block whose hash chains to the previous block. Tampering with any past entry breaks the chain and is detected at verification time.

## Operations recorded

`init`, `store`, `delete`, `convert`, `sign`, `verify`, `acl grant/revoke`, `policy apply`, `cloud push/pull`, plus custom events emitted by plugins.

## CLI

```bash
aim audit              # show recent entries
aim audit --verify     # re-hash the chain and report tampering
aim audit --export OUT # export the chain for archival
```

## Block fields

| Field       | Description                                     |
| ----------- | ----------------------------------------------- |
| `index`     | Sequential block number                         |
| `timestamp` | RFC 3339                                        |
| `principal` | ACL principal who triggered the op              |
| `operation` | Operation name                                  |
| `payload`   | Operation-specific JSON                         |
| `prev_hash` | SHA-256 of the previous block                   |
| `hash`      | SHA-256 of this block (covers all fields above) |

See [src/blockchain.rs](../src/blockchain.rs).
