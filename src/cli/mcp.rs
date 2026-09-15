//! `cronus mcp` — a Model Context Protocol server on stdio.
//!
//! Lets LLM clients (Claude Code, Claude Desktop, Cursor) author `.cronus`
//! apps with the kernel's own validation in the loop. Transport: one JSON-RPC
//! 2.0 message per line on stdin/stdout (no embedded newlines); logs go to
//! stderr only. EOF on stdin ends the server with exit code 0.
//!
//! Methods: `initialize`, `ping`, `tools/list`, `tools/call`,
//! `resources/list`, `resources/read`, `resources/templates/list`.
//! Notifications (`notifications/initialized`, `notifications/cancelled`) are
//! accepted and never answered. Tools live in `mcp/tools.rs`; the user guide
//! is `docs/MCP.md`.

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

mod ast_summary;
mod error_codes;
mod tools;

/// Protocol revisions this server speaks, newest first. `initialize` echoes
/// the client's version when it is listed, otherwise answers with the first.
pub(crate) const SUPPORTED_PROTOCOL_VERSIONS: &[&str] =
    &["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

/// Larger lines are answered with `INVALID_REQUEST` instead of being parsed.
const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;

/// JSON-RPC 2.0 error codes.
pub(crate) mod rpc {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
}

/// A JSON-RPC error object (protocol-level failure, not a tool failure).
#[derive(Debug)]
pub(crate) struct RpcError {
    pub code: i64,
    pub message: String,
}

impl RpcError {
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        RpcError {
            code,
            message: message.into(),
        }
    }
}

const RESOURCES: &[(&str, &str, &str, &str)] = &[
    (
        "cronus://llms-full.txt",
        "llms-full.txt",
        "Canonical .cronus grammar, valid section/field types, a clean example and the never-do list",
        crate::cli::context_grammar::LLMS_FULL,
    ),
    (
        "cronus://language",
        "LANGUAGE.md",
        "Long-form verified language reference, including the build --ai schema and error codes (§15.9)",
        include_str!("../../LANGUAGE.md"),
    ),
];

const INSTRUCTIONS: &str = "Author .cronus apps with validation in the loop: read cronus://llms-full.txt, write the source, call `validate` until `valid` is true, and use `explain_error` for any code you do not understand. Never emit JS, JSX, HTML or CSS into .cronus.";

pub fn cmd_mcp(_args: &[String]) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    eprintln!(
        "cronus mcp {}: MCP server on stdio (protocol {})",
        crate::cli::version::VERSION,
        SUPPORTED_PROTOCOL_VERSIONS.join(", ")
    );
    if let Err(e) = serve(stdin.lock(), stdout.lock()) {
        eprintln!("cronus mcp: {e}");
        std::process::exit(1);
    }
}

/// Reads messages until EOF, writing one response line per request.
/// A closed stdout (broken pipe) also ends the loop cleanly.
pub(crate) fn serve<R: BufRead, W: Write>(mut input: R, mut output: W) -> io::Result<()> {
    let mut line = Vec::new();
    loop {
        line.clear();
        if input.read_until(b'\n', &mut line)? == 0 {
            return Ok(());
        }
        let Some(response) = handle_line(&line) else {
            continue;
        };
        let written = writeln!(output, "{response}").and_then(|_| output.flush());
        match written {
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
            other => other?,
        }
    }
}

/// One raw line in, at most one response out (`None` for blank lines,
/// notifications and client responses).
pub(crate) fn handle_line(raw: &[u8]) -> Option<Value> {
    if raw.len() > MAX_MESSAGE_BYTES {
        return Some(error_response(
            Value::Null,
            RpcError::new(rpc::INVALID_REQUEST, "message too large"),
        ));
    }
    let Ok(text) = std::str::from_utf8(raw) else {
        return Some(error_response(
            Value::Null,
            RpcError::new(rpc::PARSE_ERROR, "message is not valid UTF-8"),
        ));
    };
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    match serde_json::from_str::<Value>(text) {
        Ok(msg) => handle_message(msg),
        Err(e) => Some(error_response(
            Value::Null,
            RpcError::new(rpc::PARSE_ERROR, format!("invalid JSON: {e}")),
        )),
    }
}

fn handle_message(msg: Value) -> Option<Value> {
    let Value::Object(obj) = &msg else {
        let why = if msg.is_array() {
            "batch requests are not supported; send one message per line"
        } else {
            "a message must be a JSON object"
        };
        return Some(error_response(
            Value::Null,
            RpcError::new(rpc::INVALID_REQUEST, why),
        ));
    };
    let id = obj.get("id").cloned();
    let id_ok = matches!(id, None | Some(Value::String(_)) | Some(Value::Number(_)));
    let reply_id = if id_ok {
        id.clone().unwrap_or(Value::Null)
    } else {
        Value::Null
    };
    let Some(method) = obj.get("method").and_then(Value::as_str) else {
        if obj.contains_key("result") || obj.contains_key("error") {
            return None; // a response to a server request; this server sends none
        }
        return Some(error_response(
            reply_id,
            RpcError::new(rpc::INVALID_REQUEST, "missing 'method'"),
        ));
    };
    if obj.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Some(error_response(
            reply_id,
            RpcError::new(rpc::INVALID_REQUEST, "'jsonrpc' must be \"2.0\""),
        ));
    }
    if !id_ok {
        return Some(error_response(
            Value::Null,
            RpcError::new(rpc::INVALID_REQUEST, "'id' must be a string or a number"),
        ));
    }
    let params = obj.get("params").cloned().unwrap_or(Value::Null);
    let Some(id) = id else {
        // Notification: never answered, unknown ones are ignored.
        return None;
    };
    Some(match dispatch(method, &params) {
        Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
        Err(e) => error_response(id, e),
    })
}

fn dispatch(method: &str, params: &Value) -> Result<Value, RpcError> {
    match method {
        "initialize" => Ok(initialize(params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(tools::list()),
        "tools/call" => tools::call(params),
        "resources/list" => Ok(resources_list()),
        "resources/templates/list" => Ok(json!({"resourceTemplates": []})),
        "resources/read" => resources_read(params),
        _ => Err(RpcError::new(
            rpc::METHOD_NOT_FOUND,
            format!("method not found: {method}"),
        )),
    }
}

fn error_response(id: Value, e: RpcError) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": e.code, "message": e.message},
    })
}

fn initialize(params: &Value) -> Value {
    let requested = params.get("protocolVersion").and_then(Value::as_str);
    let version = requested
        .filter(|v| SUPPORTED_PROTOCOL_VERSIONS.contains(v))
        .unwrap_or(SUPPORTED_PROTOCOL_VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": {
            "tools": {"listChanged": false},
            "resources": {"listChanged": false, "subscribe": false},
        },
        "serverInfo": {
            "name": "cronus",
            "title": "CRONUS",
            "version": crate::cli::version::VERSION,
        },
        "instructions": INSTRUCTIONS,
    })
}

fn resources_list() -> Value {
    let resources: Vec<Value> = RESOURCES
        .iter()
        .map(|(uri, name, description, text)| {
            json!({
                "uri": uri,
                "name": name,
                "description": description,
                "mimeType": "text/markdown",
                "size": text.len(),
            })
        })
        .collect();
    json!({ "resources": resources })
}

fn resources_read(params: &Value) -> Result<Value, RpcError> {
    let uri = params
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| RpcError::new(rpc::INVALID_PARAMS, "missing 'uri'"))?;
    let (_, _, _, text) = RESOURCES
        .iter()
        .find(|(u, ..)| *u == uri)
        .ok_or_else(|| RpcError::new(rpc::INVALID_PARAMS, format!("unknown resource: {uri}")))?;
    Ok(json!({
        "contents": [{"uri": uri, "mimeType": "text/markdown", "text": text}]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(input: &str) -> Vec<Value> {
        let mut out = Vec::new();
        serve(input.as_bytes(), &mut out).unwrap();
        String::from_utf8(out)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    #[test]
    fn mcp_serve_answers_requests_skips_notifications_and_ends_at_eof() {
        let resps = run(concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\"}}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\r\n",
            "\n",
            "{\"jsonrpc\":\"2.0\",\"id\":\"b\",\"method\":\"ping\"}"
        ));
        assert_eq!(resps.len(), 2);
        assert_eq!(resps[0]["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(resps[1]["id"], "b");
    }

    #[test]
    fn mcp_malformed_messages_get_jsonrpc_errors() {
        let line = |raw: &[u8]| handle_line(raw).expect("a response");
        assert_eq!(line(b"{nope")["error"]["code"], rpc::PARSE_ERROR);
        assert_eq!(line(&[0xff, 0xfe])["error"]["code"], rpc::PARSE_ERROR);
        assert_eq!(line(b"[]")["error"]["code"], rpc::INVALID_REQUEST);
        assert_eq!(line(b"42")["error"]["code"], rpc::INVALID_REQUEST);
        let bad_id = line(br#"{"jsonrpc":"2.0","id":{},"method":"ping"}"#);
        assert_eq!(bad_id["error"]["code"], rpc::INVALID_REQUEST);
        assert_eq!(bad_id["id"], Value::Null);
        let no_version = line(br#"{"id":7,"method":"ping"}"#);
        assert_eq!(no_version["error"]["code"], rpc::INVALID_REQUEST);
        assert_eq!(no_version["id"], 7);
        let unknown = line(br#"{"jsonrpc":"2.0","id":8,"method":"x/y"}"#);
        assert_eq!(unknown["error"]["code"], rpc::METHOD_NOT_FOUND);
        assert!(handle_line(br#"{"jsonrpc":"2.0","id":9,"result":{}}"#).is_none());
        assert!(handle_line(br#"{"jsonrpc":"2.0","method":"x/unknown-notification"}"#).is_none());
    }

    #[test]
    fn mcp_initialize_falls_back_to_newest_supported_version() {
        let r = initialize(&json!({"protocolVersion": "2000-01-01"}));
        assert_eq!(r["protocolVersion"], SUPPORTED_PROTOCOL_VERSIONS[0]);
        let r = initialize(&Value::Null);
        assert_eq!(r["protocolVersion"], SUPPORTED_PROTOCOL_VERSIONS[0]);
    }

    #[test]
    fn mcp_resources_read_serves_embedded_docs() {
        let r = resources_read(&json!({"uri": "cronus://language"})).unwrap();
        assert!(r["contents"][0]["text"].as_str().unwrap().contains("15.9"));
        assert!(resources_read(&json!({"uri": "file:///etc/passwd"})).is_err());
        assert!(resources_read(&json!({})).is_err());
    }
}
