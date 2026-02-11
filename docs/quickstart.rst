Quick Start
===========

Installation
------------

**From PyPI (with native Rust bindings):**

.. code-block:: bash

   pip install neuralvault

**From source (requires Rust toolchain):**

.. code-block:: bash

   pip install maturin
   git clone https://github.com/nervosys/ai-model-vault.git
   cd ai-model-vault
   maturin develop --features python

**Verify installation:**

.. code-block:: python

   import neuralvault
   print(neuralvault.version())       # e.g. "0.1.0"
   print(neuralvault._NATIVE)         # True if native bindings loaded


Basic Usage
-----------

Create a Vault, Store a Model
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: python

   from neuralvault import Vault, VaultConfig, ModelMetadata

   # Create or open a vault (XDG-compliant default location)
   vault = Vault()

   # Unlock with a passphrase
   vault.unlock(b"my-secure-passphrase")

   # Prepare model data and metadata
   model_data = open("model.safetensors", "rb").read()
   metadata = ModelMetadata(
       "my-model", "safetensors",
       description="Fine-tuned LLM",
       framework="PyTorch",
       task="text-generation",
       parameters=7_000_000_000,
   )

   # Store → returns a ModelVersion
   version = vault.store_model("my-model", model_data, metadata)
   print(f"Stored v{version.version}, SHA-256: {version.checksum_sha256}")

   # Lock when done (zeroizes keys)
   vault.lock()


Retrieve a Model
^^^^^^^^^^^^^^^^

.. code-block:: python

   vault.unlock(b"my-secure-passphrase")

   # Latest version
   data = vault.get_model("my-model")

   # Specific version
   data_v1 = vault.get_model("my-model", version=1)

   # Save to disk
   with open("restored_model.safetensors", "wb") as f:
       f.write(data)


Format Detection
^^^^^^^^^^^^^^^^

.. code-block:: python

   from neuralvault import ModelFormat

   fmt = ModelFormat.detect("weights.safetensors")
   print(fmt.name)       # "Safetensors"
   print(fmt.extension)  # "safetensors"


Model Cards
^^^^^^^^^^^

.. code-block:: python

   from neuralvault import ModelCard

   card = ModelCard(
       "my-model", "1.0", "transformer",
       description="Fine-tuned language model for code generation",
       developers=["NervoSys AI"],
       license="AGPL-3.0-or-later",
   )
   card.set_training_data("The Stack v2", source="HuggingFace")
   card.add_metric("accuracy", 0.92, "Top-1 accuracy on HumanEval")

   # Export in multiple formats
   print(card.to_markdown())
   json_str = card.to_json()
   yaml_str = card.to_yaml()

   # Round-trip
   card2 = ModelCard.from_json(json_str)


Version Lineage
^^^^^^^^^^^^^^^

.. code-block:: python

   vault.unlock(b"my-secure-passphrase")

   # Store incremental versions with parent tracking
   v1 = vault.store_model("gpt-finetune", data_v1, meta)
   v2 = vault.store_model("gpt-finetune", data_v2, meta, parent_version=1)
   v3 = vault.store_model("gpt-finetune", data_v3, meta, parent_version=2)

   # Trace lineage
   lineage = vault.get_lineage("gpt-finetune", version=3)
   for v in lineage:
       print(f"  v{v.version} ({v.format}, {v.size_bytes} bytes)")


Cryptographic Utilities
^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: python

   from neuralvault import sha256_hex

   digest = sha256_hex(b"model bytes here")
   print(digest)  # 64-char hex SHA-256
