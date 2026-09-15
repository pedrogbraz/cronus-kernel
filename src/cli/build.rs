//! `cronus build` — parse + validate a .cronus file.
//!
//! One report (see `build_report.rs`) drives both output modes:
//! - human (default): diagnostics on stderr, then exactly one verdict line
//! - `--ai` / `--machine` / `--json-errors` / `--strict-ai`: stdout is ONLY the
//!   JSON report (schema_version 1); nothing else is printed to stdout.
//!
//! Exit codes: 0 valid, 1 invalid, 2 usage/I-O. Documented in LANGUAGE.md §15.9.

use std::fs;
use std::io::IsTerminal;
use std::path::Path;

use crate::ast_diff;
use crate::parser::{self, AstNode};
use crate::{find_all_cronus_files, LAST_AI_ERRORS};

// Flat file names: a `build/` directory is commonly git-ignored.
#[path = "build_locate.rs"]
mod locate;
#[path = "build_report.rs"]
pub(crate) mod report;

use report::{Profile, Report};

/// Parses `source` and returns the same JSON report `cronus build --ai` prints
/// (schema_version 1, strict profile). `Err` carries the parse diagnostics
/// rendered as `CODE: message (line L, col C)`, one per line.
pub(crate) fn validate_source_ai(source: &str, file: &str) -> Result<serde_json::Value, String> {
    let nodes = parser::parse_diagnostics(source).map_err(|e| parser::diagnostic::join(&e))?;
    Ok(report::validate(file, source, &nodes, Profile { strict: true }).to_json())
}

/// Compatibility entry point for callers that already hold the AST
/// (`context --for-claude`, the llms-full drift test). Same passes and JSON as
/// `build --ai`. Source positions come from re-reading `file` when it is a
/// readable `.cronus` path; otherwise name-based locations fall back to 1:1.
/// Prefer `validate_source_ai` when the source text is at hand.
pub(crate) fn build_ai_error_json(
    nodes: &[AstNode],
    file: &str,
    _entities: usize,
    _pages: usize,
    _routes: usize,
) -> serde_json::Value {
    let source = if file.ends_with(".cronus") {
        fs::read_to_string(file).unwrap_or_default()
    } else {
        String::new()
    };
    report::validate(file, &source, nodes, Profile { strict: true }).to_json()
}

pub fn cmd_build(args: &[String]) {
    let code = run(args);
    if code != report::exit::VALID {
        std::process::exit(code);
    }
}

fn run(args: &[String]) -> i32 {
    let ai_mode = args.iter().any(|a| {
        matches!(
            a.as_str(),
            "--ai" | "--machine" | "--json-errors" | "--strict-ai"
        )
    });
    let strict = args.iter().any(|a| a == "--strict");
    let strict_audit = args.iter().any(|a| a == "--strict-audit");
    let profile = Profile {
        strict: strict || ai_mode,
    };
    let file_arg = args.iter().skip(2).find(|a| !a.starts_with("--")).cloned();

    let (report, nodes) = check(file_arg, profile);

    if ai_mode {
        let json = report.to_json();
        println!(
            "{}",
            serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string())
        );
        if let Ok(mut last) = LAST_AI_ERRORS.lock() {
            *last = Some(json);
        }
        if let (true, Some(nodes)) = (report.valid(), &nodes) {
            write_snapshot(nodes);
        }
        return report.exit_code();
    }

    let diagnostics = report.render_human(std::io::stderr().is_terminal());
    eprint!("{}", diagnostics);
    if !report.valid() {
        eprintln!("{}", report.verdict(std::io::stderr().is_terminal()));
        return report.exit_code();
    }
    println!("{}", report.verdict(std::io::stdout().is_terminal()));

    if let Some(nodes) = &nodes {
        save_ast_snapshot(nodes);
    }
    run_fidelity_audit(strict_audit)
}

/// Locate, read, parse and validate. Returns the AST when parsing succeeded.
/// Follows `import` / `compose { use }`. Several `*.cronus` files in cwd with
/// no path argument are unioned the same way as `cronus run`.
fn check(file_arg: Option<String>, profile: Profile) -> (Report, Option<Vec<AstNode>>) {
    if let Some(file) = file_arg {
        return check_file(&file, profile);
    }
    let files = find_all_cronus_files();
    match files.len() {
        0 => (
            report::usage_error(
                "",
                "IO_001",
                "no .cronus file found in the current directory".into(),
                "pass a path, e.g. 'cronus build app.cronus'",
            ),
            None,
        ),
        1 => check_file(&files[0], profile),
        _ => match parser::parse_directory_diagnostics(".") {
            Err(errors) => (report::from_parse_errors(&files[0], &errors), None),
            Ok(nodes) => {
                let source = fs::read_to_string(&files[0]).unwrap_or_default();
                let r = report::validate(&files[0], &source, &nodes, profile);
                (r, Some(nodes))
            }
        },
    }
}

fn check_file(file: &str, profile: Profile) -> (Report, Option<Vec<AstNode>>) {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            return (
                report::usage_error(
                    file,
                    "IO_001",
                    format!("cannot read '{}': {}", file, e),
                    "check the path and file permissions",
                ),
                None,
            )
        }
    };
    match parser::parse_source_at(&source, Path::new(file)) {
        Err(errors) => (report::from_parse_errors(file, &errors), None),
        Ok(nodes) => {
            let r = report::validate(file, &source, &nodes, profile);
            (r, Some(nodes))
        }
    }
}

/// Auto-audit when `.cronus/audit-ref.html` exists (human mode only).
fn run_fidelity_audit(strict_audit: bool) -> i32 {
    let audit_ref_path = ".cronus/audit-ref.html";
    if !std::path::Path::new(audit_ref_path).exists() {
        return report::exit::VALID;
    }
    println!();
    println!("  \x1b[1mFidelity Audit\x1b[0m (auto — .cronus/audit-ref.html found)");
    match crate::cli::audit_fidelity::run_audit(audit_ref_path) {
        Some(result) => {
            crate::cli::audit_fidelity::print_fidelity_line(&result);
            crate::cli::audit_fidelity::save_audit_results(&result);
            println!("  \x1b[32m✓\x1b[0m Results saved to .cronus/audit-results.json");
            if strict_audit && result.fidelity < 95 {
                println!();
                crate::cli::audit_fidelity::print_missing_top(&result, 5);
                eprintln!();
                eprintln!(
                    "  \x1b[31m✗ AUDIT FAILED\x1b[0m — {}% < 95% threshold (--strict-audit)",
                    result.fidelity
                );
                return report::exit::INVALID;
            }
        }
        None => {
            eprintln!("  \x1b[33m⚠\x1b[0m Could not run audit (no .cronus file or parse error)");
        }
    }
    report::exit::VALID
}

/// Snapshot without any output (AI mode keeps stdout JSON-only).
fn write_snapshot(nodes: &[AstNode]) {
    let snapshot = ast_diff::snapshot_from_ast(nodes);
    let _ = fs::create_dir_all(".cronus");
    if let Ok(json_str) = serde_json::to_string_pretty(&snapshot) {
        let _ = fs::write(".cronus/ast-snapshot.json", &json_str);
    }
}

pub(crate) fn save_ast_snapshot(nodes: &[AstNode]) {
    let snapshot = ast_diff::snapshot_from_ast(nodes);
    let _ = fs::create_dir_all(".cronus");
    match serde_json::to_string_pretty(&snapshot) {
        Ok(json_str) => {
            if fs::write(".cronus/ast-snapshot.json", &json_str).is_ok() {
                println!(
                    "  \x1b[32m\u{2713}\x1b[0m AST snapshot saved to .cronus/ast-snapshot.json"
                );
            }
        }
        Err(e) => {
            eprintln!(
                "  \x1b[33m\u{26a0}\x1b[0m Failed to serialize AST snapshot: {}",
                e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "entity Task {\n  title string!\n}\npage \"/\" {\n  section stats { bind Task { aggregate count } }\n  section table { bind Tsk { query all } }\n}\n";

    #[test]
    fn validate_source_ai_matches_the_ai_report() {
        let v = validate_source_ai(SRC, "app.cronus").expect("parses");
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["valid"], false);
        assert!(v["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["code"] == "RESOLVE_001" && e["location"]["line"] == 6));
        let Err(e) = validate_source_ai("entity T {\n  a strin\n}\n", "app.cronus") else {
            panic!("typo must be a parse error")
        };
        assert!(e.starts_with("TYPE_001: "), "{}", e);
    }

    #[test]
    fn build_ai_error_json_shim_has_same_verdict_and_codes() {
        let nodes = parser::parse(SRC).unwrap_or_default();
        let shim = build_ai_error_json(&nodes, "not-a-file.txt", 0, 0, 0);
        let full = validate_source_ai(SRC, "not-a-file.txt").expect("parses");
        assert_eq!(shim["valid"], full["valid"]);
        let codes = |v: &serde_json::Value| -> Vec<String> {
            v["errors"]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| e["code"].as_str().unwrap_or("").to_string())
                .collect()
        };
        assert_eq!(codes(&shim), codes(&full));
        // without source text, locations degrade to 1:1 but are never 0
        for e in shim["errors"].as_array().unwrap() {
            assert!(e["location"]["line"].as_u64().unwrap() >= 1);
        }
    }
}
