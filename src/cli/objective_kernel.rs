use std::collections::HashMap;
use std::fs;

use crate::cli::brief::{brief_toml_arr, brief_toml_arr_after_section, brief_toml_val};
use crate::parser::{
    self, ApiNode, AppNode, AstNode, AuthNode, ComponentNode, ComposeNode, EntityNode, EnvNode,
    EventNode, FieldType, ImportNode, LayoutNode, MiddlewareNode, PageNode, ServiceNode, StyleNode,
    TestNode, WorkerNode,
};

/// Count files matching a suffix in a directory (and one level of subdirs).
pub fn count_files_matching(dir: &str, suffix: &str) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().ends_with(suffix) {
                count += 1;
            }
        }
    }
    // Also check subdirectories one level deep
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                    for sub in sub_entries.flatten() {
                        if sub.file_name().to_string_lossy().ends_with(suffix) {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

/// Simple UTC ISO timestamp without chrono crate.
pub fn chrono_now_iso() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0usize;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining < md as i64 {
            m = i + 1;
            break;
        }
        remaining -= md as i64;
    }
    let d = remaining + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hours, minutes, seconds
    )
}

/// Get list of changed files from git (staged or last commit).
pub fn git_changed_files() -> Vec<String> {
    use std::process::Command;
    let mut files = Vec::new();

    // Priority 1: staged files (what's about to be committed)
    if let Ok(output) = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
    {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let t = line.trim();
                if !t.is_empty() && !files.contains(&t.to_string()) {
                    files.push(t.to_string());
                }
            }
        }
    }

    // Priority 2: if nothing staged, fall back to last commit
    if files.is_empty() {
        if let Ok(output) = Command::new("git")
            .args(["diff", "HEAD~1..HEAD", "--name-only"])
            .output()
        {
            if output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    let t = line.trim();
                    if !t.is_empty() && !files.contains(&t.to_string()) {
                        files.push(t.to_string());
                    }
                }
            }
        }
    }

    files
}

/// Get diff content from git (staged or last commit).
pub fn git_diff_content() -> String {
    use std::process::Command;

    // Priority 1: staged diff
    if let Ok(output) = Command::new("git").args(["diff", "--cached"]).output() {
        if output.status.success() {
            let staged = String::from_utf8_lossy(&output.stdout).to_string();
            if !staged.trim().is_empty() {
                return staged;
            }
        }
    }

    // Priority 2: last commit diff
    if let Ok(output) = Command::new("git").args(["diff", "HEAD~1..HEAD"]).output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).to_string();
        }
    }

    String::new()
}

/// Returns (active_task_id, files_outside_lease) — used by drift scope check
pub fn lease_check_scope() -> (String, Vec<String>) {
    let mut task_id = String::new();
    let mut write_scope: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(".cronus/tasks") {
        let mut files: Vec<_> = entries
            .flatten()
            .filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                n.starts_with("TASK-") && n.ends_with(".toml")
            })
            .collect();
        files.sort_by_key(|e| e.file_name());

        for entry in files {
            let content = fs::read_to_string(entry.path()).unwrap_or_default();
            if let Some(status) = brief_toml_val(&content, "status") {
                if status == "open" || status == "in_progress" {
                    task_id = brief_toml_val(&content, "id").unwrap_or_default();
                    // Try section-aware parsing first, fall back to simple
                    let scope = brief_toml_arr_after_section(&content, "[scope]", "write");
                    write_scope = if !scope.is_empty() {
                        scope
                    } else {
                        brief_toml_arr(&content, "write")
                    };
                    break;
                }
            }
        }
    }

    if task_id.is_empty() || write_scope.is_empty() {
        return (task_id, Vec::new());
    }

    let changed = git_changed_files();

    let mut outside = Vec::new();
    for file in &changed {
        let in_scope = write_scope
            .iter()
            .any(|scope_pattern| lease_file_allowed(file, &[scope_pattern.clone()]));
        if !in_scope {
            outside.push(file.clone());
        }
    }

    (task_id, outside)
}

/// Check if a file path is allowed by the write scope entries.
pub fn lease_file_allowed(file: &str, scope: &[String]) -> bool {
    for entry in scope {
        let entry_clean = entry.trim();
        if entry_clean.is_empty() {
            continue;
        }

        // Directory match: "tests/conformance/" matches any file under it
        if entry_clean.ends_with('/') {
            if file.starts_with(entry_clean) || file.contains(entry_clean) {
                return true;
            }
            let dir = entry_clean.trim_end_matches('/');
            if file.starts_with(&format!("{}/", dir)) {
                return true;
            }
            continue;
        }

        // Glob pattern with * (e.g. "tests/conformance/auth-*.cronus")
        if entry_clean.contains('*') {
            if lease_glob_match(entry_clean, file) {
                return true;
            }
            continue;
        }

        // Exact match
        if file == entry_clean {
            return true;
        }

        // Partial match: "main.rs" matches "cronus-kernel/src/main.rs"
        if file.ends_with(entry_clean) {
            return true;
        }

        // Partial match: scope "cronus-kernel/src/main.rs" matches file "src/main.rs"
        if entry_clean.ends_with(file) {
            return true;
        }
    }
    false
}

/// Simple glob matching — supports single * wildcard
pub fn lease_glob_match(pattern: &str, text: &str) -> bool {
    if let Some(star_pos) = pattern.find('*') {
        let prefix = &pattern[..star_pos];
        let suffix = &pattern[star_pos + 1..];

        // Direct match
        if text.starts_with(prefix) && text.ends_with(suffix) {
            return true;
        }

        // Partial path match
        if text.ends_with(suffix) {
            if let Some(idx) = text.find(prefix) {
                let remaining = &text[idx + prefix.len()..];
                if remaining.ends_with(suffix) {
                    return true;
                }
            }
        }
    }
    pattern == text
}

#[allow(dead_code)]
pub enum DriftResult {
    Ok(String),
    Warn(String, Vec<String>),
}

/// Format unix timestamp as YYYY-MM-DD.
pub fn format_unix_date(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let mut y = 1970i64;
    let mut remaining = days;

    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }

    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0usize;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining < md as i64 {
            m = i + 1;
            break;
        }
        remaining -= md as i64;
    }
    if m == 0 {
        m = 12;
    }
    let d = remaining + 1;
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Calculate days between two YYYY-MM-DD date strings (target - from).
pub fn status_days_between(from: &str, target: &str) -> i64 {
    let from_days = status_parse_date_to_days(from);
    let target_days = status_parse_date_to_days(target);
    target_days - from_days
}

/// Parse YYYY-MM-DD to days since epoch (for simple subtraction).
pub fn status_parse_date_to_days(date: &str) -> i64 {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return 0;
    }
    let y: i64 = parts[0].parse().unwrap_or(1970);
    let m: i64 = parts[1].parse().unwrap_or(1);
    let d: i64 = parts[2].parse().unwrap_or(1);

    let mut total: i64 = 0;
    for yr in 1970..y {
        let leap = yr % 4 == 0 && (yr % 100 != 0 || yr % 400 == 0);
        total += if leap { 366 } else { 365 };
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let md = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    for i in 0..(m as usize - 1).min(11) {
        total += md[i] as i64;
    }
    total += d;
    total
}

/// Map a FieldType enum to its .cronus text representation.
pub fn reconcile_field_type_str(ft: &FieldType) -> &'static str {
    match ft {
        FieldType::String => "string",
        FieldType::Text => "text",
        FieldType::Email => "email",
        FieldType::Url => "url",
        FieldType::Slug => "slug",
        FieldType::Phone => "phone",
        FieldType::Number => "number",
        FieldType::Money => "money",
        FieldType::Percentage => "percentage",
        FieldType::Boolean => "boolean",
        FieldType::Date => "date",
        FieldType::Ulid => "ulid",
        FieldType::Json => "json",
        FieldType::Enum => "enum",
        FieldType::Ip => "ip",
        FieldType::Relation => "relation",
    }
}

/// Merge two named HashMaps: unique to A kept, unique to B added, overlap = conflict
pub fn reconcile_named_map<T: Clone>(
    merged: &mut Vec<AstNode>,
    conflicts: &mut Vec<String>,
    mut map_a: HashMap<String, T>,
    mut map_b: HashMap<String, T>,
    kind: &str,
    wrap: fn(T) -> AstNode,
) {
    let mut all_names: Vec<String> = map_a.keys().cloned().collect();
    for name in map_b.keys() {
        if !all_names.contains(name) {
            all_names.push(name.clone());
        }
    }
    all_names.sort();

    for name in &all_names {
        match (map_a.remove(name), map_b.remove(name)) {
            (Some(a), None) => merged.push(wrap(a)),
            (None, Some(b)) => merged.push(wrap(b)),
            (Some(a), Some(_b)) => {
                conflicts.push(format!("{} \"{}\" exists in both files", kind, name));
                merged.push(wrap(a));
            }
            (None, None) => {}
        }
    }
}

/// Emit a Vec<AstNode> as valid .cronus text
pub fn reconcile_emit(nodes: &[AstNode]) -> String {
    let mut out = String::new();

    for node in nodes {
        match node {
            AstNode::App(app) => {
                out.push_str(&format!("app \"{}\" {{\n", app.name));
                out.push_str(&format!("  stack {}\n", app.stack.join(", ")));
                out.push_str(&format!("  port {}\n", app.port));
                if let Some(db) = &app.database {
                    if let Some(path) = &db.path {
                        out.push_str(&format!("  database {} \"{}\"\n", db.db_type, path));
                    } else {
                        out.push_str(&format!("  database {}\n", db.db_type));
                    }
                }
                out.push_str("}\n\n");
            }
            AstNode::Style(style) => {
                out.push_str("style {\n");
                if let Some(theme) = &style.theme {
                    out.push_str(&format!("  theme {}\n", theme));
                }
                if let Some(accent) = &style.accent {
                    out.push_str(&format!("  accent {}\n", accent));
                }
                if let Some(radius) = &style.radius {
                    out.push_str(&format!("  radius {}\n", radius));
                }
                if let Some(font) = &style.font {
                    out.push_str(&format!("  font \"{}\"\n", font));
                }
                for (k, v) in &style.config {
                    out.push_str(&format!("  {} {}\n", k, v));
                }
                out.push_str("}\n\n");
            }
            AstNode::Auth(auth) => {
                out.push_str(&format!("auth {} {{\n", auth.entity));
                out.push_str(&format!("  login {}\n", auth.login_fields.join(", ")));
                out.push_str(&format!("  session {}", auth.session_type));
                if !auth.session_config.is_empty() {
                    let cfg: Vec<String> = auth
                        .session_config
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect();
                    out.push_str(&format!(" {}", cfg.join(", ")));
                }
                out.push('\n');
                if !auth.roles.is_empty() {
                    out.push_str(&format!("  roles {}\n", auth.roles.join(", ")));
                }
                out.push_str("}\n\n");
            }
            AstNode::Entity(entity) => {
                out.push_str(&format!("entity {} {{\n", entity.name));
                for field in &entity.fields {
                    let ft = reconcile_field_type_str(&field.field_type);
                    let mut modifiers = Vec::new();
                    if field.required {
                        modifiers.push("required");
                    }
                    if field.unique {
                        modifiers.push("unique");
                    }
                    if field.optional {
                        modifiers.push("optional");
                    }
                    if field.sensitive {
                        modifiers.push("sensitive");
                    }
                    if field.searchable {
                        modifiers.push("searchable");
                    }
                    if field.index {
                        modifiers.push("index");
                    }
                    if field.featured {
                        modifiers.push("featured");
                    }
                    if field.formatted {
                        modifiers.push("formatted");
                    }
                    if field.array {
                        modifiers.push("array");
                    }
                    let mod_str = if modifiers.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", modifiers.join(" "))
                    };
                    if let Some(ref_name) = &field.reference {
                        out.push_str(&format!(
                            "  {} {} -> {}{}\n",
                            field.name, ft, ref_name, mod_str
                        ));
                    } else if let Some(vals) = &field.enum_values {
                        out.push_str(&format!(
                            "  {} enum({}){}\n",
                            field.name,
                            vals.join(", "),
                            mod_str
                        ));
                    } else {
                        out.push_str(&format!("  {} {}{}\n", field.name, ft, mod_str));
                    }
                }
                out.push_str("}\n\n");
            }
            AstNode::Page(page) => {
                let mut header = format!("page \"{}\"", page.route);
                if !page.page_type.is_empty() {
                    header.push_str(&format!(" type:{}", page.page_type));
                }
                if let Some(entity) = &page.entity {
                    header.push_str(&format!(" entity:{}", entity));
                }
                if let Some(title) = &page.title {
                    header.push_str(&format!(" title:\"{}\"", title));
                }
                if let Some(req) = &page.requires {
                    header.push_str(&format!(" requires:{}", req));
                }
                out.push_str(&format!("{} {{\n", header));
                for sec in &page.sections {
                    out.push_str(&format!("  section {} {{\n", sec.section_type));
                    if let Some(title) = &sec.title {
                        out.push_str(&format!("    title \"{}\"\n", title));
                    }
                    if let Some(subtitle) = &sec.subtitle {
                        out.push_str(&format!("    subtitle \"{}\"\n", subtitle));
                    }
                    for (k, v) in &sec.config {
                        out.push_str(&format!("    {} {}\n", k, v));
                    }
                    out.push_str("  }\n");
                }
                out.push_str("}\n\n");
            }
            AstNode::Api(api) => {
                out.push_str(&format!("api \"{}\" {{\n", api.prefix));
                for route in &api.routes {
                    let method = format!("{:?}", route.method);
                    out.push_str(&format!("  {} {} \"{}\"\n", route.name, method, route.path));
                }
                out.push_str("}\n\n");
            }
            AstNode::Service(svc) => {
                out.push_str(&format!("service {} {{\n", svc.name));
                if let Some(port) = svc.port {
                    out.push_str(&format!("  port {}\n", port));
                }
                for (k, v) in &svc.config {
                    out.push_str(&format!("  {} {}\n", k, v));
                }
                out.push_str("}\n\n");
            }
            AstNode::Component(comp) => {
                let mut header = format!("component {}", comp.name);
                if let Some(layout) = &comp.layout {
                    header.push_str(&format!(" layout:{}", layout));
                }
                if let Some(style) = &comp.style {
                    header.push_str(&format!(" style:{}", style));
                }
                out.push_str(&format!("{} {{\n", header));
                for item in &comp.items {
                    out.push_str(&format!("  {} \"{}\"\n", item.item_type, item.text));
                }
                out.push_str("}\n\n");
            }
            AstNode::Event(evt) => {
                out.push_str(&format!("event {} {{\n", evt.name));
                for action in &evt.actions {
                    out.push_str(&format!("  {}\n", action));
                }
                out.push_str("}\n\n");
            }
            AstNode::Worker(w) => {
                out.push_str(&format!("worker {} {{\n", w.name));
                if let Some(q) = &w.queue {
                    out.push_str(&format!("  queue \"{}\"\n", q));
                }
                if let Some(c) = w.concurrency {
                    out.push_str(&format!("  concurrency {}\n", c));
                }
                if let Some(r) = w.retry {
                    out.push_str(&format!("  retry {}\n", r));
                }
                if let Some(t) = &w.timeout {
                    out.push_str(&format!("  timeout {}\n", t));
                }
                if let Some(e) = &w.entity {
                    out.push_str(&format!("  entity {}\n", e));
                }
                out.push_str("}\n\n");
            }
            AstNode::Middleware(mw) => {
                out.push_str(&format!("middleware {} {{\n", mw.name));
                if let Some(applies) = &mw.applies_to {
                    out.push_str(&format!("  applies_to {}\n", applies.join(", ")));
                }
                for (k, v) in &mw.config {
                    out.push_str(&format!("  {} {}\n", k, v));
                }
                out.push_str("}\n\n");
            }
            AstNode::Import(imp) => {
                if imp.alias.is_empty() {
                    out.push_str(&format!("import \"{}\"\n", imp.source));
                } else {
                    out.push_str(&format!("import {} from \"{}\"\n", imp.alias, imp.source));
                }
            }
            AstNode::Env(env) => {
                if env.name.is_empty() {
                    out.push_str("env {\n");
                } else {
                    out.push_str(&format!("env {} {{\n", env.name));
                }
                for (k, v) in &env.vars {
                    out.push_str(&format!("  {} \"{}\"\n", k, v));
                }
                for var in &env.schema {
                    out.push_str(&format!(
                        "  {} {}{}",
                        var.name,
                        var.env_type.keyword(),
                        if var.required { "!" } else { "" }
                    ));
                    if var.sensitive {
                        out.push_str(" sensitive");
                    }
                    if let Some(default) = &var.default {
                        out.push_str(&format!(" default:{}", default));
                    }
                    out.push('\n');
                }
                out.push_str("}\n\n");
            }
            AstNode::Test(test) => {
                out.push_str(&format!("test \"{}\" {{\n", test.name));
                for step in &test.steps {
                    out.push_str(&format!(
                        "  {} {} expect {}\n",
                        step.action, step.entity, step.expect
                    ));
                }
                out.push_str("}\n\n");
            }
            AstNode::Compose(comp) => {
                out.push_str(&format!("compose {} {{\n", comp.name));
                for u in &comp.uses {
                    out.push_str(&format!("  use {}\n", u));
                }
                out.push_str("}\n\n");
            }
            AstNode::Layout(layout) => {
                out.push_str(&format!("layout {} {{\n", layout.name));
                for (k, v) in &layout.sidebar_config {
                    out.push_str(&format!("  sidebar.{} \"{}\"\n", k, v));
                }
                for item in &layout.sidebar_items {
                    out.push_str(&format!("  nav \"{}\" \"{}\"\n", item.label, item.route));
                }
                out.push_str("}\n\n");
            }
            AstNode::Define(def) => {
                out.push_str(&format!(
                    "define \"{}\" {{\n  # {} section(s)\n}}\n\n",
                    def.name,
                    def.sections.len()
                ));
            }
            AstNode::Webhook(wh) => {
                out.push_str(&format!("webhook {} {{\n", wh.entity));
                for h in &wh.hooks {
                    out.push_str(&format!("  on {} -> {} \"{}\"\n", h.event, h.method, h.url));
                }
                out.push_str("}\n\n");
            }
            AstNode::Deploy(_) => {
                // Deploy nodes are handled by the microservices module
            }
        }
    }

    out
}
