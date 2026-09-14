use crate::lint;
use crate::parser::{self, AstNode};
use std::fs;
use std::path::{Path, PathBuf};

/// One doctor check. Required checks decide the score; informational ones
/// only add context (a fresh project legitimately has no database yet).
#[derive(Debug)]
pub(crate) struct Check {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Default)]
pub(crate) struct Report {
    pub required: Vec<Check>,
    pub info: Vec<Check>,
    pub suggestions: Vec<String>,
}

impl Report {
    pub fn required_passed(&self) -> usize {
        self.required.iter().filter(|c| c.ok).count()
    }
}

/// `app.cronus` if present, else the first `*.cronus` (sorted) in `dir`.
fn cronus_file_in(dir: &Path) -> Option<PathBuf> {
    let app = dir.join("app.cronus");
    if app.is_file() {
        return Some(app);
    }
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("cronus"))
        .collect();
    files.sort();
    files.into_iter().next()
}

/// The database file must be creatable/writable: open it read-write if it
/// exists, otherwise check its directory accepts a new file.
fn database_writable(db_file: &Path) -> Result<String, String> {
    if db_file.exists() {
        rusqlite::Connection::open_with_flags(db_file, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
            .map(|_| format!("{} is writable", db_file.display()))
            .map_err(|e| format!("{}: {}", db_file.display(), e))
    } else {
        let parent = db_file
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let probe = parent.join(format!(".cronus-doctor-probe-{}", std::process::id()));
        fs::write(&probe, b"")
            .map(|_| {
                let _ = fs::remove_file(&probe);
                format!(
                    "{} will be created on first `cronus run`",
                    db_file.display()
                )
            })
            .map_err(|e| format!("cannot create files in {}: {}", parent.display(), e))
    }
}

fn port_available(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Runs every check for the project in `dir`.
pub(crate) fn run_checks(dir: &Path) -> Report {
    let mut report = Report::default();

    let Some(file) = cronus_file_in(dir) else {
        report.required.push(Check {
            name: "project",
            ok: false,
            detail: "no .cronus file found".into(),
        });
        report
            .suggestions
            .push("cronus new my-app   # create a project".into());
        return report;
    };
    let source = fs::read_to_string(&file).unwrap_or_default();
    let nodes = match parser::parse(&source) {
        Ok(nodes) => {
            let (e, p, r) = parser::stats(&nodes);
            report.required.push(Check {
                name: "parse",
                ok: true,
                detail: format!(
                    "{} ({} entities, {} pages, {} routes)",
                    file.display(),
                    e,
                    p,
                    r
                ),
            });
            nodes
        }
        Err(e) => {
            report.required.push(Check {
                name: "parse",
                ok: false,
                detail: e,
            });
            report
                .suggestions
                .push("cronus build --ai   # errors with fix hints".into());
            return report;
        }
    };

    // Build: the same validator as `cronus build --ai`.
    match crate::cli::build::validate_source_ai(&source, &file.to_string_lossy()) {
        Ok(v) if v["valid"].as_bool() == Some(true) => report.required.push(Check {
            name: "build",
            ok: true,
            detail: "cronus build --ai: valid".into(),
        }),
        Ok(v) => {
            let n = v["errors"].as_array().map(|a| a.len()).unwrap_or(0);
            report.required.push(Check {
                name: "build",
                ok: false,
                detail: format!("cronus build --ai: {} error(s)", n),
            });
            report
                .suggestions
                .push("cronus build --ai   # see each error and its fix".into());
        }
        Err(e) => report.required.push(Check {
            name: "build",
            ok: false,
            detail: e,
        }),
    }

    let mut port = 5175u16;
    let mut db_path: Option<String> = None;
    let mut constitution = false;
    for n in &nodes {
        if let AstNode::App(a) = n {
            port = a.port;
            db_path = a.database.as_ref().and_then(|d| d.path.clone());
            constitution = a.constitution.is_some();
        }
    }

    let port_ok = port_available(port);
    report.required.push(Check {
        name: "port",
        ok: port_ok,
        detail: if port_ok {
            format!("{} available", port)
        } else {
            format!("{} in use — try `cronus run <other-port>`", port)
        },
    });

    let db_rel = db_path.unwrap_or_else(|| "data.db".into());
    let db_rel = db_rel.trim_matches('"');
    let db_file = if dir == Path::new(".") {
        PathBuf::from(db_rel.trim_start_matches("./"))
    } else {
        dir.join(db_rel)
    };
    let db = database_writable(&db_file);
    report.required.push(Check {
        name: "database",
        ok: db.is_ok(),
        detail: db.unwrap_or_else(|e| e),
    });

    // ── Informational ──
    let lint_results = lint::lint_ast(&nodes, false);
    let warnings = lint_results
        .iter()
        .filter(|r| matches!(r.severity, lint::Severity::Warning))
        .count();
    report.info.push(Check {
        name: "lint",
        ok: warnings == 0,
        detail: format!("{} warning(s)", warnings),
    });
    report.info.push(Check {
        name: "database file",
        ok: db_file.exists(),
        detail: if db_file.exists() {
            "present".into()
        } else {
            "not created yet".into()
        },
    });
    report.info.push(Check {
        name: "constitution",
        ok: constitution || dir.join(".cronus/constitution.toml").exists(),
        detail: "optional must/never rules".into(),
    });
    report.info.push(Check {
        name: "ast snapshot",
        ok: dir.join(".cronus/ast-snapshot.json").exists(),
        detail: "written by build/run, used by changelog".into(),
    });
    report.info.push(Check {
        name: "memory",
        ok: dir.join(".cronus/memory.db").exists(),
        detail: "written by run, used by AI session tooling".into(),
    });

    if report.required.iter().all(|c| c.ok) {
        report
            .suggestions
            .push(format!("cronus run          # http://127.0.0.1:{}", port));
        report
            .suggestions
            .push("cronus test         # CRUD tests against the running server".into());
        report
            .suggestions
            .push("cronus context --for-claude   # hand the project to an AI".into());
    }
    report
}

pub fn cmd_doctor(_args: &[String]) {
    let report = run_checks(Path::new("."));
    println!();
    println!("  \x1b[36mCRONUS Doctor\x1b[0m");
    println!();
    println!("  \x1b[1mRequired\x1b[0m");
    for c in &report.required {
        let mark = if c.ok {
            "\x1b[32m✓\x1b[0m"
        } else {
            "\x1b[31m✗\x1b[0m"
        };
        println!("    {} {:<9} {}", mark, c.name, c.detail);
    }
    if !report.info.is_empty() {
        println!();
        println!("  \x1b[1mInformational\x1b[0m \x1b[90m(not scored)\x1b[0m");
        for c in &report.info {
            let mark = if c.ok {
                "\x1b[32m✓\x1b[0m"
            } else {
                "\x1b[90m·\x1b[0m"
            };
            println!("    {} {:<13} {}", mark, c.name, c.detail);
        }
    }
    println!();
    let passed = report.required_passed();
    let total = report.required.len();
    if passed == total {
        println!(
            "  \x1b[32mHealthy: {}/{} required checks passed\x1b[0m",
            passed, total
        );
    } else {
        println!(
            "  \x1b[33m{}/{} required checks passed\x1b[0m",
            passed, total
        );
    }
    if !report.suggestions.is_empty() {
        println!();
        println!("  \x1b[90mNext:\x1b[0m");
        for s in &report.suggestions {
            println!("    {}", s);
        }
    }
    println!();
    if passed != total {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "cronus-doctor-{}-{}-{}",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn fresh_project_passes_every_required_check_except_a_busy_port() {
        let root = scratch("fresh");
        let project = root.join("app");
        crate::cli::new::scaffold(&project, "saas", None).unwrap();
        let report = run_checks(&project);
        for c in &report.required {
            // The port may be taken by an unrelated dev server on this machine.
            if c.name != "port" {
                assert!(c.ok, "required check {} failed: {}", c.name, c.detail);
            }
        }
        let names: Vec<&str> = report.required.iter().map(|c| c.name).collect();
        assert_eq!(names, ["parse", "build", "port", "database"]);
        // Missing data.db / snapshot / memory are informational, never required.
        assert!(report.info.iter().any(|c| c.name == "memory" && !c.ok));
        assert!(
            !project.join("data.db").exists(),
            "doctor must not create the db"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn fresh_project_suggests_next_commands() {
        let root = scratch("suggest");
        let project = root.join("app");
        crate::cli::new::scaffold(&project, "api", None).unwrap();
        let mut src = fs::read_to_string(project.join("app.cronus")).unwrap();
        // Pick a port nothing listens on so every required check passes.
        let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = free.local_addr().unwrap().port();
        drop(free);
        src = src.replace("port 5175", &format!("port {}", port));
        fs::write(project.join("app.cronus"), src).unwrap();
        let report = run_checks(&project);
        assert_eq!(report.required_passed(), report.required.len());
        assert!(report
            .suggestions
            .iter()
            .any(|s| s.starts_with("cronus run")));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn busy_port_fails_the_port_check() {
        let held = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        assert!(!port_available(held.local_addr().unwrap().port()));
    }

    #[test]
    fn invalid_project_fails_parse_and_suggests_build_ai() {
        let dir = scratch("broken");
        fs::write(dir.join("app.cronus"), "entity {").unwrap();
        let report = run_checks(&dir);
        assert!(!report.required[0].ok);
        assert!(report.suggestions.iter().any(|s| s.contains("build --ai")));
        let _ = fs::remove_dir_all(dir);
    }
}
