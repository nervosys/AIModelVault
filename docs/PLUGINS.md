# Plugin System

Out-of-process extensions described by JSON manifests. Plugins surface their own CLI commands and event subscribers without recompiling `aim`.

## Manifest

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "0.1.0",
  "entrypoint": "./my-plugin.exe",
  "subscribes_to": ["ModelStored", "ModelDeleted"],
  "commands": [
    { "name": "hello", "description": "Say hi" }
  ]
}
```

## CLI

```bash
aim plugin discover                # scan well-known paths
aim plugin install ./manifest.json
aim plugin list
aim plugin info my-plugin
aim plugin uninstall my-plugin
```

## MCP tools

`plugin_discover`, `plugin_install`, `plugin_uninstall`, `plugin_list`, `plugin_info`.

Plugins run with the same user-account permissions as `aim` — only install plugins you trust. See [src/plugins.rs](https://github.com/nervosys/AIModelVault/blob/master/src/plugins.rs).
