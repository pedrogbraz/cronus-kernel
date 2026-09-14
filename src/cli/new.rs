use crate::parser;
use std::fs;
use std::path::{Path, PathBuf};

/// Entries every scaffolded project must keep out of git: signing keys and
/// local SQLite databases (which hold user rows and password hashes).
pub(crate) const GITIGNORE_ENTRIES: &[&str] = &[
    ".cronus/jwt.key",
    ".cronus/webhook.key",
    "*.db",
    "*.db-wal",
    "*.db-shm",
];

/// Template used by `cronus new <name>` when `--template` is not given.
pub(crate) const DEFAULT_TEMPLATE: &str = "saas";

/// Every template `cronus new` can scaffold, embedded from `templates/`.
/// `aurora` is kept as an alias of `cronus-ui`.
pub(crate) const TEMPLATES: &[(&str, &str, &str)] = &[
    (
        "saas",
        "Accounts, projects, private dashboard",
        include_str!("../../templates/saas.cronus"),
    ),
    (
        "api",
        "REST API only, no UI",
        include_str!("../../templates/api.cronus"),
    ),
    (
        "landing",
        "Public marketing page with lead capture",
        include_str!("../../templates/landing.cronus"),
    ),
    (
        "admin",
        "Admin panel: customers and orders",
        include_str!("../../templates/admin.cronus"),
    ),
    (
        "blog",
        "Posts with draft/published workflow",
        include_str!("../../templates/blog.cronus"),
    ),
    (
        "crm",
        "Contacts and deal pipeline",
        include_str!("../../templates/crm.cronus"),
    ),
    (
        "ecommerce",
        "Product catalog and orders",
        include_str!("../../templates/ecommerce.cronus"),
    ),
    (
        "helpdesk",
        "Support tickets with priorities",
        include_str!("../../templates/helpdesk.cronus"),
    ),
    (
        "cronus-ui",
        "Voodoo runtime + cronus-ui widgets",
        include_str!("../../templates/cronus-ui.cronus"),
    ),
    (
        "aurora",
        "Alias of cronus-ui",
        include_str!("../../templates/cronus-ui.cronus"),
    ),
];

const CRONUS_UI_SCRIPT: &str = r#"# Fires whenever a Lead row is created.
on Lead.create {
  log "lead created"
}
"#;

pub(crate) fn template_source(template: &str) -> Option<&'static str> {
    TEMPLATES
        .iter()
        .find(|(name, _, _)| *name == template)
        .map(|(_, _, src)| *src)
}

/// Creates `<dir>/.gitignore`, or appends whichever required entries are missing.
pub(crate) fn write_gitignore(dir: &Path) -> std::io::Result<()> {
    let path = dir.join(".gitignore");
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let present: Vec<&str> = existing.lines().map(str::trim).collect();
    let missing: Vec<&str> = GITIGNORE_ENTRIES
        .iter()
        .copied()
        .filter(|e| !present.contains(e))
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    let mut out = existing.clone();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("# Cronus: secrets and local databases\n");
    for entry in missing {
        out.push_str(entry);
        out.push('\n');
    }
    fs::write(&path, out)
}

/// What `cronus new` was asked to do.
#[derive(Debug, PartialEq)]
pub(crate) struct NewArgs {
    pub name: String,
    pub template: String,
    /// `cronus new <template>` (old form): print a deprecation note.
    pub legacy_form: bool,
}

/// Parses `cronus new <name> [--template <t>]` (args[0] = "cronus", args[1] = "new").
/// The old `cronus new <template>` still works when `<name>` is a template name.
pub(crate) fn parse_new_args(args: &[String]) -> Result<NewArgs, String> {
    let mut name: Option<String> = None;
    let mut template: Option<String> = None;
    let mut i = 2;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--template" || a == "-t" {
            i += 1;
            template = Some(
                args.get(i)
                    .cloned()
                    .ok_or_else(|| "--template requires a value".to_string())?,
            );
        } else if let Some(t) = a.strip_prefix("--template=") {
            template = Some(t.to_string());
        } else if a.starts_with('-') {
            return Err(format!("unknown option '{}'", a));
        } else if name.is_none() {
            name = Some(a.to_string());
        } else {
            return Err(format!("unexpected argument '{}'", a));
        }
        i += 1;
    }
    let name = name.ok_or_else(|| "missing project name".to_string())?;
    if name.contains(['/', '\\']) || name == "." || name == ".." {
        return Err(format!(
            "project name '{}' must be a plain directory name",
            name
        ));
    }
    let legacy_form = template.is_none() && template_source(&name).is_some();
    let template = template.unwrap_or_else(|| {
        if legacy_form {
            name.clone()
        } else {
            DEFAULT_TEMPLATE.to_string()
        }
    });
    if template_source(&template).is_none() {
        return Err(format!("unknown template '{}'", template));
    }
    Ok(NewArgs {
        name,
        template,
        legacy_form,
    })
}

/// Writes the project for `template` into `dir` (created if needed) and
/// returns the path of the generated `app.cronus`. Refuses to overwrite an
/// existing `app.cronus`. `source` overrides the embedded template text.
pub(crate) fn scaffold(
    dir: &Path,
    template: &str,
    source: Option<&str>,
) -> Result<PathBuf, String> {
    let content = source
        .or_else(|| template_source(template))
        .ok_or_else(|| format!("unknown template '{}'", template))?;
    let file_path = dir.join("app.cronus");
    if file_path.exists() {
        return Err(format!(
            "{} already exists — pick another project name",
            file_path.display()
        ));
    }
    fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {}", dir.display(), e))?;
    fs::write(&file_path, content)
        .map_err(|e| format!("cannot write {}: {}", file_path.display(), e))?;
    write_gitignore(dir).map_err(|e| format!("cannot write .gitignore: {}", e))?;
    if template == "cronus-ui" || template == "aurora" {
        fs::write(dir.join("app.scriptcronus"), CRONUS_UI_SCRIPT)
            .map_err(|e| format!("cannot write app.scriptcronus: {}", e))?;
    }
    Ok(file_path)
}

fn print_usage() {
    eprintln!("  Usage: cronus new <name> [--template <template>]");
    eprintln!();
    eprintln!("  Templates (default: {}):", DEFAULT_TEMPLATE);
    for (name, about, _) in TEMPLATES {
        eprintln!("    {:<10} {}", name, about);
    }
}

pub fn cmd_new(args: &[String]) {
    if args
        .iter()
        .skip(2)
        .any(|a| a == "--list" || a == "--help" || a == "-h")
    {
        print_usage();
        return;
    }
    let parsed = match parse_new_args(args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m {}", e);
            eprintln!();
            print_usage();
            std::process::exit(1);
        }
    };
    if parsed.legacy_form {
        eprintln!(
            "  \x1b[33mnote:\x1b[0m `cronus new {}` is deprecated — use `cronus new <name> --template {}`",
            parsed.name, parsed.template
        );
    }

    // A templates/ directory next to the binary overrides the embedded copy.
    let override_source = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .map(|dir| {
            dir.join("templates")
                .join(format!("{}.cronus", parsed.template))
        })
        .and_then(|path| fs::read_to_string(&path).ok());

    let dir = Path::new(&parsed.name);
    let file_path = match scaffold(dir, &parsed.template, override_source.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m {}", e);
            std::process::exit(1);
        }
    };
    let content = fs::read_to_string(&file_path).unwrap_or_default();

    let nodes = parser::parse(&content).ok();
    let (ent_count, pg_count, rt_count) = nodes
        .as_ref()
        .map(|n| parser::stats(n))
        .unwrap_or((0, 0, 0));
    let mut port = 5175;
    let mut has_auth = false;
    for n in nodes.iter().flatten() {
        match n {
            parser::AstNode::App(a) => port = a.port,
            parser::AstNode::Auth(_) => has_auth = true,
            _ => {}
        }
    }

    println!();
    println!(
        "  Created \x1b[1m{}\x1b[0m from template \x1b[1m{}\x1b[0m",
        file_path.display(),
        parsed.template
    );
    println!(
        "  \x1b[90m{} entities, {} pages, {} API routes{}\x1b[0m",
        ent_count,
        pg_count,
        rt_count,
        if has_auth { ", signup/login" } else { "" }
    );
    println!();
    println!("  \x1b[90mNext steps:\x1b[0m");
    println!("    cd {}", parsed.name);
    println!(
        "    cronus run          \x1b[90m# http://127.0.0.1:{}\x1b[0m",
        port
    );
    println!("    cronus build --ai   \x1b[90m# validate after edits\x1b[0m");
    println!("    cronus doctor       \x1b[90m# health check\x1b[0m");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "cronus-new-{}-{}-{}",
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

    fn argv(rest: &[&str]) -> Vec<String> {
        ["cronus", "new"]
            .iter()
            .chain(rest.iter())
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn scaffold_gitignore_excludes_secrets_and_databases() {
        let dir = scratch_dir("fresh");
        write_gitignore(&dir).unwrap();
        let content = fs::read_to_string(dir.join(".gitignore")).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert!(lines.contains(&".cronus/jwt.key"));
        assert!(lines.contains(&"*.db"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn scaffold_gitignore_appends_missing_entries_once() {
        let dir = scratch_dir("append");
        fs::write(dir.join(".gitignore"), "node_modules\n*.db").unwrap();
        write_gitignore(&dir).unwrap();
        write_gitignore(&dir).unwrap();
        let content = fs::read_to_string(dir.join(".gitignore")).unwrap();
        assert!(content.starts_with("node_modules\n*.db\n"));
        assert_eq!(content.matches("*.db\n").count(), 1);
        assert_eq!(content.matches(".cronus/jwt.key").count(), 1);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn new_takes_a_project_name_with_default_template() {
        let parsed = parse_new_args(&argv(&["my-app"])).unwrap();
        assert_eq!(parsed.name, "my-app");
        assert_eq!(parsed.template, DEFAULT_TEMPLATE);
        assert!(!parsed.legacy_form);
    }

    #[test]
    fn new_accepts_template_flag_in_any_position() {
        let a = parse_new_args(&argv(&["shop", "--template", "ecommerce"])).unwrap();
        let b = parse_new_args(&argv(&["--template=ecommerce", "shop"])).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.template, "ecommerce");
    }

    #[test]
    fn legacy_new_template_form_still_works() {
        let parsed = parse_new_args(&argv(&["blog"])).unwrap();
        assert_eq!(parsed.name, "blog");
        assert_eq!(parsed.template, "blog");
        assert!(parsed.legacy_form);
    }

    #[test]
    fn new_rejects_unknown_template_and_path_names() {
        assert!(parse_new_args(&argv(&["x", "--template", "nope"])).is_err());
        assert!(parse_new_args(&argv(&[])).is_err());
        assert!(parse_new_args(&argv(&["../evil"])).is_err());
    }

    #[test]
    fn scaffold_refuses_to_overwrite_existing_app() {
        let dir = scratch_dir("exists");
        fs::write(dir.join("app.cronus"), "mine").unwrap();
        assert!(scaffold(&dir, "saas", None).is_err());
        assert_eq!(fs::read_to_string(dir.join("app.cronus")).unwrap(), "mine");
        let _ = fs::remove_dir_all(dir);
    }

    /// The first ```cronus example in README.md is what newcomers copy: it must validate.
    #[test]
    fn readme_example_passes_build_ai() {
        let readme = include_str!("../../README.md");
        let start = readme
            .find("```cronus\n")
            .expect("README has a cronus example")
            + 10;
        let len = readme[start..].find("```").unwrap();
        let src = &readme[start..start + len];
        let report = crate::cli::build::validate_source_ai(src, "README.md").unwrap();
        assert_eq!(
            report["valid"].as_bool(),
            Some(true),
            "README example fails build --ai: {}",
            report["errors"]
        );
    }

    /// Every template, scaffolded exactly like `cronus new <name> --template <t>`,
    /// must pass the validator behind `cronus build --ai` with zero errors.
    #[test]
    fn every_template_scaffolds_and_passes_build_ai() {
        let mut failures = Vec::new();
        for (template, _, _) in TEMPLATES {
            let root = scratch_dir(template);
            let project = root.join("app");
            let file = match scaffold(&project, template, None) {
                Ok(f) => f,
                Err(e) => {
                    failures.push(format!("{}: scaffold failed: {}", template, e));
                    continue;
                }
            };
            let src = fs::read_to_string(&file).unwrap();
            // A statement that swallows `}` can merge pages and still "validate":
            // every declared page must survive parsing.
            let declared_pages = src
                .lines()
                .filter(|l| l.trim_start().starts_with("page \""))
                .count();
            if let Ok(nodes) = parser::parse(&src) {
                let (_, parsed_pages, _) = parser::stats(&nodes);
                if parsed_pages != declared_pages {
                    failures.push(format!(
                        "{}: {} pages declared but {} parsed",
                        template, declared_pages, parsed_pages
                    ));
                }
            }
            match crate::cli::build::validate_source_ai(&src, &file.to_string_lossy()) {
                Ok(report) if report["valid"].as_bool() == Some(true) => {}
                Ok(report) => {
                    let errs: Vec<String> = report["errors"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|e| format!("{} {}", e["code"], e["message"]))
                        .collect();
                    failures.push(format!("{}:\n      {}", template, errs.join("\n      ")));
                }
                Err(e) => failures.push(format!("{}: parse error: {}", template, e)),
            }
            let _ = fs::remove_dir_all(root);
        }
        assert!(
            failures.is_empty(),
            "templates failing build --ai:\n  {}",
            failures.join("\n  ")
        );
    }
}
