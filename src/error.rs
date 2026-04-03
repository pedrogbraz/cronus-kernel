//! CRONUS unified error types.
//!
//! Replaces raw String errors with typed variants.
//! Migration is incremental — existing functions can continue returning
//! Result<T, String> while new code uses CronusResult<T>.

use std::fmt;

/// Unified error type for all CRONUS operations.
#[derive(Debug)]
pub enum CronusError {
    /// .cronus file parsing failed
    Parse(String),
    /// Symbol resolution failed (entity/field/route not found)
    Resolve(String),
    /// Lint rule violation
    Lint(String),
    /// Constitution rule violation
    Constitution(String),
    /// Database operation failed
    Database(String),
    /// Authentication/authorization error
    Auth(String),
    /// File I/O error
    Io(std::io::Error),
    /// HTTP error with status code
    Http(u16, String),
    /// Generic internal error
    Internal(String),
}

impl fmt::Display for CronusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CronusError::Parse(msg) => write!(f, "Parse error: {}", msg),
            CronusError::Resolve(msg) => write!(f, "Resolve error: {}", msg),
            CronusError::Lint(msg) => write!(f, "Lint error: {}", msg),
            CronusError::Constitution(msg) => write!(f, "Constitution violation: {}", msg),
            CronusError::Database(msg) => write!(f, "Database error: {}", msg),
            CronusError::Auth(msg) => write!(f, "Auth error: {}", msg),
            CronusError::Io(err) => write!(f, "I/O error: {}", err),
            CronusError::Http(code, msg) => write!(f, "HTTP {}: {}", code, msg),
            CronusError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for CronusError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CronusError::Io(err) => Some(err),
            _ => None,
        }
    }
}

// Conversions for ergonomic ? operator usage

impl From<std::io::Error> for CronusError {
    fn from(err: std::io::Error) -> Self {
        CronusError::Io(err)
    }
}

impl From<String> for CronusError {
    fn from(msg: String) -> Self {
        CronusError::Internal(msg)
    }
}

impl From<&str> for CronusError {
    fn from(msg: &str) -> Self {
        CronusError::Internal(msg.to_string())
    }
}

impl From<rusqlite::Error> for CronusError {
    fn from(err: rusqlite::Error) -> Self {
        CronusError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for CronusError {
    fn from(err: serde_json::Error) -> Self {
        CronusError::Internal(format!("JSON error: {}", err))
    }
}

/// Convenience type alias
pub type CronusResult<T> = Result<T, CronusError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let e = CronusError::Parse("unexpected token".into());
        assert_eq!(format!("{}", e), "Parse error: unexpected token");
    }

    #[test]
    fn error_from_string() {
        let e: CronusError = "something broke".into();
        assert!(format!("{}", e).contains("something broke"));
    }

    #[test]
    fn error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let e: CronusError = io_err.into();
        assert!(format!("{}", e).contains("file not found"));
    }

    #[test]
    fn result_type_works() {
        fn may_fail(ok: bool) -> CronusResult<i32> {
            if ok {
                Ok(42)
            } else {
                Err(CronusError::Database("connection lost".into()))
            }
        }
        assert_eq!(may_fail(true).unwrap(), 42);
        assert!(may_fail(false).is_err());
    }

    #[test]
    fn error_source_chain() {
        use std::error::Error;
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let e = CronusError::Io(io_err);
        assert!(e.source().is_some());

        let e2 = CronusError::Parse("bad".into());
        assert!(e2.source().is_none());
    }
}
