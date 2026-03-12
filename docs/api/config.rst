VaultConfig API
===============

.. module:: aimodelvault
   :noindex:

.. class:: VaultConfig(vault_dir=None)

   Configuration for vault paths and crypto settings.

   By default, uses XDG-compliant directories:

   - **Linux**: ``~/.local/share/aimodelvault/``
   - **macOS**: ``~/Library/Application Support/aimodelvault/``
   - **Windows**: ``%APPDATA%\aimodelvault\``

   :param vault_dir: Override the default vault directory.
   :type vault_dir: str or None

   .. attribute:: vault_path
      :type: str

      The resolved path to the vault directory.

   **Example:**

   .. code-block:: python

      from aimodelvault import Vault, VaultConfig

      # Default XDG location
      vault = Vault()

      # Custom location
      config = VaultConfig("/path/to/my/vault")
      vault = Vault(config=config)
