# Configuration Profiles

Named bundles of config overrides — quickly switch between, e.g., a `dev` and `prod` set of cloud/KMS settings.

## CLI

```bash
aim profile create dev --description "Local dev" \
  --override storage.backend=sqlite \
  --override telemetry.enabled=false

aim profile activate dev
aim profile show
aim profile list
aim profile deactivate
aim profile remove dev
```

## MCP tools

`profile_create`, `profile_remove`, `profile_list`, `profile_activate`, `profile_deactivate`, `profile_show`.

Active profile name is persisted in the XDG config dir; overrides are merged over the base config at load time. See [src/profiles.rs](https://github.com/nervosys/AIModelVault/blob/master/src/profiles.rs).
