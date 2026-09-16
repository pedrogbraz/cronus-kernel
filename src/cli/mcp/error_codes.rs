//! Explanations for `cronus build --ai` error codes (the `explain_error` tool).
//!
//! Single source for code explanations. The test below extracts every code
//! from the "Error codes" table of LANGUAGE.md §15.9 (expanding ranges such as
//! `LINT_001`…`LINT_011`) and fails when the two sets differ, so a code added
//! to or removed from the build report must be updated in both places.

use serde_json::{json, Value};

pub(crate) struct ErrorCode {
    pub code: &'static str,
    pub severity: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub example_fix: &'static str,
}

impl ErrorCode {
    pub fn to_json(&self) -> Value {
        json!({
            "code": self.code,
            "severity": self.severity,
            "category": self.category,
            "description": self.description,
            "example_fix": self.example_fix,
            "reference": "LANGUAGE.md §15.9 (resource cronus://language)",
        })
    }
}

const STRICT: &str = "error in strict profile (build --ai, validate); warning in plain build";
const FROM_RULE: &str = "from the lint rule; strict profile promotes warnings to errors";

pub(crate) const ERROR_CODES: &[ErrorCode] = &[
    ErrorCode {
        code: "IO_001",
        severity: "error (exit 2)",
        category: "usage",
        description: "No .cronus file was found, or the file could not be read.",
        example_fix: "Pass an existing file: `cronus build app.cronus`. Over MCP, send the source text to `validate` instead of a path.",
    },
    ErrorCode {
        code: "PARSE_001",
        severity: "error",
        category: "parse",
        description: "Unexpected token: the parser expected one construct and found another (`expected X, found 'Y'`). Parsing stops at the first such error.",
        example_fix: "`list HEAD /` -> `list GET /` (only GET, POST, PUT, PATCH, DELETE are methods). Otherwise look for a missing `{`, `}` or keyword just before the reported position.",
    },
    ErrorCode {
        code: "PARSE_002",
        severity: "error",
        category: "parse",
        description: "Invalid identifier shape for an entity, field or other name (rule P040).",
        example_fix: "`entity Order-Item {` -> `entity OrderItem {` (letters, digits and `_`).",
    },
    ErrorCode {
        code: "PARSE_003",
        severity: "error",
        category: "parse",
        description: "SQL reserved word (SELECT, DROP, INSERT, DELETE, UPDATE, TABLE, FROM) used as an entity or field name (rule P041).",
        example_fix: "`from string!` -> `sender string!`",
    },
    ErrorCode {
        code: "PARSE_004",
        severity: "error",
        category: "parse",
        description: "`transition` names a field that does not exist or is not an enum.",
        example_fix: "Declare the enum first: `status enum [draft, published]`, then `transition status { draft -> published }`.",
    },
    ErrorCode {
        code: "PARSE_005",
        severity: "error",
        category: "parse",
        description: "A `transition` state or target is not one of the field's enum values.",
        example_fix: "With `status enum [draft, published]`: `draft -> publised` -> `draft -> published`.",
    },
    ErrorCode {
        code: "PARSE_006",
        severity: "error",
        category: "parse",
        description: "`on <event>` inside an entity uses an event other than create, update or delete.",
        example_fix: "`on save { log \"saved\" }` -> `on update { log \"saved\" }`",
    },
    ErrorCode {
        code: "TYPE_001",
        severity: "error",
        category: "type",
        description: "Unknown field type. Valid keywords are listed by the `list_field_types` tool; the fix suggests the closest one.",
        example_fix: "`title strin!` -> `title string!`",
    },
    ErrorCode {
        code: "TYPE_002",
        severity: "error",
        category: "type",
        description: "A field is declared without a type.",
        example_fix: "`title` -> `title string!`",
    },
    ErrorCode {
        code: "STRUCTURE_001",
        severity: "error",
        category: "structure",
        description: "A `page \"...\"` or `entity Name {` present in the source is missing from the parsed app, usually because an earlier block consumed its closing `}`.",
        example_fix: "Balance the braces of the block just before the missing page or entity so every `{` has its own `}`.",
    },
    ErrorCode {
        code: "RESOLVE_001",
        severity: "error",
        category: "resolve",
        description: "Unresolved reference: an entity, field, column or route named in a binding, column list, relation or link does not exist.",
        example_fix: "`bind Tsk { query all }` -> `bind Task { query all }` (the fix suggests a close existing name when there is one).",
    },
    ErrorCode {
        code: "RESOLVE_002",
        severity: "error",
        category: "resolve",
        description: "State-machine reference error: the transition's field is missing or is not an enum.",
        example_fix: "Make the field an enum that lists every state the transition uses: `stage enum [lead, won]`.",
    },
    ErrorCode {
        code: "CONTRACT_001",
        severity: STRICT,
        category: "contract",
        description: "Unknown section type. Valid types are listed by the `list_sections` tool; the fix suggests the closest canonical type.",
        example_fix: "`section tabel { ... }` -> `section table { ... }`",
    },
    ErrorCode {
        code: "CONTRACT_002",
        severity: STRICT,
        category: "contract",
        description: "Unknown key on a section item.",
        example_fix: "Remove the key named in the message from the `item`, or rename it to a key that section accepts.",
    },
    ErrorCode {
        code: "CONTRACT_003",
        severity: STRICT,
        category: "contract",
        description: "A section item is missing a required key.",
        example_fix: "Add the key named in the message to that `item` line.",
    },
    ErrorCode {
        code: "CONTRACT_004",
        severity: "warning",
        category: "contract",
        description: "The section type is an alias; use its canonical name.",
        example_fix: "`section stats { ... }` -> `section kpi { ... }`",
    },
    ErrorCode {
        code: "CONTRACT_005",
        severity: STRICT,
        category: "contract",
        description: "The section has fewer items than its contract requires.",
        example_fix: "Add `item` lines (or a `bind`) until the minimum given in the message is met.",
    },
    ErrorCode {
        code: "CONTRACT_006",
        severity: STRICT,
        category: "contract",
        description: "Unknown key in a section's configuration.",
        example_fix: "`section table { bind Task { query all } columns \"title\" bogus:1 }` -> remove `bogus:1`.",
    },
    ErrorCode {
        code: "LINT_001",
        severity: FROM_RULE,
        category: "lint (no-dead-text)",
        description: "Rendered text looks like a hardcoded metric instead of data.",
        example_fix: "Replace the literal number with a `kpi` item bound to data: `section kpi { bind Order { aggregate count } item \"Orders\" value:bind }`.",
    },
    ErrorCode {
        code: "LINT_002",
        severity: FROM_RULE,
        category: "lint (no-dead-links)",
        description: "A link is dead (`#`, empty) or points to a route with no page.",
        example_fix: "Point the link at a declared `page \"/reports\"`, or remove it.",
    },
    ErrorCode {
        code: "LINT_003",
        severity: FROM_RULE,
        category: "lint (bind-or-empty)",
        description: "A data section has no data source: no `bind`, no items and no template.",
        example_fix: "`section table { columns \"title\" }` -> `section table { bind Task { query all } columns \"title\" }`",
    },
    ErrorCode {
        code: "LINT_004",
        severity: FROM_RULE,
        category: "lint (no-sensitive-render)",
        description: "A `sensitive` field is listed in columns or appears in rendered output.",
        example_fix: "`columns \"email, password\"` -> `columns \"email\"`",
    },
    ErrorCode {
        code: "LINT_005",
        severity: FROM_RULE,
        category: "lint (no-sensitive-select)",
        description: "A binding would select, filter, order or group by a `sensitive` field.",
        example_fix: "`bind User { query all order password }` -> `bind User { query all order email }`",
    },
    ErrorCode {
        code: "LINT_006",
        severity: FROM_RULE,
        category: "lint (no-hardcode-user)",
        description: "User or role text is hardcoded in the page instead of coming from the session.",
        example_fix: "Remove the literal user name or role; show the signed-in user through the layout or a binding.",
    },
    ErrorCode {
        code: "LINT_007",
        severity: FROM_RULE,
        category: "lint (no-fake-state)",
        description: "Static text implies live state (for example \"online\" or \"syncing\") without data behind it.",
        example_fix: "Bind the value to an entity field (`bind Server { query all }`) or remove the status text.",
    },
    ErrorCode {
        code: "LINT_008",
        severity: FROM_RULE,
        category: "lint (no-dead-ui)",
        description: "A button has no action handler.",
        example_fix: "Give it an action block, e.g. `on click { navigate \"/deals\" }`, or remove the button.",
    },
    ErrorCode {
        code: "LINT_009",
        severity: FROM_RULE,
        category: "lint (no-orphan-reload)",
        description: "A template calls `location.reload()`, which breaks the page runtime.",
        example_fix: "Remove the script from the template; use the `refresh` action verb: `on submit { create Task  refresh }`.",
    },
    ErrorCode {
        code: "LINT_010",
        severity: FROM_RULE,
        category: "lint (form-submit-handler)",
        description: "A form section has no submit handler.",
        example_fix: "`section form { bind Task }` -> `section form { bind Task  on submit { create Task  toast \"Saved\" success } }`",
    },
    ErrorCode {
        code: "LINT_011",
        severity: FROM_RULE,
        category: "lint (shared-entity-auth)",
        description: "A route of a `shared` entity's API has no auth.",
        example_fix: "`create POST / auth:public` -> `create POST / auth:jwt`",
    },
    ErrorCode {
        code: "LINT_020",
        severity: "from the finding (strict profile only)",
        category: "lint (hardcoded content)",
        description: "Hardcoded text found in the rendered HTML.",
        example_fix: "Move literal copy into section props or items, or bind it to entity data.",
    },
    ErrorCode {
        code: "LINT_099",
        severity: FROM_RULE,
        category: "lint",
        description: "Any other lint rule; the `rule` key of the diagnostic names it.",
        example_fix: "Follow the `fix.hint` of the diagnostic for that rule.",
    },
    ErrorCode {
        code: "CONSTITUTION_001",
        severity: STRICT,
        category: "constitution",
        description: "A constitution `must` or `never` rule is violated; the `rule` key names it.",
        example_fix: "Change the app so the named rule holds, or change the constitution if the rule itself is wrong.",
    },
    ErrorCode {
        code: "FIELD_001",
        severity: "error",
        category: "validation",
        description: "A field's `match:\"…\"` pattern is not a valid regular expression (Rust regex syntax: no look-around or backreferences).",
        example_fix: "`slug slug match:\"^[a-z\"` -> `slug slug match:\"^[a-z0-9-]+$\"` (close the character class).",
    },
    ErrorCode {
        code: "FIELD_002",
        severity: "error",
        category: "validation",
        description: "`min:` is greater than `max:` on the same field (numeric bound, or length for text-like types).",
        example_fix: "`age number min:150 max:0` -> `age number min:0 max:150`.",
    },
    ErrorCode {
        code: "FIELD_003",
        severity: "error",
        category: "validation",
        description: "A `min:` or `max:` value is not a number.",
        example_fix: "`title string! min:three` -> `title string! min:3`.",
    },
    ErrorCode {
        code: "ENV_001",
        severity: "error",
        category: "env",
        description: "An `env { … }` variable declares a type other than string, number, boolean, url or email.",
        example_fix: "`APP_START date` -> `APP_START string` (parse the value in the app).",
    },
    ErrorCode {
        code: "ENV_002",
        severity: "error",
        category: "env",
        description: "An `env { … }` variable's `default:` does not match its declared type.",
        example_fix: "`APP_FEATURE_X boolean default:maybe` -> `APP_FEATURE_X boolean default:false`.",
    },
    ErrorCode {
        code: "ENV_003",
        severity: "warning",
        category: "env",
        description: "A declared `env` variable name is not SCREAMING_SNAKE with a prefix such as `APP_`, so it may collide with system variables.",
        example_fix: "`STRIPE_KEY string!` is fine; `stripe string!` -> `APP_STRIPE string!`.",
    },
    ErrorCode {
        code: "BIND_001",
        severity: "error",
        category: "bind",
        description: "A `where` clause uses an operator that is not in the language. Unknown operators used to be treated as `eq`.",
        example_fix: "`where status blah:\"paid\"` → `where status eq:\"paid\"`. Supported: eq, ne (neq), gt, gte, lt, lte, contains, starts_with, ends_with, in:[…].",
    },
    ErrorCode {
        code: "BIND_002",
        severity: "error",
        category: "bind",
        description: "`query` is not `all`, `one` or `count`. Unknown query kinds used to be treated as `all`.",
        example_fix: "`query every` -> `query all`.",
    },
    ErrorCode {
        code: "FIELD_004",
        severity: "error",
        category: "validation",
        description: "A field modifier is not recognised. Dead spellings such as `indexed`, `computed` and `onupdate:` used to be ignored.",
        example_fix: "`email string indexed` -> `email string index`. Drop `computed` and `onupdate:`.",
    },
    ErrorCode {
        code: "ACTION_001",
        severity: "error",
        category: "action",
        description: "An action verb is unknown, or is parsed but not executed (`validate`). Invented verbs used to be skipped.",
        example_fix: "`on click { log \"x\" }` -> `on click { toast \"x\" info }`. Implemented: set, toast, navigate, refresh, delete, open, close. `create`/`update` still parse (forms) and are not this error.",
    },
    ErrorCode {
        code: "LANG_001",
        severity: "error",
        category: "language",
        description: "A top-level block is accepted by the parser so the rest of the file can be checked, but the runtime does not implement it (`service`, `worker`, `middleware`, `deploy`, `test`, top-level `on`).",
        example_fix: "Remove the block. Page `requires:`, `import`, `compose { use }`, `define` + page `use`, and `webhook` are the implemented substitutes where they apply.",
    },
    ErrorCode {
        code: "COMPOSE_001",
        severity: "error",
        category: "compose",
        description: "The same declaration appears twice across the load graph (duplicate entity name, page route, app, auth, style, layout name, api prefix, component name, webhook entity, or env variable). Last-wins is gone; the first is kept and every later collision is reported. Exactly one `app {}`.",
        example_fix: "Rename or remove the later block. Split files with `import \"entities\"` / `compose { use entities }`; do not repeat `entity Task` in two files.",
    },
    ErrorCode {
        code: "COMPOSE_002",
        severity: "error",
        category: "compose",
        description: "`import` or `compose { use }` / `merge` points at a file that is not on disk. Missing imports used to warn and continue.",
        example_fix: "`import \"entities\"` looks for `entities.cronus` next to the importing file (nested imports are relative to the importer). Create the file or drop the import.",
    },
    ErrorCode {
        code: "REL_001",
        severity: "error",
        category: "relation",
        description: "`jobs <- Job.client` does not name a relation on `Job` that points at this entity, collides with a field, or repeats the same pair.",
        example_fix: "`entity Customer { jobs <- Job.client }` requires `entity Job { client -> Customer }`. The declared name is used for `expand:jobs` instead of the inferred `jobs`/`orders`.",
    },
];

/// Case-insensitive lookup (`type_001` finds `TYPE_001`).
pub(crate) fn lookup(code: &str) -> Option<&'static ErrorCode> {
    let wanted = code.trim();
    ERROR_CODES
        .iter()
        .find(|e| e.code.eq_ignore_ascii_case(wanted))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::context_grammar::code_spans;
    use std::collections::BTreeSet;

    /// `UPPER_123` shape of a build diagnostic code.
    fn is_code(s: &str) -> bool {
        s.rsplit_once('_').is_some_and(|(prefix, n)| {
            !prefix.is_empty()
                && prefix.chars().all(|c| c.is_ascii_uppercase())
                && n.len() == 3
                && n.chars().all(|c| c.is_ascii_digit())
        })
    }

    /// Codes in LANGUAGE.md §15.9's "Error codes" table.
    fn documented_codes() -> BTreeSet<String> {
        let doc = include_str!("../../../LANGUAGE.md");
        let start = doc
            .find("**Error codes:**")
            .expect("LANGUAGE.md §15.9 has no **Error codes:** table");
        let mut codes = BTreeSet::new();
        for line in doc[start..].lines().skip(1) {
            let line = line.trim();
            if line.is_empty() && !codes.is_empty() {
                break;
            }
            if !line.starts_with("| `") {
                continue;
            }
            let cell = line.split('|').nth(1).unwrap_or("");
            let spans = code_spans(cell);
            if cell.contains('…') && spans.len() == 2 {
                let (prefix, from) = spans[0].rsplit_once('_').expect("range start");
                let (_, to) = spans[1].rsplit_once('_').expect("range end");
                let width = from.len();
                let (from, to): (u32, u32) = (from.parse().unwrap(), to.parse().unwrap());
                for n in from..=to {
                    codes.insert(format!("{prefix}_{n:0width$}"));
                }
            } else {
                codes.extend(spans);
            }
            // Codes defined in another cell, e.g. "`LINT_099` = any other rule".
            codes.extend(code_spans(line).into_iter().filter(|s| is_code(s)));
        }
        codes
    }

    #[test]
    fn mcp_error_codes_match_language_md_table() {
        let doc = documented_codes();
        assert!(
            doc.contains("TYPE_001") && doc.contains("LINT_011"),
            "{doc:?}"
        );
        let code: BTreeSet<String> = ERROR_CODES.iter().map(|e| e.code.to_string()).collect();
        assert_eq!(
            code.len(),
            ERROR_CODES.len(),
            "duplicate code in ERROR_CODES"
        );
        let missing: Vec<_> = doc.difference(&code).collect();
        let extra: Vec<_> = code.difference(&doc).collect();
        assert!(
            missing.is_empty() && extra.is_empty(),
            "explain_error table drifted from LANGUAGE.md §15.9: missing {missing:?}, not documented {extra:?}"
        );
    }

    #[test]
    fn mcp_error_codes_are_complete_and_case_insensitive() {
        for e in ERROR_CODES {
            assert!(
                !e.description.is_empty() && !e.example_fix.is_empty(),
                "{}",
                e.code
            );
        }
        assert_eq!(lookup(" lint_003 ").map(|e| e.code), Some("LINT_003"));
        assert!(lookup("NOPE_001").is_none());
    }
}
