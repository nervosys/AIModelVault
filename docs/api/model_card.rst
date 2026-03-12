Model Card API
==============

.. module:: aimodelvault
   :noindex:

.. class:: ModelCard(name, version, model_type, *, description=None, developers=None, license=None, primary_use=None, out_of_scope=None)

   Model card for documentation, transparency, and responsible AI practices.
   Follows the `Model Cards for Model Reporting <https://arxiv.org/abs/1810.03993>`_ framework.

   :param name: Model name.
   :type name: str
   :param version: Model version string (e.g. ``"1.0"``).
   :type version: str
   :param model_type: Model architecture type (e.g. ``"transformer"``).
   :type model_type: str
   :param description: Free-text model description.
   :type description: str or None
   :param developers: List of developer names/organizations.
   :type developers: list[str] or None
   :param license: License identifier (e.g. ``"AGPL-3.0-or-later"``).
   :type license: str or None
   :param primary_use: Primary intended use case.
   :type primary_use: str or None
   :param out_of_scope: List of out-of-scope uses.
   :type out_of_scope: list[str] or None

   .. method:: set_training_data(description, *, source=None, preprocessing=None)

      Set training data information.

      :param description: Description of the training dataset.
      :type description: str
      :param source: Data source (e.g. ``"HuggingFace"``).
      :type source: str or None
      :param preprocessing: Preprocessing steps applied.
      :type preprocessing: str or None

   .. method:: add_metric(name, value, description)

      Add an evaluation metric.

      :param name: Metric name (e.g. ``"accuracy"``).
      :type name: str
      :param value: Metric value.
      :type value: float
      :param description: Description of what the metric measures.
      :type description: str

   .. method:: add_metadata(key, value)

      Add a custom metadata key-value pair.

      :param key: Metadata key.
      :type key: str
      :param value: Metadata value.
      :type value: str

   .. method:: to_json()

      Serialize the model card to a JSON string.

      :returns: JSON representation.
      :rtype: str

   .. method:: to_yaml()

      Serialize the model card to a YAML string.

      :returns: YAML representation.
      :rtype: str

   .. method:: to_markdown()

      Render the model card as a Markdown document.

      :returns: Markdown string.
      :rtype: str

   .. staticmethod:: from_json(json_str)

      Deserialize a model card from JSON.

      :param json_str: JSON string.
      :type json_str: str
      :returns: Reconstructed model card.
      :rtype: ModelCard

   .. staticmethod:: from_yaml(yaml_str)

      Deserialize a model card from YAML.

      :param yaml_str: YAML string.
      :type yaml_str: str
      :returns: Reconstructed model card.
      :rtype: ModelCard

   **Example:**

   .. code-block:: python

      from aimodelvault import ModelCard

      card = ModelCard(
          "llama-finetune", "2.0", "transformer",
          description="Fine-tuned LLaMA for code generation",
          developers=["NervoSys AI"],
          license="AGPL-3.0-or-later",
          primary_use="Code completion and generation",
          out_of_scope=["Medical advice", "Legal counsel"],
      )

      card.set_training_data(
          "The Stack v2 (Python subset)",
          source="HuggingFace",
          preprocessing="Deduplication, license filtering",
      )
      card.add_metric("pass@1", 0.67, "HumanEval pass@1 score")
      card.add_metric("pass@10", 0.85, "HumanEval pass@10 score")

      # Export
      print(card.to_markdown())
      card_json = card.to_json()

      # Round-trip
      restored = ModelCard.from_json(card_json)
