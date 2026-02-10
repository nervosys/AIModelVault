//! CLI helper utilities.

use ai_model_vault::Result;
use std::io::{self, Write};

/// Prompt for passphrase input (masked).
pub fn prompt_passphrase(prompt: &str) -> Result<Vec<u8>> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let passphrase = rpassword::read_password()?;
    Ok(passphrase.into_bytes())
}
