//! Cronus Audit language axis — HTML/JSX/React/Voodoo/sidecar in `.cronus` source.
//!
//! Compile-target HTML from dedicated Rust renderers is allowed. Authoring it
//! in `.cronus` is not.

use crate::cli::audit_codes::{
    self, AuditFinding, HTML_IN_SOURCE, JSX, SIDECAR_SOURCE, STACK_REACT, STACK_VOODOO, TEMPLATE,
    TW_CSS, VOODOO,
};
use crate::parser::{self, AstNode};
use regex::Regex;
use std::sync::LazyLock;

static HTML_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[A-Za-z]").unwrap());
static JSX_CLOSE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"</[A-Z]").unwrap());
static SIDECAR_SOURCE_RAW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\bsource\s*:?\s*["'][^"']+\.(html|tsx|jsx|css)\b"#).unwrap()
});
static SIDECAR_IMPORT_EXT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\.(html|tsx|jsx|css)$").unwrap());
static VOODOO_ATTR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:v-data|v-model)\b").unwrap());

const SIDECAR_EXTS: &[&str] = &[".html", ".tsx", ".jsx", ".css"];

pub fn scan_file(path: &str) -> Result<Vec<AuditFinding>, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {path}: {e}"))?;
    Ok(scan_source(&source))
}

pub fn scan_source(source: &str) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    scan_raw(source, &mut findings);
    if let Ok(nodes) = parser::parse(source) {
        scan_ast(&nodes, source, &mut findings);
    }
    findings.sort_by(|a, b| a.code.cmp(b.code).then(a.message.cmp(&b.message)));
    findings.dedup_by(|a, b| a.code == b.code && a.message == b.message);
    findings
}

fn scan_raw(source: &str, findings: &mut Vec<AuditFinding>) {
    if HTML_TAG.is_match(source) {
        findings.push(AuditFinding::fail(
            "language",
            HTML_IN_SOURCE,
            "HTML tag in .cronus source",
        ));
    }
    if source.contains("className=") || JSX_CLOSE.is_match(source) || source.contains("<>") {
        findings.push(AuditFinding::fail(
            "language",
            JSX,
            "JSX/TSX in .cronus source",
        ));
    }
    if SIDECAR_SOURCE_RAW.is_match(source) {
        findings.push(AuditFinding::fail(
            "language",
            SIDECAR_SOURCE,
            "page.config.source or sidecar path in .cronus source",
        ));
    }
    if source.contains("--tw-") || source.contains("zinc-") || source.contains("@tailwind") {
        findings.push(AuditFinding::fail(
            "language",
            TW_CSS,
            "Tailwind palette / @tailwind / --tw- in .cronus source",
        ));
    }
    if VOODOO_ATTR.is_match(source) || source.contains("voodoojs@") {
        findings.push(AuditFinding::fail(
            "language",
            VOODOO,
            "Voodoo attribute or runtime in .cronus source",
        ));
    }
}

fn scan_ast(nodes: &[AstNode], _source: &str, findings: &mut Vec<AuditFinding>) {
    for node in nodes {
        match node {
            AstNode::App(app) => {
                for item in &app.stack {
                    let l = item.to_ascii_lowercase();
                    if l == "react" {
                        findings.push(AuditFinding::fail(
                            "language",
                            STACK_REACT,
                            "stack react is forbidden in audit fixtures",
                        ));
                    }
                    if l == "voodoo" {
                        findings.push(AuditFinding::fail(
                            "language",
                            STACK_VOODOO,
                            "stack voodoo is forbidden in audit fixtures",
                        ));
                    }
                }
            }
            AstNode::Import(imp) => {
                if is_sidecar_path(&imp.source) {
                    findings.push(AuditFinding::fail(
                        "language",
                        SIDECAR_SOURCE,
                        format!("import of non-.cronus sidecar {}", imp.source),
                    ));
                }
            }
            AstNode::Page(page) => {
                if page.config.contains_key("source") {
                    findings.push(AuditFinding::fail(
                        "language",
                        SIDECAR_SOURCE,
                        "page.config.source sidecar HTML",
                    ));
                }
                if let Some(title) = &page.title {
                    scan_content_value(title, findings);
                }
                for section in &page.sections {
                    if section.template.is_some() || section.style_block.is_some() {
                        findings.push(AuditFinding::fail(
                            "language",
                            TEMPLATE,
                            "section.template / style_block in page",
                        ));
                    }
                    if let Some(t) = &section.title {
                        scan_content_value(t, findings);
                    }
                    if let Some(t) = &section.subtitle {
                        scan_content_value(t, findings);
                    }
                    if let Some(t) = &section.template {
                        scan_content_value(t, findings);
                    }
                    for item in &section.items {
                        for v in item.values() {
                            scan_content_value(v, findings);
                        }
                    }
                    for v in section.config.values() {
                        scan_content_value(v, findings);
                    }
                }
            }
            AstNode::Component(comp) => {
                if let Some(t) = &comp.template {
                    findings.push(AuditFinding::fail(
                        "language",
                        TEMPLATE,
                        format!("component {} has template", comp.name),
                    ));
                    scan_content_value(t, findings);
                }
                for item in &comp.items {
                    scan_content_value(&item.text, findings);
                    for v in item.config.values() {
                        scan_content_value(v, findings);
                    }
                }
                for v in comp.props.values() {
                    scan_content_value(v, findings);
                }
            }
            _ => {}
        }
    }
}

fn scan_content_value(value: &str, findings: &mut Vec<AuditFinding>) {
    if HTML_TAG.is_match(value) {
        findings.push(AuditFinding::fail(
            "language",
            HTML_IN_SOURCE,
            format!("HTML tag in content field: {value}"),
        ));
    }
}

fn is_sidecar_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    SIDECAR_EXTS.iter().any(|ext| lower.ends_with(ext)) || SIDECAR_IMPORT_EXT.is_match(path)
}

pub fn codes_of(findings: &[AuditFinding]) -> Vec<&'static str> {
    let mut codes: Vec<&'static str> = findings.iter().map(|f| f.code).collect();
    codes.sort();
    codes.dedup();
    codes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(src: &str) -> Vec<&'static str> {
        codes_of(&scan_source(src))
    }

    fn clean_fixture() -> &'static str {
        r#"
app "audit-fixtures" {
  port 5176
}

style {
  preset aurora
  theme dark
}

component ButtonPrimaryMd layout:inline style:button+primary+md {
  label "Save profile"
}

page "/audit/button/primary-md" type:custom {
  use ButtonPrimaryMd
}
"#
    }

    #[test]
    fn clean_fixture_passes() {
        assert!(scan_source(clean_fixture()).is_empty(), "{:?}", scan_source(clean_fixture()));
    }

    #[test]
    fn html_tag_in_source_fails() {
        let src = r#"
app "x" { port 1 }
component B layout:inline style:button+primary {
  label "<div>"
}
"#;
        assert!(codes(src).contains(&HTML_IN_SOURCE));
    }

    #[test]
    fn html_inside_label_draft_fails() {
        let src = r#"
app "x" { port 1 }
component B layout:inline style:button+primary {
  label "Save <draft>"
}
"#;
        assert!(codes(src).contains(&HTML_IN_SOURCE));
    }

    #[test]
    fn comparison_without_letter_passes() {
        let src = r#"
app "x" { port 1 }
component B layout:inline style:button+primary {
  label "2 < 3"
}
"#;
        assert!(!codes(src).contains(&HTML_IN_SOURCE));
    }

    #[test]
    fn jsx_classname_fails() {
        let src = r#"
app "x" { port 1 }
component B layout:inline style:button+primary {
  label "className=foo"
}
"#;
        // raw contains className=
        let with_attr = src.replace("className=foo", "Save");
        let jsx = with_attr.replace("label \"Save\"", "label \"x\"\n  # className=\"px-2\"");
        // comments are stripped by tokenizer but raw scan still sees them...
        // '#' comments are discarded from tokens but remain in the raw string
        // until end of line. A real JSX cheat is:
        let jsx_src = "app \"x\" { port 1 }\ncomponent B layout:inline style:button {\n  label \"Hi\"\n}\n<div className=\"x\"></div>\n";
        assert!(codes(jsx_src).contains(&JSX));
        assert!(codes(jsx_src).contains(&HTML_IN_SOURCE));
        let _ = jsx;
    }

    #[test]
    fn page_source_html_is_sidecar() {
        let src = r#"
app "audit-fixtures" { port 5176 }
component ButtonPrimaryMd layout:inline style:button+primary+md {
  label "Save profile"
}
page "/audit/button/primary-md" type:custom {
  source:"./stolen.html"
  use ButtonPrimaryMd
}
"#;
        assert!(codes(src).contains(&SIDECAR_SOURCE));
        let nodes = parser::parse(src).unwrap();
        let page = nodes.iter().find_map(|n| match n {
            AstNode::Page(p) => Some(p),
            _ => None,
        }).unwrap();
        assert_eq!(page.config.get("source").map(String::as_str), Some("./stolen.html"));
    }

    #[test]
    fn import_tsx_is_sidecar() {
        let src = r#"
import Stolen from "./stolen.tsx"
app "x" { port 1 }
"#;
        assert!(codes(src).contains(&SIDECAR_SOURCE));
    }

    #[test]
    fn stack_react_fails() {
        let src = r#"
app "x" {
  stack react + tailwind
  port 1
}
"#;
        assert!(codes(src).contains(&STACK_REACT));
    }

    #[test]
    fn stack_voodoo_fails() {
        let src = r#"
app "x" {
  stack voodoo
  port 1
}
"#;
        assert!(codes(src).contains(&STACK_VOODOO));
    }

    #[test]
    fn voodoo_attr_fails_style_block_does_not() {
        let style_only = r#"
app "x" { port 1 }
style {
  preset aurora
  theme dark
}
"#;
        assert!(!codes(style_only).contains(&VOODOO));
        let voodoo = "app \"x\" { port 1 }\ncomponent B layout:inline style:button {\n  label \"v-model=x\"\n}\n";
        assert!(codes(voodoo).contains(&VOODOO));
    }

    #[test]
    fn zinc_palette_fails() {
        let src = "app \"x\" { port 1 }\nstyle { accent zinc-900 }\n";
        assert!(codes(src).contains(&TW_CSS));
    }

    #[test]
    fn template_section_fails() {
        // Dump-emitted template HTML inside a section is cheat 3.
        // Parser stores template when the dump path sets it; a raw `<div>` in
        // the file is already HTML_IN_SOURCE. This covers the AST Some(template)
        // path via a constructed node.
        let mut findings = Vec::new();
        let page = parser::PageNode {
            route: "/x".into(),
            page_type: "custom".into(),
            entity: None,
            title: None,
            sections: vec![parser::SectionNode {
                section_type: "hero".into(),
                title: None,
                subtitle: None,
                config: Default::default(),
                items: vec![],
                plans: vec![],
                binding: None,
                actions: vec![],
                visibility: None,
                template: Some("<div>hi</div>".into()),
                style_block: None,
                doc: None,
            }],
            config: Default::default(),
            components: vec![],
            requires: None,
            doc: None,
        };
        scan_ast(&[AstNode::Page(page)], "", &mut findings);
        let codes: Vec<_> = findings.iter().map(|f| f.code).collect();
        assert!(codes.contains(&TEMPLATE));
        assert!(codes.contains(&HTML_IN_SOURCE));
        let _ = audit_codes::SIDECAR_RENDERER;
    }
}
