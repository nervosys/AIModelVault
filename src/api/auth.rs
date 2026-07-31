//! JWT authentication for the REST API.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Revoked JWT IDs, each mapped to the `exp` of the token it came from.
///
/// Keeping the expiry is what makes the list prunable. A revoked JTI only has
/// to be remembered until the token would have expired on its own — after that
/// `Validation` rejects it regardless — so entries past their `exp` are dead
/// weight. The previous `HashSet<String>` had no way to know when an entry
/// stopped mattering, so the set grew for the life of the process and a busy
/// server that revoked on every logout leaked memory indefinitely.
#[allow(clippy::incompatible_msrv)]
static REVOKED_TOKENS: std::sync::LazyLock<RwLock<RevocationList>> =
    std::sync::LazyLock::new(|| RwLock::new(RevocationList::default()));

/// JTI → expiry, plus the optional file the list is mirrored to.
#[derive(Default)]
struct RevocationList {
    entries: HashMap<String, u64>,
    /// When set, every mutation is persisted here so revocations survive a
    /// restart. Without it the list is process-local: restarting the server
    /// silently un-revokes every token that has not yet expired.
    store: Option<PathBuf>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl RevocationList {
    /// Drop entries whose token has expired anyway.
    fn prune(&mut self) {
        let now = now_secs();
        self.entries.retain(|_, exp| *exp > now);
    }

    /// Rewrite the backing file, if one is configured.
    ///
    /// The whole map is rewritten rather than appended to: revocations are
    /// rare, the file stays small because it is pruned first, and a full
    /// rewrite through a temporary file plus rename means a crash mid-write
    /// leaves the previous list intact rather than a truncated one. A
    /// truncated list is the dangerous failure — it un-revokes tokens.
    fn persist(&self) -> std::io::Result<()> {
        let Some(path) = &self.store else {
            return Ok(());
        };

        let json = serde_json::to_vec(&self.entries)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &json)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

/// Persist revocations to `path`, loading any already recorded there.
///
/// Call once at startup. Until it is called the revocation list lives only in
/// this process: a restart re-admits every unexpired token that had been
/// revoked, which is the difference between "logged out" and "logged out until
/// the next deploy".
///
/// A missing file is a valid empty list (first run). A file that exists but
/// cannot be read or parsed is an error rather than an empty list, because
/// starting with a silently empty list is exactly the un-revocation this is
/// meant to prevent.
///
/// # Errors
///
/// Returns an error if `path` exists but cannot be read or parsed, or if the
/// list cannot be written back.
///
/// # Note on horizontal scaling
///
/// This is a single-node store. Replicas do not share it, so a token revoked
/// on one replica stays valid on the others. Multi-replica deployments need a
/// shared backend (Redis, a database) or short token lifetimes; the chart's
/// default `token_expiry_secs` of one hour bounds the exposure.
pub fn configure_revocation_store(path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = path.as_ref().to_path_buf();

    let entries: HashMap<String, u64> = match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Revocation store {} is corrupt: {e}", path.display()),
            )
        })?,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
        Err(err) => return Err(err),
    };

    let mut list = REVOKED_TOKENS
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    list.entries = entries;
    list.store = Some(path);
    list.prune();
    list.persist()
}

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

    // Check if token has been revoked.
    //
    // A poisoned lock is recovered rather than skipped: `if let Ok(..)` treated
    // poisoning as "not revoked" and would have admitted every revoked token
    // for the rest of the process's life.
    {
        let revoked = REVOKED_TOKENS
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if revoked.entries.contains_key(&data.claims.jti) {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }
    }

    Ok(data.claims)
}

/// Revoke the token these claims came from.
///
/// This is the entry point to prefer: the claims carry the `exp` needed to
/// retire the entry once the token would have expired anyway.
///
/// Returns an error only if a revocation store is configured and could not be
/// written. The in-memory list is updated either way, so a storage failure
/// degrades to process-local revocation rather than to no revocation — but it
/// is surfaced, because silently losing a revocation on restart is a security
/// event the operator needs to see.
///
/// # Errors
///
/// Returns an error if the configured revocation store cannot be written.
pub fn revoke_claims(claims: &Claims) -> std::io::Result<()> {
    let mut revoked = REVOKED_TOKENS
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    revoked.entries.insert(claims.jti.clone(), claims.exp);
    revoked.prune();
    revoked.persist()
}

/// Revoke a token by its JTI (JWT ID), with no expiry.
///
/// Prefer [`revoke_claims`], which supplies the token's `exp`. Without it the
/// entry can never be retired — it is stored with an effectively infinite
/// expiry and stays in the list (and in the store, if one is configured) for
/// as long as the process runs.
#[deprecated(
    since = "4.0.0",
    note = "use `revoke_claims`, which records the token's expiry so the entry can be pruned"
)]
pub fn revoke_token(jti: &str) {
    let mut revoked = REVOKED_TOKENS
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    revoked.entries.insert(jti.to_string(), u64::MAX);
    let _ = revoked.persist();
}

/// Number of revocations currently held, after pruning expired entries.
#[must_use]
pub fn revoked_count() -> usize {
    let mut revoked = REVOKED_TOKENS
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    revoked.prune();
    revoked.entries.len()
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

    /// The revocation list is process-global and cargo runs tests on parallel
    /// threads, so every test that touches it must serialise here.
    ///
    /// This originally guarded only `store`, on the reasoning that the file was
    /// the shared resource. That was wrong: `configure_revocation_store`
    /// replaces `entries` wholesale, so a test holding the lock wiped the
    /// entries of a test that was not holding it. `test_revoke_claims` revoked
    /// a token and had the revocation deleted out from under it before it could
    /// verify — it failed on macOS and passed elsewhere purely because of
    /// thread scheduling, which is what a missing lock looks like rather than a
    /// platform difference.
    static REVOCATION_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Serialise against every other revocation test and start from an empty
    /// list.
    ///
    /// The reset happens on *acquire* rather than on release: a test that
    /// panics never runs its cleanup, and leaving the next test to inherit that
    /// state turns one failure into an unrelated cascade.
    fn revocation_guard() -> std::sync::MutexGuard<'static, ()> {
        let guard = REVOCATION_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let mut list = REVOKED_TOKENS
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        list.entries.clear();
        list.store = None;

        drop(list);
        guard
    }

    #[test]
    fn test_revoke_claims() {
        let _guard = revocation_guard();

        let secret = "test-revoke-secret";
        let token = create_token(secret, 3600).unwrap();
        let claims = verify_token(&token, secret).unwrap();

        revoke_claims(&claims).unwrap();

        let result = verify_token(&token, secret);
        assert!(result.is_err(), "a revoked token must not verify");
    }

    #[test]
    fn test_expired_revocations_are_pruned() {
        let _guard = revocation_guard();

        let before = revoked_count();

        // An entry whose token expired an hour ago carries no information —
        // `Validation` already rejects the token on its own.
        {
            let mut list = REVOKED_TOKENS
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            list.entries
                .insert("stale-jti-for-pruning-test".into(), now_secs() - 3600);
        }

        // `revoked_count` prunes, so the stale entry must not be counted.
        assert_eq!(
            revoked_count(),
            before,
            "an entry past its exp should have been pruned"
        );

        let list = REVOKED_TOKENS
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(!list.entries.contains_key("stale-jti-for-pruning-test"));
    }

    #[test]
    fn test_unexpired_revocations_are_kept() {
        let _guard = revocation_guard();

        let jti = "live-jti-for-pruning-test";
        {
            let mut list = REVOKED_TOKENS
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            list.entries.insert(jti.into(), now_secs() + 3600);
        }

        let _ = revoked_count(); // triggers a prune

        let mut list = REVOKED_TOKENS
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            list.entries.contains_key(jti),
            "pruning must not drop an entry whose token is still valid"
        );
        list.entries.remove(jti);
    }

    #[test]
    fn test_revocations_survive_a_restart() {
        let _guard = revocation_guard();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("revocations.json");

        // First "run": configure a store and revoke a token.
        configure_revocation_store(&path).unwrap();
        let secret = "test-persistence-secret";
        let token = create_token(secret, 3600).unwrap();
        let claims = verify_token(&token, secret).unwrap();
        revoke_claims(&claims).unwrap();

        // Simulate a restart: wipe the in-memory list entirely, then reload.
        {
            let mut list = REVOKED_TOKENS
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            list.entries.clear();
            list.store = None;
        }
        assert!(
            verify_token(&token, secret).is_ok(),
            "precondition: with the list wiped and no store, the token verifies \
             again — this is exactly the bug persistence fixes"
        );

        configure_revocation_store(&path).unwrap();
        assert!(
            verify_token(&token, secret).is_err(),
            "a revocation recorded before the restart must still apply after it"
        );
    }

    #[test]
    fn test_a_corrupt_revocation_store_is_an_error_not_an_empty_list() {
        let _guard = revocation_guard();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("revocations.json");
        std::fs::write(&path, b"{not json").unwrap();

        let err = configure_revocation_store(&path).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(
            err.to_string().contains("corrupt"),
            "the operator needs to know the store was unreadable, not silently \
             start with zero revocations; got: {err}"
        );
    }

    #[test]
    fn test_a_missing_revocation_store_is_a_valid_empty_list() {
        let _guard = revocation_guard();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist-yet.json");

        // First run on a fresh volume must not fail.
        configure_revocation_store(&path).unwrap();
        assert!(path.exists(), "the store should be created on first use");
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
