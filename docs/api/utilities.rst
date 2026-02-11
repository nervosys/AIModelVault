Utilities API
=============

.. module:: neuralvault
   :noindex:

Standalone utility functions available from the native bindings.

sha256_hex
----------

.. function:: sha256_hex(data)

   Compute the SHA-256 hex digest of arbitrary data.
   Uses the same FIPS-compliant implementation as the vault.

   :param data: Input bytes.
   :type data: bytes
   :returns: 64-character lowercase hex string.
   :rtype: str

   .. code-block:: python

      from neuralvault import sha256_hex

      digest = sha256_hex(b"Hello, world!")
      print(digest)  # "315f5bdb76d0..."

version
-------

.. function:: version()

   Return the library version string.

   When native Rust bindings are active, returns the Cargo package version.
   Otherwise, returns the Python package ``__version__``.

   :returns: Version string (e.g. ``"0.1.0"``).
   :rtype: str

   .. code-block:: python

      import neuralvault
      print(neuralvault.version())  # "0.1.0"
