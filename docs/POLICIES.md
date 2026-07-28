# Retention Policies

Declarative version pruning rules, enforced explicitly via `aim policy apply` (never automatic).

## Policy fields

| Field          | Meaning                                       |
| -------------- | --------------------------------------------- |
| `max_versions` | Keep only the newest N versions               |
| `max_age_days` | Delete versions older than N days             |
| `keep_minimum` | Floor — never reduce below this many versions |

## CLI

```bash
aim policy set my-llm --max-versions 5 --keep-minimum 2
aim policy list
aim policy apply my-llm --dry-run
aim policy apply-all
aim policy remove my-llm
```

## MCP tools

`policy_set`, `policy_remove`, `policy_list`, `policy_apply`, `policy_apply_all`.

`--dry-run` reports the actions that would be taken without deleting anything. See [src/policies.rs](https://github.com/nervosys/AIModelVault/blob/master/src/policies.rs).
