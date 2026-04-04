#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Auth Engine — JWT (jsonwebtoken) + Password Hashing (argon2)
//!
//! Production-grade auth using industry-standard crates.
//! JWT via HS256, passwords via Argon2id.

use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

// ══════════════════════════════════════════════════
// JWT
// ══════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

/// Create a JWT token (HS256, 7-day expiry)
pub fn create_token(user_id: &str, role: &str, secret: &str) -> String {
    let exp = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() + 7 * 24 * 3600) as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .unwrap_or_default()
}

/// Verify a JWT token, returns Claims if valid
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| e.to_string())
}

// ══════════════════════════════════════════════════
// PASSWORD HASHING (Argon2id)
// ══════════════════════════════════════════════════

/// Hash a password using Argon2id (industry standard)
pub fn hash_password(password: &str) -> String {
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::SaltString;
    use rand_core::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .unwrap_or_else(|e| panic!("CRITICAL: Argon2 hashing failed — refusing to store password: {}", e))
}

/// Verify a password against an Argon2 hash string
pub fn verify_password(password: &str, stored: &str) -> bool {
    use argon2::{Argon2, PasswordVerifier};
    use argon2::password_hash::PasswordHash;

    // Try Argon2 format first
    if let Ok(hash) = PasswordHash::new(stored) {
        return Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok();
    }

    // Fallback: old SHA-256 "salt:hash" format (for migration)
    if stored.contains(':') && !stored.starts_with('$') {
        return verify_sha256_legacy(password, stored);
    }

    // Fallback: plain SHA-256 hex (no salt, very old format)
    if stored.len() == 64 && stored.chars().all(|c| c.is_ascii_hexdigit()) {
        use sha2::{Sha256, Digest};
        let hash = hex::encode(Sha256::digest(password.as_bytes()));
        return hash == stored;
    }

    false
}

/// Verify a password against a legacy SHA-256 "salt:hash" format.
/// Tries both `sha256(salt + password)` and `sha256(password + salt)`.
fn verify_sha256_legacy(password: &str, stored: &str) -> bool {
    use sha2::{Sha256, Digest};

    let Some((salt, expected_hash)) = stored.split_once(':') else {
        return false;
    };

    // Try salt + password
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let candidate = hex::encode(hasher.finalize());
    if candidate == expected_hash {
        return true;
    }

    // Try password + salt
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let candidate = hex::encode(hasher.finalize());
    candidate == expected_hash
}

// ══════════════════════════════════════════════════
// AUTH MIDDLEWARE
// ══════════════════════════════════════════════════

/// Extract user claims from Authorization header
pub fn extract_user(auth_header: Option<&str>, secret: &str) -> Option<Claims> {
    let header = auth_header?;
    let token = if header.starts_with("Bearer ") {
        &header[7..]
    } else {
        header
    };
    verify_token(token, secret).ok()
}

/// Check if claims have the required role
pub fn require_role(claims: &Claims, role: &str) -> bool {
    if role == "public" { return true; }
    if role == "jwt" { return true; } // any authenticated user
    claims.role == role || claims.role == "admin"
}

/// Default JWT secret — persisted to `.cronus/jwt.key` so sessions survive restarts.
///
/// Priority: JWT_SECRET env var > .cronus/jwt.key file > generate + save.
pub fn default_secret() -> String {
    // 1. Check env var
    if let Ok(s) = std::env::var("JWT_SECRET") {
        if !s.is_empty() { return s; }
    }

    use std::sync::OnceLock;
    static SECRET: OnceLock<String> = OnceLock::new();
    SECRET.get_or_init(|| {
        let key_path = std::path::Path::new(".cronus/jwt.key");

        // 2. Read from persisted file
        if let Ok(s) = std::fs::read_to_string(key_path) {
            let s = s.trim().to_string();
            if !s.is_empty() { return s; }
        }

        // 3. Generate and persist
        use rand::Rng;
        let secret: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        // Save to file
        let _ = std::fs::create_dir_all(".cronus");
        if std::fs::write(key_path, &secret).is_ok() {
            eprintln!("  \x1b[32m✓\x1b[0m JWT secret persisted to .cronus/jwt.key");
        }

        secret
    }).clone()
}

// ══════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_roundtrip() {
        let secret = "test-secret-key";
        let token = create_token("user123", "admin", secret);
        assert!(!token.is_empty());

        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_jwt_invalid_secret() {
        let token = create_token("user123", "admin", "secret1");
        let result = verify_token(&token, "secret2");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_hash_verify() {
        let password = "MyStr0ngP@ss!";
        let hash = hash_password(password);

        // Hash should be Argon2 format
        assert!(hash.starts_with("$argon2"));

        // Verify should work
        assert!(verify_password(password, &hash));

        // Wrong password should fail
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn test_password_verify_sha256_legacy_salted() {
        use sha2::{Sha256, Digest};
        // Simulate legacy salt:hash format (salt + password)
        let salt = "randomsalt123";
        let password = "MyOldPassword!";
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(password.as_bytes());
        let hash = hex::encode(hasher.finalize());
        let stored = format!("{}:{}", salt, hash);

        assert!(verify_password(password, &stored));
        assert!(!verify_password("wrong", &stored));
    }

    #[test]
    fn test_password_verify_sha256_plain_hex() {
        use sha2::{Sha256, Digest};
        // Plain SHA-256 hex (no salt)
        let password = "PlainHash123";
        let hash = hex::encode(Sha256::digest(password.as_bytes()));

        assert!(verify_password(password, &hash));
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn test_extract_user() {
        let secret = "test-secret";
        let token = create_token("u1", "user", secret);

        // With Bearer prefix
        let claims = extract_user(Some(&format!("Bearer {}", token)), secret);
        assert!(claims.is_some());
        assert_eq!(claims.unwrap().sub, "u1");

        // Without prefix
        let claims = extract_user(Some(&token), secret);
        assert!(claims.is_some());

        // None header
        assert!(extract_user(None, secret).is_none());
    }

    #[test]
    fn test_require_role() {
        let claims = Claims { sub: "u1".into(), role: "admin".into(), exp: 0 };
        assert!(require_role(&claims, "admin"));
        assert!(require_role(&claims, "user")); // admin can do anything
        assert!(require_role(&claims, "jwt"));
        assert!(require_role(&claims, "public"));

        let user_claims = Claims { sub: "u2".into(), role: "user".into(), exp: 0 };
        assert!(require_role(&user_claims, "user"));
        assert!(!require_role(&user_claims, "admin"));
    }
}
