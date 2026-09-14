#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Auth Engine — JWT (jsonwebtoken) + Password Hashing (argon2)
//!
//! Production-grade auth using industry-standard crates.
//! JWT via HS256, passwords via Argon2id.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// ══════════════════════════════════════════════════
// JWT
// ══════════════════════════════════════════════════

/// Session JWT claims. `iat` and `jti` are required: tokens minted before
/// they existed (7-day, JS-readable) no longer verify and force a re-login.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
    /// Unique token id — the hook for a future server-side revocation list.
    pub jti: String,
}

pub const DEFAULT_SESSION_TTL_SECS: u64 = 24 * 3600;

/// Session lifetime from `auth { session jwt expires:<dur> }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionPolicy {
    pub ttl_secs: u64,
}

impl Default for SessionPolicy {
    fn default() -> Self {
        Self {
            ttl_secs: DEFAULT_SESSION_TTL_SECS,
        }
    }
}

impl SessionPolicy {
    /// Validated once at startup from the auth block's session config.
    pub fn from_session_config(
        config: &std::collections::HashMap<String, String>,
    ) -> Result<Self, String> {
        match config.get("expires") {
            None => Ok(Self::default()),
            Some(raw) => parse_duration_secs(raw)
                .map(|ttl_secs| Self { ttl_secs })
                .map_err(|e| format!("auth session expires:{}: {}", raw, e)),
        }
    }
}

/// Parse `30s`, `15m`, `24h`, `7d`, `2w`. Zero and unknown units are rejected.
pub fn parse_duration_secs(raw: &str) -> Result<u64, String> {
    let raw = raw.trim();
    let split = raw.find(|c: char| !c.is_ascii_digit()).unwrap_or(raw.len());
    let (digits, unit) = raw.split_at(split);
    let amount: u64 = digits
        .parse()
        .map_err(|_| "expected <number><unit>, e.g. 24h".to_string())?;
    let unit_secs = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86_400,
        "w" => 604_800,
        _ => return Err("unit must be one of s, m, h, d, w".to_string()),
    };
    if amount == 0 {
        return Err("duration must be greater than zero".to_string());
    }
    amount
        .checked_mul(unit_secs)
        .ok_or_else(|| "duration too large".to_string())
}

fn new_jti() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Create a session JWT (HS256) that expires after `ttl_secs`.
pub fn create_session_token(user_id: &str, role: &str, secret: &str, ttl_secs: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: (now + ttl_secs) as usize,
        iat: now as usize,
        jti: new_jti(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap_or_default()
}

/// Create a JWT with the default session lifetime (24h).
pub fn create_token(user_id: &str, role: &str, secret: &str) -> String {
    create_session_token(user_id, role, secret, DEFAULT_SESSION_TTL_SECS)
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

/// Minimum length for a single-factor password (NIST SP 800-63B-4).
/// No composition rules and no periodic expiry, by design.
pub const MIN_PASSWORD_CHARS: usize = 15;

/// Argon2id memory cost in KiB, iterations and parallelism.
const ARGON2_M_COST: u32 = 19_456;
const ARGON2_T_COST: u32 = 2;
const ARGON2_P_COST: u32 = 1;

/// Argon2id with explicit parameters (m=19456, t=2, p=1), so a crate
/// default change can never silently weaken new hashes.
fn argon2id() -> argon2::Argon2<'static> {
    use argon2::{Algorithm, Argon2, Params, Version};
    let params =
        Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, None).unwrap_or(Params::DEFAULT);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// Signup-time policy: at least `MIN_PASSWORD_CHARS` characters (Unicode
/// scalar values), nothing else.
pub fn validate_new_password(password: &str) -> Result<(), String> {
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(format!(
            "password must be at least {} characters",
            MIN_PASSWORD_CHARS
        ));
    }
    Ok(())
}

/// Hash a password using Argon2id (industry standard)
pub fn hash_password(password: &str) -> String {
    use argon2::password_hash::SaltString;
    use argon2::PasswordHasher;
    use rand_core::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    argon2id()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .unwrap_or_else(|e| {
            panic!(
                "CRITICAL: Argon2 hashing failed — refusing to store password: {}",
                e
            )
        })
}

/// Verify a password against a PHC-format Argon2 hash string. Anything
/// else (including legacy unsalted/salted SHA-256 hex) is rejected.
pub fn verify_password(password: &str, stored: &str) -> bool {
    use argon2::password_hash::PasswordHash;
    use argon2::PasswordVerifier;

    match PasswordHash::new(stored) {
        Ok(hash) if hash.algorithm.as_str().starts_with("argon2") => argon2id()
            .verify_password(password.as_bytes(), &hash)
            .is_ok(),
        _ => false,
    }
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
    if role == "public" {
        return true;
    }
    if role == "jwt" {
        return true;
    } // any authenticated user
    claims.role == role || claims.role == "admin"
}

/// Minimum length for any HMAC signing secret supplied by the operator
/// (HS256 key should carry at least 256 bits).
pub const MIN_SECRET_BYTES: usize = 32;

/// Validates the `JWT_SECRET` env value. Unset or empty → `Ok(None)`
/// (fall back to the key file). Set but shorter than [`MIN_SECRET_BYTES`] → `Err`.
pub fn secret_from_env(value: Option<&str>) -> Result<Option<String>, String> {
    match value {
        None | Some("") => Ok(None),
        Some(s) if s.len() < MIN_SECRET_BYTES => Err(format!(
            "JWT_SECRET is {} bytes; it must be at least {} bytes (generate one with `openssl rand -base64 48`)",
            s.len(),
            MIN_SECRET_BYTES
        )),
        Some(s) => Ok(Some(s.to_string())),
    }
}

/// Startup check: call before binding the server so a weak `JWT_SECRET`
/// refuses to start instead of silently signing tokens with it.
pub fn check_secret_config() -> Result<(), String> {
    secret_from_env(std::env::var("JWT_SECRET").ok().as_deref()).map(|_| ())
}

/// Default JWT secret — persisted to `.cronus/jwt.key` so sessions survive restarts.
///
/// Priority: JWT_SECRET env var (>= 32 bytes) > .cronus/jwt.key file > generate + save (mode 0600).
pub fn default_secret() -> String {
    match secret_from_env(std::env::var("JWT_SECRET").ok().as_deref()) {
        Ok(Some(s)) => return s,
        Ok(None) => {}
        Err(e) => {
            // Unreachable after `check_secret_config` at startup; never sign with a weak key.
            eprintln!("  \x1b[31m✗\x1b[0m {}", e);
            std::process::exit(1);
        }
    }

    use std::sync::OnceLock;
    static SECRET: OnceLock<String> = OnceLock::new();
    SECRET
        .get_or_init(|| load_or_create_key_file(std::path::Path::new(".cronus/jwt.key")))
        .clone()
}

/// Reads a persisted secret, or generates 64 random alphanumerics and writes
/// them with owner-only permissions. An existing group/world-readable key file
/// is tightened to 0600. If persisting fails the secret is still returned
/// (valid for this process only).
pub fn load_or_create_key_file(path: &std::path::Path) -> String {
    if let Ok(s) = std::fs::read_to_string(path) {
        let s = s.trim().to_string();
        if !s.is_empty() {
            restrict_key_file_permissions(path);
            return s;
        }
    }

    use rand::Rng;
    let secret: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    match write_key_file(path, &secret) {
        Ok(()) => eprintln!("  \x1b[32m✓\x1b[0m Secret persisted to {} (mode 0600)", path.display()),
        Err(e) => eprintln!(
            "  \x1b[33m⚠\x1b[0m Could not persist secret to {} ({}); sessions will not survive restart",
            path.display(),
            e.kind()
        ),
    }
    secret
}

/// Creates `path` exclusively with mode 0600 (unix). Never overwrites.
pub fn write_key_file(path: &std::path::Path, secret: &str) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut file = opts.open(path)?;
    file.write_all(secret.as_bytes())?;
    file.sync_all()
}

#[cfg(unix)]
fn restrict_key_file_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.permissions().mode() & 0o077 != 0 {
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
            eprintln!(
                "  \x1b[33m⚠\x1b[0m {} was readable by other users; permissions set to 0600",
                path.display()
            );
        }
    }
}

#[cfg(not(unix))]
fn restrict_key_file_permissions(_path: &std::path::Path) {}

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
    fn test_password_hash_uses_explicit_argon2id_params() {
        let hash = hash_password("correct horse battery staple");
        assert!(
            hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"),
            "unexpected PHC string: {hash}"
        );
    }

    #[test]
    fn test_password_verify_rejects_sha256_legacy_salted() {
        use sha2::{Digest, Sha256};
        let salt = "randomsalt123";
        let password = "MyOldPassword!";
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(password.as_bytes());
        let stored = format!("{}:{}", salt, hex::encode(hasher.finalize()));

        assert!(!verify_password(password, &stored));
    }

    #[test]
    fn test_password_verify_rejects_sha256_plain_hex() {
        use sha2::{Digest, Sha256};
        let password = "PlainHash123";
        let hash = hex::encode(Sha256::digest(password.as_bytes()));

        assert!(!verify_password(password, &hash));
        assert!(!verify_password("", ""));
    }

    #[test]
    fn test_validate_new_password_length_only() {
        assert!(validate_new_password("").is_err());
        assert!(validate_new_password("fourteen-chars").is_err());
        assert!(validate_new_password("fifteen-chars!!").is_ok());
        // No composition rules: 15 lowercase letters are fine.
        assert!(validate_new_password("aaaaaaaaaaaaaaa").is_ok());
        // Counts characters, not bytes: 14 multi-byte chars is still too short.
        assert!(validate_new_password("ééééééééééééé").is_err());
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

    fn scratch_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "cronus-auth-{}-{}-{}",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn jwt_secret_env_requires_32_bytes() {
        assert_eq!(secret_from_env(None), Ok(None));
        assert_eq!(secret_from_env(Some("")), Ok(None));
        let err = secret_from_env(Some("short-secret")).unwrap_err();
        assert!(err.contains("at least 32 bytes"));
        assert!(
            !err.contains("short-secret"),
            "error must not echo the secret"
        );
        assert!(secret_from_env(Some(&"x".repeat(31))).is_err());
        let ok = "k".repeat(32);
        assert_eq!(secret_from_env(Some(&ok)), Ok(Some(ok.clone())));
    }

    #[cfg(unix)]
    #[test]
    fn generated_key_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch_dir("mode");
        let path = dir.join(".cronus").join("jwt.key");
        let secret = load_or_create_key_file(&path);
        assert_eq!(secret.len(), 64);
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "key file mode was {:o}", mode);
        // second load returns the persisted value
        assert_eq!(load_or_create_key_file(&path), secret);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn existing_world_readable_key_file_is_tightened() {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch_dir("tighten");
        let path = dir.join("jwt.key");
        std::fs::write(&path, "persisted-secret").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(load_or_create_key_file(&path), "persisted-secret");
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn write_key_file_never_overwrites() {
        let dir = scratch_dir("excl");
        let path = dir.join("k");
        write_key_file(&path, "one").unwrap();
        assert!(write_key_file(&path, "two").is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "one");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_require_role() {
        let claims = Claims {
            sub: "u1".into(),
            role: "admin".into(),
            exp: 0,
            iat: 0,
            jti: "t1".into(),
        };
        assert!(require_role(&claims, "admin"));
        assert!(require_role(&claims, "user")); // admin can do anything
        assert!(require_role(&claims, "jwt"));
        assert!(require_role(&claims, "public"));

        let user_claims = Claims {
            sub: "u2".into(),
            role: "user".into(),
            exp: 0,
            iat: 0,
            jti: "t2".into(),
        };
        assert!(require_role(&user_claims, "user"));
        assert!(!require_role(&user_claims, "admin"));
    }

    #[test]
    fn session_token_carries_iat_jti_and_policy_lifetime() {
        let secret = "test-secret";
        let a = verify_token(&create_session_token("u1", "user", secret, 3600), secret).unwrap();
        let b = verify_token(&create_session_token("u1", "user", secret, 3600), secret).unwrap();
        assert_eq!(a.exp - a.iat, 3600);
        assert_eq!(a.jti.len(), 32);
        assert_ne!(a.jti, b.jti, "every token gets a fresh jti");
        let default = verify_token(&create_token("u1", "user", secret), secret).unwrap();
        assert_eq!((default.exp - default.iat) as u64, DEFAULT_SESSION_TTL_SECS);
    }

    #[test]
    fn token_without_iat_or_jti_is_rejected() {
        #[derive(Serialize)]
        struct Legacy {
            sub: String,
            role: String,
            exp: usize,
        }
        let secret = "test-secret";
        let exp = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600) as usize;
        let legacy = encode(
            &Header::default(),
            &Legacy {
                sub: "u1".into(),
                role: "admin".into(),
                exp,
            },
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();
        assert!(verify_token(&legacy, secret).is_err());
    }

    #[test]
    fn session_policy_reads_expires() {
        let mut cfg = std::collections::HashMap::new();
        assert_eq!(
            SessionPolicy::from_session_config(&cfg).unwrap().ttl_secs,
            DEFAULT_SESSION_TTL_SECS
        );
        for (raw, secs) in [
            ("30s", 30),
            ("15m", 900),
            ("24h", 86_400),
            ("7d", 604_800),
            ("2w", 1_209_600),
        ] {
            cfg.insert("expires".to_string(), raw.to_string());
            assert_eq!(
                SessionPolicy::from_session_config(&cfg).unwrap().ttl_secs,
                secs,
                "{raw}"
            );
        }
        for bad in ["0h", "h", "24", "10y", "-1h"] {
            cfg.insert("expires".to_string(), bad.to_string());
            assert!(SessionPolicy::from_session_config(&cfg).is_err(), "{bad}");
        }
    }
}
