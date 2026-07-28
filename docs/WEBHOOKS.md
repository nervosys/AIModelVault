# Webhooks

HTTP notification fan-out. Subscribers receive a JSON POST whenever a vault event fires (model stored, version deleted, signature verified, etc.).

## CLI

```bash
aim webhook add --url https://example.com/hook --secret S3CR3T
aim webhook list
aim webhook test <ID>
aim webhook remove <ID>
```

## MCP tools

`webhook_add`, `webhook_remove`, `webhook_list`, `webhook_test`.

## REST

`/api/v1/webhooks` (CRUD).

## Payload

```json
{
  "event": "ModelStored",
  "model": "my-llm",
  "version": 3,
  "timestamp": "2025-01-01T00:00:00Z"
}
```

If a secret is configured, the request includes `X-AIM-Signature: sha256=<HMAC>` over the raw body. See [src/webhooks.rs](https://github.com/nervosys/AIModelVault/blob/master/src/webhooks.rs).
