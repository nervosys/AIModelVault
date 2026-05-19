# Cross-Model Lineage DAG

While `aim lineage` shows the parent/child tree *within* one model, `aim lineage-graph` tracks derivations *across* models — e.g. `llama-base → llama-instruct → llama-quant`.

## Edge kinds

`fine-tune`, `distill`, `quantize`, `convert`, `merge`, `lora`.

## CLI

```bash
aim lineage-graph add --child llama-instruct --parents llama-base --kind fine-tune
aim lineage-graph add --child llama-q4 --parents llama-instruct --kind quantize
aim lineage-graph show
aim lineage-graph ancestors llama-q4
aim lineage-graph descendants llama-base
```

## MCP tools

`lineage_graph_add`, `lineage_graph_show`, `lineage_graph_ancestors`, `lineage_graph_descendants`.

The store is a DAG — cycles are rejected at insert time. See [src/lineage_graph.rs](../src/lineage_graph.rs).
