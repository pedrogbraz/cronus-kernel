//! Bound entity data for cronus-ui widgets (request-scoped).
//!
//! Set around `render_section` so `style:metric` / `data-table` can read
//! `bind Lead { query count }` without every helper taking extra args.
//! Same task-local discipline as Voodoo: never a process-wide flag.

use crate::binding::ResolvedData;
use crate::parser::{BindingNode, ComponentNode, SectionNode};
use std::cell::RefCell;

thread_local! {
    static ENTITY: RefCell<String> = RefCell::new(String::new());
    static DATA: RefCell<ResolvedData> = RefCell::new(ResolvedData::None);
}

pub fn with_binding<R>(entity: &str, data: &ResolvedData, f: impl FnOnce() -> R) -> R {
    ENTITY.with(|e| {
        DATA.with(|d| {
            let prev_e = e.replace(entity.to_string());
            let prev_d = d.replace(data.clone());
            let out = f();
            e.replace(prev_e);
            d.replace(prev_d);
            out
        })
    })
}

pub fn entity() -> String {
    ENTITY.with(|e| e.borrow().clone())
}

pub fn api_path() -> Option<String> {
    let e = entity();
    if e.is_empty() {
        return None;
    }
    Some(format!("/api/{}", e.to_lowercase()))
}

/// Scalar for metric/progress: count, row len, or first numeric field.
pub fn scalar() -> Option<String> {
    DATA.with(|d| match &*d.borrow() {
        ResolvedData::Count(n) => Some(n.to_string()),
        ResolvedData::Rows(rows) if !rows.is_empty() => {
            if let Some(v) = rows[0].get("value") {
                return Some(json_to_string(v));
            }
            Some(rows.len().to_string())
        }
        ResolvedData::Record(Some(rec)) => rec
            .get("value")
            .or_else(|| rec.get("amount"))
            .or_else(|| rec.get("total"))
            .map(json_to_string),
        _ => None,
    })
}

pub fn rows() -> Vec<serde_json::Value> {
    DATA.with(|d| match &*d.borrow() {
        ResolvedData::Rows(rows) => rows.clone(),
        ResolvedData::Record(Some(rec)) => vec![rec.clone()],
        _ => Vec::new(),
    })
}

pub fn has_data() -> bool {
    DATA.with(|d| !matches!(*d.borrow(), ResolvedData::None))
}

fn json_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

pub fn component_from_section(family: &str, section: &SectionNode) -> ComponentNode {
    let mut items = Vec::new();
    if let Some(ref title) = section.title {
        items.push(crate::parser::ComponentItemNode {
            item_type: "title".into(),
            text: title.clone(),
            link: None,
            tone: None,
            config: Default::default(),
        });
    }
    for it in &section.items {
        let text = it
            .get("title")
            .or_else(|| it.get("label"))
            .or_else(|| it.get("name"))
            .or_else(|| it.get("text"))
            .cloned()
            .unwrap_or_default();
        if !text.is_empty() {
            items.push(crate::parser::ComponentItemNode {
                item_type: "item".into(),
                text,
                link: None,
                tone: None,
                config: Default::default(),
            });
        }
    }
    let mut props = section.config.clone();
    if let Some(ref b) = section.binding {
        props.insert("entity".into(), b.entity.clone());
    }
    ComponentNode {
        name: section.title.clone().unwrap_or_else(|| family.to_string()),
        layout: Some("stack".into()),
        style: Some(family.to_string()),
        items,
        props,
        params: vec![],
        template: None,
        sections: vec![],
        state: vec![],
        tests: vec![],
        binding: section.binding.clone(),
    }
}

pub fn entity_of(comp: &ComponentNode) -> Option<String> {
    let live = entity();
    if !live.is_empty() {
        return Some(live);
    }
    comp.props.get("entity").cloned().or_else(|| {
        comp.binding
            .as_ref()
            .map(|b: &BindingNode| b.entity.clone())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        assert!(entity().is_empty());
        assert!(scalar().is_none());
        assert!(!has_data());
    }

    #[test]
    fn count_becomes_scalar() {
        with_binding("Lead", &ResolvedData::Count(7), || {
            assert_eq!(entity(), "Lead");
            assert_eq!(scalar().as_deref(), Some("7"));
            assert_eq!(api_path().as_deref(), Some("/api/lead"));
        });
        assert!(entity().is_empty());
    }
}
