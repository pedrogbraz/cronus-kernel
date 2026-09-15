//! End-to-end contract of `cronus build` / `cronus build --ai`:
//! stdout purity, JSON schema, locations and exit codes (LANGUAGE.md §15.9).
//! Runs the real binary; each test works in its own temp directory because
//! the build writes `.cronus/ast-snapshot.json` into the current directory.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn workdir(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("cronus-build-cli-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn cronus(dir: &PathBuf, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cronus"))
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("run cronus");
    (
        out.status.code().expect("exit code"),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

fn json_stdout(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout)
        .unwrap_or_else(|e| panic!("stdout must be exactly one JSON document ({e}):\n{stdout}"))
}

const TYPO: &str = "app \"T\" {\n  port 5175\n}\nentity Task {\n  title strin!\n  qty numbr\n}\n";

#[test]
fn ai_type_typos_are_reported_with_location_and_fix() {
    let dir = workdir("typo");
    fs::write(dir.join("app.cronus"), TYPO).unwrap();
    let (code, stdout, _) = cronus(&dir, &["build", "--ai", "app.cronus"]);
    let j = json_stdout(&stdout);
    assert_eq!(code, 1);
    assert_eq!(j["schema_version"], 1);
    assert_eq!(j["valid"], false);
    let errs = j["errors"].as_array().unwrap();
    assert_eq!(errs.len(), 2, "{j}");
    assert_eq!(errs[0]["code"], "TYPE_001");
    assert_eq!(errs[0]["location"]["file"], "app.cronus");
    assert_eq!(errs[0]["location"]["line"], 5);
    assert_eq!(errs[0]["location"]["col"], 9);
    assert_eq!(errs[0]["location"]["span"]["end"]["col"], 14);
    assert_eq!(errs[0]["fix"]["replacement"], "string");
    assert_eq!(errs[1]["location"]["line"], 6);
    assert_eq!(errs[1]["location"]["col"], 7);
    assert_eq!(errs[1]["fix"]["replacement"], "number");
    assert!(j["warnings"].is_array());
}

#[test]
fn ai_stdout_is_json_only_even_when_pages_render_with_warnings() {
    let dir = workdir("pages");
    let src = "app \"T\" {\n  port 5175\n}\nentity Task {\n  title string!\n}\npage \"/\" {\n  section stats { bind Task { aggregate count } }\n  section table { bind Task { query all } columns \"title\" bogus:1 }\n}\n";
    fs::write(dir.join("app.cronus"), src).unwrap();
    let (code, stdout, _) = cronus(&dir, &["build", "--ai", "app.cronus"]);
    let j = json_stdout(&stdout);
    assert_eq!(code, if j["valid"] == true { 0 } else { 1 });
    let alias = j["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["code"] == "CONTRACT_004")
        .unwrap_or_else(|| panic!("{j}"));
    assert_eq!(alias["fix"]["replacement"], "kpi");
    assert_eq!(alias["location"]["line"], 8);
    for d in j["errors"]
        .as_array()
        .unwrap()
        .iter()
        .chain(j["warnings"].as_array().unwrap())
    {
        assert!(
            d["location"]["line"].as_u64().unwrap() >= 1,
            "line 0 in {d}"
        );
        assert!(d["location"]["col"].as_u64().unwrap() >= 1, "col 0 in {d}");
    }
}

#[test]
fn ai_valid_file_exits_0() {
    let dir = workdir("valid");
    fs::write(
        dir.join("app.cronus"),
        "app \"T\" {\n  port 5175\n}\nentity Task {\n  title string!\n  qty int\n}\n",
    )
    .unwrap();
    let (code, stdout, _) = cronus(&dir, &["build", "--ai", "app.cronus"]);
    let j = json_stdout(&stdout);
    assert_eq!((code, &j["valid"]), (0, &serde_json::json!(true)), "{j}");
}

#[test]
fn ai_syntax_error_is_english_and_located() {
    let dir = workdir("syntax");
    fs::write(
        dir.join("app.cronus"),
        "entity Item {\n  name string!\n}\napi /items {\n  check HEAD /\n}\n",
    )
    .unwrap();
    let (code, stdout, _) = cronus(&dir, &["build", "--ai", "app.cronus"]);
    let j = json_stdout(&stdout);
    assert_eq!(code, 1);
    let e = &j["errors"][0];
    assert_eq!(e["code"], "PARSE_001");
    assert_eq!(
        e["message"],
        "expected an HTTP method (GET, POST, PUT, PATCH, DELETE), found 'HEAD'"
    );
    assert_eq!(
        (
            e["location"]["line"].as_u64(),
            e["location"]["col"].as_u64()
        ),
        (Some(5), Some(9))
    );
}

#[test]
fn missing_file_exits_2_in_both_modes() {
    let dir = workdir("missing");
    let (code, stdout, _) = cronus(&dir, &["build", "--ai", "nope.cronus"]);
    assert_eq!(code, 2);
    let j = json_stdout(&stdout);
    assert_eq!(j["errors"][0]["code"], "IO_001");
    assert_eq!(j["errors"][0]["location"]["line"], 1);

    let (code, stdout, stderr) = cronus(&dir, &["build", "nope.cronus"]);
    assert_eq!(code, 2);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("build failed"), "{stderr}");

    // no path and no .cronus file in the directory
    let (code, _, _) = cronus(&dir, &["build", "--ai"]);
    assert_eq!(code, 2);
}

#[test]
fn plain_build_gives_a_single_verdict() {
    let dir = workdir("plain");
    fs::write(dir.join("bad.cronus"), TYPO).unwrap();
    let (code, stdout, stderr) = cronus(&dir, &["build", "bad.cronus"]);
    assert_eq!(code, 1);
    let all = format!("{stdout}{stderr}");
    assert!(!all.contains("Valid") && !all.contains("valid —"), "{all}");
    assert!(
        stderr.contains(
            "bad.cronus:5:9: error[TYPE_001]: unknown field type 'strin' for field 'title'"
        ),
        "{stderr}"
    );
    assert_eq!(stderr.matches("build blocked").count(), 1, "{stderr}");

    fs::write(
        dir.join("good.cronus"),
        "app \"T\" {\n  port 5175\n}\nentity Task {\n  title string!\n}\n",
    )
    .unwrap();
    let (code, stdout, stderr) = cronus(&dir, &["build", "good.cronus"]);
    assert_eq!(code, 0, "{stdout}{stderr}");
    assert!(stdout.contains("good.cronus is valid"), "{stdout}");
    assert!(!format!("{stdout}{stderr}").contains("blocked"));
}

#[test]
fn ai_unknown_where_operator_is_bind_001() {
    let dir = workdir("bind-op");
    fs::write(
        dir.join("app.cronus"),
        "app \"T\" { port 5175 }\nentity Task { title string! }\npage \"/\" {\n  section table {\n    bind Task { query all where title blah \"x\" }\n    columns \"title\"\n  }\n}\n",
    )
    .unwrap();
    let (code, stdout, _) = cronus(&dir, &["build", "--ai", "app.cronus"]);
    let j = json_stdout(&stdout);
    assert_eq!(code, 1);
    assert_eq!(j["valid"], false);
    let e = j["errors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["code"] == "BIND_001")
        .unwrap_or_else(|| panic!("{j}"));
    assert_eq!(e["category"], "bind");
    assert!(e["location"]["line"].as_u64().unwrap() >= 1);
    assert!(e["message"].as_str().unwrap().contains("blah"), "{e}");
}
