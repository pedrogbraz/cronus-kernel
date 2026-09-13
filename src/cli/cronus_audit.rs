//! `cronus audit` dispatcher.
//!
//! Dump-text fidelity is `legacy`. `cronus audit foo.html` aliases to legacy
//! permanently when the file exists.

use crate::cli::audit_codes::AuditFinding;
use crate::cli::source_language_scan;
use crate::cli::stub_renderer_gate;
use crate::parser::{self, AstNode};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditMode {
    Language,
    Logic,
    Visual,
    All,
    Legacy,
    Usage,
}

pub fn resolve_audit_mode(args: &[String]) -> AuditMode {
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("");
    if !sub.is_empty() && sub.ends_with(".html") && Path::new(sub).is_file() {
        return AuditMode::Legacy;
    }
    match sub {
        "language" => AuditMode::Language,
        "logic" => AuditMode::Logic,
        "visual" => AuditMode::Visual,
        "all" => AuditMode::All,
        "legacy" => AuditMode::Legacy,
        "" | "--help" | "-h" | "help" => AuditMode::Usage,
        _ => AuditMode::Usage,
    }
}

pub fn print_audit_usage() {
    eprintln!(
        "Usage: cronus audit <language|logic|visual|all|legacy> [options]
       cronus audit <reference.html> [--threshold 95]

  language --source <file>
  logic    --source <file> --fixture <json>
  visual                         (exit 2: pixel SoT is Playwright in cooud-ui)
  all      --source <file> [--fixture <json>] [--fixture-dir <dir>]
  legacy   <reference.html> [--threshold 95]

  cronus run --audit-canvas [port]   bind 127.0.0.1, exclusive /audit/* path
"
    );
}

fn flag_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn want_json(args: &[String]) -> bool {
    args.iter().any(|a| a == "--json")
}

fn print_findings(findings: &[AuditFinding], json: bool) {
    if json {
        let arr: Vec<_> = findings.iter().map(AuditFinding::to_json).collect();
        println!("{}", serde_json::to_string_pretty(&arr).unwrap());
        return;
    }
    if findings.is_empty() {
        println!("  \x1b[32m✓\x1b[0m Cronus Audit: pass");
        return;
    }
    println!("  \x1b[31m✗\x1b[0m Cronus Audit: {} finding(s)", findings.len());
    for f in findings {
        println!("    [{}] {}: {}", f.axis, f.code, f.message);
    }
}

fn source_arg(args: &[String]) -> Result<String, i32> {
    if let Some(s) = flag_value(args, "--source") {
        return Ok(s.to_string());
    }
    if let Some(f) = crate::find_cronus_file() {
        return Ok(f);
    }
    eprintln!("  \x1b[31m✗\x1b[0m --source <file> required (or a .cronus in cwd)");
    Err(2)
}

fn run_language(path: &str) -> Vec<AuditFinding> {
    match source_language_scan::scan_file(path) {
        Ok(f) => f,
        Err(e) => vec![AuditFinding::fail("language", "CRONUS_AUDIT_IO", e)],
    }
}

fn run_stubs(path: &str) -> Vec<AuditFinding> {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            return vec![AuditFinding::fail(
                "logic",
                "CRONUS_AUDIT_IO",
                format!("{e}"),
            )]
        }
    };
    match parser::parse(&source) {
        Ok(nodes) => stub_renderer_gate::check_ast(&nodes),
        Err(_) => Vec::new(),
    }
}

fn render_first_component(path: &str) -> Result<String, String> {
    let source = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let nodes = parser::parse(&source).map_err(|e| e.to_string())?;
    for node in &nodes {
        if let AstNode::Component(comp) = node {
            if let Some(html) = crate::cronus_ui_widgets::render(comp) {
                return Ok(html);
            }
        }
    }
    Err("no renderable component".into())
}

fn run_logic(path: &str, fixture: &str) -> Vec<AuditFinding> {
    match render_first_component(path) {
        Ok(html) => match crate::cli::logic_parity::compare_fixture_file(&html, fixture) {
            Ok(f) => f,
            Err(e) => vec![AuditFinding::fail("logic", "CRONUS_AUDIT_IO", e)],
        },
        Err(e) => vec![AuditFinding::fail("logic", "CRONUS_AUDIT_IO", e)],
    }
}

/// Returns process exit code. `legacy` may exit on its own.
pub fn run_audit_command(args: &[String]) -> i32 {
    match resolve_audit_mode(args) {
        AuditMode::Usage => {
            print_audit_usage();
            2
        }
        AuditMode::Visual => {
            eprintln!(
                "cronus audit visual: pixel SoT lives in cooud-ui Playwright\n  bunx playwright test -c playwright.audit.config.ts"
            );
            2
        }
        AuditMode::Legacy => {
            let path = if args.get(2).map(|s| s.as_str()) == Some("legacy") {
                args.get(3).cloned().unwrap_or_default()
            } else {
                args.get(2).cloned().unwrap_or_default()
            };
            let forwarded = vec![
                args.first().cloned().unwrap_or_else(|| "cronus".into()),
                "audit".into(),
                path,
            ];
            crate::cli::audit_fidelity::cmd_audit_fidelity(&forwarded);
            1
        }
        AuditMode::Language => {
            let path = match source_arg(args) {
                Ok(p) => p,
                Err(c) => return c,
            };
            let findings = run_language(&path);
            print_findings(&findings, want_json(args));
            if findings.is_empty() {
                0
            } else {
                1
            }
        }
        AuditMode::Logic => {
            let path = match source_arg(args) {
                Ok(p) => p,
                Err(c) => return c,
            };
            let Some(fixture) = flag_value(args, "--fixture") else {
                eprintln!("  \x1b[31m✗\x1b[0m logic requires --fixture <json>");
                return 2;
            };
            let findings = run_logic(&path, fixture);
            print_findings(&findings, want_json(args));
            if findings.is_empty() {
                0
            } else {
                1
            }
        }
        AuditMode::All => {
            let path = match source_arg(args) {
                Ok(p) => p,
                Err(c) => return c,
            };
            let mut findings = run_language(&path);
            findings.extend(run_stubs(&path));
            if let Some(fixture) = flag_value(args, "--fixture") {
                findings.extend(run_logic(&path, fixture));
            }
            print_findings(&findings, want_json(args));
            if findings.is_empty() {
                0
            } else {
                1
            }
        }
    }
}

pub fn cmd_audit(args: &[String]) {
    std::process::exit(run_audit_command(args));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_when_bare() {
        let args = vec!["cronus".into(), "audit".into()];
        assert_eq!(resolve_audit_mode(&args), AuditMode::Usage);
        assert_eq!(run_audit_command(&args), 2);
    }

    #[test]
    fn visual_exits_2() {
        let args = vec!["cronus".into(), "audit".into(), "visual".into()];
        assert_eq!(run_audit_command(&args), 2);
    }

    #[test]
    fn html_file_aliases_legacy() {
        let dir = std::env::temp_dir();
        let path = dir.join("cronus-audit-legacy-alias.html");
        std::fs::write(&path, "<p>hi</p>").unwrap();
        let args = vec![
            "cronus".into(),
            "audit".into(),
            path.to_string_lossy().into(),
        ];
        assert_eq!(resolve_audit_mode(&args), AuditMode::Legacy);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn language_clean_fixture_passes() {
        let dir = std::env::temp_dir();
        let path = dir.join("cronus-audit-clean.cronus");
        std::fs::write(
            &path,
            r#"
app "audit-fixtures" { port 5176 }
component ButtonPrimaryMd layout:inline style:button+primary+md {
  label "Save profile"
}
page "/audit/button/primary-md" type:custom {
  use ButtonPrimaryMd
}
"#,
        )
        .unwrap();
        let args = vec![
            "cronus".into(),
            "audit".into(),
            "language".into(),
            "--source".into(),
            path.to_string_lossy().into(),
        ];
        assert_eq!(run_audit_command(&args), 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn language_html_fails() {
        let dir = std::env::temp_dir();
        let path = dir.join("cronus-audit-html.cronus");
        std::fs::write(
            &path,
            "app \"x\" { port 1 }\ncomponent B layout:inline style:button { label \"<div>\" }\n",
        )
        .unwrap();
        let args = vec![
            "cronus".into(),
            "audit".into(),
            "language".into(),
            "--source".into(),
            path.to_string_lossy().into(),
        ];
        assert_eq!(run_audit_command(&args), 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn all_fails_stub_chart() {
        let dir = std::env::temp_dir();
        let path = dir.join("cronus-audit-chart.cronus");
        std::fs::write(
            &path,
            r#"
app "x" { port 1 }
component Revenue layout:stack style:sankey-chart {
  label "Revenue"
}
"#,
        )
        .unwrap();
        let args = vec![
            "cronus".into(),
            "audit".into(),
            "all".into(),
            "--source".into(),
            path.to_string_lossy().into(),
        ];
        assert_eq!(run_audit_command(&args), 1);
        let _ = std::fs::remove_file(path);
    }
}
