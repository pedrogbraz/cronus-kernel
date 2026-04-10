#![allow(dead_code, unused_imports, unused_variables)]
//! Next.js / VINEXT → .cronus converter
//!
//! Scans a Next.js project directory (App Router + Pages Router)
//! and emits a complete .cronus file with entities, pages, API routes,
//! auth, layout, middleware, and style configuration.
//!
//! Follows the exact Next.js file-system routing conventions
//! as documented by VINEXT's routing scanner.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// Public data structures
// ─────────────────────────────────────────────────────────────────────────────

pub struct NextJsProject {
    pub name: String,
    pub port: u16,
    pub has_app_router: bool,
    pub has_pages_router: bool,
    pub app_dir: Option<PathBuf>,
    pub pages_dir: Option<PathBuf>,
    pub pages: Vec<NextPage>,
    pub api_routes: Vec<NextApiRoute>,
    pub layouts: Vec<NextLayout>,
    pub middleware: Option<MiddlewareInfo>,
    pub config: NextConfig,
    pub auth: AuthInfo,
    pub style: StyleInfo,
    pub prisma_entities: String,
}

pub struct NextPage {
    pub route: String,
    pub file_path: PathBuf,
    pub title: String,
    pub is_dynamic: bool,
    pub params: Vec<String>,
    pub has_loading: bool,
    pub has_error: bool,
    pub requires_auth: bool,
    pub page_type: String,
    pub data_fetching: Option<String>,
}

pub struct NextApiRoute {
    pub route: String,
    pub methods: Vec<String>,
    pub has_auth: bool,
    pub is_dynamic: bool,
    pub handler_type: String,
}

pub struct NextLayout {
    pub route_prefix: String,
    pub file_path: PathBuf,
    pub has_sidebar: bool,
    pub has_topbar: bool,
    pub brand: Option<String>,
    pub nav_items: Vec<NavItem>,
}

pub struct NavItem {
    pub label: String,
    pub href: String,
    pub icon: Option<String>,
}

pub struct MiddlewareInfo {
    pub matchers: Vec<String>,
    pub has_auth_check: bool,
    pub has_redirect: bool,
}

pub struct NextConfig {
    pub base_path: Option<String>,
    pub trailing_slash: bool,
    pub i18n: Option<I18nConfig>,
    pub redirects: Vec<RedirectRule>,
    pub images_domains: Vec<String>,
}

pub struct I18nConfig {
    pub locales: Vec<String>,
    pub default_locale: String,
}

pub struct RedirectRule {
    pub source: String,
    pub destination: String,
    pub permanent: bool,
}

pub struct AuthInfo {
    pub detected: bool,
    pub provider: String,
    pub roles: Vec<String>,
}

pub struct StyleInfo {
    pub theme: String,
    pub accent: Option<String>,
    pub font: Option<String>,
    pub has_tailwind: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Main entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Scans a Next.js project and returns complete .cronus source
pub fn dump_nextjs(dir: &Path) -> String {
    eprintln!(
        "\n  \x1b[36m⚡\x1b[0m CRONUS × VINEXT: absorbing Next.js project...\n"
    );

    let project = scan_project(dir);

    eprintln!("  \x1b[1m{}\x1b[0m", project.name);
    eprintln!(
        "  Router: {}",
        match (project.has_app_router, project.has_pages_router) {
            (true, true) => "App + Pages (hybrid)",
            (true, false) => "App Router",
            (false, true) => "Pages Router",
            _ => "none detected",
        }
    );
    eprintln!("  Pages: {}", project.pages.len());
    eprintln!("  API routes: {}", project.api_routes.len());
    eprintln!("  Layouts: {}", project.layouts.len());
    if project.middleware.is_some() {
        eprintln!("  Middleware: \x1b[32m✓\x1b[0m");
    }
    if project.auth.detected {
        eprintln!("  Auth: {} \x1b[32m✓\x1b[0m", project.auth.provider);
    }
    eprintln!();

    emit_cronus(&project)
}

// ─────────────────────────────────────────────────────────────────────────────
// Project scanning
// ─────────────────────────────────────────────────────────────────────────────

fn scan_project(dir: &Path) -> NextJsProject {
    let name = detect_name(dir);
    let port = detect_port(dir);

    // Detect router type
    let (app_dir, pages_dir) = detect_router_dirs(dir);
    let has_app_router = app_dir.is_some();
    let has_pages_router = pages_dir.is_some();

    // Scan pages
    let mut pages = Vec::new();
    if let Some(ref app) = app_dir {
        pages.extend(scan_app_router_pages(app, app));
    }
    if let Some(ref pgs) = pages_dir {
        pages.extend(scan_pages_router_pages(pgs, pgs));
    }

    // Scan API routes
    let mut api_routes = Vec::new();
    if let Some(ref app) = app_dir {
        let api = app.join("api");
        if api.exists() {
            scan_app_api_routes(&api, "/api", &mut api_routes);
        }
    }
    if let Some(ref pgs) = pages_dir {
        let api = pgs.join("api");
        if api.exists() {
            scan_pages_api_routes(&api, "/api", &mut api_routes);
        }
    }

    // Scan layouts
    let mut layouts = Vec::new();
    if let Some(ref app) = app_dir {
        scan_layouts(app, app, &mut layouts);
    }

    // Detect middleware
    let middleware = detect_middleware(dir);

    // Detect config
    let config = detect_next_config(dir);

    // Detect auth
    let auth = detect_auth(dir);

    // Detect style
    let style = detect_style(dir);

    // Detect Prisma entities
    let prisma_entities = detect_prisma(dir);

    // Infer auth requirements on pages from middleware/layout patterns
    if let Some(ref mw) = middleware {
        if mw.has_auth_check {
            for page in &mut pages {
                for matcher in &mw.matchers {
                    if page.route.starts_with(matcher) || matcher == "/" {
                        page.requires_auth = true;
                    }
                }
            }
        }
    }

    NextJsProject {
        name,
        port,
        has_app_router,
        has_pages_router,
        app_dir,
        pages_dir,
        pages,
        api_routes,
        layouts,
        middleware,
        config,
        auth,
        style,
        prisma_entities,
    }
}

fn detect_name(dir: &Path) -> String {
    let pkg = dir.join("package.json");
    if let Ok(content) = fs::read_to_string(&pkg) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                return name.to_string();
            }
        }
    }
    dir.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

fn detect_port(dir: &Path) -> u16 {
    let pkg = dir.join("package.json");
    if let Ok(content) = fs::read_to_string(&pkg) {
        if content.contains("--port") {
            // Try to extract port from scripts
            if let Some(pos) = content.find("--port") {
                let after = &content[pos + 6..];
                let trimmed = after.trim_start().trim_start_matches('=').trim_start();
                let num: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(p) = num.parse::<u16>() {
                    return p;
                }
            }
        }
    }
    3000
}

fn detect_router_dirs(dir: &Path) -> (Option<PathBuf>, Option<PathBuf>) {
    let candidates_app = [
        dir.join("app"),
        dir.join("src/app"),
    ];
    let candidates_pages = [
        dir.join("pages"),
        dir.join("src/pages"),
    ];

    let app_dir = candidates_app.iter().find(|p| p.exists()).cloned();
    let pages_dir = candidates_pages.iter().find(|p| p.exists()).cloned();

    (app_dir, pages_dir)
}

// ─────────────────────────────────────────────────────────────────────────────
// App Router page scanning (follows VINEXT conventions exactly)
// ─────────────────────────────────────────────────────────────────────────────

fn scan_app_router_pages(dir: &Path, app_root: &Path) -> Vec<NextPage> {
    let mut pages = Vec::new();
    scan_app_dir_recursive(dir, app_root, &mut pages);
    pages
}

fn scan_app_dir_recursive(dir: &Path, app_root: &Path, pages: &mut Vec<NextPage>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            // Skip private dirs, node_modules, .next
            if name.starts_with('_') || name == "node_modules" || name == ".next" {
                continue;
            }
            scan_app_dir_recursive(&path, app_root, pages);
        } else if is_page_file(&name) {
            // Convert file path to route pattern
            let route = file_to_app_route(&path, app_root);
            let has_loading = dir.join("loading.tsx").exists() || dir.join("loading.js").exists();
            let has_error = dir.join("error.tsx").exists() || dir.join("error.js").exists();

            // Read file to detect data fetching patterns
            let content = fs::read_to_string(&path).unwrap_or_default();
            let data_fetching = detect_data_fetching(&content);
            let requires_auth = content.contains("auth")
                || content.contains("session")
                || content.contains("getServerSession");

            let page_type = infer_page_type(&route, &content);

            pages.push(NextPage {
                route: route.clone(),
                file_path: path,
                title: route_to_title(&route),
                is_dynamic: route.contains(':'),
                params: extract_params(&route),
                has_loading,
                has_error,
                requires_auth,
                page_type,
                data_fetching,
            });
        }
    }
}

fn is_page_file(name: &str) -> bool {
    name == "page.tsx" || name == "page.ts" || name == "page.jsx" || name == "page.js"
}

fn is_route_file(name: &str) -> bool {
    name == "route.tsx" || name == "route.ts" || name == "route.js"
}

fn file_to_app_route(file: &Path, app_root: &Path) -> String {
    let relative = file.parent().unwrap_or(file)
        .strip_prefix(app_root)
        .unwrap_or(Path::new(""));

    let mut parts = Vec::new();
    for component in relative.components() {
        let seg = component.as_os_str().to_string_lossy().to_string();

        // Skip invisible segments
        if seg.starts_with('(') && seg.ends_with(')') {
            continue; // Route group — transparent
        }
        if seg.starts_with('@') {
            continue; // Parallel slot
        }
        if seg == "." {
            continue;
        }

        // Convert dynamic segments
        if seg.starts_with("[[...") && seg.ends_with("]]") {
            // Optional catch-all
            let param = &seg[5..seg.len() - 2];
            parts.push(format!(":{}", param));
        } else if seg.starts_with("[...") && seg.ends_with(']') {
            // Catch-all
            let param = &seg[4..seg.len() - 1];
            parts.push(format!(":{}", param));
        } else if seg.starts_with('[') && seg.ends_with(']') {
            // Dynamic segment
            let param = &seg[1..seg.len() - 1];
            parts.push(format!(":{}", param));
        } else {
            parts.push(seg);
        }
    }

    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Pages Router page scanning
// ─────────────────────────────────────────────────────────────────────────────

fn scan_pages_router_pages(dir: &Path, pages_root: &Path) -> Vec<NextPage> {
    let mut pages = Vec::new();
    scan_pages_dir_recursive(dir, pages_root, &mut pages);
    pages
}

fn scan_pages_dir_recursive(dir: &Path, pages_root: &Path, pages: &mut Vec<NextPage>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            if name == "api" || name == "node_modules" || name == "_app" || name == "_document" {
                continue;
            }
            scan_pages_dir_recursive(&path, pages_root, pages);
        } else if is_page_extension(&name) {
            // Skip special Next.js files
            let stem = name.split('.').next().unwrap_or("");
            if stem.starts_with('_') {
                continue;
            }

            let route = file_to_pages_route(&path, pages_root);
            let content = fs::read_to_string(&path).unwrap_or_default();
            let data_fetching = detect_pages_data_fetching(&content);
            let requires_auth = content.contains("getServerSession")
                || content.contains("useSession")
                || content.contains("withAuth");

            pages.push(NextPage {
                route: route.clone(),
                file_path: path,
                title: route_to_title(&route),
                is_dynamic: route.contains(':'),
                params: extract_params(&route),
                has_loading: false,
                has_error: false,
                requires_auth,
                page_type: infer_page_type(&route, &content),
                data_fetching,
            });
        }
    }
}

fn is_page_extension(name: &str) -> bool {
    name.ends_with(".tsx") || name.ends_with(".ts")
        || name.ends_with(".jsx") || name.ends_with(".js")
}

fn file_to_pages_route(file: &Path, pages_root: &Path) -> String {
    let relative = file
        .strip_prefix(pages_root)
        .unwrap_or(Path::new(""));

    let mut route = relative.to_string_lossy().to_string();

    // Strip extension
    for ext in &[".tsx", ".ts", ".jsx", ".js"] {
        if route.ends_with(ext) {
            route = route[..route.len() - ext.len()].to_string();
            break;
        }
    }

    // Convert index to /
    if route == "index" {
        return "/".to_string();
    }
    if route.ends_with("/index") {
        route = route[..route.len() - 6].to_string();
    }

    // Convert dynamic segments
    route = route
        .replace("[[...", "[...")  // normalize optional catch-all
        .replace("]]", "]");

    let parts: Vec<String> = route.split('/')
        .map(|seg| {
            if seg.starts_with("[...") && seg.ends_with(']') {
                let param = &seg[4..seg.len() - 1];
                format!(":{}", param)
            } else if seg.starts_with('[') && seg.ends_with(']') {
                let param = &seg[1..seg.len() - 1];
                format!(":{}", param)
            } else {
                seg.to_string()
            }
        })
        .collect();

    format!("/{}", parts.join("/"))
}

// ─────────────────────────────────────────────────────────────────────────────
// API route scanning
// ─────────────────────────────────────────────────────────────────────────────

fn scan_app_api_routes(dir: &Path, prefix: &str, routes: &mut Vec<NextApiRoute>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            // Route groups — skip segment
            if name.starts_with('(') && name.ends_with(')') {
                scan_app_api_routes(&path, prefix, routes);
                continue;
            }

            let segment = if name.starts_with('[') && name.ends_with(']') {
                format!(":{}", &name[1..name.len() - 1].replace("...", ""))
            } else {
                name
            };

            scan_app_api_routes(&path, &format!("{}/{}", prefix, segment), routes);
        } else if is_route_file(&name) {
            let content = fs::read_to_string(&path).unwrap_or_default();
            let methods = detect_exported_methods(&content);
            let has_auth = content.contains("auth")
                || content.contains("token")
                || content.contains("session")
                || content.contains("getServerSession");

            if !methods.is_empty() {
                routes.push(NextApiRoute {
                    route: prefix.to_string(),
                    methods,
                    has_auth,
                    is_dynamic: prefix.contains(':'),
                    handler_type: "app-route".to_string(),
                });
            }
        }
    }
}

fn scan_pages_api_routes(dir: &Path, prefix: &str, routes: &mut Vec<NextApiRoute>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            let segment = if name.starts_with('[') && name.ends_with(']') {
                format!(":{}", &name[1..name.len() - 1].replace("...", ""))
            } else {
                name
            };
            scan_pages_api_routes(&path, &format!("{}/{}", prefix, segment), routes);
        } else if is_page_extension(&name) && !name.starts_with('_') {
            let stem = name.split('.').next().unwrap_or("");
            let route = if stem == "index" {
                prefix.to_string()
            } else {
                let segment = if stem.starts_with('[') && stem.ends_with(']') {
                    format!(":{}", &stem[1..stem.len() - 1].replace("...", ""))
                } else {
                    stem.to_string()
                };
                format!("{}/{}", prefix, segment)
            };

            let content = fs::read_to_string(&path).unwrap_or_default();
            let has_auth = content.contains("auth")
                || content.contains("token")
                || content.contains("session");

            // Pages API routes export default handler — all methods
            routes.push(NextApiRoute {
                route,
                methods: vec![
                    "GET".into(), "POST".into(), "PUT".into(),
                    "PATCH".into(), "DELETE".into(),
                ],
                has_auth,
                is_dynamic: prefix.contains(':'),
                handler_type: "pages-api".to_string(),
            });
        }
    }
}

fn detect_exported_methods(content: &str) -> Vec<String> {
    let mut methods = Vec::new();
    for method in &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"] {
        if content.contains(&format!("export async function {}", method))
            || content.contains(&format!("export function {}", method))
            || content.contains(&format!("export const {} =", method))
            || content.contains(&format!("export const {}", method))
        {
            methods.push(method.to_string());
        }
    }
    methods
}

// ─────────────────────────────────────────────────────────────────────────────
// Layout scanning
// ─────────────────────────────────────────────────────────────────────────────

fn scan_layouts(dir: &Path, app_root: &Path, layouts: &mut Vec<NextLayout>) {
    let layout_file = find_file(dir, "layout");
    if let Some(ref lf) = layout_file {
        let content = fs::read_to_string(lf).unwrap_or_default();
        let route_prefix = file_to_app_route(&dir.join("page.tsx"), app_root);

        let nav_items = extract_nav_items(&content);
        let has_sidebar = content.contains("sidebar")
            || content.contains("Sidebar")
            || content.contains("aside")
            || content.contains("nav-sidebar");
        let has_topbar = content.contains("header")
            || content.contains("Header")
            || content.contains("navbar")
            || content.contains("Navbar")
            || content.contains("topbar");

        let brand = extract_brand(&content);

        layouts.push(NextLayout {
            route_prefix,
            file_path: lf.clone(),
            has_sidebar,
            has_topbar,
            brand,
            nav_items,
        });
    }

    // Recurse into subdirs
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir()
                && !name.starts_with('_')
                && !name.starts_with('.')
                && name != "node_modules"
                && name != ".next"
                && name != "api"
            {
                scan_layouts(&path, app_root, layouts);
            }
        }
    }
}

fn find_file(dir: &Path, stem: &str) -> Option<PathBuf> {
    for ext in &["tsx", "ts", "jsx", "js"] {
        let p = dir.join(format!("{}.{}", stem, ext));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn extract_nav_items(content: &str) -> Vec<NavItem> {
    let mut items = Vec::new();

    // Look for href="/path" patterns near text content
    let mut pos = 0;
    while let Some(href_pos) = content[pos..].find("href=") {
        let abs = pos + href_pos + 5;
        if abs >= content.len() { break; }

        let quote = content.as_bytes().get(abs).copied().unwrap_or(0);
        if quote == b'"' || quote == b'\'' || quote == b'{' {
            let q = if quote == b'{' { b'}' } else { quote };
            let start = abs + 1;
            // For template literals {`/path`}, skip the backtick
            let start = if quote == b'{' && content.as_bytes().get(start) == Some(&b'`') {
                start + 1
            } else {
                start
            };

            if let Some(end) = content[start..].find(|c: char| c as u8 == q || c == '`') {
                let href = &content[start..start + end];
                if href.starts_with('/') && href.len() < 80 && !href.contains("api/") {
                    let label = route_to_title(href);
                    items.push(NavItem {
                        label,
                        href: href.to_string(),
                        icon: None,
                    });
                }
            }
        }
        pos = abs + 1;
    }

    // Deduplicate
    let mut seen = HashSet::new();
    items.retain(|item| seen.insert(item.href.clone()));
    items
}

fn extract_brand(content: &str) -> Option<String> {
    // Look for brand/logo text near <Link href="/">
    if let Some(pos) = content.find("href=\"/\"") {
        let window = &content[pos..std::cmp::min(pos + 200, content.len())];
        // Look for string literal after the link
        if let Some(q_start) = window.find('>') {
            let after = &window[q_start + 1..];
            let text: String = after.chars()
                .take_while(|c| *c != '<')
                .collect();
            let text = text.trim().to_string();
            if !text.is_empty() && text.len() < 40 && !text.contains('{') {
                return Some(text);
            }
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// Middleware detection
// ─────────────────────────────────────────────────────────────────────────────

fn detect_middleware(dir: &Path) -> Option<MiddlewareInfo> {
    let candidates = [
        dir.join("middleware.ts"),
        dir.join("middleware.js"),
        dir.join("src/middleware.ts"),
        dir.join("src/middleware.js"),
    ];

    let file = candidates.iter().find(|p| p.exists())?;
    let content = fs::read_to_string(file).ok()?;

    let has_auth_check = content.contains("auth")
        || content.contains("token")
        || content.contains("session")
        || content.contains("getToken")
        || content.contains("NextAuth");

    let has_redirect = content.contains("redirect")
        || content.contains("NextResponse.redirect");

    // Extract matchers from config
    let mut matchers = Vec::new();
    if let Some(pos) = content.find("matcher") {
        let window = &content[pos..std::cmp::min(pos + 500, content.len())];
        let mut in_string = false;
        let mut current = String::new();

        for ch in window.chars() {
            if ch == '"' || ch == '\'' || ch == '`' {
                if in_string {
                    if current.starts_with('/') {
                        matchers.push(current.clone());
                    }
                    current.clear();
                }
                in_string = !in_string;
            } else if in_string {
                current.push(ch);
            }
        }
    }

    if matchers.is_empty() {
        matchers.push("/".to_string());
    }

    Some(MiddlewareInfo {
        matchers,
        has_auth_check,
        has_redirect,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Config detection
// ─────────────────────────────────────────────────────────────────────────────

fn detect_next_config(dir: &Path) -> NextConfig {
    let candidates = [
        dir.join("next.config.ts"),
        dir.join("next.config.mjs"),
        dir.join("next.config.js"),
    ];

    let content = candidates.iter()
        .find(|p| p.exists())
        .and_then(|p| fs::read_to_string(p).ok())
        .unwrap_or_default();

    NextConfig {
        base_path: extract_string_value(&content, "basePath"),
        trailing_slash: content.contains("trailingSlash: true")
            || content.contains("trailingSlash:true"),
        i18n: detect_i18n(&content),
        redirects: Vec::new(), // Complex to parse, handled at runtime
        images_domains: Vec::new(),
    }
}

fn extract_string_value(content: &str, key: &str) -> Option<String> {
    let patterns = [
        format!("{}: \"", key),
        format!("{}: '", key),
        format!("{}:\"", key),
        format!("{}:'", key),
    ];

    for pat in &patterns {
        if let Some(pos) = content.find(pat.as_str()) {
            let start = pos + pat.len();
            let quote = pat.chars().last().unwrap();
            if let Some(end) = content[start..].find(quote) {
                return Some(content[start..start + end].to_string());
            }
        }
    }
    None
}

fn detect_i18n(content: &str) -> Option<I18nConfig> {
    if !content.contains("i18n") {
        return None;
    }

    // Simplified: detect locales array and defaultLocale
    let default = extract_string_value(content, "defaultLocale")
        .unwrap_or_else(|| "en".to_string());

    Some(I18nConfig {
        locales: vec![default.clone()],
        default_locale: default,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Auth detection
// ─────────────────────────────────────────────────────────────────────────────

fn detect_auth(dir: &Path) -> AuthInfo {
    let pkg = dir.join("package.json");
    let content = fs::read_to_string(&pkg).unwrap_or_default();

    let (detected, provider) = if content.contains("next-auth") || content.contains("@auth/") {
        (true, "next-auth".to_string())
    } else if content.contains("@clerk") {
        (true, "clerk".to_string())
    } else if content.contains("lucia") {
        (true, "lucia".to_string())
    } else if content.contains("supabase") && content.contains("auth") {
        (true, "supabase".to_string())
    } else if content.contains("firebase") {
        (true, "firebase".to_string())
    } else {
        // Check for custom auth files
        let auth_files = [
            dir.join("lib/auth.ts"),
            dir.join("src/lib/auth.ts"),
            dir.join("utils/auth.ts"),
            dir.join("app/api/auth"),
        ];
        if auth_files.iter().any(|p| p.exists()) {
            (true, "custom".to_string())
        } else {
            (false, String::new())
        }
    };

    AuthInfo {
        detected,
        provider,
        roles: if detected {
            vec!["admin".into(), "user".into()]
        } else {
            vec![]
        },
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Style detection
// ─────────────────────────────────────────────────────────────────────────────

fn detect_style(dir: &Path) -> StyleInfo {
    let has_tailwind = dir.join("tailwind.config.ts").exists()
        || dir.join("tailwind.config.js").exists()
        || dir.join("tailwind.config.mjs").exists()
        || {
            // Tailwind v4: check postcss or CSS imports
            let globals = dir.join("app/globals.css");
            if globals.exists() {
                let css = fs::read_to_string(&globals).unwrap_or_default();
                css.contains("@tailwind") || css.contains("@import \"tailwindcss\"")
                    || css.contains("@import 'tailwindcss'")
            } else {
                false
            }
        };

    // Detect theme from globals.css or tailwind config
    let mut theme = "light".to_string();
    let mut accent = None;
    let mut font = None;

    let globals_candidates = [
        dir.join("app/globals.css"),
        dir.join("src/app/globals.css"),
        dir.join("styles/globals.css"),
    ];

    for globals_path in &globals_candidates {
        if let Ok(css) = fs::read_to_string(globals_path) {
            if css.contains("dark") && css.contains(":root") {
                theme = "dark".to_string();
            }

            // Extract accent from CSS variables
            for line in css.lines() {
                let trimmed = line.trim();
                if (trimmed.contains("primary") || trimmed.contains("accent"))
                    && trimmed.contains(':')
                    && !trimmed.starts_with("//")
                {
                    if let Some(val) = trimmed.split(':').nth(1) {
                        let val = val.trim().trim_end_matches(';').trim();
                        if !val.is_empty() && val.len() < 30 {
                            accent = Some(val.to_string());
                            break;
                        }
                    }
                }
            }

            // Detect font from CSS
            for line in css.lines() {
                if line.contains("font-family") || line.contains("--font") {
                    if let Some(val) = line.split(':').nth(1) {
                        let val = val.trim().trim_end_matches(';').trim();
                        let font_name = val.split(',').next().unwrap_or("").trim()
                            .trim_matches('"').trim_matches('\'');
                        if !font_name.is_empty()
                            && font_name != "sans-serif"
                            && font_name != "inherit"
                            && font_name != "var"
                        {
                            font = Some(font_name.to_string());
                            break;
                        }
                    }
                }
            }

            break; // Use first found globals
        }
    }

    StyleInfo {
        theme,
        accent,
        font,
        has_tailwind,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Prisma detection
// ─────────────────────────────────────────────────────────────────────────────

fn detect_prisma(dir: &Path) -> String {
    let candidates = [
        dir.join("prisma/schema.prisma"),
        dir.join("schema.prisma"),
    ];

    for path in &candidates {
        if let Ok(schema) = fs::read_to_string(path) {
            let result = super::prisma::dump_prisma(&schema);
            // Remove the app block since we'll generate our own
            return remove_app_block(&result);
        }
    }

    String::new()
}

fn remove_app_block(cronus: &str) -> String {
    let mut result = String::new();
    let mut skip = false;
    let mut depth = 0i32;

    for line in cronus.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("app ") && trimmed.contains('{') {
            skip = true;
            depth = 1;
            continue;
        }
        if skip {
            for ch in trimmed.chars() {
                match ch {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
            }
            if depth <= 0 {
                skip = false;
            }
            continue;
        }
        if !trimmed.starts_with("# Generated") {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn detect_data_fetching(content: &str) -> Option<String> {
    if content.contains("\"use server\"") || content.contains("'use server'") {
        Some("server-action".into())
    } else if content.contains("fetch(") || content.contains("axios") {
        Some("fetch".into())
    } else if content.contains("prisma") || content.contains("drizzle") || content.contains("db.") {
        Some("database".into())
    } else {
        None
    }
}

fn detect_pages_data_fetching(content: &str) -> Option<String> {
    if content.contains("getServerSideProps") {
        Some("getServerSideProps".into())
    } else if content.contains("getStaticProps") {
        Some("getStaticProps".into())
    } else {
        None
    }
}

fn route_to_title(route: &str) -> String {
    let segments: Vec<&str> = route.split('/')
        .filter(|s| !s.is_empty() && !s.starts_with(':'))
        .collect();

    if segments.is_empty() {
        return "Home".to_string();
    }

    let last = segments.last().unwrap();
    let title = last.replace('-', " ").replace('_', " ");

    // Capitalize first letter of each word
    title.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => format!("{}{}", c.to_uppercase(), chars.collect::<String>()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_params(route: &str) -> Vec<String> {
    route.split('/')
        .filter(|s| s.starts_with(':'))
        .map(|s| s.trim_start_matches(':').to_string())
        .collect()
}

fn infer_page_type(route: &str, content: &str) -> String {
    if route == "/" {
        if content.contains("dashboard") || content.contains("Dashboard") {
            "dashboard".into()
        } else {
            "custom".into()
        }
    } else if route.contains(':') {
        "detail".into()
    } else if content.contains("<form") || content.contains("Form") || content.contains("onSubmit") {
        "form".into()
    } else if content.contains("<table") || content.contains("Table") || content.contains("DataTable") {
        "list".into()
    } else if content.contains("dashboard") || content.contains("Dashboard") {
        "dashboard".into()
    } else {
        "custom".into()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// .cronus emission
// ─────────────────────────────────────────────────────────────────────────────

fn emit_cronus(project: &NextJsProject) -> String {
    let mut out = String::new();

    // Header
    out.push_str("# Generated by cronus dump --nextjs\n");
    out.push_str(&format!(
        "# Source: Next.js {} project\n",
        if project.has_app_router { "App Router" } else { "Pages Router" }
    ));
    out.push_str(&format!("# Pages: {} | API Routes: {} | Layouts: {}\n\n",
        project.pages.len(),
        project.api_routes.len(),
        project.layouts.len(),
    ));

    // App block
    out.push_str(&format!("app \"{}\" {{\n", project.name));
    let stack = if project.style.has_tailwind {
        "react + tailwind"
    } else {
        "react"
    };
    out.push_str(&format!("  stack {}\n", stack));
    out.push_str(&format!("  port {}\n", project.port));
    out.push_str("  database sqlite \"./data.db\"\n");
    if let Some(ref bp) = project.config.base_path {
        out.push_str(&format!("  basepath \"{}\"\n", bp));
    }
    out.push_str("}\n\n");

    // Auth block
    if project.auth.detected {
        out.push_str("auth {\n");
        out.push_str("  entity User\n");
        out.push_str("  login email + password\n");
        out.push_str("  session jwt expires:24h\n");
        if !project.auth.roles.is_empty() {
            out.push_str(&format!(
                "  roles [{}]\n",
                project.auth.roles.join(", ")
            ));
        }
        out.push_str("}\n\n");
    }

    // Style block
    out.push_str("style {\n");
    out.push_str(&format!("  theme {}\n", project.style.theme));
    if let Some(ref accent) = project.style.accent {
        out.push_str(&format!("  accent {}\n", accent));
    }
    if let Some(ref font) = project.style.font {
        out.push_str(&format!("  font \"{}\"\n", font));
    }
    out.push_str("}\n\n");

    // Entities from Prisma
    if !project.prisma_entities.is_empty() {
        out.push_str("# ── Entities (from Prisma schema) ──\n\n");
        out.push_str(&project.prisma_entities);
        out.push('\n');
    }

    // Layout
    for layout in &project.layouts {
        if layout.has_sidebar && !layout.nav_items.is_empty() {
            out.push_str("layout Main {\n");
            if let Some(ref brand) = layout.brand {
                out.push_str(&format!("  brand \"{}\"\n", brand));
            }
            out.push_str("  sidebar {\n");
            for item in &layout.nav_items {
                let icon_str = item.icon.as_deref()
                    .map(|i| format!(" icon:{}", i))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "    \"{}\" -> \"{}\"{}\n",
                    item.label, item.href, icon_str
                ));
            }
            out.push_str("  }\n");
            out.push_str("}\n\n");
            break; // Only emit the first sidebar layout
        }
    }

    // Pages
    if !project.pages.is_empty() {
        out.push_str("# ── Pages ──\n\n");
        for page in &project.pages {
            let auth_str = if page.requires_auth {
                " requires:auth"
            } else {
                ""
            };
            let type_str = if page.page_type != "custom" {
                format!(" type:{}", page.page_type)
            } else {
                String::new()
            };

            out.push_str(&format!(
                "page \"{}\"{}{} {{\n",
                page.route, type_str, auth_str
            ));
            out.push_str(&format!("  title \"{}\"\n", page.title));

            // Emit data binding hint if we know the data source
            if let Some(ref df) = page.data_fetching {
                out.push_str(&format!("  # data: {}\n", df));
            }

            if page.page_type == "list" {
                out.push_str("  section table {\n");
                out.push_str("    # bind Entity { query all }\n");
                out.push_str("  }\n");
            } else if page.page_type == "detail" && !page.params.is_empty() {
                out.push_str("  section card {\n");
                out.push_str(&format!(
                    "    # bind Entity {{ query one where {} }}\n",
                    page.params.iter()
                        .map(|p| format!("{} eq:route.{}", p, p))
                        .collect::<Vec<_>>()
                        .join(" ")
                ));
                out.push_str("  }\n");
            } else if page.page_type == "form" {
                out.push_str("  section form {\n");
                out.push_str("    # entity Entity\n");
                out.push_str("  }\n");
            }

            out.push_str("}\n\n");
        }
    }

    // API routes
    if !project.api_routes.is_empty() {
        out.push_str("# ── API Routes ──\n\n");

        // Group by base path
        let mut groups: HashMap<String, Vec<&NextApiRoute>> = HashMap::new();
        for route in &project.api_routes {
            let base = route.route.split('/')
                .take(3)
                .collect::<Vec<_>>()
                .join("/");
            groups.entry(if base.is_empty() { "/api".to_string() } else { base })
                .or_default()
                .push(route);
        }

        let mut keys: Vec<&String> = groups.keys().collect();
        keys.sort();

        for prefix in keys {
            let routes = &groups[prefix];
            out.push_str(&format!("api {} {{\n", prefix));

            for route in routes {
                let relative = route.route.trim_start_matches(prefix.as_str());
                let relative = if relative.is_empty() { "/" } else { relative };
                let auth = if route.has_auth { "jwt" } else { "public" };

                for method in &route.methods {
                    let name = match method.as_str() {
                        "GET" => if route.is_dynamic { "detail" } else { "list" },
                        "POST" => "create",
                        "PUT" | "PATCH" => "update",
                        "DELETE" => "delete",
                        _ => "handle",
                    };
                    out.push_str(&format!(
                        "  {:<12} {:<8} {:<20} auth:{}\n",
                        name, method, relative, auth
                    ));
                }
            }

            out.push_str("}\n\n");
        }
    }

    // Summary
    let total_features = project.pages.len()
        + project.api_routes.len()
        + if project.auth.detected { 1 } else { 0 }
        + project.layouts.len();

    eprintln!("  \x1b[32m✓\x1b[0m Generated {} blocks from Next.js project", total_features);
    eprintln!(
        "  \x1b[32m✓\x1b[0m {} → .cronus ({}x fewer files)\n",
        project.name,
        std::cmp::max(1, project.pages.len() + project.api_routes.len())
    );

    out
}
