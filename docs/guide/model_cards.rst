Model Cards Guide
=================

NeuralVault includes built-in support for `Model Cards
<https://arxiv.org/abs/1810.03993>`_ — structured documentation for AI models
covering intended use, training data, evaluation metrics, and ethical considerations.


Creating a Model Card
---------------------

.. code-block:: python

   from neuralvault import ModelCard

   card = ModelCard(
       name="code-llama-finetune",
       version="2.0",
       model_type="transformer",
       description="Fine-tuned Code Llama for Python code generation",
       developers=["NervoSys AI", "Research Team"],
       license="AGPL-3.0-or-later",
       primary_use="Code completion and generation",
       out_of_scope=["Medical advice", "Legal counsel", "Autonomous systems"],
   )


Adding Training Data
--------------------

.. code-block:: python

   card.set_training_data(
       "The Stack v2 — Python subset (deduped)",
       source="HuggingFace Datasets",
       preprocessing="License filtering, deduplication, PII removal",
   )


Adding Evaluation Metrics
-------------------------

.. code-block:: python

   card.add_metric("pass@1", 0.67, "HumanEval pass@1")
   card.add_metric("pass@10", 0.85, "HumanEval pass@10")
   card.add_metric("MBPP", 0.72, "Mostly Basic Python Problems accuracy")


Custom Metadata
---------------

.. code-block:: python

   card.add_metadata("training_compute", "8x A100 80GB, 72 hours")
   card.add_metadata("carbon_footprint", "~120 kg CO2")
   card.add_metadata("base_model", "codellama/CodeLlama-7b-hf")


Export Formats
--------------

Model cards can be exported in three formats:

.. code-block:: python

   # Markdown — for README files, documentation
   md = card.to_markdown()

   # JSON — for programmatic consumption, APIs
   json_str = card.to_json()

   # YAML — for HuggingFace model card metadata
   yaml_str = card.to_yaml()


Round-Trip Serialization
------------------------

.. code-block:: python

   # Save
   with open("model_card.json", "w") as f:
       f.write(card.to_json())

   # Load
   with open("model_card.json") as f:
       restored = ModelCard.from_json(f.read())

   # Also works with YAML
   restored_yaml = ModelCard.from_yaml(card.to_yaml())
