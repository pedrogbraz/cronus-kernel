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

use crate::ast_diff;
use crate::parser::{self, AstNode};
use crate::{find_cronus_file, LAST_AI_ERRORS};

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
fn check(file_arg: Option<String>, profile: Profile) -> (Report, Option<Vec<AstNode>>) {
    let Some(file) = file_arg.or_else(find_cronus_file) else {
        return (
            report::usage_error(
                "",
                "IO_001",
                "no .cronus file found in the current directory".into(),
                "pass a path, e.g. 'cronus build app.cronus'",
            ),
            None,
        );
    };
    let source = match fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            return (
                report::usage_error(
                    &file,
                    "IO_001",
                    format!("cannot read '{}': {}", file, e),
                    "check the path and file permissions",
                ),
                None,
            )
        }
    };
    match parser::parse_diagnostics(&source) {
        Err(errors) => (report::from_parse_errors(&file, &errors), None),
        Ok(nodes) => {
            let r = report::validate(&file, &source, &nodes, profile);
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
