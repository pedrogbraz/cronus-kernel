//! Logic parity: compare Cronus HTML against the React DOM contract as shipped.
//!
//! `data-slot` is required. `data-variant` only when React emits it.
//! `data-size` is NOT CONTRACT — ignored even if the kernel emits it.

use crate::cli::audit_codes::AuditFinding;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct LogicExpect {
    pub slot: String,
    pub tag: Option<String>,
    pub attrs: HashMap<String, String>,
    pub href: Option<String>,
    pub disabled: bool,
    pub aria_invalid: Option<String>,
}

impl LogicExpect {
    pub fn from_fixture_json(v: &Value) -> Result<Self, String> {
        let expect = v.get("expect").ok_or("fixture missing expect")?;
        let slot = expect
            .get("slot")
            .and_then(Value::as_str)
            .ok_or("expect.slot required")?
            .to_string();
        let tag = expect
            .get("tag")
            .and_then(Value::as_str)
            .map(str::to_string);
        let mut attrs = HashMap::new();
        if let Some(obj) = expect.get("attrs").and_then(Value::as_object) {
            for (k, val) in obj {
                if k == "data-size" {
                    // Not CONTRACT. React Button does not emit it.
                    continue;
                }
                if let Some(s) = val.as_str() {
                    attrs.insert(k.clone(), s.to_string());
                }
            }
        }
        let href = expect
            .get("href")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                v.get("props")
                    .and_then(|p| p.get("href"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            });
        let disabled = v
            .get("props")
            .and_then(|p| p.get("disabled"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let aria_invalid = expect
            .get("attrs")
            .and_then(|a| a.get("aria-invalid"))
            .and_then(Value::as_str)
            .map(str::to_string);
        Ok(Self {
            slot,
            tag,
            attrs,
            href,
            disabled,
            aria_invalid,
        })
    }
}

pub fn compare_html(html: &str, expect: &LogicExpect) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    let fragment = Html::parse_fragment(html);
    let sel = match Selector::parse(&format!("[data-slot=\"{}\"]", expect.slot)) {
        Ok(s) => s,
        Err(_) => {
            findings.push(AuditFinding::fail(
                "logic",
                "CRONUS_AUDIT_LOGIC",
                "invalid data-slot selector",
            ));
            return findings;
        }
    };
    let el = match fragment.select(&sel).next() {
        Some(e) => e,
        None => {
            findings.push(AuditFinding::fail(
                "logic",
                "CRONUS_AUDIT_LOGIC",
                format!("missing data-slot=\"{}\"", expect.slot),
            ));
            return findings;
        }
    };
    let tag = el.value().name();
    if let Some(want) = &expect.tag {
        if tag != want {
            findings.push(AuditFinding::fail(
                "logic",
                "CRONUS_AUDIT_LOGIC",
                format!("tag {tag} != {want}"),
            ));
        }
    }
    for (key, want) in &expect.attrs {
        if key == "data-size" {
            continue;
        }
        let got = el.value().attr(key).unwrap_or("");
        if got != want {
            findings.push(AuditFinding::fail(
                "logic",
                "CRONUS_AUDIT_LOGIC",
                format!("attr {key}: {got} != {want}"),
            ));
        }
    }
    if let Some(href) = &expect.href {
        if el.value().attr("href") != Some(href.as_str()) {
            findings.push(AuditFinding::fail(
                "logic",
                "CRONUS_AUDIT_LOGIC",
                format!("href {:?} != {href}", el.value().attr("href")),
            ));
        }
        if tag != "a" {
            findings.push(AuditFinding::fail(
                "logic",
                "CRONUS_AUDIT_LOGIC",
                "href fixture must render <a>",
            ));
        }
    }
    if expect.disabled {
        if el.value().attr("disabled").is_none() {
            findings.push(AuditFinding::fail(
                "logic",
                "CRONUS_AUDIT_LOGIC",
                "expected disabled",
            ));
        }
    }
    if let Some(inv) = &expect.aria_invalid {
        if el.value().attr("aria-invalid") != Some(inv.as_str()) {
            findings.push(AuditFinding::fail(
                "logic",
                "CRONUS_AUDIT_LOGIC",
                format!(
                    "aria-invalid {:?} != {inv}",
                    el.value().attr("aria-invalid")
                ),
            ));
        }
    }
    if html.contains("data-slot=\"input-control\"") && expect.slot == "input" {
        findings.push(AuditFinding::fail(
            "logic",
            "CRONUS_AUDIT_LOGIC",
            "input must not use interact wrapper (input-control)",
        ));
    }
    findings
}

pub fn compare_fixture_file(html: &str, fixture_path: &str) -> Result<Vec<AuditFinding>, String> {
    let raw = std::fs::read_to_string(fixture_path)
        .map_err(|e| format!("cannot read fixture {fixture_path}: {e}"))?;
    let v: Value = serde_json::from_str(&raw).map_err(|e| format!("fixture json: {e}"))?;
    let expect = LogicExpect::from_fixture_json(&v)?;
    Ok(compare_html(html, &expect))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui::button_ex;

    #[test]
    fn button_slot_and_variant_without_requiring_data_size() {
        let html = button_ex("Save profile", "primary", "md", None, false);
        let mut expect = LogicExpect {
            slot: "button".into(),
            tag: Some("button".into()),
            ..Default::default()
        };
        expect.attrs.insert("data-slot".into(), "button".into());
        expect.attrs.insert("data-variant".into(), "primary".into());
        expect
            .attrs
            .insert("data-size".into(), "must-ignore".into());
        let findings = compare_html(&html, &expect);
        assert!(findings.is_empty(), "{findings:?}");
        assert!(html.contains("data-size=\"md\""));
    }

    #[test]
    fn button_href_is_anchor() {
        let html = button_ex("Docs", "link", "md", Some("/docs"), false);
        let mut expect = LogicExpect {
            slot: "button".into(),
            tag: Some("a".into()),
            href: Some("/docs".into()),
            ..Default::default()
        };
        expect.attrs.insert("data-variant".into(), "link".into());
        let findings = compare_html(&html, &expect);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn button_variant_matrix_does_not_require_data_size() {
        for variant in [
            "primary",
            "secondary",
            "outline",
            "ghost",
            "destructive",
            "link",
        ] {
            for size in ["sm", "md", "lg", "icon"] {
                let html = button_ex("Save", variant, size, None, false);
                let mut expect = LogicExpect {
                    slot: "button".into(),
                    tag: Some("button".into()),
                    ..Default::default()
                };
                expect.attrs.insert("data-slot".into(), "button".into());
                expect.attrs.insert("data-variant".into(), variant.into());
                let findings = compare_html(&html, &expect);
                assert!(findings.is_empty(), "{variant}/{size}: {findings:?}");
            }
        }
    }

    #[test]
    fn fixture_json_drops_data_size() {
        let v = serde_json::json!({
            "expect": {
                "slot": "button",
                "tag": "button",
                "attrs": {
                    "data-slot": "button",
                    "data-variant": "primary",
                    "data-size": "md"
                }
            }
        });
        let expect = LogicExpect::from_fixture_json(&v).unwrap();
        assert!(!expect.attrs.contains_key("data-size"));
        assert_eq!(expect.attrs.get("data-variant").unwrap(), "primary");
    }
}
