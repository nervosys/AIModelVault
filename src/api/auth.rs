//! JWT authentication for the REST API.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Set of revoked JWT token IDs.
#[allow(clippy::incompatible_msrv)]
static REVOKED_TOKENS: std::sync::LazyLock<RwLock<HashSet<String>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashSet::new()));

/// Roles that control access to audit logs and admin operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Full access to all operations including audit logs.
    Admin,
    /// Standard model operations; audit logs are filtered to own actions.
    Operator,
    /// Read-only access; cannot modify models or view security events.
    Viewer,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Operator => write!(f, "operator"),
            Role::Viewer => write!(f, "viewer"),
        }
    }
}

/// JWT claims payload.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (unique session identifier).
    pub sub: String,
    /// JWT ID for revocation tracking.
    pub jti: String,
    /// Role for RBAC.
    #[serde(default = "default_role")]
    pub role: Role,
    /// Issued-at (Unix epoch seconds).
    pub iat: u64,
    /// Expiration (Unix epoch seconds).
    pub exp: u64,
}

fn default_role() -> Role {
    Role::Admin
}

/// Create a signed JWT token with a unique session ID.
pub fn create_token(secret: &str, expiry_secs: u64) -> Result<String, jsonwebtoken::errors::Error> {
    create_token_with_role(secret, expiry_secs, Role::Admin)
}

/// Create a signed JWT token with a specific role.
pub fn create_token_with_role(
    secret: &str,
    expiry_secs: u64,
    role: Role,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let jti = uuid::Uuid::new_v4().to_string();

    let claims = Claims {
        sub: "vault-user".into(),
        jti,
        role,
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
///
/// Rejects expired tokens, tokens with invalid signatures,
/// and tokens that have been revoked.
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    // Check if token has been revoked
    if let Ok(revoked) = REVOKED_TOKENS.read() {
        if revoked.contains(&data.claims.jti) {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }
    }

    Ok(data.claims)
}

/// Revoke a token by its JTI (JWT ID).
pub fn revoke_token(jti: &str) {
    if let Ok(mut revoked) = REVOKED_TOKENS.write() {
        revoked.insert(jti.to_string());
    }
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
        assert!(!claims.jti.is_empty());
        assert_eq!(claims.role, Role::Admin);
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
            jti: uuid::Uuid::new_v4().to_string(),
            role: Role::Admin,
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

    #[test]
    fn test_revoke_token() {
        let secret = "test-revoke-secret";
        let token = create_token(secret, 3600).unwrap();
        let claims = verify_token(&token, secret).unwrap();
        revoke_token(&claims.jti);
        let result = verify_token(&token, secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_token_with_role() {
        let secret = "test-role-secret";
        let token = create_token_with_role(secret, 3600, Role::Operator).unwrap();
        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.role, Role::Operator);

        let token2 = create_token_with_role(secret, 3600, Role::Viewer).unwrap();
        let claims2 = verify_token(&token2, secret).unwrap();
        assert_eq!(claims2.role, Role::Viewer);
    }
}
