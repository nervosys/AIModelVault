# Telemetry and OTLP export

`aim` ships with **anonymous, opt-in** telemetry — off by default unless the
user explicitly enables it. Nothing is collected or transmitted until then.

## Opt-out (multiple ways, any one suffices)

```bash
export AIM_TELEMETRY_ENABLED=false
export AIM_TELEMETRY_DISABLED=1
export DO_NOT_TRACK=1
```

Or `aim telemetry disable`, or set `telemetry.enabled = false` in `config.toml`.

`aim telemetry status` reports the current state and the device ID.

## What is sent (when enabled)

Two events. `AppStart`, once per process:

| Field | Example |
|---|---|
| `app.version` | `4.2.1` |
| `os.type` | `linux` |
| `host.arch` | `x86_64` |
| `app.features` | `api,sqlite` |
| `device.id` | random UUID v4, generated on first run |
| `session.id` | random UUID v4, per process |

Neither identifier is derived from anything about the machine or the user, so
neither can be correlated back to an identity.

And `CommandRun`, once per invocation, added in 4.2.0:

| Field | Example |
|---|---|
| `command.name` | `cloud` |
| `command.subcommand` | `push`, or absent |
| `command.duration_ms` | `1420` |
| `command.success` | `true` |

Both names come from clap's registered command table, not from the command
line. `ArgMatches::subcommand_name` can only return a literal declared in
`args.rs`, so the set of values this field can ever hold is the set of
subcommands — a model name, path, or token has no route into it. There is a
test asserting that argument values do not appear in the pair. The failure
*reason* is deliberately not recorded, only the boolean: error messages
interpolate paths and model names.

Still not emitted: `ModelOperation`, `Conversion`, `ApiCall`, `Error`, and
`FeatureUsed`. Those event types and their `track_*` helpers are public, but
nothing in the crate calls them.

## Where events go

The built-in sender posts to a compiled-in default:

```
https://telemetry.nervosys.ai/v1/events
```

That is the project's own collector, operated by NERVOSYS. Nothing is sent
there unless you have explicitly enabled telemetry — `enabled` defaults to
`false`, so a default install never contacts it.

Override it in `config.toml` to send events somewhere you control instead:

```toml
[telemetry]
enabled = true
endpoint = "https://collector.internal.example.com/v1/events"
```

Or bypass the built-in sender entirely and use the `otel` feature with a
standard OTLP collector, described below.

This endpoint went undocumented until 4.2.1, and this page previously stated
that no default collector existed. It did, and it does — for the built-in
sender. The statement was written about the OTLP path and should have said so.

## What is **never** sent

- Model names, file paths, vault contents, passphrases, keys, ACL principals
- Free-form text from any flag

If you wire up the unused event types, note that `Error::context`,
`ApiCall::endpoint` and `FeatureUsed::detail` are free-form strings and the only
way the guarantees above can break — error messages carry file paths, and a
resolved request path carries the model name. Pass constants and route
templates, never formatted messages. `telemetry_otlp` has a test pinning the
exported attribute key set, so adding one requires a deliberate edit.

## OTLP export

Build with the `otel` feature, then configure with the standard OpenTelemetry
environment variables. Any OTLP collector or vendor endpoint works.

```bash
cargo install ai-model-vault --features otel
```

| Variable | Meaning |
|---|---|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Collector endpoint. Unset means no export. |
| `OTEL_EXPORTER_OTLP_LOGS_ENDPOINT` | Signal-specific override; takes precedence. |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | `http/protobuf` (default) or `http/json` |
| `OTEL_EXPORTER_OTLP_HEADERS` | e.g. `Authorization=Bearer <token>` |
| `OTEL_SERVICE_NAME` | Reported as `service.name` |

Two rules the implementation enforces:

1. **Setting an endpoint does not enable telemetry.** Configuring an exporter
   and consenting to collection are separate decisions, usually made by
   different people. Both are required.
2. **No OTLP endpoint or token is baked into the binary.** The OTLP exporter
   has no default collector and no default credential; unset means no export.
   A credential compiled into an AGPL crate published to a public registry is
   readable by everyone who installs it.

   This applies to the OTLP path only. The built-in sender *does* have a
   compiled-in default endpoint — see [Where events go](#where-events-go).

Building without the `otel` feature while `OTEL_EXPORTER_OTLP_ENDPOINT` is set
warns on stderr rather than silently dropping the configuration.

## Service-scoped configuration

Prefer per-service settings over machine-global ones. A bearer token in
`/etc/environment` or a `profile.d` script is inherited by every process on the
host, including ones that dump their environment on crash.

### systemd

`deploy/systemd/` has a unit and an example environment file.

```bash
sudo cp deploy/systemd/aim-server.service /etc/systemd/system/
sudo install -d -m 0755 /etc/aim
sudo cp deploy/systemd/aim-server.env.example /etc/aim/server.env
sudo chown root:root /etc/aim/server.env
sudo chmod 0600 /etc/aim/server.env
$EDITOR /etc/aim/server.env
sudo systemctl daemon-reload && sudo systemctl enable --now aim-server
```

The unit uses `EnvironmentFile=`, not `Environment=`. `Environment=` values are
visible in `systemctl show` and `systemd-analyze dump` to any local user, which
for a bearer token means every account on the machine can read it.

### Kubernetes / Helm

Create the credential Secret out of band — never in `values.yaml`, which is
committed and printed back by `helm get values`:

```bash
kubectl create secret generic aim-otlp \
  --from-literal=headers='Authorization=Bearer <token>'
```

```yaml
telemetry:
  enabled: true
  otlp:
    endpoint: "https://collector.example.com/otlp"
    protocol: "http/protobuf"
    serviceName: "ai-model-vault"
    headersSecret:
      existingSecret: "aim-otlp"
      key: headers
```

The chart injects these into the Deployment's containers only. Two releases in
the same cluster can report to different collectors, and nothing else on the
node inherits the settings.

## Rotating a leaked token

A bearer token grants write access to your telemetry backend. Treat it as
compromised if it has ever appeared in a shell history, a chat window, a CI
log, a screenshot, or a commit — including one later amended or force-pushed,
since the object usually survives in the reflog and on any fork.

Rotate at the provider, then update `/etc/aim/server.env` or the Kubernetes
Secret. No application change is needed; the value is read from the environment
at process start.

See [src/telemetry.rs](https://github.com/nervosys/AIModelVault/blob/master/src/telemetry.rs)
and [src/telemetry_otlp.rs](https://github.com/nervosys/AIModelVault/blob/master/src/telemetry_otlp.rs).
