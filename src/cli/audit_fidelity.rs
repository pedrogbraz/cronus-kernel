//! CRONUS Dump Audit — Server-side fidelity check
//!
//! Compares rendered .cronus output against reference HTML.
//! Exit code 1 if fidelity < threshold — blocks compilation loop.
//!
//! Usage: cronus audit <reference.html> [--threshold 95]

use crate::parser::{self, AstNode};
use crate::ui::page::render_page;
use std::collections::HashMap;

/// Entry point for `cronus audit` command.
pub fn cmd_audit_fidelity(args: &[String]) {
    let ref_path = args.get(2).cloned().unwrap_or_default();
    if ref_path.is_empty() || ref_path.starts_with('-') {
        eprintln!("  \x1b[31m✗\x1b[0m Usage: cronus audit <reference.html> [--threshold 95]");
        std::process::exit(1);
    }

    let threshold: u32 = args
        .iter()
        .position(|a| a == "--threshold")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(95);

    // 1. Read reference HTML
    let ref_html = match std::fs::read_to_string(&ref_path) {
        Ok(h) => h,
        Err(e) => {
            eprintln!(
                "  \x1b[31m✗\x1b[0m Cannot read reference: {}: {}",
                ref_path, e
            );
            std::process::exit(1);
        }
    };

    // 2. Find and parse .cronus file
    let cronus_file = match crate::find_cronus_file() {
        Some(f) => f,
        None => {
            eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found in current directory");
            std::process::exit(1);
        }
    };
    let source = std::fs::read_to_string(&cronus_file).unwrap_or_default();
    let nodes = match parser::parse(&source) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // 3. Extract style config
    let mut accent = "blue".to_string();
    let mut theme = "dark".to_string();
    for node in &nodes {
        if let AstNode::Style(style) = node {
            if let Some(a) = &style.accent {
                accent = a.clone();
            }
            if let Some(t) = &style.theme {
                theme = t.clone();
            }
        }
    }

    // 4. Render all pages
    let mut rendered_html = String::new();
    let entities: Vec<crate::parser::EntityNode> = nodes
        .iter()
        .filter_map(|n| {
            if let AstNode::Entity(e) = n {
                Some(e.clone())
            } else {
                None
            }
        })
        .collect();

    for node in &nodes {
        if let AstNode::Page(page) = node {
            let empty_params: HashMap<String, String> = HashMap::new();
            let html = render_page(
                page,
                &entities,
                &accent,
                &theme,
                None,
                &empty_params,
                &crate::access::Access::anonymous(),
            );
            rendered_html.push_str(&html);
        }
    }

    // 5. Extract reference values
    let ref_text = extract_visible_text(&ref_html);
    let rendered_text = extract_visible_text(&rendered_html);

    let ref_numbers = extract_numbers(&ref_text);
    let ref_strings = extract_strings(&ref_text);
    let rendered_numbers = extract_numbers(&rendered_text);
    let rendered_strings = extract_strings(&rendered_text);

    // 6. Compare
    let mut num_matched = 0u32;
    let mut num_missing: Vec<String> = Vec::new();
    for n in &ref_numbers {
        if rendered_numbers.contains(n) {
            num_matched += 1;
        } else {
            num_missing.push(format!("{}", n));
        }
    }

    // Normalize rendered text for substring search — collapse whitespace
    let rendered_lower = rendered_text
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let mut str_matched = 0u32;
    let mut str_missing: Vec<String> = Vec::new();
    for s in &ref_strings {
        // Check both: extracted strings AND full rendered text (substring)
        if rendered_lower.contains(s.as_str())
            || rendered_strings
                .iter()
                .any(|rs| rs.contains(s.as_str()) || s.contains(rs.as_str()))
        {
            str_matched += 1;
        } else {
            str_missing.push(s.clone());
        }
    }

    let mut num_extra: Vec<String> = Vec::new();
    for n in &rendered_numbers {
        if !ref_numbers.contains(n) {
            num_extra.push(format!("{}", n));
        }
    }

    let ref_lower = ref_text
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let mut str_extra: Vec<String> = Vec::new();
    for s in &rendered_strings {
        if !ref_lower.contains(s.as_str())
            && !ref_strings
                .iter()
                .any(|rs| rs.contains(s.as_str()) || s.contains(rs.as_str()))
        {
            str_extra.push(s.clone());
        }
    }

    let total_ref = ref_numbers.len() + ref_strings.len();
    let total_matched = num_matched as usize + str_matched as usize;
    let fidelity = if total_ref > 0 {
        (total_matched as f64 / total_ref as f64 * 100.0) as u32
    } else {
        100
    };

    let total_missing = num_missing.len() + str_missing.len();
    let total_extra = num_extra.len() + str_extra.len();

    // 7. Report
    let dot = if fidelity >= 95 {
        "\x1b[32m●\x1b[0m"
    } else if fidelity >= 70 {
        "\x1b[33m●\x1b[0m"
    } else {
        "\x1b[31m●\x1b[0m"
    };

    println!();
    println!("  \x1b[1mCRONUS Dump Audit\x1b[0m");
    println!("  Reference: {}", ref_path);
    println!("  Source:    {}", cronus_file);
    println!();
    println!(
        "  {} \x1b[1m{}% fidelity\x1b[0m  (threshold: {}%)",
        dot, fidelity, threshold
    );
    println!("  Numbers: {}/{} matched", num_matched, ref_numbers.len());
    println!("  Strings: {}/{} matched", str_matched, ref_strings.len());
    println!();

    if !num_missing.is_empty() {
        println!("  \x1b[31m✗ Missing numbers:\x1b[0m");
        for n in &num_missing {
            println!("    NUM  {}", n);
        }
    }

    if !str_missing.is_empty() {
        println!("  \x1b[31m✗ Missing strings:\x1b[0m");
        for (i, s) in str_missing.iter().enumerate() {
            if i < 30 {
                println!("    STR  \"{}\"", s);
            }
        }
        if str_missing.len() > 30 {
            println!("    ... +{} more", str_missing.len() - 30);
        }
    }

    if !num_extra.is_empty() || !str_extra.is_empty() {
        println!();
        println!("  \x1b[33m! Extra in output (not in reference):\x1b[0m");
        for n in &num_extra {
            println!("    NUM  {}", n);
        }
        for s in str_extra.iter().take(10) {
            println!("    STR  \"{}\"", s);
        }
    }

    println!();
    println!("  Total: {} missing, {} extra", total_missing, total_extra);
    println!();

    // 8. Gate — exit 1 if below threshold
    if fidelity < threshold {
        eprintln!(
            "  \x1b[31m✗ AUDIT FAILED\x1b[0m — {}% < {}% threshold",
            fidelity, threshold
        );
        eprintln!(
            "  Fix the missing items above and re-run: cronus audit {}",
            ref_path
        );
        std::process::exit(1);
    } else {
        println!(
            "  \x1b[32m✓ AUDIT PASSED\x1b[0m — {}% >= {}% threshold",
            fidelity, threshold
        );
    }
}

// ── Public audit runner (reusable from build/run/dump) ──

/// Structured audit result for persistence and programmatic use.
#[derive(Clone, Debug)]
pub struct AuditResult {
    pub fidelity: u32,
    pub num_matched: u32,
    pub num_total: usize,
    pub str_matched: u32,
    pub str_total: usize,
    pub missing_numbers: Vec<String>,
    pub missing_strings: Vec<String>,
    pub extra_numbers: Vec<String>,
    pub extra_strings: Vec<String>,
}

/// Run fidelity audit comparing the current .cronus output against a reference HTML file.
/// Returns None if files can't be read or parsed; returns Some(AuditResult) otherwise.
pub fn run_audit(ref_html_path: &str) -> Option<AuditResult> {
    let ref_html = std::fs::read_to_string(ref_html_path).ok()?;
    let cronus_file = crate::find_cronus_file()?;
    let source = std::fs::read_to_string(&cronus_file).ok()?;
    let nodes = crate::parser::parse(&source).ok()?;

    run_audit_from_nodes(&ref_html, &nodes)
}

/// Run fidelity audit with pre-parsed AST nodes (avoids re-parsing).
pub fn run_audit_from_nodes(
    ref_html: &str,
    nodes: &[crate::parser::AstNode],
) -> Option<AuditResult> {
    use crate::parser::AstNode;
    use std::collections::HashMap;

    let mut accent = "blue".to_string();
    let mut theme = "dark".to_string();
    for node in nodes {
        if let AstNode::Style(style) = node {
            if let Some(a) = &style.accent {
                accent = a.clone();
            }
            if let Some(t) = &style.theme {
                theme = t.clone();
            }
        }
    }

    let mut rendered_html = String::new();
    let entities: Vec<crate::parser::EntityNode> = nodes
        .iter()
        .filter_map(|n| {
            if let AstNode::Entity(e) = n {
                Some(e.clone())
            } else {
                None
            }
        })
        .collect();

    for node in nodes {
        if let AstNode::Page(page) = node {
            let empty_params: HashMap<String, String> = HashMap::new();
            let html = crate::ui::page::render_page(
                page,
                &entities,
                &accent,
                &theme,
                None,
                &empty_params,
                &crate::access::Access::anonymous(),
            );
            rendered_html.push_str(&html);
        }
    }

    let ref_text = extract_visible_text(ref_html);
    let rendered_text = extract_visible_text(&rendered_html);

    let ref_numbers = extract_numbers(&ref_text);
    let ref_strings = extract_strings(&ref_text);
    let rendered_numbers = extract_numbers(&rendered_text);
    let rendered_strings = extract_strings(&rendered_text);

    let mut num_matched = 0u32;
    let mut missing_numbers: Vec<String> = Vec::new();
    for n in &ref_numbers {
        if rendered_numbers.contains(n) {
            num_matched += 1;
        } else {
            missing_numbers.push(format!("{}", n));
        }
    }

    let rendered_lower = rendered_text
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let mut str_matched = 0u32;
    let mut missing_strings: Vec<String> = Vec::new();
    for s in &ref_strings {
        if rendered_lower.contains(s.as_str())
            || rendered_strings
                .iter()
                .any(|rs| rs.contains(s.as_str()) || s.contains(rs.as_str()))
        {
            str_matched += 1;
        } else {
            missing_strings.push(s.clone());
        }
    }

    let mut extra_numbers: Vec<String> = Vec::new();
    for n in &rendered_numbers {
        if !ref_numbers.contains(n) {
            extra_numbers.push(format!("{}", n));
        }
    }

    let ref_lower = ref_text
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut extra_strings: Vec<String> = Vec::new();
    for s in &rendered_strings {
        if !ref_lower.contains(s.as_str())
            && !ref_strings
                .iter()
                .any(|rs| rs.contains(s.as_str()) || s.contains(rs.as_str()))
        {
            extra_strings.push(s.clone());
        }
    }

    let total_ref = ref_numbers.len() + ref_strings.len();
    let total_matched = num_matched as usize + str_matched as usize;
    let fidelity = if total_ref > 0 {
        (total_matched as f64 / total_ref as f64 * 100.0) as u32
    } else {
        100
    };

    Some(AuditResult {
        fidelity,
        num_matched,
        num_total: ref_numbers.len(),
        str_matched,
        str_total: ref_strings.len(),
        missing_numbers,
        missing_strings,
        extra_numbers,
        extra_strings,
    })
}

/// Save audit results to `.cronus/audit-results.json` with timestamp.
pub fn save_audit_results(result: &AuditResult) {
    let _ = std::fs::create_dir_all(".cronus");
    let timestamp = chrono_now_iso();

    // Load existing results to append
    let mut history: Vec<serde_json::Value> =
        if let Ok(existing) = std::fs::read_to_string(".cronus/audit-results.json") {
            serde_json::from_str(&existing).unwrap_or_default()
        } else {
            Vec::new()
        };

    let entry = serde_json::json!({
        "timestamp": timestamp,
        "fidelity": result.fidelity,
        "numbers": { "matched": result.num_matched, "total": result.num_total },
        "strings": { "matched": result.str_matched, "total": result.str_total },
        "missing": {
            "numbers": result.missing_numbers,
            "strings": result.missing_strings.iter().take(50).collect::<Vec<_>>(),
        },
        "extra": {
            "numbers": result.extra_numbers,
            "strings": result.extra_strings.iter().take(50).collect::<Vec<_>>(),
        },
    });

    history.push(entry);

    // Keep last 100 entries
    if history.len() > 100 {
        history = history.split_off(history.len() - 100);
    }

    if let Ok(json_str) = serde_json::to_string_pretty(&history) {
        let _ = std::fs::write(".cronus/audit-results.json", json_str);
    }
}

/// Print a compact fidelity summary line.
pub fn print_fidelity_line(result: &AuditResult) {
    let dot = if result.fidelity >= 95 {
        "\x1b[32m●\x1b[0m"
    } else if result.fidelity >= 70 {
        "\x1b[33m●\x1b[0m"
    } else {
        "\x1b[31m●\x1b[0m"
    };
    println!(
        "  {} \x1b[1m{}% fidelity\x1b[0m  (numbers: {}/{}, strings: {}/{})",
        dot,
        result.fidelity,
        result.num_matched,
        result.num_total,
        result.str_matched,
        result.str_total
    );
}

/// Print the top N missing items as warnings.
pub fn print_missing_top(result: &AuditResult, n: usize) {
    let mut items: Vec<String> = Vec::new();
    for s in result.missing_numbers.iter().take(n) {
        items.push(format!("NUM {}", s));
    }
    let remaining = n.saturating_sub(items.len());
    for s in result.missing_strings.iter().take(remaining) {
        items.push(format!("STR \"{}\"", s));
    }
    if !items.is_empty() {
        println!("  \x1b[33m⚠ Top missing items:\x1b[0m");
        for item in &items {
            println!("    {}", item);
        }
    }
}

fn chrono_now_iso() -> String {
    // Simple ISO timestamp without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let mins = (time_secs % 3600) / 60;
    let s = time_secs % 60;
    // Approximate date calculation (good enough for logging)
    let mut y = 1970i64;
    let mut remaining_days = days as i64;
    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0usize;
    while m < 12 && remaining_days >= month_days[m] {
        remaining_days -= month_days[m];
        m += 1;
    }
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m + 1,
        remaining_days + 1,
        hours,
        mins,
        s
    )
}

// ── Helpers (pub for reuse) ──

pub fn extract_visible_text(html: &str) -> String {
    let mut result = String::new();
    let body_start = html
        .find("<body")
        .and_then(|pos| html[pos..].find('>').map(|p| pos + p + 1))
        .unwrap_or(0);
    let body_end = html.rfind("</body>").unwrap_or(html.len());
    let body = &html[body_start..body_end];

    let mut in_tag = false;
    let mut skip_depth = 0u32;
    let skip_tags = [
        "script",
        "style",
        "svg",
        "noscript",
        "canvas",
        "iconify-icon",
    ];
    let mut tag_buf = String::new();
    // Also skip material icon spans
    let mut in_icon_span = false;

    for ch in body.chars() {
        if ch == '<' {
            in_tag = true;
            tag_buf.clear();
            continue;
        }
        if in_tag {
            if ch == '>' {
                in_tag = false;
                let tag_lower = tag_buf.to_lowercase();
                let tag_name = tag_lower.split_whitespace().next().unwrap_or("");
                if tag_name.starts_with('/') {
                    let name = &tag_name[1..];
                    if name == "span" && in_icon_span {
                        in_icon_span = false;
                    }
                    if skip_tags.contains(&name) && skip_depth > 0 {
                        skip_depth -= 1;
                    }
                } else {
                    // Insert space for block-level tags and <br>
                    if matches!(
                        tag_name,
                        "br" | "div"
                            | "p"
                            | "h1"
                            | "h2"
                            | "h3"
                            | "h4"
                            | "li"
                            | "td"
                            | "th"
                            | "section"
                            | "article"
                            | "footer"
                            | "header"
                            | "nav"
                    ) {
                        if skip_depth == 0 {
                            result.push(' ');
                        }
                    }
                    // Skip material icon spans
                    if tag_name == "span" && tag_buf.contains("material-symbols") {
                        in_icon_span = true;
                    }
                    if skip_tags.contains(&tag_name) {
                        skip_depth += 1;
                    }
                }
            } else {
                tag_buf.push(ch);
            }
            continue;
        }
        if skip_depth == 0 && !in_icon_span {
            result.push(ch);
        }
    }
    // Decode HTML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

pub fn extract_numbers(text: &str) -> Vec<f64> {
    let mut nums = Vec::new();
    let re_like = |s: &str| -> Option<f64> {
        let cleaned = s
            .replace(',', "")
            .replace('%', "")
            .replace('$', "")
            .replace('£', "")
            .replace('€', "")
            .replace('+', "");
        cleaned.trim().parse::<f64>().ok()
    };

    // Split by whitespace and non-alphanumeric, find number-like tokens
    for word in text.split(|c: char| {
        !c.is_alphanumeric() && c != '.' && c != ',' && c != '%' && c != '$' && c != '£' && c != '+'
    }) {
        let word = word.trim();
        if word.is_empty() {
            continue;
        }
        if let Some(n) = re_like(word) {
            if n.is_finite() && n >= 3.0 && n <= 1e9 {
                // Skip CSS noise
                let is_hex = n == n.floor() && n >= 100000.0 && n <= 999999.0;
                let is_css = [24.0, 32.0, 36.0, 48.0, 64.0, 96.0, 128.0, 256.0, 512.0].contains(&n);
                if !is_hex && !is_css && !nums.contains(&n) {
                    nums.push(n);
                }
            }
        }
    }
    nums
}

pub fn extract_strings(text: &str) -> Vec<String> {
    let mut strings = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.len() >= 3 && trimmed.len() <= 120 {
            let lower = trimmed.to_lowercase();
            if !lower.contains("tailwind")
                && !lower.contains("rgba(")
                && !lower.contains("gradient")
                && !lower.contains("animation")
                && !lower.starts_with('.')
                && !lower.starts_with('{')
                && !lower.starts_with('@')
                && !lower.starts_with('#')
            {
                if !strings.contains(&lower) {
                    strings.push(lower);
                }
            }
        }
    }
    strings
}
