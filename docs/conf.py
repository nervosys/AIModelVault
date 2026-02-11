# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

import os
import sys

# -- Path setup ---------------------------------------------------------------
sys.path.insert(0, os.path.abspath(os.path.join("..", "src")))

# -- Project information ------------------------------------------------------
project = "NeuralVault"
copyright = "2026, NervoSys AI"
author = "NervoSys AI Team"
release = "0.3.0"
version = "0.3.0"

# -- General configuration ----------------------------------------------------
extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",        # Google/NumPy-style docstrings
    "sphinx.ext.viewcode",        # Source links
    "sphinx.ext.intersphinx",     # Cross-ref external docs
    "sphinx.ext.autosummary",     # Auto-generate summary tables
]

templates_path = ["_templates"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

# -- Options for HTML output --------------------------------------------------
html_theme = "sphinx_rtd_theme"
html_static_path = ["_static"]
html_title = "NeuralVault Documentation"
html_short_title = "NeuralVault"

# -- Extension configuration --------------------------------------------------

# Napoleon settings (Google-style docstrings)
napoleon_google_docstring = True
napoleon_numpy_docstring = True
napoleon_include_init_with_doc = True
napoleon_use_param = True
napoleon_use_rtype = True

# Autodoc settings
autodoc_default_options = {
    "members": True,
    "undoc-members": False,
    "show-inheritance": True,
    "member-order": "bysource",
}
autodoc_typehints = "description"

# Intersphinx mapping
intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
}

# Autosummary
autosummary_generate = True
