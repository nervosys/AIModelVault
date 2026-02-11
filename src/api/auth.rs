//! JWT authentication for the REST API.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT claims payload.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (always "vault-user" for single-tenant mode).
    pub sub: String,
    /// Issued-at (Unix epoch seconds).
    pub iat: u64,
    /// Expiration (Unix epoch seconds).
    pub exp: u64,
}

/// Create a signed JWT token.
pub fn create_token(secret: &str, expiry_secs: u64) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_secs();

    let claims = Claims {
        sub: "vault-user".into(),
        iat: now,
        exp: now + expiry_secs,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Verify and decode a JWT token, returning the claims.
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_token() {
        let secret = "test-secret-key-for-jwt-signing";
        let token = create_token(secret, 3600).unwrap();
        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.sub, "vault-user");
    }

    #[test]
    fn test_invalid_secret_rejects() {
        let token = create_token("good-secret", 3600).unwrap();
        let result = verify_token(&token, "wrong-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token_rejects() {
        let secret = "test-secret";
        // Token that expired well beyond the default leeway (60s)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = Claims {
            sub: "vault-user".into(),
            iat: now - 7200,
            exp: now - 3600,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();
        let result = verify_token(&token, secret);
        assert!(result.is_err());
    }
}
