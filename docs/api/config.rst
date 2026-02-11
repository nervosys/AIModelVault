VaultConfig API
===============

.. module:: neuralvault
   :noindex:

.. class:: VaultConfig(vault_dir=None)

   Configuration for vault paths and crypto settings.

   By default, uses XDG-compliant directories:

   - **Linux**: ``~/.local/share/neuralvault/``
   - **macOS**: ``~/Library/Application Support/neuralvault/``
   - **Windows**: ``%APPDATA%\neuralvault\``

   :param vault_dir: Override the default vault directory.
   :type vault_dir: str or None

   .. attribute:: vault_path
      :type: str

      The resolved path to the vault directory.

   **Example:**

   .. code-block:: python

      from neuralvault import Vault, VaultConfig

      # Default XDG location
      vault = Vault()

      # Custom location
      config = VaultConfig("/path/to/my/vault")
      vault = Vault(config=config)
