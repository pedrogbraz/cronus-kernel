#![allow(dead_code, unused_imports, unused_variables)]
//! Route detection for CRONUS Black Hole dump system
//!
//! Detects API routes from source code and directory structure
//! for Next.js, React Router, Express, and Elysia projects.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::project::ProjectProfile;

pub struct DetectedRoute {
    pub method: String,
    pub path: String,
    pub name: String,
    pub auth: String,
}

/// Detect routes from project source files based on stack type
pub fn detect_routes(profile: &ProjectProfile) -> Vec<DetectedRoute> {
    match profile.stack.as_str() {
        "nextjs" => detect_nextjs_routes(&profile.src_dir),
        "react" => detect_react_router_routes(&profile.ts_files),
        "express" => detect_code_routes(&profile.ts_files, "express"),
        "elysia" => detect_code_routes(&profile.ts_files, "elysia"),
        _ => detect_code_routes(&profile.ts_files, "any"),
    }
}

/// Emit detected routes as .cronus api blocks
pub fn emit_routes(routes: &[DetectedRoute]) -> String {
    if routes.is_empty() {
        return String::new();
    }

    // Group by prefix (first two path segments)
    let mut groups: HashMap<String, Vec<&DetectedRoute>> = HashMap::new();

    for route in routes {
        let prefix = route
            .path
            .split('/')
            .take(3)
            .collect::<Vec<_>>()
            .join("/");
        let prefix = if prefix.is_empty() {
            "/".to_string()
        } else {
            prefix
        };
        groups.entry(prefix).or_default().push(route);
    }

    let mut out = String::new();

    let mut keys: Vec<&String> = groups.keys().collect();
    keys.sort();

    for prefix in keys {
        let group_routes = &groups[prefix];
        out.push_str(&format!("api {} {{\n", prefix));
        for r in group_routes {
            let relative = r.path.trim_start_matches(prefix.as_str());
            let relative = if relative.is_empty() { "/" } else { relative };

            out.push_str(&format!(
                "  {:<12} {:<8} {:<20} auth:{}\n",
                r.name, r.method, relative, r.auth
            ));
        }
        out.push_str("}\n\n");
    }

    out
}

// ---------------------------------------------------------------------------
// Next.js App Router detection
// ---------------------------------------------------------------------------

fn detect_nextjs_routes(src_dir: &Path) -> Vec<DetectedRoute> {
    let mut routes = Vec::new();

    // Scan app/api/ for route.ts files
    let api_dir = src_dir.join("app/api");
    if api_dir.exists() {
        scan_nextjs_api_dir(&api_dir, "/api", &mut routes);
    }

    // Also check src/app/api/
    let src_api_dir = src_dir.join("src/app/api");
    if src_api_dir.exists() {
        scan_nextjs_api_dir(&src_api_dir, "/api", &mut routes);
    }

    routes
}

fn scan_nextjs_api_dir(dir: &Path, prefix: &str, routes: &mut Vec<DetectedRoute>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if path.is_dir() {
                // Skip route groups like (auth) — recurse without adding segment
                if name.starts_with('(') {
                    scan_nextjs_api_dir(&path, prefix, routes);
                    continue;
                }

                // Convert [param] to :param
                let segment = if name.starts_with('[') && name.ends_with(']') {
                    format!(":{}", &name[1..name.len() - 1])
                } else {
                    name.clone()
                };

                scan_nextjs_api_dir(&path, &format!("{}/{}", prefix, segment), routes);
            } else if name == "route.ts" || name == "route.js" {
                // Read file to detect which HTTP methods are exported
                if let Ok(content) = fs::read_to_string(&path) {
                    for method in &["GET", "POST", "PUT", "PATCH", "DELETE"] {
                        if content.contains(&format!("export async function {}", method))
                            || content.contains(&format!("export function {}", method))
                            || content.contains(&format!("export const {} =", method))
                        {
                            let action = match *method {
                                "GET" => {
                                    if prefix.contains(':') {
                                        "detail"
                                    } else {
                                        "list"
                                    }
                                }
                                "POST" => "create",
                                "PUT" | "PATCH" => "update",
                                "DELETE" => "delete",
                                _ => "handle",
                            };

                            routes.push(DetectedRoute {
                                method: method.to_string(),
                                path: prefix.to_string(),
                                name: action.to_string(),
                                auth: if content.contains("auth")
                                    || content.contains("token")
                                    || content.contains("session")
                                {
                                    "jwt"
                                } else {
                                    "public"
                                }
                                .to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// React Router detection (from source code)
// ---------------------------------------------------------------------------

fn detect_react_router_routes(ts_files: &[PathBuf]) -> Vec<DetectedRoute> {
    let mut routes = Vec::new();

    for file in ts_files {
        if let Ok(source) = fs::read_to_string(file) {
            // Look for <Route path="/x" ...> or path="/x" in router configs
            let mut pos = 0;
            while let Some(route_pos) = source[pos..].find("path=") {
                let abs_pos = pos + route_pos + 5;
                if abs_pos >= source.len() {
                    break;
                }

                let quote = source.as_bytes().get(abs_pos).copied().unwrap_or(0);
                if quote == b'"' || quote == b'\'' {
                    if let Some(end) = source[abs_pos + 1..].find(quote as char) {
                        if end > 0 {
                            let path = &source[abs_pos + 1..abs_pos + 1 + end];
                            if path.starts_with('/') && path.len() < 100 {
                                routes.push(DetectedRoute {
                                    method: "GET".to_string(),
                                    path: path.replace('[', ":").replace(']', ""),
                                    name: path_to_name(path),
                                    auth: "public".to_string(),
                                });
                            }
                        }
                    }
                }
                pos = abs_pos + 1;
            }
        }
    }

    routes
}

// ---------------------------------------------------------------------------
// Express / Elysia / generic code-based detection
// ---------------------------------------------------------------------------

fn detect_code_routes(ts_files: &[PathBuf], framework: &str) -> Vec<DetectedRoute> {
    let mut routes = Vec::new();

    let prefixes: Vec<&str> = match framework {
        "elysia" => vec!["."],
        "express" => vec!["app.", "router."],
        _ => vec![".", "app.", "router."],
    };

    for file in ts_files {
        if let Ok(source) = fs::read_to_string(file) {
            for prefix in &prefixes {
                for method in &["get", "post", "put", "patch", "delete"] {
                    let pattern = format!("{}{}", prefix, method);
                    let mut pos = 0;

                    while let Some(found) = source[pos..].find(&pattern) {
                        let abs_pos = pos + found + pattern.len();
                        // Find the path in quotes after (
                        if let Some(paren) = source[abs_pos..].find('(') {
                            let after_paren = abs_pos + paren + 1;
                            if after_paren < source.len() {
                                let trimmed = source[after_paren..].trim_start();
                                if let Some(first_char) = trimmed.chars().next() {
                                    if first_char == '"' || first_char == '\'' || first_char == '`'
                                    {
                                        if let Some(quote_offset) =
                                            source[after_paren..].find(first_char)
                                        {
                                            let quote_start = after_paren + quote_offset + 1;
                                            if quote_start < source.len() {
                                                if let Some(quote_end) =
                                                    source[quote_start..].find(first_char)
                                                {
                                                    if quote_end > 0 && quote_end < 200 {
                                                        let path = &source
                                                            [quote_start..quote_start + quote_end];
                                                        if path.starts_with('/') {
                                                            let action = match *method {
                                                                "get" => {
                                                                    if path.contains(':')
                                                                        || path.contains('{')
                                                                    {
                                                                        "detail"
                                                                    } else {
                                                                        "list"
                                                                    }
                                                                }
                                                                "post" => "create",
                                                                "put" | "patch" => "update",
                                                                "delete" => "delete",
                                                                _ => "handle",
                                                            };

                                                            routes.push(DetectedRoute {
                                                                method: method.to_uppercase(),
                                                                path: path
                                                                    .replace('{', ":")
                                                                    .replace('}', ""),
                                                                name: action.to_string(),
                                                                auth: "public".to_string(),
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        pos = abs_pos + 1;
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn path_to_name(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return "home".to_string();
    }
    let last = segments.last().unwrap_or(&"page");
    if last.starts_with(':') || last.starts_with('[') {
        "detail".to_string()
    } else {
        last.to_string()
    }
}
