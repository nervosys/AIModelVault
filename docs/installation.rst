Installation
============

Requirements
------------

- **Python** 3.9 or later
- **Rust toolchain** (for building from source)
- **maturin** 1.7+ (for building native bindings)


Install from PyPI
-----------------

.. code-block:: bash

   pip install aimodelvault

This installs the pre-built wheel with native Rust bindings. No Rust toolchain needed.


Install from Source
-------------------

.. code-block:: bash

   # Clone the repository
   git clone https://github.com/nervosys/AIModelVault.git
   cd AIModelVault

   # Create a virtual environment
   python -m venv .venv
   source .venv/bin/activate   # Linux/macOS
   # .venv\Scripts\Activate.ps1  # Windows PowerShell

   # Install with native bindings
   pip install maturin
   maturin develop --features python

   # Verify
   python -c "import aimodelvault; print(aimodelvault._NATIVE)"  # True


Optional Dependencies
---------------------

**ML frameworks** (for working with specific model formats):

.. code-block:: bash

   pip install aimodelvault[ml]

This installs PyTorch, TensorFlow, ONNX, SafeTensors, NumPy, and h5py.

**Development tools:**

.. code-block:: bash

   pip install aimodelvault[dev]

**Security scanning:**

.. code-block:: bash

   pip install aimodelvault[security]

**Documentation building:**

.. code-block:: bash

   pip install aimodelvault[docs]


Verify Native Bindings
-----------------------

.. code-block:: python

   import aimodelvault

   # True  → native Rust bindings active (fast, full API)
   # False → pure-Python fallback (CLI wrappers, limited API)
   print(f"Native: {aimodelvault._NATIVE}")
   print(f"Version: {aimodelvault.version()}")


Rust CLI
--------

The Rust binary ``aim`` provides a command-line interface:

.. code-block:: bash

   cargo build --release
   ./target/release/aim --help

See the :doc:`CLI documentation <../docs/CLI>` for details.
