#![allow(dead_code, unused_imports)]
//! Utility helpers shared across UI submodules.

use crate::parser::ComponentItemNode;

/// Get the text of the first item matching a given kind.
pub(super) fn item_by_kind<'a>(items: &'a [ComponentItemNode], kind: &str) -> Option<&'a str> {
    items
        .iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
}

/// Get all items matching a given kind.
pub(super) fn items_by_kind<'a>(
    items: &'a [ComponentItemNode],
    kind: &str,
) -> Vec<&'a ComponentItemNode> {
    items.iter().filter(|i| i.item_type == kind).collect()
}
