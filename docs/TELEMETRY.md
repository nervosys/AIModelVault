# Telemetry

`aim` ships with **anonymous, opt-in** telemetry — off by default unless the user explicitly enables it.

## Opt-out (multiple ways, any one suffices)

```bash
export AIM_TELEMETRY_ENABLED=false
export AIM_TELEMETRY_DISABLED=1
export DO_NOT_TRACK=1
```

Or set `telemetry.enabled = false` in `config.toml`.

## What is sent (when enabled)

- `aim` version, OS family, architecture.
- Anonymous install ID (random UUID, never reused across machines).
- Command name only — never arguments, paths, model names, or content.

## What is **never** sent

- Model names, file paths, vault contents, passphrases, keys, ACL principals.
- Free-form text from any flag.

## REST

`GET /api/v1/telemetry/status` reports `{ "enabled": bool, "do_not_track": bool }`.

See [src/telemetry.rs](https://github.com/nervosys/AIModelVault/blob/master/src/telemetry.rs).
