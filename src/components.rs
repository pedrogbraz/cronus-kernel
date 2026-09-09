#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Component Library — 40+ UI Components
//!
//! Replaces shadcn/ui + Radix + Material UI.
//! Each component is a pure function returning HTML String.
//! Dark theme, Tailwind classes, zero JS dependencies.

// ══════════════════════════════════════════════════
// BUTTONS
// ══════════════════════════════════════════════════

/// Button — cronus-ui CONTRACT (semantic --cronus-* tokens).
/// variant: primary, secondary, outline, ghost, destructive (danger alias), link
/// size: sm, md, lg, icon, icon-sm
pub fn button(label: &str, variant: &str, size: &str, href: Option<&str>) -> String {
    crate::cronus_ui::button(label, variant, size, href)
}

/// Icon button (square)
pub fn icon_button(icon_svg: &str, variant: &str, size: &str) -> String {
    let size_cls = match size {
        "sm" => "w-7 h-7 text-xs",
        "lg" => "w-11 h-11 text-base",
        _ => "w-9 h-9 text-sm",
    };
    let variant_cls = match variant {
        "primary" => "bg-amber-500 text-black hover:bg-amber-400",
        "ghost" => "bg-transparent text-neutral-400 hover:bg-neutral-800 hover:text-white",
        "danger" => "bg-red-500/10 text-red-400 hover:bg-red-500/20",
        _ => "bg-neutral-800 text-white hover:bg-neutral-700",
    };
    format!(
        "<button class=\"inline-flex items-center justify-center {size_cls} {variant_cls} rounded transition-all duration-200 focus:ring-2 focus:ring-amber-500/30 outline-none cursor-pointer\">{icon_svg}</button>",
    )
}

/// Button group (horizontal)
pub fn button_group(buttons: &[String]) -> String {
    let inner = buttons.join("\n    ");
    format!(
        "<div class=\"inline-flex items-center gap-0 [&>*]:rounded-none [&>*:first-child]:rounded-l [&>*:last-child]:rounded-r [&>*:not(:last-child)]:border-r-0\">\n    {inner}\n  </div>",
    )
}

// ══════════════════════════════════════════════════
// INPUTS
// ══════════════════════════════════════════════════

/// Text input
pub fn input(name: &str, input_type: &str, placeholder: &str, required: bool) -> String {
    let req = if required { " required" } else { "" };
    format!(
        "<input type=\"{input_type}\" name=\"{name}\" placeholder=\"{placeholder}\"{req} class=\"w-full px-4 py-2.5 bg-[#0a0a0a] border border-neutral-800 rounded text-sm text-white placeholder-neutral-600 transition-all duration-200 focus:border-amber-500/50 focus:ring-2 focus:ring-amber-500/20 outline-none hover:border-neutral-700\">",
    )
}

/// Textarea
pub fn textarea(name: &str, placeholder: &str, rows: u32) -> String {
    format!(
        "<textarea name=\"{name}\" placeholder=\"{placeholder}\" rows=\"{rows}\" class=\"w-full px-4 py-2.5 bg-[#0a0a0a] border border-neutral-800 rounded text-sm text-white placeholder-neutral-600 transition-all duration-200 focus:border-amber-500/50 focus:ring-2 focus:ring-amber-500/20 outline-none hover:border-neutral-700 resize-y min-h-[80px]\"></textarea>",
    )
}

/// Select dropdown
pub fn select(name: &str, options: &[(&str, &str)], placeholder: &str) -> String {
    let opts: String = std::iter::once(format!("<option value=\"\" disabled selected class=\"text-neutral-600\">{placeholder}</option>"))
        .chain(options.iter().map(|(val, label)| format!("<option value=\"{val}\" class=\"bg-neutral-900\">{label}</option>")))
        .collect::<Vec<_>>().join("\n      ");
    format!(
        "<select name=\"{name}\" class=\"w-full px-4 py-2.5 bg-[#0a0a0a] border border-neutral-800 rounded text-sm text-white transition-all duration-200 focus:border-amber-500/50 focus:ring-2 focus:ring-amber-500/20 outline-none hover:border-neutral-700 appearance-none cursor-pointer\">\n      {opts}\n    </select>",
    )
}

/// Checkbox
pub fn checkbox(name: &str, label: &str, checked: bool) -> String {
    let chk = if checked { " checked" } else { "" };
    format!(
        "<label class=\"inline-flex items-center gap-3 cursor-pointer group\">\
        <input type=\"checkbox\" name=\"{name}\"{chk} class=\"w-4 h-4 rounded border-neutral-700 bg-[#0a0a0a] text-amber-500 focus:ring-amber-500/30 focus:ring-2 cursor-pointer\">\
        <span class=\"text-sm text-neutral-300 group-hover:text-white transition-colors\">{label}</span>\
        </label>",
    )
}

/// Toggle switch
pub fn toggle(name: &str, label: &str, checked: bool) -> String {
    let bg = if checked { "bg-amber-500" } else { "bg-neutral-700" };
    let dot_pos = if checked { "translate-x-5" } else { "translate-x-0.5" };
    let chk = if checked { " checked" } else { "" };
    format!(
        "<label class=\"inline-flex items-center gap-3 cursor-pointer group\">\
        <input type=\"checkbox\" name=\"{name}\"{chk} class=\"sr-only peer\">\
        <div class=\"relative w-10 h-5 {bg} rounded-full transition-colors peer-focus:ring-2 peer-focus:ring-amber-500/30\">\
        <div class=\"absolute top-0.5 w-4 h-4 bg-white rounded-full transition-transform {dot_pos}\"></div>\
        </div>\
        <span class=\"text-sm text-neutral-300 group-hover:text-white transition-colors\">{label}</span>\
        </label>",
    )
}

/// Radio group
pub fn radio_group(name: &str, options: &[(&str, &str)], selected: &str) -> String {
    let radios: String = options.iter().map(|(val, label)| {
        let chk = if *val == selected { " checked" } else { "" };
        format!(
            "<label class=\"inline-flex items-center gap-2 cursor-pointer group\">\
            <input type=\"radio\" name=\"{name}\" value=\"{val}\"{chk} class=\"w-4 h-4 border-neutral-700 bg-[#0a0a0a] text-amber-500 focus:ring-amber-500/30\">\
            <span class=\"text-sm text-neutral-300 group-hover:text-white transition-colors\">{label}</span>\
            </label>"
        )
    }).collect::<Vec<_>>().join("\n    ");
    format!("<div class=\"flex flex-col gap-3\">\n    {radios}\n  </div>")
}

/// Search input with icon
pub fn search_input(placeholder: &str, _entity: &str) -> String {
    format!(
        "<div class=\"relative\">\
        <svg class=\"absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-neutral-500\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" viewBox=\"0 0 24 24\"><circle cx=\"11\" cy=\"11\" r=\"8\"/><path d=\"m21 21-4.3-4.3\"/></svg>\
        <input type=\"search\" placeholder=\"{placeholder}\" class=\"w-full pl-10 pr-4 py-2.5 bg-[#0a0a0a] border border-neutral-800 rounded text-sm text-white placeholder-neutral-600 transition-all duration-200 focus:border-amber-500/50 focus:ring-2 focus:ring-amber-500/20 outline-none\">\
        </div>",
    )
}

/// File upload
pub fn file_upload(name: &str, accept: &str) -> String {
    format!(
        "<label class=\"flex flex-col items-center justify-center w-full h-32 border-2 border-dashed border-neutral-800 rounded-lg cursor-pointer hover:border-neutral-600 hover:bg-neutral-900/50 transition-all duration-200\">\
        <svg class=\"w-8 h-8 text-neutral-600 mb-2\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.5\" viewBox=\"0 0 24 24\"><path d=\"M12 16V4m0 0L8 8m4-4l4 4M4 17v2a2 2 0 002 2h12a2 2 0 002-2v-2\"/></svg>\
        <span class=\"text-xs text-neutral-500\">Click to upload or drag & drop</span>\
        <input type=\"file\" name=\"{name}\" accept=\"{accept}\" class=\"sr-only\">\
        </label>",
    )
}

/// Range slider
pub fn range_slider(name: &str, min: i32, max: i32, value: i32) -> String {
    format!(
        "<div class=\"flex items-center gap-3\">\
        <span class=\"text-xs text-neutral-500 font-mono\">{min}</span>\
        <input type=\"range\" name=\"{name}\" min=\"{min}\" max=\"{max}\" value=\"{value}\" class=\"flex-1 h-1 bg-neutral-800 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-amber-500\">\
        <span class=\"text-xs text-neutral-500 font-mono\">{max}</span>\
        </div>",
    )
}

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

/// Avatar
pub fn avatar(name: &str, src: Option<&str>, size: &str) -> String {
    let size_cls = match size {
        "xs" => "w-6 h-6 text-[9px]",
        "sm" => "w-8 h-8 text-[10px]",
        "lg" => "w-12 h-12 text-sm",
        _ => "w-10 h-10 text-xs",
    };
    let initials: String = name.split_whitespace().filter_map(|w| w.chars().next()).take(2).collect::<String>().to_uppercase();
    if let Some(url) = src {
        format!("<img src=\"{url}\" alt=\"{name}\" class=\"{size_cls} rounded-full object-cover border border-neutral-800\">")
    } else {
        format!("<div class=\"{size_cls} rounded-full bg-neutral-800 border border-neutral-700 flex items-center justify-center font-mono font-medium text-neutral-400\">{initials}</div>")
    }
}

/// Avatar group (stacked)
pub fn avatar_group(avatars: &[String]) -> String {
    let inner: String = avatars.iter().map(|a| format!("<div class=\"-ml-2 first:ml-0 ring-2 ring-[#080808] rounded-full\">{a}</div>")).collect::<Vec<_>>().join("\n    ");
    format!("<div class=\"flex items-center\">\n    {inner}\n  </div>")
}

/// Stat card
pub fn stat_card(label: &str, value: &str, change_pct: Option<f32>, icon: &str) -> String {
    let change_html = change_pct.map(|p| {
        let (color, arrow) = if p >= 0.0 { ("var(--success)", "↑") } else { ("var(--danger)", "↓") };
        format!("<span class=\"text-xs font-mono\" style=\"color:{color}\">{arrow} {:.1}%</span>", p.abs())
    }).unwrap_or_default();
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

/// Progress bar
pub fn progress_bar(value: u32, max: u32, color: &str) -> String {
    let pct = if max > 0 { (value as f32 / max as f32 * 100.0) as u32 } else { 0 };
    let bg = match color {
        "emerald" => "bg-emerald-500",
        "red" => "bg-red-500",
        "blue" => "bg-blue-500",
        _ => "bg-amber-500",
    };
    format!(
        "<div class=\"w-full h-1.5 bg-neutral-800 rounded-full overflow-hidden\">\
        <div class=\"h-full {bg} rounded-full transition-all duration-500\" style=\"width:{pct}%\"></div>\
        </div>",
    )
}

/// Tooltip (CSS-only)
pub fn tooltip(trigger: &str, content: &str) -> String {
    format!(
        "<span class=\"relative group cursor-help\">{trigger}\
        <span class=\"absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-1.5 bg-neutral-800 border border-neutral-700 rounded text-xs text-neutral-300 whitespace-nowrap opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 pointer-events-none\">{content}</span>\
        </span>",
    )
}

/// Tag (closable)
pub fn tag(text: &str, color: &str, closable: bool) -> String {
    let cls = match color {
        "emerald" => "bg-emerald-500/10 text-emerald-400",
        "amber" => "bg-amber-500/10 text-amber-400",
        "red" => "bg-red-500/10 text-red-400",
        "blue" => "bg-blue-500/10 text-blue-400",
        _ => "bg-neutral-800 text-neutral-300",
    };
    let close_btn = if closable {
        " <button class=\"ml-1 hover:text-white transition-colors\">&times;</button>"
    } else { "" };
    format!(
        "<span class=\"inline-flex items-center px-2.5 py-1 text-xs font-medium rounded {cls}\">{text}{close_btn}</span>",
    )
}

/// Code block
pub fn code_block(code: &str, _language: &str) -> String {
    let escaped = code.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    format!(
        "<div class=\"relative group\">\
        <pre class=\"bg-[#0a0a0a] border border-neutral-800 rounded-lg p-4 overflow-x-auto\">\
        <code class=\"text-sm font-mono text-neutral-300 leading-relaxed\">{escaped}</code>\
        </pre>\
        <button class=\"absolute top-2 right-2 px-2 py-1 text-[10px] font-mono text-neutral-600 hover:text-neutral-300 bg-neutral-800 rounded opacity-0 group-hover:opacity-100 transition-all\" onclick=\"navigator.clipboard.writeText(this.parentElement.querySelector('code').textContent)\">COPY</button>\
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

/// Skeleton loading placeholder
pub fn skeleton(width: &str, height: &str) -> String {
    format!(
        "<div class=\"animate-pulse bg-neutral-800 rounded\" style=\"width:{width};height:{height}\"></div>",
    )
}

/// Separator (horizontal line)
pub fn separator() -> String {
    "<hr class=\"border-neutral-800 my-6\">".to_string()
}

// ══════════════════════════════════════════════════
// FEEDBACK
// ══════════════════════════════════════════════════

/// Alert
pub fn alert(message: &str, variant: &str, closable: bool) -> String {
    let (bg, border, text, icon) = match variant {
        "success" => ("bg-emerald-500/5", "border-emerald-500/20", "text-emerald-400", "✓"),
        "warning" => ("bg-amber-500/5", "border-amber-500/20", "text-amber-400", "⚠"),
        "error" => ("bg-red-500/5", "border-red-500/20", "text-red-400", "✗"),
        _ => ("bg-blue-500/5", "border-blue-500/20", "text-blue-400", "ℹ"),
    };
    let close = if closable {
        " <button class=\"ml-auto text-neutral-600 hover:text-white transition-colors\" onclick=\"this.parentElement.remove()\">&times;</button>"
    } else { "" };
    format!(
        "<div class=\"flex items-center gap-3 px-4 py-3 {bg} border {border} rounded text-sm {text}\">\
        <span>{icon}</span>\
        <span class=\"flex-1\">{message}</span>\
        {close}\
        </div>",
    )
}

/// Toast container (inject once, use JS to add toasts)
pub fn toast_container() -> String {
    "<div id=\"cronus-toasts\" class=\"fixed top-4 right-4 z-[999] flex flex-col gap-2 pointer-events-none\"></div>\
    <script>\
    window.cronusToast=function(msg,type){\
      var el=document.createElement('div');\
      var colors={success:'border-emerald-500/30 text-emerald-400',error:'border-red-500/30 text-red-400',info:'border-blue-500/30 text-blue-400'};\
      el.className='pointer-events-auto px-4 py-3 bg-[#0a0a0a] border rounded text-sm font-mono animate-[slideIn_0.3s_ease-out] '+(colors[type]||colors.info);\
      el.textContent=msg;\
      document.getElementById('cronus-toasts').appendChild(el);\
      setTimeout(function(){el.style.opacity='0';el.style.transition='opacity 0.3s';setTimeout(function(){el.remove()},300);},3000);\
    };\
    </script>".to_string()
}

/// Loading spinner
pub fn loading_spinner(size: &str) -> String {
    let sz = match size {
        "sm" => "w-4 h-4",
        "lg" => "w-8 h-8",
        _ => "w-6 h-6",
    };
    format!(
        "<svg class=\"{sz} animate-spin text-amber-500\" viewBox=\"0 0 24 24\" fill=\"none\">\
        <circle class=\"opacity-25\" cx=\"12\" cy=\"12\" r=\"10\" stroke=\"currentColor\" stroke-width=\"4\"></circle>\
        <path class=\"opacity-75\" fill=\"currentColor\" d=\"M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z\"></path>\
        </svg>",
    )
}

/// Loading dots
pub fn loading_dots() -> String {
    "<div class=\"flex items-center gap-1\">\
    <div class=\"w-1.5 h-1.5 bg-neutral-500 rounded-full animate-bounce\" style=\"animation-delay:0s\"></div>\
    <div class=\"w-1.5 h-1.5 bg-neutral-500 rounded-full animate-bounce\" style=\"animation-delay:0.1s\"></div>\
    <div class=\"w-1.5 h-1.5 bg-neutral-500 rounded-full animate-bounce\" style=\"animation-delay:0.2s\"></div>\
    </div>".to_string()
}

// ══════════════════════════════════════════════════
// NAVIGATION
// ══════════════════════════════════════════════════

/// Breadcrumb
pub fn breadcrumb(items: &[(&str, &str)]) -> String {
    let crumbs: String = items.iter().enumerate().map(|(i, (label, href))| {
        let sep = if i > 0 { "<span class=\"text-neutral-700 mx-2\">/</span>" } else { "" };
        let is_last = i == items.len() - 1;
        if is_last {
            format!("{sep}<span class=\"text-neutral-300 text-sm\">{label}</span>")
        } else {
            format!("{sep}<a href=\"{href}\" class=\"text-neutral-500 text-sm hover:text-white transition-colors\">{label}</a>")
        }
    }).collect::<Vec<_>>().join("");
    format!("<nav class=\"flex items-center font-mono text-xs tracking-wider\">{crumbs}</nav>")
}

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

    let tab_contents: String = items.iter().enumerate().map(|(i, (_, content))| {
        let display = if i == active { "block" } else { "none" };
        format!("<div data-tab-content=\"{i}\" style=\"display:{display}\">{content}</div>")
    }).collect::<Vec<_>>().join("\n    ");

    format!(
        "<div>\
        <div class=\"flex items-center border-b gap-0\" style=\"border-color:var(--border)\">\n    {tab_buttons}\n  </div>\
        <div class=\"mt-4\">\n    {tab_contents}\n  </div>\
        </div>",
    )
}

/// Pagination
pub fn pagination(current: u32, total: u32, per_page: u32) -> String {
    let total_pages = if total > 0 { (total + per_page - 1) / per_page } else { 1 };
    let mut pages = Vec::new();

    for i in 1..=total_pages.min(7) {
        let cls = if i == current {
            "bg-amber-500 text-black font-medium"
        } else {
            "bg-neutral-800 text-neutral-400 hover:bg-neutral-700 hover:text-white"
        };
        pages.push(format!("<button class=\"w-8 h-8 text-xs rounded {cls} transition-all\">{i}</button>"));
    }

    let prev_cls = if current > 1 { "text-neutral-400 hover:text-white" } else { "text-neutral-700 cursor-not-allowed" };
    let next_cls = if current < total_pages { "text-neutral-400 hover:text-white" } else { "text-neutral-700 cursor-not-allowed" };

    format!(
        "<div class=\"flex items-center gap-1.5\">\
        <button class=\"w-8 h-8 text-xs rounded bg-neutral-800 {prev_cls} transition-all\">&laquo;</button>\
        {}\
        <button class=\"w-8 h-8 text-xs rounded bg-neutral-800 {next_cls} transition-all\">&raquo;</button>\
        <span class=\"ml-3 text-xs text-neutral-600 font-mono\">{current}/{total_pages}</span>\
        </div>",
        pages.join("\n    "),
    )
}

/// Steps / stepper
pub fn steps(items: &[(&str, &str)], current: usize) -> String {
    let step_items: String = items.iter().enumerate().map(|(i, (label, status))| {
        let (dot_cls, text_cls, line_cls) = match *status {
            "complete" => ("bg-emerald-500", "text-emerald-400", "bg-emerald-500"),
            "current" => ("bg-amber-500 ring-4 ring-amber-500/20", "text-white", "bg-neutral-700"),
            _ => ("bg-neutral-700", "text-neutral-500", "bg-neutral-800"),
        };
        let line = if i < items.len() - 1 {
            format!("<div class=\"flex-1 h-px {line_cls} mx-3\"></div>")
        } else { String::new() };
        format!(
            "<div class=\"flex items-center gap-2 flex-shrink-0\">\
            <div class=\"w-3 h-3 rounded-full {dot_cls} transition-all\"></div>\
            <span class=\"text-xs font-mono tracking-wider {text_cls}\">{label}</span>\
            </div>{line}"
        )
    }).collect::<Vec<_>>().join("\n    ");

    format!("<div class=\"flex items-center w-full\">\n    {step_items}\n  </div>")
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

/// Table wrapper
pub fn table(headers: &[&str], rows: &[Vec<String>], accent: &str) -> String {
    let th: String = headers.iter().map(|h| {
        format!("<th class=\"px-4 py-3 text-left text-[10px] font-mono tracking-[0.15em] text-neutral-500 uppercase\">{h}</th>")
    }).collect::<Vec<_>>().join("");

    let tr: String = rows.iter().map(|row| {
        let tds: String = row.iter().map(|cell| {
            format!("<td class=\"px-4 py-3 text-sm text-neutral-300\">{cell}</td>")
        }).collect::<Vec<_>>().join("");
        format!("<tr class=\"border-t border-neutral-800/50 hover:bg-neutral-900/50 transition-colors\">{tds}</tr>")
    }).collect::<Vec<_>>().join("\n      ");

    format!(
        "<div class=\"bg-[#080808] border border-neutral-800 rounded-lg overflow-hidden\">\
        <table class=\"w-full\">\
        <thead class=\"border-b border-neutral-800\"><tr>{th}</tr></thead>\
        <tbody>\n      {tr}\n    </tbody>\
        </table>\
        </div>",
    )
}

// ══════════════════════════════════════════════════
// SIDEBAR NAV
// ══════════════════════════════════════════════════

pub fn sidebar_nav(items: &[(&str, &str, &str)], active: &str) -> String {
    let links: String = items.iter().map(|(label, href, icon)| {
        let is_active = *label == active || *href == active;
        let bg = if is_active { "var(--card-soft)" } else { "transparent" };
        let color = if is_active { "var(--foreground)" } else { "var(--foreground-muted)" };
        format!(r#"<a href="{href}" style="display:flex;align-items:center;gap:10px;padding:8px 14px;border-radius:10px;font-size:13px;font-weight:500;color:{color};background:{bg};text-decoration:none;transition:all .15s">{icon}<span>{label}</span></a>"#, href=href, icon=icon, label=label, color=color, bg=bg)
    }).collect::<Vec<_>>().join("\n    ");
    format!(r#"<nav style="width:220px;flex-shrink:0;display:flex;flex-direction:column;gap:2px;padding:8px">{links}</nav>"#, links=links)
}

// ══════════════════════════════════════════════════
// HERO SECTION
// ══════════════════════════════════════════════════

pub fn hero(badge_text: &str, title: &str, subtitle: &str, ctas: &[(&str, &str, &str)]) -> String {
    let badge = if badge_text.is_empty() { String::new() } else {
        format!(r#"<div style="display:inline-flex;align-items:center;gap:6px;padding:4px 14px;border-radius:20px;border:1px solid var(--border);font-size:11px;font-weight:500;color:var(--foreground-muted);margin-bottom:24px"><span style="width:5px;height:5px;border-radius:50%;background:var(--accent)"></span>{}</div>"#, badge_text)
    };
    let buttons: String = ctas.iter().map(|(label, href, variant)| {
        let style = if *variant == "primary" { "background:var(--foreground);color:var(--background)" } else { "background:transparent;color:var(--foreground-muted);border:1px solid var(--border)" };
        format!(r#"<a href="{}" style="padding:10px 24px;border-radius:10px;font-size:14px;font-weight:500;text-decoration:none;{}">{}</a>"#, href, style, label)
    }).collect::<Vec<_>>().join("\n      ");
    format!(r#"<section style="text-align:center;padding:80px 0">{badge}<h1 style="font-size:48px;font-weight:700;letter-spacing:-0.03em;color:var(--foreground);line-height:1.1;margin:0 0 16px">{title}</h1><p style="font-size:18px;color:var(--foreground-muted);max-width:600px;margin:0 auto 32px;line-height:1.6">{subtitle}</p><div style="display:flex;gap:12px;justify-content:center">{buttons}</div></section>"#, badge=badge, title=title, subtitle=subtitle, buttons=buttons)
}

// ══════════════════════════════════════════════════
// PRICING GRID
// ══════════════════════════════════════════════════

pub fn pricing_grid(plans: &[(&str, &str, &[&str], bool)]) -> String {
    let cards: String = plans.iter().map(|(name, price, features, featured)| {
        let border = if *featured { "oklch(0.488 0.243 264/40%)" } else { "var(--border)" };
        let badge = if *featured { r#"<span style="position:absolute;top:-10px;left:50%;transform:translateX(-50%);padding:2px 12px;border-radius:12px;font-size:10px;font-weight:600;background:var(--accent);color:white">Popular</span>"# } else { "" };
        let feats: String = features.iter().map(|f| format!(r#"<li style="display:flex;align-items:center;gap:8px;font-size:13px;color:var(--foreground-muted)"><span style="color:var(--success)">&#10003;</span>{}</li>"#, f)).collect::<Vec<_>>().join("");
        format!(r#"<div style="position:relative;border-radius:22px;border:1px solid {border};background:var(--card);padding:28px;display:flex;flex-direction:column">{badge}<h3 style="font-size:16px;font-weight:600;color:var(--foreground);margin:0 0 8px">{name}</h3><p style="font-size:32px;font-weight:700;color:var(--foreground);margin:0 0 24px">{price}</p><ul style="list-style:none;padding:0;margin:0 0 24px;display:flex;flex-direction:column;gap:10px;flex:1">{feats}</ul><button style="width:100%;padding:10px;border-radius:10px;font-size:14px;font-weight:500;border:none;cursor:pointer;background:var(--foreground);color:var(--background)">Comecar</button></div>"#, border=border, badge=badge, name=name, price=price, feats=feats)
    }).collect::<Vec<_>>().join("");
    format!(r#"<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:16px">{cards}</div>"#, cards=cards)
}

// ══════════════════════════════════════════════════
// COMMAND PALETTE (Ctrl+K)
// ══════════════════════════════════════════════════

pub fn command_palette(commands: &[(&str, &str)]) -> String {
    let items: String = commands.iter().map(|(label, shortcut)| {
        format!(r#"<div class="cmd-item" style="display:flex;align-items:center;justify-content:space-between;padding:8px 14px;border-radius:8px;cursor:pointer" onmouseover="this.style.background='var(--card-soft)'" onmouseout="this.style.background='transparent'"><span style="font-size:13px;color:var(--foreground)">{}</span><kbd style="font-size:11px;padding:2px 6px;border-radius:4px;background:var(--card);border:1px solid var(--border);color:var(--foreground-muted)">{}</kbd></div>"#, label, shortcut)
    }).collect::<Vec<_>>().join("");
    format!(r##"<div id="cronus-command" style="display:none;position:fixed;inset:0;z-index:9999;background:oklch(0 0 0/60%);backdrop-filter:blur(4px);align-items:flex-start;justify-content:center;padding-top:20vh" onclick="if(event.target===this)this.style.display='none'">
  <div style="width:480px;border-radius:16px;border:1px solid var(--border);background:var(--background);overflow:hidden;box-shadow:0 20px 60px oklch(0 0 0/40%)">
    <div style="padding:12px 14px;border-bottom:1px solid var(--border)"><input id="cmd-search" type="text" placeholder="Type a command..." style="width:100%;background:transparent;border:none;outline:none;font-size:14px;color:var(--foreground)" oninput="var q=this.value.toLowerCase();document.querySelectorAll('.cmd-item').forEach(function(e){{e.style.display=e.textContent.toLowerCase().includes(q)?'flex':'none'}})"></div>
    <div style="padding:8px;max-height:300px;overflow-y:auto">{items}</div>
  </div>
</div>
<script>document.addEventListener('keydown',function(e){{if((e.metaKey||e.ctrlKey)&&e.key==='k'){{e.preventDefault();var p=document.getElementById('cronus-command');p.style.display=p.style.display==='none'?'flex':'none';if(p.style.display==='flex')document.getElementById('cmd-search').focus()}}}});</script>"##, items=items)
}

// ══════════════════════════════════════════════════
// SHEET (slide-in panel)
// ══════════════════════════════════════════════════

pub fn sheet(title: &str, content: &str, id: &str) -> String {
    format!(r##"<div id="{id}" style="display:none;position:fixed;inset:0;z-index:9998;background:oklch(0 0 0/50%)" onclick="if(event.target===this)this.style.display='none'">
  <div style="position:absolute;right:0;top:0;bottom:0;width:400px;background:var(--background);border-left:1px solid var(--border);display:flex;flex-direction:column;transform:translateX(0);transition:transform .2s">
    <div style="display:flex;align-items:center;justify-content:space-between;padding:16px 20px;border-bottom:1px solid var(--border)">
      <h2 style="font-size:16px;font-weight:600;color:var(--foreground);margin:0">{title}</h2>
      <button onclick="document.getElementById('{id}').style.display='none'" style="background:none;border:none;color:var(--foreground-muted);cursor:pointer;font-size:18px">&#10005;</button>
    </div>
    <div style="padding:20px;flex:1;overflow-y:auto">{content}</div>
  </div>
</div>"##, id=id, title=title, content=content)
}

// ══════════════════════════════════════════════════
// METRIC ROW
// ══════════════════════════════════════════════════

pub fn metric_row(icon_svg: &str, label: &str, value: &str, color: &str) -> String {
    format!(r#"<div style="display:flex;align-items:center;gap:12px;padding:10px 0;border-bottom:1px solid oklch(1 0 0/4%)"><div style="width:32px;height:32px;border-radius:8px;background:{color};display:flex;align-items:center;justify-content:center;opacity:0.15"><div style="color:{color};opacity:1">{icon}</div></div><span style="flex:1;font-size:13px;color:var(--foreground-muted)">{label}</span><span style="font-size:14px;font-weight:600;color:var(--foreground);font-variant-numeric:tabular-nums">{value}</span></div>"#, icon=icon_svg, label=label, value=value, color=color)
}
