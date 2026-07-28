# Tags & Search

Free-form labels and key/value annotations to organize and search models.

## CLI

```bash
aim tag add my-llm production v2 stable
aim tag annotate my-llm --key team --value llm-platform
aim tag list my-llm
aim tag remove my-llm production

aim search llm
aim search "" --tag production
aim search llm --tag production --format json
```

## MCP tools

`tag_add`, `tag_remove`, `tag_list`, `tag_annotate`, `model_search`.

## REST

`/api/v1/models/{name}/tags`, `/api/v1/search`.

Tags are case-sensitive; search matches name substring AND every supplied tag/annotation filter. See [src/tags.rs](https://github.com/nervosys/AIModelVault/blob/master/src/tags.rs).
