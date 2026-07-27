//! CLI helper utilities.

use ai_model_vault::{kms, Result, Vault, VaultBuilder, VaultConfig};
use std::io::{self, BufRead, IsTerminal, Write};

/// Environment variable holding the vault passphrase for unattended use.
///
/// The value is either the passphrase itself or a KMS URI
/// (`env://`, `file://`, `aws-sm://`, `azure-kv://`, `vault://`) — see
/// [`ai_model_vault::kms`].
pub const PASSPHRASE_ENV: &str = "aimodelvault_PASSPHRASE";

/// Obtain the vault passphrase, in descending order of precedence:
///
/// 1. `$aimodelvault_PASSPHRASE` — a literal value or a KMS URI to resolve.
/// 2. A line piped on stdin, when stdin is not a terminal.
/// 3. An interactive masked prompt.
///
/// Steps 1 and 2 make every passphrase-gated command usable from CI and from
/// agents; step 3 preserves the interactive behaviour for humans.
pub fn prompt_passphrase(prompt: &str) -> Result<Vec<u8>> {
    if let Ok(value) = std::env::var(PASSPHRASE_ENV) {
        if !value.is_empty() {
            let resolved = kms::resolve(&value)?;
            return Ok(resolved.as_bytes().to_vec());
        }
    }

    let stdin = io::stdin();
    if !stdin.is_terminal() {
        let mut line = String::new();
        // A closed/empty stdin is not a passphrase — fall through to the prompt
        // rather than silently unlocking with "".
        if stdin.lock().read_line(&mut line)? > 0 {
            let trimmed = line.trim_end_matches(['\n', '\r']);
            if !trimmed.is_empty() {
                return Ok(trimmed.as_bytes().to_vec());
            }
        }
    }

    print!("{}", prompt);
    io::stdout().flush()?;

    let passphrase = rpassword::read_password()?;
    if passphrase.is_empty() {
        // A closed or non-interactive stdin reads as "" here. Deriving a key
        // from an empty passphrase would silently unlock the vault with no
        // secret at all, so refuse it.
        return Err(ai_model_vault::VaultError::InvalidInput(format!(
            "No passphrase provided. Set ${PASSPHRASE_ENV} (a literal value or a \
             KMS URI), pipe it on stdin, or run interactively."
        )));
    }
    Ok(passphrase.into_bytes())
}

/// Build a [`Vault`] using the standard builder, optionally enabling SQLite
/// version storage when `use_sqlite` is true.
///
/// All CLI handlers should use this instead of `Vault::new()` directly.
pub fn build_vault(config: VaultConfig, use_sqlite: bool) -> Result<Vault> {
    let mut builder = VaultBuilder::new().config(config);

    if use_sqlite {
        #[cfg(feature = "sqlite")]
        {
            builder = builder.sqlite_versions();
        }
        #[cfg(not(feature = "sqlite"))]
        {
            return Err(ai_model_vault::VaultError::ConfigError(
                "SQLite version backend requires the `sqlite` feature".to_string(),
            ));
        }
    }

    builder.build()
}
