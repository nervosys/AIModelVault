Vault User Guide
================

The vault is the core component of AI Model Vault — a secure, encrypted container
for storing and managing AI model files with version control and lineage tracking.

Architecture
------------

.. code-block:: text

   ┌─────────────────────────────────┐
   │           Application           │
   ├─────────────────────────────────┤
   │   Vault API (Python / Rust)     │
   ├─────────────────────────────────┤
   │  AES-256-GCM  │  Argon2id KDF  │
   ├─────────────────────────────────┤
   │  Version Control  │  Audit Log  │
   ├─────────────────────────────────┤
   │      XDG-compliant Storage      │
   └─────────────────────────────────┘

- **Encryption**: AES-256-GCM authenticated encryption (FIPS 140-3 compliant)
- **Key Derivation**: Argon2id with persistent salt for reproducible key derivation
- **Integrity**: SHA-256 checksums on every stored model
- **Zeroization**: Keys are securely wiped from memory on ``lock()``


Lifecycle
---------

1. **Create** a vault (happens automatically on first use)
2. **Unlock** with a passphrase — derives encryption key via Argon2id
3. **Store** models — encrypted at rest with AES-256-GCM
4. **Retrieve** models — decrypted on-the-fly, integrity verified
5. **Lock** — zeroizes all key material from memory


Custom Vault Location
---------------------

By default, AI Model Vault uses XDG-compliant directories. Override with ``VaultConfig``:

.. code-block:: python

   from aimodelvault import Vault, VaultConfig

   config = VaultConfig("/secure/nvme/models")
   vault = Vault(config=config)


Changing Passphrases
--------------------

.. code-block:: python

   vault.unlock(b"old-passphrase")
   count = vault.change_passphrase(b"new-stronger-passphrase")
   print(f"Re-encrypted {count} models")
   # Vault remains unlocked with the new passphrase


Vault Statistics
----------------

.. code-block:: python

   stats = vault.get_stats()
   print(f"Models: {stats['model_count']}")
   print(f"Versions: {stats['total_versions']}")
   print(f"Total size: {stats['total_size_bytes'] / 1e9:.2f} GB")
