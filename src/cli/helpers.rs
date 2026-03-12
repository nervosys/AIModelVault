//! CLI helper utilities.

use ai_model_vault::{Result, Vault, VaultBuilder, VaultConfig};
use std::io::{self, Write};

/// Prompt for passphrase input (masked).
pub fn prompt_passphrase(prompt: &str) -> Result<Vec<u8>> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let passphrase = rpassword::read_password()?;
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
