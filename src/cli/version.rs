//! `cronus --version` / `cronus version`.

/// Crate version from `Cargo.toml` — the single source of truth for release
/// tags (`v<VERSION>`) and for the binary pinned by `cronus deploy`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Commit the binary was built from. The release workflow sets
/// `CRONUS_GIT_SHA` at build time; local builds report `unknown`.
pub fn git_sha() -> &'static str {
    match option_env!("CRONUS_GIT_SHA") {
        Some(sha) if !sha.trim().is_empty() => sha.trim(),
        _ => "unknown",
    }
}

/// `cronus <version> (<git sha>)`
pub fn version_line() -> String {
    format!("cronus {} ({})", VERSION, git_sha())
}

pub fn cmd_version() {
    println!("{}", version_line());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_line_has_pkg_version_and_sha() {
        let line = version_line();
        assert_eq!(
            line,
            format!("cronus {} ({})", env!("CARGO_PKG_VERSION"), git_sha())
        );
        let sha = line
            .strip_prefix(&format!("cronus {} (", VERSION))
            .and_then(|rest| rest.strip_suffix(')'))
            .expect("format is `cronus <version> (<sha>)`");
        assert!(!sha.is_empty());
        assert!(!sha.contains(char::is_whitespace));
    }

    #[test]
    fn version_is_semver_like() {
        let core = VERSION.split(['-', '+']).next().unwrap();
        let parts: Vec<&str> = core.split('.').collect();
        assert_eq!(parts.len(), 3, "{VERSION}");
        assert!(parts.iter().all(|p| p.parse::<u64>().is_ok()), "{VERSION}");
    }
}
