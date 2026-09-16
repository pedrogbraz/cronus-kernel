//! Multi-file composition: `import`, `compose { use }`, and directory union.
//!
//! One load graph. Each path is read once. Declarations union; a second
//! `entity`/`page`/`app`/… of the same key is `COMPOSE_001`. A missing file is
//! `COMPOSE_002`. Last-wins is gone.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::diagnostic::codes;
use super::{parse_collect, AstNode, PageNode, ParseError, SectionNode, Span};

/// Parse `source` and follow `import` / `compose { use }` relative to `base_dir`.
pub fn parse_with_imports(source: &str, base_dir: &str) -> Result<Vec<AstNode>, String> {
    parse_with_imports_diagnostics(source, base_dir).map_err(|e| super::diagnostic::join(&e))
}

/// Same as [`parse_with_imports`], with structured diagnostics.
pub fn parse_with_imports_diagnostics(
    source: &str,
    base_dir: &str,
) -> Result<Vec<AstNode>, Vec<ParseError>> {
    let base = PathBuf::from(if base_dir.is_empty() { "." } else { base_dir });
    parse_source_at(source, &base.join("<entry>"))
}

/// Parse a file from disk so its real path is the load-graph origin
/// (cycles that `import` this file skip it instead of loading it twice).
pub fn parse_file_diagnostics(path: &Path) -> Result<Vec<AstNode>, Vec<ParseError>> {
    match std::fs::read_to_string(path) {
        Ok(src) => parse_source_at(&src, path),
        Err(e) => Err(vec![ParseError::new(
            codes::MISSING_IMPORT,
            format!("import '{}' not found", display_spec(path)),
            1,
            1,
        )
        .with_target(path.display().to_string())
        .with_hint(e.to_string())]),
    }
}

/// Parse `source` as if it lived at `origin` (imports resolve next to that path;
/// `origin` is already loaded).
pub fn parse_source_at(source: &str, origin: &Path) -> Result<Vec<AstNode>, Vec<ParseError>> {
    let mut loader = Loader::new();
    loader.seen.insert(path_key(origin));
    loader.ingest_loaded(source, origin);
    loader.finish()
}

/// Union every `*.cronus` file in `dir` (non-recursive, sorted) through the
/// same load graph as [`parse_with_imports`]. Imported files that also sit in
/// `dir` are not loaded twice.
pub fn parse_directory(dir: &str) -> Result<Vec<AstNode>, String> {
    parse_directory_diagnostics(dir).map_err(|e| super::diagnostic::join(&e))
}

/// Same as [`parse_directory`], with structured diagnostics.
pub fn parse_directory_diagnostics(dir: &str) -> Result<Vec<AstNode>, Vec<ParseError>> {
    let mut files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".cronus") && entry.path().is_file() {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(vec![ParseError::new(
            codes::MISSING_IMPORT,
            "no .cronus files found in directory",
            1,
            1,
        )
        .with_hint(
            "pass a path, or put a .cronus file in the working directory",
        )]);
    }
    let mut loader = Loader::new();
    for file in files {
        loader.load_file(&file);
    }
    loader.finish()
}

struct Loader {
    seen: HashSet<String>,
    nodes: Vec<AstNode>,
    errors: Vec<ParseError>,
}

impl Loader {
    fn new() -> Self {
        Loader {
            seen: HashSet::new(),
            nodes: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn load_file(&mut self, path: &Path) {
        let key = path_key(path);
        if !self.seen.insert(key) {
            return;
        }
        match std::fs::read_to_string(path) {
            Ok(src) => self.ingest_loaded(&src, path),
            Err(e) => self.errors.push(
                ParseError::new(
                    codes::MISSING_IMPORT,
                    format!("import '{}' not found", display_spec(path)),
                    1,
                    1,
                )
                .with_target(path.display().to_string())
                .with_hint(e.to_string()),
            ),
        }
    }

    fn ingest_loaded(&mut self, source: &str, from: &Path) {
        let (nodes, mut diags) = parse_collect(source);
        self.errors.append(&mut diags);
        let from_dir = from.parent().unwrap_or(from);
        let mut follow: Vec<PathBuf> = Vec::new();
        for node in &nodes {
            match node {
                AstNode::Import(imp) => match resolve_spec(from_dir, &imp.source, "import") {
                    Ok(path) => follow.push(path),
                    Err(err) => self.errors.push(err.at(imp.line, imp.col)),
                },
                AstNode::Compose(c) => {
                    for spec in c.uses.iter().chain(c.merges.iter().map(|(s, _)| s)) {
                        match resolve_spec(from_dir, spec, "use") {
                            Ok(path) => follow.push(path),
                            Err(err) => self.errors.push(err.at(c.line, c.col)),
                        }
                    }
                }
                _ => {}
            }
        }
        for node in nodes {
            if !matches!(node, AstNode::Import(_) | AstNode::Compose(_)) {
                self.nodes.push(node);
            }
        }
        for path in follow {
            self.load_file(&path);
        }
    }

    fn finish(self) -> Result<Vec<AstNode>, Vec<ParseError>> {
        let (mut nodes, mut compose_errs) = union_conflict(self.nodes);
        expand_defines(&mut nodes);
        let ents: Vec<super::EntityNode> = nodes
            .iter()
            .filter_map(|n| match n {
                AstNode::Entity(e) => Some(e.clone()),
                _ => None,
            })
            .collect();
        compose_errs.extend(crate::relations::validate_reverses(&ents));
        let mut errors = self.errors;
        errors.append(&mut compose_errs);
        if errors.is_empty() {
            Ok(nodes)
        } else {
            Err(errors)
        }
    }
}

struct Missing {
    kind: &'static str,
    spec: String,
    looked: PathBuf,
}

impl Missing {
    fn at(self, line: usize, col: usize) -> ParseError {
        ParseError::new(
            codes::MISSING_IMPORT,
            format!("{} '{}' not found", self.kind, self.spec),
            line,
            col,
        )
        .with_target(self.spec)
        .with_hint(format!("looked for {}", self.looked.display()))
    }
}

fn resolve_spec(from_dir: &Path, spec: &str, kind: &'static str) -> Result<PathBuf, Missing> {
    let spec = spec.trim();
    let raw = if spec.starts_with('/') {
        PathBuf::from(spec)
    } else {
        from_dir.join(spec)
    };
    let path = with_cronus(&raw);
    if path.is_file() {
        Ok(path)
    } else {
        Err(Missing {
            kind,
            spec: spec.to_string(),
            looked: path,
        })
    }
}

fn with_cronus(path: &Path) -> PathBuf {
    if path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("cronus"))
    {
        path.to_path_buf()
    } else {
        let mut s = path.as_os_str().to_os_string();
        s.push(".cronus");
        PathBuf::from(s)
    }
}

fn path_key(path: &Path) -> String {
    match path.canonicalize() {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => normalize(path),
    }
}

fn normalize(path: &Path) -> String {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    let mut out = PathBuf::new();
    for c in abs.components() {
        match c {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out.to_string_lossy().into_owned()
}

/// Expand `page { use Name }` into the sections of `define Name { … }`.
/// Names that are not defines stay on `page.components` (kit `component`).
pub(crate) fn expand_defines(nodes: &mut [AstNode]) {
    let mut defines: HashMap<String, Vec<SectionNode>> = HashMap::new();
    for node in nodes.iter() {
        if let AstNode::Define(d) = node {
            defines.insert(d.name.clone(), d.sections.clone());
        }
    }
    if defines.is_empty() {
        return;
    }
    for node in nodes.iter_mut() {
        if let AstNode::Page(page) = node {
            splice_defines(page, &defines);
        }
    }
}

fn splice_defines(page: &mut PageNode, defines: &HashMap<String, Vec<SectionNode>>) {
    let mut expanded = Vec::new();
    let mut leftover = Vec::new();
    for name in &page.components {
        if let Some(secs) = defines.get(name) {
            for mut sec in secs.clone() {
                if sec.section_type == "sidebar" {
                    for item in &mut sec.items {
                        let href = item.get("href").map(|s| s.as_str()).unwrap_or("");
                        if !href.is_empty() && href == page.route {
                            item.insert("active".into(), "true".into());
                        } else {
                            item.remove("active");
                        }
                    }
                }
                if sec.section_type == "topbar" {
                    let route_parts: Vec<&str> =
                        page.route.split('/').filter(|s| !s.is_empty()).collect();
                    if let Some(first) = route_parts.first() {
                        if !first.is_empty() {
                            let capitalized =
                                format!("{}{}", first[..1].to_uppercase(), &first[1..]);
                            sec.config.insert("active_nav".into(), capitalized);
                        }
                    }
                }
                expanded.push(sec);
            }
        } else {
            leftover.push(name.clone());
        }
    }
    if expanded.is_empty() {
        return;
    }
    expanded.append(&mut page.sections);
    page.sections = expanded;
    page.components = leftover;
}

fn display_spec(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn union_conflict(nodes: Vec<AstNode>) -> (Vec<AstNode>, Vec<ParseError>) {
    let mut out = Vec::new();
    let mut errors = Vec::new();
    let mut seen_entity: HashSet<String> = HashSet::new();
    let mut seen_page: HashSet<String> = HashSet::new();
    let mut seen_layout: HashSet<String> = HashSet::new();
    let mut seen_api: HashSet<String> = HashSet::new();
    let mut seen_component: HashSet<String> = HashSet::new();
    let mut seen_define: HashSet<String> = HashSet::new();
    let mut seen_webhook: HashSet<String> = HashSet::new();
    let mut seen_env_var: HashSet<String> = HashSet::new();
    let mut have_app = false;
    let mut have_auth = false;
    let mut have_style = false;

    for node in nodes {
        let conflict = match &node {
            AstNode::Entity(e) => take(&mut seen_entity, &e.name, "entity"),
            AstNode::Page(p) => take(&mut seen_page, &p.route, "page"),
            AstNode::Layout(l) => take(&mut seen_layout, &l.name, "layout"),
            AstNode::Api(a) => take(&mut seen_api, &a.prefix, "api"),
            AstNode::Component(c) => take(&mut seen_component, &c.name, "component"),
            AstNode::Define(d) => take(&mut seen_define, &d.name, "define"),
            AstNode::Webhook(w) => take(&mut seen_webhook, &w.entity, "webhook"),
            AstNode::App(_) => singleton(&mut have_app, "app"),
            AstNode::Auth(_) => singleton(&mut have_auth, "auth"),
            AstNode::Style(_) => singleton(&mut have_style, "style"),
            AstNode::Env(env) => env_conflict(&mut seen_env_var, env),
            _ => None,
        };
        if let Some(kind) = conflict {
            let (line, col) = node_span(&node);
            errors.push(
                ParseError::new(
                    codes::DUPLICATE_DECL,
                    format!("duplicate {kind}"),
                    line,
                    col,
                )
                .with_hint("the first declaration is kept; rename or remove the later one"),
            );
            continue;
        }
        out.push(node);
    }
    (out, errors)
}

fn node_span(node: &AstNode) -> (usize, usize) {
    let span = match node {
        AstNode::Entity(n) => n.span,
        AstNode::Page(n) => n.span,
        AstNode::Layout(n) => n.span,
        AstNode::Api(n) => n.span,
        AstNode::Component(n) => n.span,
        AstNode::Define(n) => n.span,
        AstNode::Webhook(n) => n.span,
        AstNode::App(n) => n.span,
        AstNode::Auth(n) => n.span,
        AstNode::Style(n) => n.span,
        AstNode::Env(n) => n.span,
        _ => Span::default(),
    };
    (span.line, span.col)
}

fn take(seen: &mut HashSet<String>, name: &str, kind: &str) -> Option<String> {
    if !seen.insert(name.to_string()) {
        Some(format!("{kind} '{name}'"))
    } else {
        None
    }
}

fn singleton(flag: &mut bool, kind: &str) -> Option<String> {
    if *flag {
        Some(kind.to_string())
    } else {
        *flag = true;
        None
    }
}

fn env_conflict(seen: &mut HashSet<String>, env: &super::EnvNode) -> Option<String> {
    for spec in &env.schema {
        if !seen.insert(spec.name.clone()) {
            return Some(format!("env variable '{}'", spec.name));
        }
    }
    for key in env.vars.keys() {
        if !seen.insert(key.clone()) {
            return Some(format!("env variable '{key}'"));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{parse, parse_diagnostics, parse_with_imports};
    use std::fs;

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "cronus-t4-{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(dir: &Path, name: &str, src: &str) {
        fs::write(dir.join(name), src).unwrap();
    }

    fn codes_of(err: Vec<ParseError>) -> Vec<&'static str> {
        err.into_iter().map(|e| e.code).collect()
    }

    fn expect_err(r: Result<Vec<AstNode>, Vec<ParseError>>) -> Vec<ParseError> {
        match r {
            Err(e) => e,
            Ok(_) => panic!("expected compose diagnostics"),
        }
    }

    fn entity_names(nodes: &[AstNode]) -> Vec<String> {
        nodes
            .iter()
            .filter_map(|n| match n {
                AstNode::Entity(e) => Some(e.name.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn compose_block_is_not_lang_001() {
        let nodes = parse("compose App { }\n").expect("compose parses");
        assert!(matches!(nodes[0], AstNode::Compose(_)));
        assert!(parse_diagnostics("worker jobs { }\n").is_err());
    }

    #[test]
    fn import_loads_and_strips_import_nodes() {
        let dir = tmp("import-load");
        write(&dir, "entities.cronus", "entity Task { title string }\n");
        write(
            &dir,
            "app.cronus",
            "import Tasks from \"entities\"\napp \"X\" { port 1 }\nentity Tag { name string }\n",
        );
        let src = fs::read_to_string(dir.join("app.cronus")).unwrap();
        let nodes = parse_with_imports(&src, dir.to_str().unwrap()).expect("compose");
        assert!(nodes.iter().all(|n| !matches!(n, AstNode::Import(_))));
        assert_eq!(
            entity_names(&nodes),
            vec!["Tag".to_string(), "Task".to_string()]
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_bare_string_form() {
        let dir = tmp("import-bare");
        write(&dir, "entities.cronus", "entity Task { title string }\n");
        write(
            &dir,
            "app.cronus",
            "import \"entities\"\napp \"X\" { port 1 }\n",
        );
        let src = fs::read_to_string(dir.join("app.cronus")).unwrap();
        let nodes = parse_with_imports(&src, dir.to_str().unwrap()).expect("bare import");
        assert_eq!(entity_names(&nodes), vec!["Task".to_string()]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_import_is_compose_002() {
        let dir = tmp("missing");
        write(
            &dir,
            "app.cronus",
            "import \"nope\"\napp \"X\" { port 1 }\n",
        );
        let src = fs::read_to_string(dir.join("app.cronus")).unwrap();
        let err = expect_err(parse_with_imports_diagnostics(&src, dir.to_str().unwrap()));
        assert!(codes_of(err).contains(&codes::MISSING_IMPORT));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compose_001_points_at_second_entity_line() {
        let src = "app \"X\" { port 1 }\nentity Foo { n string }\n\nentity Foo { n string }\n";
        let err = expect_err(parse_source_at(src, Path::new("app.cronus")));
        let e = err
            .iter()
            .find(|e| e.code == codes::DUPLICATE_DECL)
            .unwrap_or_else(|| panic!("expected COMPOSE_001, got {err:?}"));
        assert!(e.message.contains("entity 'Foo'"), "{e:?}");
        assert_eq!(e.line, 4, "second entity is on line 4, got {e:?}");
        assert_ne!(e.line, 1);
    }

    #[test]
    fn duplicate_entity_is_compose_001() {
        let dir = tmp("dup-entity");
        write(&dir, "a.cronus", "entity Task { title string }\n");
        write(
            &dir,
            "b.cronus",
            "import \"a\"\nentity Task { title string }\napp \"X\" { port 1 }\n",
        );
        let src = fs::read_to_string(dir.join("b.cronus")).unwrap();
        let err = expect_err(parse_with_imports_diagnostics(&src, dir.to_str().unwrap()));
        assert!(
            err.iter()
                .any(|e| e.code == codes::DUPLICATE_DECL && e.message.contains("entity 'Task'")),
            "{err:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn two_apps_is_compose_001() {
        let dir = tmp("two-apps");
        write(&dir, "a.cronus", "app \"A\" { port 1 }\n");
        write(&dir, "b.cronus", "app \"B\" { port 2 }\n");
        let err = expect_err(parse_directory_diagnostics(dir.to_str().unwrap()));
        assert!(
            err.iter()
                .any(|e| e.code == codes::DUPLICATE_DECL && e.message.contains("duplicate app")),
            "{err:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn duplicate_page_auth_style_are_compose_001() {
        let dir = tmp("dup-others");
        write(
            &dir,
            "a.cronus",
            "app \"A\" { port 1 }\nauth { entity User login email + password session jwt }\nstyle { theme dark }\npage \"/\" { section kpi { bind Task { aggregate count } } }\nentity Task { title string }\n",
        );
        write(
            &dir,
            "b.cronus",
            "auth { entity User login email + password session jwt }\nstyle { theme light }\npage \"/\" { section kpi { bind Task { aggregate count } } }\n",
        );
        let err = expect_err(parse_directory_diagnostics(dir.to_str().unwrap()));
        let msgs: Vec<_> = err.iter().map(|e| e.message.as_str()).collect();
        assert!(msgs.iter().any(|m| m.contains("auth")), "{msgs:?}");
        assert!(msgs.iter().any(|m| m.contains("style")), "{msgs:?}");
        assert!(msgs.iter().any(|m| m.contains("page '/'")), "{msgs:?}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_directory_does_not_double_load_imported_file() {
        let dir = tmp("dir-union");
        write(&dir, "entities.cronus", "entity Task { title string }\n");
        write(
            &dir,
            "app.cronus",
            "import \"entities\"\napp \"X\" { port 1 }\npage \"/p\" { section kpi { bind Task { aggregate count } } }\n",
        );
        let nodes = parse_directory(dir.to_str().unwrap()).expect("directory union");
        assert_eq!(entity_names(&nodes), vec!["Task".to_string()]);
        assert_eq!(
            nodes
                .iter()
                .filter(|n| matches!(n, AstNode::App(_)))
                .count(),
            1
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compose_use_and_merge_load_like_import() {
        let dir = tmp("compose-use");
        write(&dir, "entities.cronus", "entity Task { title string }\n");
        write(
            &dir,
            "pages.cronus",
            "page \"/p\" { section kpi { bind Task { aggregate count } } }\n",
        );
        write(
            &dir,
            "app.cronus",
            "compose App {\n  use entities\n  merge pages\n}\napp \"X\" { port 1 }\n",
        );
        let src = fs::read_to_string(dir.join("app.cronus")).unwrap();
        let nodes = parse_with_imports(&src, dir.to_str().unwrap()).expect("compose use");
        assert!(nodes.iter().all(|n| !matches!(n, AstNode::Compose(_))));
        assert_eq!(entity_names(&nodes), vec!["Task".to_string()]);
        assert!(nodes
            .iter()
            .any(|n| matches!(n, AstNode::Page(p) if p.route == "/p")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unnamed_compose_use_loads() {
        let dir = tmp("compose-unnamed");
        write(&dir, "entities.cronus", "entity Task { title string }\n");
        write(
            &dir,
            "app.cronus",
            "compose {\n  use entities\n}\napp \"X\" { port 1 }\n",
        );
        let src = fs::read_to_string(dir.join("app.cronus")).unwrap();
        let nodes = parse_with_imports(&src, dir.to_str().unwrap()).expect("unnamed compose");
        assert_eq!(entity_names(&nodes), vec!["Task".to_string()]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn nested_import_is_relative_to_importer() {
        let dir = tmp("nested");
        fs::create_dir_all(dir.join("mod")).unwrap();
        write(
            &dir.join("mod"),
            "tag.cronus",
            "entity Tag { name string }\n",
        );
        write(
            &dir.join("mod"),
            "entities.cronus",
            "import \"tag\"\nentity Task { title string }\n",
        );
        write(
            &dir,
            "app.cronus",
            "import \"mod/entities\"\napp \"X\" { port 1 }\n",
        );
        let src = fs::read_to_string(dir.join("app.cronus")).unwrap();
        let nodes = parse_with_imports(&src, dir.to_str().unwrap()).expect("nested");
        let mut names = entity_names(&nodes);
        names.sort();
        assert_eq!(names, vec!["Tag".to_string(), "Task".to_string()]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cycle_loads_each_file_once() {
        let dir = tmp("cycle");
        write(&dir, "a.cronus", "import \"b\"\nentity A { n string }\n");
        write(&dir, "b.cronus", "import \"a\"\nentity B { n string }\n");
        let nodes = match parse_file_diagnostics(&dir.join("a.cronus")) {
            Ok(n) => n,
            Err(e) => panic!("cycle: {}", crate::parser::diagnostic::join(&e)),
        };
        let mut names = entity_names(&nodes);
        names.sort();
        assert_eq!(names, vec!["A".to_string(), "B".to_string()]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unique_env_vars_union_duplicate_env_var_conflicts() {
        let dir = tmp("env");
        write(
            &dir,
            "a.cronus",
            "env { APP_A string! }\napp \"X\" { port 1 }\n",
        );
        write(&dir, "b.cronus", "env { APP_B boolean default:false }\n");
        let nodes = parse_directory(dir.to_str().unwrap()).expect("unique env");
        let vars: Vec<_> = nodes
            .iter()
            .filter_map(|n| match n {
                AstNode::Env(e) => Some(e),
                _ => None,
            })
            .flat_map(|e| e.schema.iter().map(|s| s.name.as_str()))
            .collect();
        assert_eq!(vars, vec!["APP_A", "APP_B"]);

        write(&dir, "c.cronus", "env { APP_A number }\n");
        let err = expect_err(parse_directory_diagnostics(dir.to_str().unwrap()));
        assert!(
            err.iter()
                .any(|e| e.code == codes::DUPLICATE_DECL
                    && e.message.contains("env variable 'APP_A'")),
            "{err:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn keep_first_reports_every_conflict() {
        let dir = tmp("all-conflicts");
        write(
            &dir,
            "a.cronus",
            "entity Task { title string }\nentity Tag { name string }\n",
        );
        write(
            &dir,
            "b.cronus",
            "entity Task { title string }\nentity Tag { name string }\napp \"X\" { port 1 }\n",
        );
        let err = expect_err(parse_directory_diagnostics(dir.to_str().unwrap()));
        let dups: Vec<_> = err
            .iter()
            .filter(|e| e.code == codes::DUPLICATE_DECL)
            .map(|e| e.message.as_str())
            .collect();
        assert!(dups.iter().any(|m| m.contains("Task")), "{dups:?}");
        assert!(dups.iter().any(|m| m.contains("Tag")), "{dups:?}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_compose_use_is_compose_002() {
        let dir = tmp("missing-use");
        write(
            &dir,
            "app.cronus",
            "compose App { use missing }\napp \"X\" { port 1 }\n",
        );
        let src = fs::read_to_string(dir.join("app.cronus")).unwrap();
        let err = expect_err(parse_with_imports_diagnostics(&src, dir.to_str().unwrap()));
        assert!(
            err.iter()
                .any(|e| e.code == codes::MISSING_IMPORT && e.message.contains("use 'missing'")),
            "{err:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn define_is_not_lang_001_and_use_splices_sections() {
        let src = r#"
define Header {
  section page-header { title "Bench" }
}
page "/" {
  use Header
  section kpi { bind Task { aggregate count } }
}
entity Task { title string }
app "X" { port 1 }
"#;
        let nodes = parse_with_imports(src, ".").expect("define expands");
        let page = nodes
            .iter()
            .find_map(|n| match n {
                AstNode::Page(p) => Some(p),
                _ => None,
            })
            .expect("page");
        assert!(page.components.is_empty(), "{:?}", page.components);
        assert_eq!(page.sections[0].section_type, "page-header");
        assert_eq!(page.sections[1].section_type, "kpi");
        assert!(nodes.iter().any(|n| matches!(n, AstNode::Define(_))));
    }

    #[test]
    fn duplicate_define_is_compose_001() {
        let dir = tmp("dup-define");
        write(
            &dir,
            "a.cronus",
            "define Header { section page-header { title \"A\" } }\n",
        );
        write(
            &dir,
            "b.cronus",
            "define Header { section page-header { title \"B\" } }\napp \"X\" { port 1 }\n",
        );
        let err = expect_err(parse_directory_diagnostics(dir.to_str().unwrap()));
        assert!(
            err.iter()
                .any(|e| e.code == codes::DUPLICATE_DECL && e.message.contains("define 'Header'")),
            "{err:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn saas_billing_directory_unions() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("demos/saas-billing");
        let nodes = parse_directory(dir.to_str().unwrap()).expect("saas-billing is one app");
        assert!(nodes
            .iter()
            .any(|n| matches!(n, AstNode::App(a) if a.name.contains("NovaPay"))));
        assert!(entity_names(&nodes).contains(&"User".to_string()));
        assert!(entity_names(&nodes).contains(&"Invoice".to_string()));
        assert!(nodes
            .iter()
            .any(|n| matches!(n, AstNode::Page(p) if p.route == "/")));
    }
}
