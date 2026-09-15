//! Tools exposed by `cronus mcp` (`tools/list`, `tools/call`).
//!
//! Every tool calls the kernel's own entry points (parser, `build --ai`
//! report, `context --for-claude`, grammar tables, templates) so answers
//! cannot drift from the CLI. Failures the model can fix (bad arguments,
//! unparseable source, unknown template) are tool results with
//! `isError: true`; only an unknown tool name is a JSON-RPC error.

use serde_json::{json, Value};
use std::path::{Component, Path, PathBuf};

use super::{ast_summary, error_codes, rpc, RpcError};
use crate::cli::build::{self, report};
use crate::cli::context_grammar as grammar;
use crate::cli::new;
use crate::parser;

/// Result of a tool body: `Ok` is shown to the model as a success, `Err` as a
/// tool execution error.
type ToolResult = Result<Output, String>;

enum Output {
    /// Structured JSON, sent as `structuredContent` plus its text form.
    Json(Value),
    /// Plain text (Markdown).
    Text(String),
}

struct Tool {
    name: &'static str,
    title: &'static str,
    description: &'static str,
    schema: fn() -> Value,
    run: fn(&Value) -> ToolResult,
}

const TOOLS: &[Tool] = &[
    Tool {
        name: "validate",
        title: "Validate .cronus source",
        description: "Validate .cronus source text exactly like `cronus build --ai` (strict profile). Returns the build JSON: schema_version, valid, exit_code, errors and warnings with 1-based locations and fix suggestions. An invalid app is a normal result with valid:false.",
        schema: || {
            json!({
                "type": "object",
                "properties": {
                    "source": {"type": "string", "description": "Complete .cronus source text"},
                    "file": {"type": "string", "description": "File name used in locations (label only, never read). Default: app.cronus"}
                },
                "required": ["source"]
            })
        },
        run: validate,
    },
    Tool {
        name: "parse_ast",
        title: "Summarize the AST",
        description: "Parse .cronus source and return a compact JSON summary of the AST: app, auth, entities (fields, types, modifiers, transitions), apis (routes, methods, auth), pages (sections, bindings), layouts and style.",
        schema: || {
            json!({
                "type": "object",
                "properties": {"source": {"type": "string", "description": "Complete .cronus source text"}},
                "required": ["source"]
            })
        },
        run: parse_ast,
    },
    Tool {
        name: "list_sections",
        title: "List section types",
        description: "Section types the renderer accepts: canonical built-ins, aliases with their canonical name (aliases produce CONTRACT_004 warnings), and cronus-ui family sections.",
        schema: no_args,
        run: list_sections,
    },
    Tool {
        name: "list_field_types",
        title: "List field types",
        description: "Valid entity field type keywords (unknown types are TYPE_001 errors) and the relation syntax.",
        schema: no_args,
        run: list_field_types,
    },
    Tool {
        name: "list_families",
        title: "List cronus-ui families",
        description: "cronus-ui component families usable as section types.",
        schema: no_args,
        run: list_families,
    },
    Tool {
        name: "explain_error",
        title: "Explain a build error code",
        description: "Describe a `cronus build --ai` error code (e.g. TYPE_001, RESOLVE_001, CONTRACT_004, LINT_003) with its severity and an example fix.",
        schema: || {
            json!({
                "type": "object",
                "properties": {"code": {"type": "string", "description": "Error code, e.g. TYPE_001"}},
                "required": ["code"]
            })
        },
        run: explain_error,
    },
    Tool {
        name: "context",
        title: "Project context",
        description: "Markdown project context, same as `cronus context --for-claude`: project summary, current build errors, grammar and types. `path` is a project directory or .cronus file inside the server's working directory (default: the working directory).",
        schema: || {
            json!({
                "type": "object",
                "properties": {"path": {"type": "string", "description": "Directory or .cronus file, relative to the working directory"}}
            })
        },
        run: context,
    },
    Tool {
        name: "new_app",
        title: "Starter app source",
        description: "Return the source text of a `cronus new` template (nothing is written to disk). Omit `template` for the default; an unknown template lists the available ones.",
        schema: || {
            json!({
                "type": "object",
                "properties": {
                    "template": {"type": "string", "description": "Template name, e.g. saas, api, landing, admin, blog, crm, ecommerce, helpdesk, cronus-ui"},
                    "name": {"type": "string", "description": "App name to put in the app \"...\" block"}
                }
            })
        },
        run: new_app,
    },
];

fn no_args() -> Value {
    json!({"type": "object", "properties": {}})
}

pub(super) fn list() -> Value {
    let tools: Vec<Value> = TOOLS
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "title": t.title,
                "description": t.description,
                "inputSchema": (t.schema)(),
                "annotations": {"readOnlyHint": true, "openWorldHint": false},
            })
        })
        .collect();
    json!({ "tools": tools })
}

pub(super) fn call(params: &Value) -> Result<Value, RpcError> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| RpcError::new(rpc::INVALID_PARAMS, "missing tool 'name'"))?;
    let tool = TOOLS
        .iter()
        .find(|t| t.name == name)
        .ok_or_else(|| RpcError::new(rpc::INVALID_PARAMS, format!("unknown tool: {name}")))?;
    let args = match params.get("arguments") {
        None | Some(Value::Null) => json!({}),
        Some(v @ Value::Object(_)) => v.clone(),
        Some(_) => {
            return Err(RpcError::new(
                rpc::INVALID_PARAMS,
                "'arguments' must be an object",
            ))
        }
    };
    Ok(match (tool.run)(&args) {
        Ok(Output::Json(v)) => json!({
            "content": [{"type": "text", "text": v.to_string()}],
            "structuredContent": v,
        }),
        Ok(Output::Text(t)) => json!({"content": [{"type": "text", "text": t}]}),
        Err(message) => json!({
            "content": [{"type": "text", "text": message}],
            "isError": true,
        }),
    })
}

fn str_arg<'a>(args: &'a Value, key: &str) -> Result<Option<&'a str>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(_) => Err(format!("argument '{key}' must be a string")),
    }
}

fn required_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    str_arg(args, key)?.ok_or_else(|| format!("missing required argument '{key}'"))
}

// ── validate / parse_ast ────────────────────────────────────────────────────

/// The JSON `cronus build --ai` prints for this source.
pub(super) fn validation_json(source: &str, file: &str) -> Value {
    match parser::parse_diagnostics(source) {
        Err(errors) => report::from_parse_errors(file, &errors).to_json(),
        Ok(_) => build::validate_source_ai(source, file).unwrap_or_else(|e| {
            json!({"schema_version": 1, "valid": false, "file": file, "errors": [{"code": "PARSE_001", "severity": "error", "message": e}], "warnings": []})
        }),
    }
}

fn validate(args: &Value) -> ToolResult {
    let source = required_str(args, "source")?;
    let file = str_arg(args, "file")?.unwrap_or("app.cronus");
    Ok(Output::Json(validation_json(source, file)))
}

fn parse_ast(args: &Value) -> ToolResult {
    let source = required_str(args, "source")?;
    match parser::parse_diagnostics(source) {
        Ok(nodes) => Ok(Output::Json(ast_summary::summarize(&nodes))),
        Err(errors) => Err(format!(
            "the source does not parse; call `validate` for locations and fixes:\n{}",
            parser::diagnostic::join(&errors)
        )),
    }
}

// ── grammar lists ───────────────────────────────────────────────────────────

fn list_sections(_: &Value) -> ToolResult {
    let canonical: Vec<&str> = grammar::BUILTIN_SECTION_TYPES
        .iter()
        .copied()
        .filter(|s| !grammar::SECTION_ALIASES.contains(s))
        .collect();
    let aliases: Vec<Value> = grammar::SECTION_ALIASES
        .iter()
        .map(|a| {
            json!({
                "name": a,
                "canonical": crate::contracts::ContractRegistry::resolve_alias(a),
            })
        })
        .collect();
    Ok(Output::Json(json!({
        "builtin": canonical,
        "aliases": aliases,
        "families": grammar::family_section_types(),
        "usage": "section <type> { ... }; data sections need `bind Entity { ... }` or items",
    })))
}

fn list_field_types(_: &Value) -> ToolResult {
    Ok(Output::Json(json!({
        "field_types": grammar::FIELD_TYPES,
        "relation": "author -> User",
        "enum": "status enum [draft, published]",
        "required_marker": "title string!  (`!` = required)",
    })))
}

fn list_families(_: &Value) -> ToolResult {
    let families = grammar::family_section_types();
    Ok(Output::Json(json!({
        "count": families.len(),
        "families": families,
    })))
}

fn explain_error(args: &Value) -> ToolResult {
    let code = required_str(args, "code")?;
    match error_codes::lookup(code) {
        Some(e) => Ok(Output::Json(e.to_json())),
        None => Err(format!(
            "unknown error code '{}'. Known codes: {}",
            code,
            error_codes::ERROR_CODES
                .iter()
                .map(|e| e.code)
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

// ── context ─────────────────────────────────────────────────────────────────

/// Resolves `path` (default `.`) to an existing path inside `root`.
/// Rejects `..` components, and anything that canonicalizes (symlinks
/// included) outside `root`.
pub(super) fn confine(root: &Path, path: Option<&str>) -> Result<PathBuf, String> {
    let raw = path.unwrap_or(".");
    let p = Path::new(raw);
    if raw.contains('\0') || p.components().any(|c| c == Component::ParentDir) {
        return Err("path must stay inside the working directory ('..' is not allowed)".into());
    }
    let root = root
        .canonicalize()
        .map_err(|_| "working directory is not accessible".to_string())?;
    let joined = if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    };
    let canon = joined
        .canonicalize()
        .map_err(|_| format!("path not found: {raw}"))?;
    if !canon.starts_with(&root) {
        return Err("path must stay inside the working directory".into());
    }
    Ok(canon)
}

/// The `.cronus` file for a confined path: the file itself, or `app.cronus`
/// (else the first `.cronus` file by name) in a directory.
fn project_file(path: &Path) -> Result<PathBuf, String> {
    if path.is_file() {
        return if path.extension().is_some_and(|e| e == "cronus") {
            Ok(path.to_path_buf())
        } else {
            Err("path is not a .cronus file".into())
        };
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(path)
        .map_err(|_| "cannot read directory".to_string())?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "cronus"))
        .collect();
    files.sort();
    files
        .iter()
        .find(|p| p.file_name().is_some_and(|n| n == "app.cronus"))
        .or_else(|| files.first())
        .cloned()
        .ok_or_else(|| "no .cronus file in that directory".to_string())
}

fn context(args: &Value) -> ToolResult {
    let cwd = std::env::current_dir().map_err(|_| "working directory is not accessible")?;
    let target = confine(&cwd, str_arg(args, "path")?)?;
    let file = project_file(&target)?;
    let root = cwd
        .canonicalize()
        .map_err(|_| "working directory is not accessible")?;
    // Relative label: also lets the build pass re-read the file from cwd.
    let label = file
        .strip_prefix(&root)
        .map(|rel| {
            rel.components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/")
        })
        .map_err(|_| "path must stay inside the working directory".to_string())?;
    let source = std::fs::read_to_string(&file).map_err(|_| format!("cannot read {label}"))?;
    Ok(Output::Text(crate::cli::context::render_for_claude(
        &label, &source,
    )))
}

// ── new_app ─────────────────────────────────────────────────────────────────

/// Replaces the name in the first `app "…"` declaration.
fn rename_app(source: &str, name: &str) -> String {
    let start = source
        .lines()
        .scan(0usize, |offset, line| {
            let at = *offset;
            *offset += line.len() + 1;
            Some((at, line))
        })
        .find(|(_, line)| line.trim_start().starts_with("app \""))
        .map(|(at, line)| at + line.find("app \"").unwrap_or(0) + "app \"".len());
    let Some(start) = start else {
        return source.to_string();
    };
    let Some(len) = source[start..].find('"') else {
        return source.to_string();
    };
    format!("{}{}{}", &source[..start], name, &source[start + len..])
}

fn new_app(args: &Value) -> ToolResult {
    let template = str_arg(args, "template")?.unwrap_or(new::DEFAULT_TEMPLATE);
    let names = || {
        new::TEMPLATES
            .iter()
            .map(|(n, d, _)| format!("{n} ({d})"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let source = new::template_source(template)
        .ok_or_else(|| format!("unknown template '{template}'. Available: {}", names()))?;
    let source = match str_arg(args, "name")? {
        None => source.to_string(),
        Some(name) => {
            let bad = name.trim().is_empty()
                || name.chars().count() > 80
                || name
                    .chars()
                    .any(|c| c == '"' || c == '\\' || c.is_control());
            if bad {
                return Err(
                    "argument 'name' must be 1-80 characters without quotes, backslashes or control characters"
                        .into(),
                );
            }
            rename_app(source, name)
        }
    };
    Ok(Output::Json(json!({
        "template": template,
        "file": "app.cronus",
        "source": source,
        "next": "call `validate` with this source, then save it as app.cronus and run `cronus run`",
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cronus-mcp-tools-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("inner")).unwrap();
        dir
    }

    #[test]
    fn mcp_confine_rejects_escapes() {
        let root = scratch("confine");
        std::fs::write(root.join("inner/app.cronus"), "entity T {\n  a string\n}\n").unwrap();
        let inner = root.join("inner");
        assert!(confine(&inner, None).is_ok());
        assert!(confine(&inner, Some("app.cronus")).is_ok());
        assert!(confine(&inner, Some("..")).is_err());
        assert!(confine(&inner, Some("./x/../../")).is_err());
        assert!(confine(&inner, Some("/")).is_err());
        let abs_inside = inner.join("app.cronus");
        assert!(confine(&inner, abs_inside.to_str()).is_ok());
        assert!(confine(&inner, Some("missing.cronus")).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&root, inner.join("up")).unwrap();
            assert!(confine(&inner, Some("up")).is_err(), "symlink escape");
        }
    }

    #[test]
    fn mcp_unknown_tool_is_a_protocol_error_and_bad_args_a_tool_error() {
        let e = call(&json!({"name": "nope"})).unwrap_err();
        assert_eq!(e.code, rpc::INVALID_PARAMS);
        let r = call(&json!({"name": "validate", "arguments": {"source": 3}})).unwrap();
        assert_eq!(r["isError"], true);
        assert!(call(&json!({"name": "validate", "arguments": []})).is_err());
    }

    #[test]
    fn mcp_every_template_renames_and_validates() {
        for (name, _, _) in new::TEMPLATES {
            let r = call(
                &json!({"name": "new_app", "arguments": {"template": name, "name": "Renamed App"}}),
            )
            .unwrap();
            let source = r["structuredContent"]["source"].as_str().unwrap();
            assert!(source.contains("app \"Renamed App\""), "{name}");
            assert_eq!(
                validation_json(source, "app.cronus")["valid"],
                true,
                "{name}"
            );
        }
        let r = call(&json!({"name": "new_app", "arguments": {"name": "bad\"name"}})).unwrap();
        assert_eq!(r["isError"], true);
    }

    #[test]
    fn mcp_list_sections_splits_aliases_from_canonical() {
        let r = call(&json!({"name": "list_sections"})).unwrap();
        let s = &r["structuredContent"];
        let builtin: Vec<&str> = s["builtin"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(builtin.contains(&"kpi") && !builtin.contains(&"stats"));
        let stats = s["aliases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["name"] == "stats")
            .unwrap();
        assert_eq!(stats["canonical"], "kpi");
        assert_eq!(
            s["families"].as_array().unwrap().len(),
            grammar::family_section_types().len()
        );
    }
}
