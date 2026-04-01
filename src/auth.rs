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
        // Old format — cannot verify without reimplementing SHA-256
        // Return false to force password reset
        return false;
    }

    false
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

/// Default JWT secret (from env or generated per-run)
///
/// If JWT_SECRET is not set, generates a random secret and warns on stderr.
/// This means tokens are invalidated on restart — set JWT_SECRET for persistence.
pub fn default_secret() -> String {
    match std::env::var("JWT_SECRET") {
        Ok(s) if !s.is_empty() => s,
        _ => {
            eprintln!("\x1b[33m[CRONUS] WARNING: JWT_SECRET not set — using random per-run secret. Tokens will not survive restarts.\x1b[0m");
            eprintln!("\x1b[33m[CRONUS] Set JWT_SECRET env var for persistent sessions.\x1b[0m");
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now().hash(&mut hasher);
            std::process::id().hash(&mut hasher);
            format!("cronus-ephemeral-{:x}-{:x}", hasher.finish(), std::process::id())
        }
    }
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
