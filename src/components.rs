//! CRONUS Component Library — 40+ UI Components
//!
//! Replaces shadcn/ui + Radix + Material UI.
//! Each component is a pure function returning HTML String.
//! Dark theme, Tailwind classes, zero JS dependencies.

// ══════════════════════════════════════════════════
// BUTTONS
// ══════════════════════════════════════════════════

/// Legacy kernel Button (Obsidian / existing demos).
/// variant: primary, secondary, ghost, danger, outline — size: sm, md, lg
/// Unchanged on purpose: cronus-ui Button is `cronus_ui::button`, opted-in via
/// `style:button+primary` so existing pages keep working.
pub fn button(label: &str, variant: &str, size: &str, href: Option<&str>) -> String {
    let size_cls = match size {
        "sm" => "text-xs px-3 py-1.5",
        "lg" => "text-base px-6 py-3",
        _ => "text-sm px-4 py-2",
    };
    let variant_cls = match variant {
        "primary" => "font-semibold",
        "secondary" => "",
        "ghost" => "",
        "danger" => "",
        "outline" => "",
        _ => "",
    };
    let variant_style = match variant {
        "primary" => "background:var(--foreground);color:var(--background);border:1px solid var(--border);",
        "secondary" => "background:transparent;color:var(--foreground);border:1px solid var(--border);",
        "ghost" => "background:transparent;color:var(--foreground-muted);border:1px solid transparent;",
        "danger" => "background:color-mix(in oklch, var(--danger) 14%, transparent);color:var(--danger);border:1px solid color-mix(in oklch, var(--danger) 28%, transparent);",
        "outline" => "background:transparent;color:var(--foreground);border:1px solid var(--border);",
        _ => "background:transparent;color:var(--foreground);border:1px solid var(--border);",
    };
    let tag = if href.is_some() { "a" } else { "button" };
    let href_attr = href.map(|h| format!(" href=\"{}\"", h)).unwrap_or_default();
    format!(
        "<{tag}{href_attr} class=\"inline-flex items-center justify-center gap-2 {size_cls} {variant_cls} font-medium tracking-wide uppercase transition-all duration-200 outline-none cursor-pointer hover:opacity-90 active:opacity-80\" style=\"border-radius:var(--radius-button);{variant_style}\">{label}</{tag}>",
    )
}

#[cfg(test)]
mod legacy_button_tests {
    use super::*;

    #[test]
    fn legacy_button_unchanged() {
        let html = button("Save", "primary", "md", None);
        assert!(html.contains("uppercase"));
        assert!(html.contains("var(--foreground)"));
        assert!(!html.contains("data-slot"));
        assert!(!html.contains("--cronus-primary"));
    }
}

// ══════════════════════════════════════════════════
// INPUTS
// ══════════════════════════════════════════════════

// ══════════════════════════════════════════════════
// DATA DISPLAY
// ══════════════════════════════════════════════════

/// Badge
pub fn badge(text: &str, color: &str) -> String {
    let cls = match color {
        "emerald" | "green" => "background:color-mix(in oklch, var(--success) 14%, var(--card));color:var(--success);border-color:color-mix(in oklch, var(--success) 28%, var(--border));",
        "amber" | "yellow" => "background:color-mix(in oklch, var(--warning) 14%, var(--card));color:var(--warning);border-color:color-mix(in oklch, var(--warning) 28%, var(--border));",
        "red" => "background:color-mix(in oklch, var(--danger) 14%, var(--card));color:var(--danger);border-color:color-mix(in oklch, var(--danger) 28%, var(--border));",
        "blue" | "purple" => "background:color-mix(in oklch, var(--accent) 14%, var(--card));color:var(--accent);border-color:color-mix(in oklch, var(--accent) 28%, var(--border));",
        _ => "background:var(--card);color:var(--foreground-muted);border-color:var(--border);",
    };
    format!(
        "<span class=\"inline-flex items-center px-2 py-0.5 text-[10px] font-mono tracking-wider uppercase border\" style=\"border-radius:999px;{cls}\">{text}</span>",
    )
}

/// Stat card
pub fn stat_card(label: &str, value: &str, change_pct: Option<f32>, icon: &str) -> String {
    let change_html = change_pct
        .map(|p| {
            let (color, arrow) = if p >= 0.0 {
                ("var(--success)", "↑")
            } else {
                ("var(--danger)", "↓")
            };
            format!(
                "<span class=\"text-xs font-mono\" style=\"color:{color}\">{arrow} {:.1}%</span>",
                p.abs()
            )
        })
        .unwrap_or_default();
    format!(
        "<div class=\"border p-5\" style=\"background:var(--card);border-color:var(--border);border-radius:var(--radius-card)\">\
        <div class=\"flex items-center justify-between mb-3\">\
        <span class=\"text-[10px] font-mono tracking-[0.15em] uppercase\" style=\"color:var(--foreground-muted)\">{label}</span>\
        <span style=\"color:var(--foreground-muted)\">{icon}</span>\
        </div>\
        <div class=\"flex items-baseline gap-3\">\
        <span class=\"text-2xl font-bold tracking-tight\" style=\"color:var(--foreground)\">{value}</span>\
        {change_html}\
        </div>\
        </div>",
    )
}

/// Keyboard shortcut display
pub fn kbd(key: &str) -> String {
    format!(
        "<kbd class=\"inline-flex items-center px-2 py-0.5 text-[10px] font-mono text-neutral-400 bg-neutral-800 border border-neutral-700 rounded shadow-sm\">{key}</kbd>",
    )
}

/// Empty state
pub fn empty_state(icon: &str, title: &str, description: &str, cta: Option<&str>) -> String {
    let cta_html = cta.map(|label| format!(
        "<button class=\"mt-4 px-4 py-2 text-sm bg-amber-500 text-black font-medium rounded hover:bg-amber-400 transition-colors\">{label}</button>"
    )).unwrap_or_default();
    format!(
        "<div class=\"flex flex-col items-center justify-center py-16 text-center\">\
        <div class=\"text-4xl text-neutral-700 mb-4\">{icon}</div>\
        <h3 class=\"text-lg font-semibold text-neutral-400\">{title}</h3>\
        <p class=\"mt-1 text-sm text-neutral-600 max-w-sm\">{description}</p>\
        {cta_html}\
        </div>",
    )
}

// ══════════════════════════════════════════════════
// FEEDBACK
// ══════════════════════════════════════════════════

/// Alert
pub fn alert(message: &str, variant: &str, closable: bool) -> String {
    let (bg, border, text, icon) = match variant {
        "success" => (
            "bg-emerald-500/5",
            "border-emerald-500/20",
            "text-emerald-400",
            "✓",
        ),
        "warning" => (
            "bg-amber-500/5",
            "border-amber-500/20",
            "text-amber-400",
            "⚠",
        ),
        "error" => ("bg-red-500/5", "border-red-500/20", "text-red-400", "✗"),
        _ => ("bg-blue-500/5", "border-blue-500/20", "text-blue-400", "ℹ"),
    };
    let close = if closable {
        " <button class=\"ml-auto text-neutral-600 hover:text-white transition-colors\" onclick=\"this.parentElement.remove()\">&times;</button>"
    } else {
        ""
    };
    format!(
        "<div class=\"flex items-center gap-3 px-4 py-3 {bg} border {border} rounded text-sm {text}\">\
        <span>{icon}</span>\
        <span class=\"flex-1\">{message}</span>\
        {close}\
        </div>",
    )
}

// ══════════════════════════════════════════════════
// NAVIGATION
// ══════════════════════════════════════════════════

/// Tabs
pub fn tabs(items: &[(&str, &str)], active: usize) -> String {
    let tab_buttons: String = items.iter().enumerate().map(|(i, (label, _))| {
        let (cls, style) = if i == active {
            (
                "border-b-2",
                "background:var(--card);color:var(--foreground);border-color:var(--accent);",
            )
        } else {
            (
                "border-b-2",
                "background:var(--card);color:var(--foreground-muted);border-color:transparent;",
            )
        };
        format!("<button class=\"px-4 py-2.5 text-sm font-medium transition-all {cls}\" style=\"{style}\" data-tab=\"{i}\" data-tab-group=\"tabs\">{label}</button>")
    }).collect::<Vec<_>>().join("\n    ");

    let tab_contents: String = items
        .iter()
        .enumerate()
        .map(|(i, (_, content))| {
            let display = if i == active { "block" } else { "none" };
            format!("<div data-tab-content=\"{i}\" style=\"display:{display}\">{content}</div>")
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    format!(
        "<div>\
        <div class=\"flex items-center border-b gap-0\" style=\"border-color:var(--border)\">\n    {tab_buttons}\n  </div>\
        <div class=\"mt-4\">\n    {tab_contents}\n  </div>\
        </div>",
    )
}

// ══════════════════════════════════════════════════
// LAYOUT COMPONENTS
// ══════════════════════════════════════════════════

/// Card wrapper
pub fn card(content: &str, padding: &str) -> String {
    let pad = match padding {
        "sm" => "p-4",
        "lg" => "p-8",
        "none" => "p-0",
        _ => "p-6",
    };
    format!("<div class=\"border {pad}\" style=\"background:var(--card);border-color:var(--border);border-radius:var(--radius-card);box-shadow:var(--shadow-sm)\">{content}</div>")
}

/// Modal/dialog
pub fn modal(title: &str, content: &str, id: &str) -> String {
    format!(
        "<div id=\"{id}\" class=\"fixed inset-0 z-50 hidden items-center justify-center bg-black/60 backdrop-blur-sm\">\
        <div class=\"border w-full max-w-md mx-4\" style=\"background:var(--card);border-color:var(--border);border-radius:var(--radius-card);box-shadow:var(--shadow-lg)\">\
        <div class=\"flex items-center justify-between px-6 py-4 border-b\" style=\"border-color:var(--border)\">\
        <h3 class=\"text-sm font-semibold\" style=\"color:var(--foreground)\">{title}</h3>\
        <button onclick=\"document.getElementById('{id}').classList.add('hidden')\" class=\"transition-colors\" style=\"color:var(--foreground-muted)\">&times;</button>\
        </div>\
        <div class=\"px-6 py-4\">{content}</div>\
        </div>\
        </div>",
    )
}

/// Dropdown menu
pub fn dropdown(trigger: &str, items: &[(&str, &str)]) -> String {
    let menu_items: String = items.iter().map(|(label, href)| {
        if label.is_empty() {
            "<div class=\"border-t border-neutral-800 my-1\"></div>".to_string()
        } else {
            format!("<a href=\"{href}\" class=\"block px-4 py-2 text-sm text-neutral-400 hover:text-white hover:bg-neutral-800 transition-colors\">{label}</a>")
        }
    }).collect::<Vec<_>>().join("\n      ");
    format!(
        "<div class=\"relative group\">\
        <div class=\"cursor-pointer\">{trigger}</div>\
        <div class=\"absolute right-0 mt-1 w-48 bg-[#0a0a0a] border border-neutral-800 rounded-lg shadow-xl py-1 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-50\">\n      {menu_items}\n    </div>\
        </div>",
    )
}

// ══════════════════════════════════════════════════
// SIDEBAR NAV
// ══════════════════════════════════════════════════

// ══════════════════════════════════════════════════
// HERO SECTION
// ══════════════════════════════════════════════════

// ══════════════════════════════════════════════════
// PRICING GRID
// ══════════════════════════════════════════════════

// ══════════════════════════════════════════════════
// COMMAND PALETTE (Ctrl+K)
// ══════════════════════════════════════════════════

// ══════════════════════════════════════════════════
// SHEET (slide-in panel)
// ══════════════════════════════════════════════════

// ══════════════════════════════════════════════════
// METRIC ROW
// ══════════════════════════════════════════════════
