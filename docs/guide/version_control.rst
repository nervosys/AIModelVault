Version Control Guide
=====================

AI Model Vault provides built-in version control for models with full lineage
tracking — no external VCS required.


Storing Versions
----------------

Each call to ``store_model`` creates a new version:

.. code-block:: python

   from aimodelvault import Vault, ModelMetadata

   vault = Vault()
   vault.unlock(b"passphrase")

   meta = ModelMetadata("gpt-finetune", "safetensors",
                        description="GPT fine-tune checkpoints")

   # Version 1 (initial)
   v1 = vault.store_model("gpt-finetune", epoch1_data, meta)
   print(f"v{v1.version}: {v1.checkpoint_id}")

   # Version 2 (with parent lineage)
   v2 = vault.store_model("gpt-finetune", epoch2_data, meta, parent_version=1)

   # Version 3
   v3 = vault.store_model("gpt-finetune", epoch3_data, meta, parent_version=2)


Listing Versions
----------------

.. code-block:: python

   versions = vault.list_versions("gpt-finetune")
   for v in versions:
       print(f"v{v.version} | {v.format} | {v.size_bytes} bytes | {v.timestamp}")


Tracing Lineage
---------------

.. code-block:: python

   # Get full ancestry chain for version 3
   lineage = vault.get_lineage("gpt-finetune", version=3)
   for v in lineage:
       parent = f"← v{v.parent_version}" if v.parent_version else "(root)"
       print(f"  v{v.version} {parent}")

   # Output:
   #   v1 (root)
   #   v2 ← v1
   #   v3 ← v2


Integrity Verification
----------------------

Every version stores a SHA-256 checksum computed before encryption:

.. code-block:: python

   from aimodelvault import sha256_hex

   data = vault.get_model("gpt-finetune", version=2)
   v2 = vault.list_versions("gpt-finetune")[1]

   assert sha256_hex(data) == v2.checksum_sha256
   print("Integrity verified ✓")


Deleting Versions
-----------------

.. code-block:: python

   deleted = vault.delete_version("gpt-finetune", version=1)
   print(f"Deleted: {deleted}")  # True
