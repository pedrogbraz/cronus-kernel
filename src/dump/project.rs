#![allow(dead_code, unused_imports, unused_variables)]
//! Black Hole project orchestrator v2
//!
//! Chains all dump phases to absorb an entire project directory
//! and emit a single .cronus file describing the full stack.
//! v2: page extraction, entity filtering, fetch route detection, CSS style.

use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;

pub struct ProjectProfile {
    pub name: String,
    pub stack: String,
    pub src_dir: PathBuf,
    pub prisma_path: Option<PathBuf>,
    pub openapi_path: Option<PathBuf>,
    pub tailwind_path: Option<PathBuf>,
    pub env_path: Option<PathBuf>,
    pub ts_files: Vec<PathBuf>,
    pub css_files: Vec<PathBuf>,
    pub port: u16,
}

pub struct PageDef {
    pub route: String,
    pub title: String,
    pub requires_auth: bool,
}

struct FetchRoute {
    method: String,
    path: String,
    name: String,
    auth: String,
}

/// Main entry: dump an entire project directory into .cronus
pub fn dump_project(dir: &Path) -> String {
    eprintln!(
        "\n  \x1b[36m⚡\x1b[0m Black Hole v2: absorbing {}...\n",
        dir.display()
    );

    // Phase 1: DETECT
    let profile = detect_project(dir);
    eprintln!("  Stack: {}", profile.stack);
    eprintln!("  Source files: {} TS, {} CSS", profile.ts_files.len(), profile.css_files.len());

    let mut entities_cronus = String::new();
    let mut routes_cronus = String::new();
    let auth_cronus;
    let mut style_cronus = String::new();

    // Phase 2: EXTRACT DATA LAYER
    // Try Prisma first (highest fidelity)
    if let Some(ref prisma_path) = profile.prisma_path {
        eprintln!(
            "  \x1b[32m✓\x1b[0m Prisma schema found: {}",
            prisma_path.display()
        );
        if let Ok(schema) = fs::read_to_string(prisma_path) {
            entities_cronus.push_str(&format!("# Entities from {}\n", prisma_path.display()));
            entities_cronus.push_str(&super::prisma::dump_prisma(&schema));
            entities_cronus = remove_app_block(&entities_cronus);
        }
    }

    // Try OpenAPI (high fidelity for routes)
    if let Some(ref openapi_path) = profile.openapi_path {
        eprintln!(
            "  \x1b[32m✓\x1b[0m OpenAPI spec found: {}",
            openapi_path.display()
        );
        if let Ok(json) = fs::read_to_string(openapi_path) {
            let dump = super::openapi::dump_openapi(&json);
            let dump_clean = remove_app_block(&dump);
            routes_cronus.push_str(&format!("# Routes from {}\n", openapi_path.display()));
            routes_cronus.push_str(&extract_api_blocks(&dump_clean));

            // Only add entities if Prisma didn't already provide them
            if entities_cronus.is_empty() {
                entities_cronus.push_str(&extract_entity_blocks(&dump_clean));
            }
        }
    }

    // TypeScript interface extraction (fallback if no Prisma/OpenAPI)
    // v2: path-aware filtering — only scan domain files
    if entities_cronus.is_empty() && !profile.ts_files.is_empty() {
        eprintln!("  \x1b[33m⚡\x1b[0m Scanning TypeScript interfaces (filtered)...");
        let mut all_entities = Vec::new();
        let mut scanned = 0u32;
        for file in &profile.ts_files {
            if !super::typescript::is_domain_file(file) {
                continue;
            }
            scanned += 1;
            if let Ok(source) = fs::read_to_string(file) {
                let extracted = super::typescript::extract_entities_from_file(&source, file);
                all_entities.extend(extracted);
            }
        }
        // Deduplicate by name
        let mut seen = std::collections::HashSet::new();
        all_entities.retain(|e| seen.insert(e.name.clone()));

        if !all_entities.is_empty() {
            eprintln!(
                "  \x1b[32m✓\x1b[0m Found {} entities from {} domain files",
                all_entities.len(), scanned
            );
            entities_cronus.push_str("# Entities extracted from TypeScript interfaces\n");
            entities_cronus.push_str(&super::typescript::emit_entities(&all_entities));
        }
    }

    // Phase 3: EXTRACT ROUTES (if OpenAPI didn't provide them)
    if routes_cronus.is_empty() {
        let detected = super::routes::detect_routes(&profile);
        if !detected.is_empty() {
            eprintln!(
                "  \x1b[32m✓\x1b[0m Detected {} routes",
                detected.len()
            );
            routes_cronus.push_str("# Routes detected from source code\n");
            routes_cronus.push_str(&super::routes::emit_routes(&detected));
        }
    }

    // Phase 3.5: EXTRACT PAGES
    let pages = detect_pages(&profile);
    let mut pages_cronus = String::new();
    if !pages.is_empty() {
        eprintln!(
            "  \x1b[32m✓\x1b[0m Detected {} pages",
            pages.len()
        );
        pages_cronus.push_str("# Pages detected from project structure\n");
        for page in &pages {
            pages_cronus.push_str(&format!(
                "page \"{}\" type:custom {}{{\n  title \"{}\"\n}}\n\n",
                page.route,
                if page.requires_auth { "requires:auth " } else { "" },
                page.title
            ));
        }
    }

    // Phase 3.7: EXTRACT FETCH ROUTES (API calls from frontend code)
    let fetch_routes = detect_fetch_routes(&profile.ts_files);
    let mut fetch_cronus = String::new();
    if !fetch_routes.is_empty() {
        eprintln!(
            "  \x1b[32m✓\x1b[0m Detected {} fetch API endpoints",
            fetch_routes.len()
        );
        fetch_cronus.push_str("# API endpoints detected from fetch calls\n");
        fetch_cronus.push_str(&emit_fetch_routes(&fetch_routes));
    }

    // Phase 4: EXTRACT STYLE
    // Try tailwind config first
    if let Some(ref tw_path) = profile.tailwind_path {
        eprintln!("  \x1b[32m✓\x1b[0m Tailwind config found");
        if let Ok(config) = fs::read_to_string(tw_path) {
            style_cronus = super::style_extract::extract_style(&config, &profile.ts_files);
        }
    }

    // v2: If no tailwind config, check CSS files for @theme or color variables
    if style_cronus.is_empty() {
        for css_file in &profile.css_files {
            if let Ok(css) = fs::read_to_string(css_file) {
                if css.contains("--color-") || css.contains("@theme") {
                    eprintln!(
                        "  \x1b[32m✓\x1b[0m CSS theme found: {}",
                        css_file.file_name().unwrap_or_default().to_string_lossy()
                    );
                    style_cronus = super::style_extract::extract_style_from_css(&css, &profile.ts_files);
                    break;
                }
            }
        }
    }

    // Phase 5: DETECT AUTH
    auth_cronus = detect_auth(&profile);

    // Phase 6: ASSEMBLE
    let mut output = String::new();

    // Header comment
    output.push_str(&format!("# Generated by: cronus dump {}\n", dir.display()));
    output.push_str(&format!("# Stack: {}\n\n", profile.stack));

    // App block
    output.push_str(&format!("app \"{}\" {{\n", profile.name));
    output.push_str("  stack react + tailwind\n");
    output.push_str(&format!("  port {}\n", profile.port));
    output.push_str("  database sqlite \"./data.db\"\n");
    output.push_str("  theme dark\n");
    output.push_str("}\n\n");

    // Auth
    if !auth_cronus.is_empty() {
        output.push_str(&auth_cronus);
        output.push('\n');
    }

    // Style
    if !style_cronus.is_empty() {
        output.push_str(&style_cronus);
        output.push('\n');
    }

    // Pages
    if !pages_cronus.is_empty() {
        output.push_str(&pages_cronus);
    }

    // Entities
    if !entities_cronus.is_empty() {
        output.push_str(&entities_cronus);
    }

    // Routes
    if !routes_cronus.is_empty() {
        output.push_str(&routes_cronus);
    }

    // Fetch routes (if no explicit routes were found)
    if routes_cronus.is_empty() && !fetch_cronus.is_empty() {
        output.push_str(&fetch_cronus);
    }

    // Summary
    let entity_count = output.matches("\nentity ").count() + if output.starts_with("entity ") { 1 } else { 0 };
    let page_count = output.matches("\npage ").count() + if output.starts_with("page ") { 1 } else { 0 };
    let api_count = output.matches("\napi ").count() + if output.starts_with("api ") { 1 } else { 0 };
    let style_count = if output.contains("\nstyle ") || output.starts_with("style ") { 1 } else { 0 };
    eprintln!(
        "\n  \x1b[32m✓\x1b[0m Generated: {} entities, {} pages, {} api blocks, {} style",
        entity_count, page_count, api_count, style_count
    );

    output
}

fn detect_project(dir: &Path) -> ProjectProfile {
    let mut profile = ProjectProfile {
        name: dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        stack: "unknown".to_string(),
        src_dir: dir.to_path_buf(),
        prisma_path: None,
        openapi_path: None,
        tailwind_path: None,
        env_path: None,
        ts_files: Vec::new(),
        css_files: Vec::new(),
        port: 3000,
    };

    // Read package.json
    let pkg_path = dir.join("package.json");
    if let Ok(content) = fs::read_to_string(&pkg_path) {
        if let Ok(pkg) = serde_json::from_str::<Value>(&content) {
            // Name
            if let Some(name) = pkg.get("name").and_then(|v| v.as_str()) {
                profile.name = name.to_string();
            }

            // Stack detection
            let deps_str = format!(
                "{}{}",
                pkg.get("dependencies").unwrap_or(&Value::Null),
                pkg.get("devDependencies").unwrap_or(&Value::Null)
            );

            if deps_str.contains("\"next\"") {
                profile.stack = "nextjs".to_string();
            } else if deps_str.contains("\"react\"") {
                profile.stack = "react".to_string();
            } else if deps_str.contains("\"vue\"") {
                profile.stack = "vue".to_string();
            } else if deps_str.contains("\"svelte\"") {
                profile.stack = "svelte".to_string();
            } else if deps_str.contains("\"elysia\"") {
                profile.stack = "elysia".to_string();
            } else if deps_str.contains("\"express\"") {
                profile.stack = "express".to_string();
            } else if deps_str.contains("\"@angular/core\"") {
                profile.stack = "angular".to_string();
            }

            // Port from scripts
            if let Some(scripts) = pkg.get("scripts").and_then(|v| v.as_object()) {
                if let Some(dev) = scripts.get("dev").and_then(|v| v.as_str()) {
                    if let Some(port_pos) = dev.find("--port ").or(dev.find("-p ")) {
                        let after = &dev[port_pos..];
                        let num_start = after.find(char::is_numeric).unwrap_or(0);
                        let num_str: String = after[num_start..]
                            .chars()
                            .take_while(|c| c.is_numeric())
                            .collect();
                        if let Ok(p) = num_str.parse::<u16>() {
                            profile.port = p;
                        }
                    }
                }
            }
        }
    }

    // Find Prisma schema
    for candidate in &[
        "prisma/schema.prisma",
        "schema.prisma",
        "db/schema.prisma",
    ] {
        let p = dir.join(candidate);
        if p.exists() {
            profile.prisma_path = Some(p);
            break;
        }
    }

    // Find OpenAPI
    for candidate in &[
        "openapi.json",
        "swagger.json",
        "api/openapi.json",
        "docs/openapi.json",
        "public/openapi.json",
    ] {
        let p = dir.join(candidate);
        if p.exists() {
            profile.openapi_path = Some(p);
            break;
        }
    }

    // Find Tailwind
    for candidate in &[
        "tailwind.config.ts",
        "tailwind.config.js",
        "tailwind.config.mjs",
        "tailwind.config.cjs",
    ] {
        let p = dir.join(candidate);
        if p.exists() {
            profile.tailwind_path = Some(p);
            break;
        }
    }

    // Find .env
    for candidate in &[".env.example", ".env.local", ".env"] {
        let p = dir.join(candidate);
        if p.exists() {
            profile.env_path = Some(p);
            break;
        }
    }

    // Collect TS/JS files (max 500)
    collect_source_files(dir, &mut profile.ts_files, &mut profile.css_files, 0);
    profile.ts_files.truncate(500);

    profile
}

fn collect_source_files(dir: &Path, ts_files: &mut Vec<PathBuf>, css_files: &mut Vec<PathBuf>, depth: usize) {
    if depth > 10 || ts_files.len() >= 500 {
        return;
    }

    let skip_dirs = [
        "node_modules",
        ".git",
        "target",
        "dist",
        ".next",
        "build",
        "__pycache__",
        ".cronus-build",
        ".vite",
        "coverage",
        ".turbo",
    ];

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if path.is_dir() {
                if !skip_dirs.contains(&name.as_str()) && !name.starts_with('.') {
                    collect_source_files(&path, ts_files, css_files, depth + 1);
                }
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if matches!(ext, "ts" | "tsx" | "js" | "jsx") {
                    // Skip files > 100KB
                    if let Ok(meta) = fs::metadata(&path) {
                        if meta.len() <= 100_000 {
                            ts_files.push(path);
                        }
                    }
                } else if ext == "css" {
                    css_files.push(path);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Page detection
// ---------------------------------------------------------------------------

fn detect_pages(profile: &ProjectProfile) -> Vec<PageDef> {
    let mut pages = Vec::new();

    // Next.js: scan app/ or pages/ directory for page.tsx files
    let dirs_to_try = [
        profile.src_dir.join("src/app"),
        profile.src_dir.join("app"),
        profile.src_dir.join("src/pages"),
        profile.src_dir.join("pages"),
    ];

    for dir in &dirs_to_try {
        if dir.exists() {
            let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();
            if dir_name == "app" {
                scan_app_dir(dir, "", &mut pages);
            } else {
                scan_pages_dir(dir, "", &mut pages);
            }
            if !pages.is_empty() {
                return pages;
            }
        }
    }

    // React: scan for route definitions in App.tsx or routes.tsx
    if pages.is_empty() {
        for file in &profile.ts_files {
            let name = file.file_name().unwrap_or_default().to_string_lossy().to_string();
            if name == "App.tsx" || name == "app.tsx" || name == "routes.tsx" || name == "router.tsx" {
                if let Ok(source) = fs::read_to_string(file) {
                    extract_react_routes(&source, &mut pages);
                    if !pages.is_empty() {
                        break;
                    }
                }
            }
        }
    }

    pages
}

/// Scan Next.js app/ directory for page.tsx files
fn scan_app_dir(dir: &Path, prefix: &str, pages: &mut Vec<PageDef>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if path.is_dir() {
                // Skip route groups, api routes, and hidden dirs
                if name.starts_with('.') || name == "api" {
                    continue;
                }

                // Route groups: (auth) → don't add segment
                let segment = if name.starts_with('(') && name.ends_with(')') {
                    String::new()
                } else if name.starts_with('[') && name.ends_with(']') {
                    format!("/:{}", &name[1..name.len() - 1])
                } else {
                    format!("/{}", name)
                };

                scan_app_dir(&path, &format!("{}{}", prefix, segment), pages);
            } else if name == "page.tsx" || name == "page.jsx" || name == "page.ts" || name == "page.js" {
                let route = if prefix.is_empty() { "/".to_string() } else { prefix.to_string() };
                let title = route_to_title(&route);
                let requires_auth = check_has_auth_layout(dir);
                pages.push(PageDef { route, title, requires_auth });
            }
        }
    }
}

/// Scan traditional pages/ directory (Next.js pages router or file-based)
fn scan_pages_dir(dir: &Path, prefix: &str, pages: &mut Vec<PageDef>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if path.is_dir() {
                if name.starts_with('.') || name == "api" || name == "_app" || name == "_document" {
                    continue;
                }
                let segment = if name.starts_with('[') && name.ends_with(']') {
                    format!("/:{}", &name[1..name.len() - 1])
                } else {
                    format!("/{}", name)
                };
                scan_pages_dir(&path, &format!("{}{}", prefix, segment), pages);
            } else {
                let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                let ext = path.extension().unwrap_or_default().to_string_lossy().to_string();

                if !matches!(ext.as_str(), "tsx" | "jsx" | "ts" | "js") {
                    continue;
                }
                // Skip _app, _document, _error, 404, 500
                if stem.starts_with('_') || stem == "404" || stem == "500" {
                    continue;
                }

                let route = if stem == "index" {
                    if prefix.is_empty() { "/".to_string() } else { prefix.to_string() }
                } else {
                    format!("{}/{}", prefix, stem.to_lowercase())
                };

                let title = route_to_title(&route);
                pages.push(PageDef {
                    route,
                    title,
                    requires_auth: false,
                });
            }
        }
    }
}

/// Extract routes from React Router patterns in source code
fn extract_react_routes(source: &str, pages: &mut Vec<PageDef>) {
    // Pattern 1: PATH_TO_PAGE or route map objects like { "/path": "name" }
    // Look for string literal keys that look like routes
    for line in source.lines() {
        let line = line.trim();

        // Match: "/route": "pageName" or path="/route"
        if let Some(route) = extract_route_from_line(line) {
            if route.len() > 1 && route.len() < 80 && !pages.iter().any(|p| p.route == route) {
                let title = route_to_title(&route);
                pages.push(PageDef {
                    route,
                    title,
                    requires_auth: false,
                });
            }
        }
    }
}

fn extract_route_from_line(line: &str) -> Option<String> {
    // Pattern: "/something": "value"  (route map)
    let line = line.trim();
    if line.starts_with('"') || line.starts_with('\'') {
        let quote = line.chars().next()?;
        let end = line[1..].find(quote)?;
        let path = &line[1..1 + end];
        if path.starts_with('/') && !path.contains("api") && !path.contains("http") {
            return Some(path.to_string());
        }
    }

    // Pattern: path="/something"
    if let Some(pos) = line.find("path=") {
        let after = &line[pos + 5..];
        let quote = after.chars().next()?;
        if quote == '"' || quote == '\'' {
            let end = after[1..].find(quote)?;
            let path = &after[1..1 + end];
            if path.starts_with('/') && path.len() < 80 {
                return Some(path.to_string());
            }
        }
    }

    None
}

fn route_to_title(route: &str) -> String {
    if route == "/" {
        return "Home".to_string();
    }
    let last = route.split('/').filter(|s| !s.is_empty() && !s.starts_with(':')).last().unwrap_or("Page");
    // Capitalize first letter, replace hyphens
    let mut title = last.replace('-', " ");
    if let Some(first) = title.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    title
}

fn check_has_auth_layout(dir: &Path) -> bool {
    // Check if layout.tsx in this or parent dir contains auth references
    let layout = dir.join("layout.tsx");
    if let Ok(content) = fs::read_to_string(&layout) {
        return content.contains("auth") || content.contains("session") || content.contains("protect");
    }
    false
}

// ---------------------------------------------------------------------------
// Fetch route detection (API calls from frontend code)
// ---------------------------------------------------------------------------

fn detect_fetch_routes(ts_files: &[PathBuf]) -> Vec<FetchRoute> {
    let mut routes = Vec::new();

    let patterns = ["\"/api/", "'/api/", "`/api/", "\"/tasks", "\"/agents", "\"/events", "\"/db/"];

    for file in ts_files {
        if let Ok(source) = fs::read_to_string(file) {
            for line in source.lines() {
                for pat in &patterns {
                    if let Some(pos) = line.find(pat) {
                        let start = pos + 1; // skip the opening quote
                        let rest = &line[start..];
                        let end = rest
                            .find(|c: char| c == '"' || c == '\'' || c == '`' || c == '$' || c == '{')
                            .unwrap_or(rest.len());
                        let path = &rest[..end];
                        if path.len() > 3 && path.len() < 100 {
                            let method = if line.contains("POST") || line.contains("post") || line.contains("method: \"POST\"") {
                                "POST"
                            } else if line.contains("DELETE") || line.contains("delete") {
                                "DELETE"
                            } else if line.contains("PATCH") || line.contains("patch") || line.contains("PUT") || line.contains("put") {
                                "PATCH"
                            } else {
                                "GET"
                            };

                            routes.push(FetchRoute {
                                method: method.to_string(),
                                path: path.to_string(),
                                name: infer_fetch_name(path, method),
                                auth: "jwt".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Deduplicate
    routes.sort_by(|a, b| (&a.path, &a.method).cmp(&(&b.path, &b.method)));
    routes.dedup_by(|a, b| a.path == b.path && a.method == b.method);
    routes
}

fn infer_fetch_name(path: &str, method: &str) -> String {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let resource = segments.last().unwrap_or(&"resource");

    match method {
        "POST" => format!("create_{}", resource),
        "DELETE" => format!("delete_{}", resource),
        "PATCH" | "PUT" => format!("update_{}", resource),
        _ => format!("list_{}", resource),
    }
}

fn emit_fetch_routes(routes: &[FetchRoute]) -> String {
    if routes.is_empty() {
        return String::new();
    }

    // Group by path prefix
    let mut groups: std::collections::HashMap<String, Vec<&FetchRoute>> = std::collections::HashMap::new();

    for route in routes {
        let parts: Vec<&str> = route.path.split('/').filter(|s| !s.is_empty()).collect();
        let prefix = if parts.len() >= 2 {
            format!("/{}/{}", parts[0], parts[1])
        } else {
            format!("/{}", parts.first().unwrap_or(&"api"))
        };
        groups.entry(prefix).or_default().push(route);
    }

    let mut out = String::new();
    let mut keys: Vec<&String> = groups.keys().collect();
    keys.sort();

    for prefix in keys {
        let group = &groups[prefix];
        out.push_str(&format!("api {} {{\n", prefix));
        for r in group {
            let relative = r.path.trim_start_matches(prefix.as_str());
            let relative = if relative.is_empty() { "/" } else { relative };
            out.push_str(&format!(
                "  {:<20} {:<8} {:<24} auth:{}\n",
                r.name, r.method, relative, r.auth
            ));
        }
        out.push_str("}\n\n");
    }

    out
}

// ---------------------------------------------------------------------------
// Auth detection
// ---------------------------------------------------------------------------

fn detect_auth(profile: &ProjectProfile) -> String {
    // Check package.json deps for auth libraries
    let pkg_path = profile.src_dir.join("package.json");
    if let Ok(content) = fs::read_to_string(&pkg_path) {
        let has_auth = content.contains("next-auth")
            || content.contains("@auth/")
            || content.contains("passport")
            || content.contains("jsonwebtoken")
            || content.contains("bcrypt")
            || content.contains("argon2")
            || content.contains("lucia")
            || content.contains("clerk");

        if has_auth {
            return "auth {\n  entity User\n  login email + password\n  session jwt\n}\n"
                .to_string();
        }
    }

    // Check source files for auth patterns
    for file in profile.ts_files.iter().take(100) {
        if let Ok(source) = fs::read_to_string(file) {
            if source.contains("signIn")
                || source.contains("sign_in")
                || source.contains("login")
                || source.contains("authenticate")
                || source.contains("JWT")
                || source.contains("jwt")
            {
                return "auth {\n  entity User\n  login email + password\n  session jwt\n}\n"
                    .to_string();
            }
        }
    }

    String::new()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn remove_app_block(cronus: &str) -> String {
    let mut result = String::new();
    let mut in_app_block = false;
    let mut brace_depth = 0;

    for line in cronus.lines() {
        if line.trim().starts_with("app ") || line.trim().starts_with("app \"") {
            in_app_block = true;
            brace_depth = 0;
        }

        if in_app_block {
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth += 1;
                }
                if ch == '}' {
                    brace_depth -= 1;
                }
            }
            if brace_depth <= 0 && line.contains('}') {
                in_app_block = false;
            }
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

fn extract_entity_blocks(cronus: &str) -> String {
    let mut result = String::new();
    let mut in_entity = false;
    let mut brace_depth = 0;

    for line in cronus.lines() {
        if line.trim().starts_with("entity ") {
            in_entity = true;
            brace_depth = 0;
        }

        if in_entity {
            result.push_str(line);
            result.push('\n');
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth += 1;
                }
                if ch == '}' {
                    brace_depth -= 1;
                }
            }
            if brace_depth <= 0 && line.contains('}') {
                in_entity = false;
                result.push('\n');
            }
        }
    }

    result
}

fn extract_api_blocks(cronus: &str) -> String {
    let mut result = String::new();
    let mut in_api = false;
    let mut brace_depth = 0;

    for line in cronus.lines() {
        if line.trim().starts_with("api ") {
            in_api = true;
            brace_depth = 0;
        }

        if in_api {
            result.push_str(line);
            result.push('\n');
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth += 1;
                }
                if ch == '}' {
                    brace_depth -= 1;
                }
            }
            if brace_depth <= 0 && line.contains('}') {
                in_api = false;
                result.push('\n');
            }
        }
    }

    result
}
