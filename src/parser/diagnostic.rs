//! Structured parser diagnostics.
//!
//! Every parser failure is a [`ParseError`] with a stable `code`, an English
//! `message` (no location inside it) and a separate 1-based `line`/`col`.
//! `build --ai` serialises these fields directly; the string API
//! (`parser::parse`) renders them with [`ParseError`]'s `Display`.

use std::fmt;

/// Stable parser error codes. Documented in LANGUAGE.md §15.9.
pub mod codes {
    /// Unexpected token (the parser expected a different kind of token).
    pub const UNEXPECTED_TOKEN: &str = "PARSE_001";
    /// Identifier does not match `[a-zA-Z][a-zA-Z0-9_]{0,63}` (P040).
    pub const INVALID_IDENTIFIER: &str = "PARSE_002";
    /// Identifier is a SQL reserved word (P041).
    pub const RESERVED_IDENTIFIER: &str = "PARSE_003";
    /// `transition` block references a field that is missing or not an enum.
    pub const INVALID_TRANSITION_FIELD: &str = "PARSE_004";
    /// `transition` state is not a value of the enum.
    pub const INVALID_TRANSITION_STATE: &str = "PARSE_005";
    /// `on <event>` inside an entity is not create/update/delete.
    pub const INVALID_EFFECT_EVENT: &str = "PARSE_006";
    /// Unknown field type.
    pub const UNKNOWN_FIELD_TYPE: &str = "TYPE_001";
    /// Field declared without a type.
    pub const MISSING_FIELD_TYPE: &str = "TYPE_002";
    /// `match:"…"` is not a valid regular expression.
    pub const INVALID_PATTERN: &str = "FIELD_001";
    /// `min:` is greater than `max:` on the same field.
    pub const MIN_GREATER_THAN_MAX: &str = "FIELD_002";
    /// `min:`/`max:` value is not a number.
    pub const INVALID_BOUND: &str = "FIELD_003";
    /// `env { … }` variable type is not string/number/boolean/url/email.
    pub const UNSUPPORTED_ENV_TYPE: &str = "ENV_001";
    /// `env { … }` variable `default:` does not match its type.
    pub const INVALID_ENV_DEFAULT: &str = "ENV_002";
    /// `where` uses an operator that is not in the language.
    pub const UNKNOWN_FILTER_OP: &str = "BIND_001";
    /// `query` is not `all`, `one` or `count`.
    pub const UNKNOWN_QUERY: &str = "BIND_002";
    /// Field modifier is not a recognised keyword (`index`, not `indexed`).
    pub const UNKNOWN_FIELD_MODIFIER: &str = "FIELD_004";
    /// Action verb is unknown, or parsed but not executed (`validate`).
    pub const UNIMPLEMENTED_ACTION: &str = "ACTION_001";
    /// Top-level block that parses but has no runtime (`service`, `worker`, …).
    pub const UNIMPLEMENTED_BLOCK: &str = "LANG_001";
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub code: &'static str,
    /// English message, without code or location.
    pub message: String,
    /// 1-based line.
    pub line: usize,
    /// 1-based column (chars).
    pub col: usize,
    /// Width in chars of the offending token (0 when unknown, e.g. end of file).
    pub len: usize,
    /// Source text the fix applies to (usually the offending token).
    pub target: Option<String>,
    /// Concrete replacement text for `target`, when one can be derived.
    pub replacement: Option<String>,
    /// Free-form hint when no concrete replacement exists.
    pub hint: Option<String>,
}

impl ParseError {
    pub fn new(code: &'static str, message: impl Into<String>, line: usize, col: usize) -> Self {
        ParseError {
            code,
            message: message.into(),
            line: line.max(1),
            col: col.max(1),
            len: 0,
            target: None,
            replacement: None,
            hint: None,
        }
    }

    pub fn with_len(mut self, len: usize) -> Self {
        self.len = len;
        self
    }

    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    pub fn with_replacement(mut self, replacement: impl Into<String>) -> Self {
        self.replacement = Some(replacement.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// `true` for `message` text; used by tests that grep error content.
    pub fn contains(&self, needle: &str) -> bool {
        self.message.contains(needle)
    }
}

/// `<code>: <message> (line L, col C)`
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} (line {}, col {})",
            self.code, self.message, self.line, self.col
        )
    }
}

/// Render a list of diagnostics as the legacy single-string error.
pub fn join(errors: &[ParseError]) -> String {
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Edit distance over chars (optimal string alignment: insertions, deletions,
/// substitutions and adjacent transpositions each cost 1, so `tabel` → `table`
/// is 1, matching how people actually mistype).
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for j in 0..=b.len() {
        d[0][j] = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut best = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = best;
        }
    }
    d[a.len()][b.len()]
}

/// Closest candidate by case-insensitive edit distance. Only suggests when the
/// distance is small relative to the word (≤ 1/3 of its length, min 1, max 3).
pub fn closest<'a>(needle: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let lower = needle.to_lowercase();
    let budget = (lower.chars().count() / 3).clamp(1, 3);
    candidates
        .iter()
        .map(|c| (edit_distance(&lower, &c.to_lowercase()), *c))
        .filter(|(d, _)| *d <= budget)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_distance_counts_insertions_and_substitutions() {
        assert_eq!(edit_distance("strin", "string"), 1);
        assert_eq!(edit_distance("numbr", "number"), 1);
        assert_eq!(edit_distance("bolean", "boolean"), 1);
        assert_eq!(edit_distance("same", "same"), 0);
        assert_eq!(edit_distance("tabel", "table"), 1, "adjacent transposition");
        assert_eq!(edit_distance("nmuber", "number"), 1);
        assert_eq!(edit_distance("", "abc"), 3);
    }

    #[test]
    fn closest_suggests_only_near_matches() {
        let types = ["string", "number", "boolean", "date"];
        assert_eq!(closest("strin", &types), Some("string"));
        assert_eq!(closest("numbr", &types), Some("number"));
        assert_eq!(closest("String", &types), Some("string"));
        assert_eq!(closest("dat", &types), Some("date"));
        assert_eq!(closest("xyzzy", &types), None);
    }

    #[test]
    fn display_is_code_message_then_location() {
        let e = ParseError::new(
            codes::UNKNOWN_FIELD_TYPE,
            "unknown field type 'strin'",
            3,
            9,
        );
        assert_eq!(
            e.to_string(),
            "TYPE_001: unknown field type 'strin' (line 3, col 9)"
        );
    }
}
