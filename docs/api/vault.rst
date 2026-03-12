Vault API
=========

.. module:: aimodelvault
   :synopsis: Secure vault for AI model storage and management.

The :class:`Vault` class is the primary interface for securely storing,
retrieving, and managing AI models with FIPS 140-3 compliant encryption.

Vault
-----

.. class:: Vault(config=None)

   Create or open a secure model vault.

   :param config: Optional :class:`VaultConfig` to override default XDG paths.
   :type config: VaultConfig or None

   .. method:: unlock(passphrase)

      Unlock the vault with a passphrase. Derives an AES-256-GCM key using Argon2id.

      :param passphrase: The vault passphrase.
      :type passphrase: bytes
      :raises RuntimeError: If passphrase is incorrect or data is corrupted.

   .. method:: lock()

      Lock the vault and zeroize all key material from memory.

   .. attribute:: is_unlocked
      :type: bool

      Whether the vault is currently unlocked.

   .. method:: store_model(name, data, metadata, parent_version=None)

      Encrypt and store a model in the vault.

      :param name: Model name (identifier).
      :type name: str
      :param data: Raw model bytes.
      :type data: bytes
      :param metadata: Model metadata.
      :type metadata: ModelMetadata
      :param parent_version: Parent version number for lineage tracking.
      :type parent_version: int or None
      :returns: The newly created version.
      :rtype: ModelVersion

   .. method:: get_model(name, version=None)

      Decrypt and retrieve a model from the vault.

      :param name: Model name.
      :type name: str
      :param version: Specific version number, or ``None`` for latest.
      :type version: int or None
      :returns: Decrypted model bytes.
      :rtype: bytes
      :raises ValueError: If model or version not found.

   .. method:: list_models()

      List all model names in the vault.

      :returns: List of model name strings.
      :rtype: list[str]

   .. method:: list_versions(name)

      List all versions of a model.

      :param name: Model name.
      :type name: str
      :returns: List of version snapshots.
      :rtype: list[ModelVersion]

   .. method:: get_lineage(name, version)

      Trace the full version lineage (ancestry chain) for a model version.

      :param name: Model name.
      :type name: str
      :param version: Version number to trace from.
      :type version: int
      :returns: Ordered list from oldest ancestor to the given version.
      :rtype: list[ModelVersion]

   .. method:: delete_version(name, version)

      Delete a specific model version.

      :param name: Model name.
      :type name: str
      :param version: Version number to delete.
      :type version: int
      :returns: ``True`` if the version existed and was deleted.
      :rtype: bool

   .. method:: get_stats()

      Get vault statistics.

      :returns: Dictionary with ``model_count``, ``total_versions``, ``total_size_bytes``.
      :rtype: dict

   .. method:: change_passphrase(new_passphrase)

      Change the vault passphrase. Re-encrypts all stored models.

      :param new_passphrase: The new passphrase.
      :type new_passphrase: bytes
      :returns: Number of models re-encrypted.
      :rtype: int

   .. attribute:: config
      :type: VaultConfig

      The vault's configuration (read-only).
