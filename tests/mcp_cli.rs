//! End-to-end contract of `cronus mcp`: a Model Context Protocol server
//! speaking newline-delimited JSON-RPC 2.0 on stdio (docs/MCP.md).
//! Each test runs the real binary in its own temp directory, writes a whole
//! session to stdin, closes it and reads every response line from stdout.

use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn workdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cronus-mcp-cli-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Runs one stdio session. Returns (exit code, parsed stdout lines, stderr).
fn session(dir: &Path, input: &str) -> (i32, Vec<Value>, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cronus"))
        .arg("mcp")
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn cronus mcp");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    // stdin is dropped here: EOF must end the server gracefully.
    let out = child.wait_with_output().expect("wait cronus mcp");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines = stdout
        .lines()
        .map(|l| {
            serde_json::from_str(l)
                .unwrap_or_else(|e| panic!("stdout line is not one JSON message ({e}): {l:?}"))
        })
        .collect();
    (
        out.status.code().expect("exit code"),
        lines,
        String::from_utf8(out.stderr).unwrap(),
    )
}

fn lines(msgs: &[Value]) -> String {
    msgs.iter().map(|m| format!("{m}\n")).collect()
}

fn init() -> Value {
    json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{
        "protocolVersion":"2025-06-18","capabilities":{},
        "clientInfo":{"name":"test","version":"0"}}})
}

fn initialized() -> Value {
    json!({"jsonrpc":"2.0","method":"notifications/initialized"})
}

fn call(id: i64, name: &str, args: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":args}})
}

fn by_id(resps: &[Value], id: i64) -> &Value {
    resps
        .iter()
        .find(|r| r["id"] == id)
        .unwrap_or_else(|| panic!("no response for id {id}: {resps:?}"))
}

/// Structured result of a successful tool call.
fn tool_json(resp: &Value) -> &Value {
    let result = &resp["result"];
    assert!(result.is_object(), "not a result: {resp}");
    assert_eq!(result["content"][0]["type"], "text", "{resp}");
    &result["structuredContent"]
}

const VALID: &str = "app \"T\" {\n  port 5175\n}\nentity Task {\n  title string!\n  done boolean default:false\n}\n";
const TYPO: &str = "app \"T\" {\n  port 5175\n}\nentity Task {\n  title strin!\n}\n";

#[test]
fn mcp_initialize_handshake_negotiates_version_and_capabilities() {
    let dir = workdir("init");
    let unsupported = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "protocolVersion":"1999-01-01","capabilities":{},"clientInfo":{"name":"t","version":"0"}}});
    let ping = json!({"jsonrpc":"2.0","id":2,"method":"ping"});
    let (code, resps, _) = session(&dir, &lines(&[init(), initialized(), unsupported, ping]));
    assert_eq!(code, 0);
    // The notification gets no response: exactly three replies.
    assert_eq!(resps.len(), 3, "{resps:?}");
    let r = &by_id(&resps, 0)["result"];
    assert_eq!(by_id(&resps, 0)["jsonrpc"], "2.0");
    assert_eq!(r["protocolVersion"], "2025-06-18");
    assert_eq!(r["serverInfo"]["name"], "cronus");
    assert!(r["capabilities"]["tools"].is_object());
    assert!(r["capabilities"]["resources"].is_object());
    // An unknown version is answered with the latest one the server supports.
    let latest = by_id(&resps, 1)["result"]["protocolVersion"]
        .as_str()
        .unwrap();
    assert!(latest.starts_with("20"), "{latest}");
    assert_ne!(latest, "1999-01-01");
    assert_eq!(by_id(&resps, 2)["result"], json!({}));
}

#[test]
fn mcp_tools_list_contains_every_tool_with_schema() {
    let dir = workdir("list");
    let list = json!({"jsonrpc":"2.0","id":1,"method":"tools/list"});
    let (_, resps, _) = session(&dir, &lines(&[init(), initialized(), list]));
    let tools = by_id(&resps, 1)["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    for expected in [
        "validate",
        "parse_ast",
        "list_sections",
        "list_field_types",
        "list_families",
        "explain_error",
        "context",
        "new_app",
    ] {
        assert!(names.contains(&expected), "{expected} missing: {names:?}");
    }
    for t in tools {
        assert_eq!(t["inputSchema"]["type"], "object", "{t}");
        assert!(!t["description"].as_str().unwrap().is_empty());
    }
}

#[test]
fn mcp_validate_matches_build_ai_on_valid_and_invalid_source() {
    let dir = workdir("validate");
    let (_, resps, _) = session(
        &dir,
        &lines(&[
            init(),
            call(1, "validate", json!({"source": VALID})),
            call(2, "validate", json!({"source": TYPO, "file": "app.cronus"})),
        ]),
    );
    let ok = tool_json(by_id(&resps, 1));
    assert_eq!(ok["schema_version"], 1);
    assert_eq!(ok["valid"], true, "{ok}");
    assert_eq!(ok["exit_code"], 0);
    assert!(ok["errors"].as_array().unwrap().is_empty());

    let bad = tool_json(by_id(&resps, 2));
    assert_eq!(bad["valid"], false);
    assert_eq!(bad["file"], "app.cronus");
    let e = &bad["errors"][0];
    assert_eq!(e["code"], "TYPE_001");
    assert_eq!(e["location"]["line"], 5);
    assert_eq!(e["fix"]["replacement"], "string");
    // An invalid app is a successful tool call carrying the verdict.
    assert_ne!(by_id(&resps, 2)["result"]["isError"], true);

    // Same JSON as the CLI.
    fs::write(dir.join("app.cronus"), TYPO).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cronus"))
        .args(["build", "--ai", "app.cronus"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let cli: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(&cli, bad);
}

#[test]
fn mcp_parse_ast_is_a_compact_json_summary() {
    let dir = workdir("ast");
    let src = "app \"Shop\" {\n  port 5175\n}\nentity Order {\n  total money!\n  status enum [open, paid]\n}\napi /orders {\n  list GET / auth:jwt\n}\npage \"/\" requires:auth {\n  section table { bind Order { query all order total desc } columns \"total, status\" }\n}\n";
    let (_, resps, _) = session(
        &dir,
        &lines(&[init(), call(1, "parse_ast", json!({"source": src}))]),
    );
    let ast = tool_json(by_id(&resps, 1));
    assert_eq!(ast["app"]["name"], "Shop");
    assert_eq!(ast["app"]["port"], 5175);
    let order = &ast["entities"][0];
    assert_eq!(order["name"], "Order");
    assert_eq!(order["fields"][0]["type"], "money");
    assert_eq!(order["fields"][0]["required"], true);
    assert_eq!(order["fields"][1]["enum_values"], json!(["open", "paid"]));
    assert_eq!(ast["apis"][0]["routes"][0]["method"], "GET");
    assert_eq!(ast["apis"][0]["routes"][0]["auth"], "jwt");
    let section = &ast["pages"][0]["sections"][0];
    assert_eq!(section["type"], "table");
    assert_eq!(section["binding"]["entity"], "Order");
    assert_eq!(section["binding"]["query"], "all");

    let (_, resps, _) = session(
        &dir,
        &lines(&[init(), call(2, "parse_ast", json!({"source": TYPO}))]),
    );
    let r = &by_id(&resps, 2)["result"];
    assert_eq!(r["isError"], true, "{r}");
    assert!(r["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("TYPE_001"));
}

#[test]
fn mcp_context_confines_paths_to_the_working_directory() {
    let root = workdir("confine");
    let project = root.join("project");
    fs::create_dir_all(project.join("sub")).unwrap();
    fs::write(project.join("sub/app.cronus"), VALID).unwrap();
    fs::write(root.join("outside.cronus"), VALID).unwrap();
    let outside_abs = root.join("outside.cronus").to_string_lossy().to_string();
    let (_, resps, _) = session(
        &project,
        &lines(&[
            init(),
            call(1, "context", json!({"path": "sub"})),
            call(2, "context", json!({"path": "../outside.cronus"})),
            call(3, "context", json!({"path": outside_abs})),
            call(4, "context", json!({"path": "sub/../../outside.cronus"})),
        ]),
    );
    let ok = &by_id(&resps, 1)["result"];
    assert_ne!(ok["isError"], true, "{ok}");
    assert!(ok["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("# Project: T"));
    for id in [2, 3, 4] {
        let r = by_id(&resps, id);
        let rejected = r["result"]["isError"] == true || r["error"].is_object();
        assert!(rejected, "path escape not rejected: {r}");
        assert!(!r.to_string().contains("# Project"), "{r}");
    }
}

#[test]
fn mcp_errors_for_unknown_tool_method_and_malformed_input() {
    let dir = workdir("errors");
    let input = format!(
        "{}\nthis is not json\n\n{}\n{}\n{}\n{}\n{}\n",
        init(),
        call(1, "no_such_tool", json!({})),
        json!({"jsonrpc":"2.0","id":2,"method":"no/such/method"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"validate","arguments":{}}}),
        json!({"id":4,"method":"ping"}),
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"explain_error","arguments":{"code":"type_001"}}}),
    );
    let (code, resps, stderr) = session(&dir, &input);
    assert_eq!(code, 0, "{stderr}");
    let parse_err = resps
        .iter()
        .find(|r| r["error"]["code"] == -32700)
        .unwrap_or_else(|| panic!("no parse error: {resps:?}"));
    assert_eq!(parse_err["id"], Value::Null);
    assert_eq!(by_id(&resps, 1)["error"]["code"], -32602);
    assert_eq!(by_id(&resps, 2)["error"]["code"], -32601);
    let missing_arg = by_id(&resps, 3);
    assert!(
        missing_arg["error"]["code"] == -32602 || missing_arg["result"]["isError"] == true,
        "{missing_arg}"
    );
    assert_eq!(by_id(&resps, 4)["error"]["code"], -32600);
    let explained = tool_json(by_id(&resps, 5));
    assert_eq!(explained["code"], "TYPE_001");
    assert!(explained["example_fix"]
        .as_str()
        .unwrap()
        .contains("string"));
}

#[test]
fn mcp_resources_and_new_app() {
    let dir = workdir("resources");
    let (_, resps, _) = session(
        &dir,
        &lines(&[
            init(),
            json!({"jsonrpc":"2.0","id":1,"method":"resources/list"}),
            json!({"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"cronus://llms-full.txt"}}),
            json!({"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"cronus://nope"}}),
            call(4, "new_app", json!({"template": "blog", "name": "My Blog"})),
            call(5, "new_app", json!({"template": "does-not-exist"})),
        ]),
    );
    let uris: Vec<&str> = by_id(&resps, 1)["result"]["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["uri"].as_str().unwrap())
        .collect();
    assert!(uris.contains(&"cronus://llms-full.txt") && uris.contains(&"cronus://language"));
    let text = by_id(&resps, 2)["result"]["contents"][0]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains("<!-- begin:grammar -->"));
    assert!(by_id(&resps, 3)["error"].is_object());
    let app = tool_json(by_id(&resps, 4));
    assert!(
        app["source"].as_str().unwrap().contains("app \"My Blog\""),
        "{app}"
    );
    assert!(
        !dir.join("app.cronus").exists(),
        "new_app must not write files"
    );
    assert_eq!(by_id(&resps, 5)["result"]["isError"], true);
}
