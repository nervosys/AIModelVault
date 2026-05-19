# External KMS Integration

`aim` can fetch passphrases, HMAC keys, and cloud credentials from external secret managers instead of env vars.

## Supported sources

| URI prefix              | Backend                        |
| ----------------------- | ------------------------------ |
| `env://NAME`            | Environment variable           |
| `aws-sm://path/secret`  | AWS Secrets Manager            |
| `azure-kv://vault/name` | Azure Key Vault                |
| `vault://path/key`      | HashiCorp Vault                |
| `file://path`           | Local file (mode 600 enforced) |

## Usage

```bash
aim init                       # uses env or prompt
aimodelvault_PASSPHRASE="aws-sm://prod/aim-passphrase" aim store ...
aim sign my-llm --key "azure-kv://my-vault/hmac-key"
```

Keys are read once, zeroized after use. See [src/kms.rs](../src/kms.rs).
