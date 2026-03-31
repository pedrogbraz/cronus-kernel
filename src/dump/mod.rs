#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Dump — HTML→.cronus converter
//!
//! Analyzes HTML pages and generates .cronus files that describe
//! the same content using CRONUS language primitives.

pub mod dom;
pub mod detect;
pub mod patterns;
pub mod emit;
pub mod openapi;
pub mod prisma;

use std::collections::HashMap;

/// Main entry point: takes raw HTML, returns .cronus source
pub fn dump_html(html: &str) -> String {
    // 1. Parse DOM
    let nodes = dom::parse_html(html);

    // 2. Detect theme
    let theme = dom::detect_theme_with_html(html, &nodes);

    // 3. Extract app info
    let app_name = dom::extract_title(html);

    // 4. Detect sections
    let sections = detect::detect_sections(&nodes);

    // 5. Build CronusFile
    let file = emit::CronusFile {
        app_name,
        port: 5175,
        theme: theme.to_string(),
        style_config: {
            let mut m = HashMap::new();
            m.insert("font".into(), "Inter".into());
            m.insert("accent".into(), if theme == "dark" { "blue".into() } else { "black".into() });
            m
        },
        sections,
    };

    // 6. Emit .cronus
    emit::emit_cronus(&file)
}
