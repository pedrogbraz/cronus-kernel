//! Native HTML for interactive cronus-ui families.
//!
//! Controls work without JavaScript (checkbox, dialog, details, range,
//! progress, tabs via small onclick). Voodoo attributes are additive and
//! only emitted when `crate::voodoo::enabled()` is true — never as authoring.

use crate::parser::ComponentNode;
use crate::voodoo;

const BASE: &str =
    "color:var(--cronus-fg);font-family:var(--cronus-font-sans,inherit);box-sizing:border-box;";
const SURF: &str = "background:var(--cronus-surface-raised,var(--cronus-surface-overlay,transparent));border:1px solid var(--cronus-border);border-radius:var(--cronus-radius,14px);";
const CTRL: &str = "height:2.5rem;padding:0 0.75rem;border-radius:0.5rem;border:1px solid var(--cronus-border);background:var(--cronus-surface-inset,transparent);color:var(--cronus-fg);outline:none;";

pub fn render(family: &str, comp: &ComponentNode) -> Option<String> {
    let html = match family {
        "checkbox" => checkbox(comp),
        "switch" | "toggle" => switch(family, comp),
        "toggle-group" | "segmented-control" | "radio-group" => radios(family, comp),
        "slider" => slider(comp),
        "progress" | "usage-meter" | "scroll-progress" => progress(family, comp),
        "rating" => rating(comp),
        "input" | "floating-label-input" | "input-group" => input(family, comp, "text"),
        "password-input" => input(family, comp, "password"),
        "number-input" | "currency-input" => input(family, comp, "number"),
        "phone-input" => input(family, comp, "tel"),
        "credit-card-input" => input(family, comp, "text"),
        "color-picker" => input(family, comp, "color"),
        "date-picker" => input(family, comp, "date"),
        "date-range-picker" => date_range(comp),
        "time-picker" => input(family, comp, "time"),
        "file-dropzone" => input(family, comp, "file"),
        "textarea" | "rich-text-editor" => textarea(family, comp),
        "select" | "combobox" | "autocomplete" | "multi-select" | "tags-input" => {
            select(family, comp)
        }
        "input-otp" => otp(comp),
        "field" | "form" => field_form(family, comp),
        "signature-pad" => signature(comp),
        "dialog" | "alert-dialog" | "confirmation-dialog" | "invite-dialog" | "sheet"
        | "drawer" | "lightbox" => dialog(family, comp),
        "popover" | "hover-card" | "tooltip" | "dropdown-menu" | "context-menu"
        | "morphing-popover" | "command" | "menubar" | "notification-center" | "sonner" => {
            popover(family, comp)
        }
        "tabs" | "code-tabs" | "expandable-tabs" => tabs(family, comp),
        "accordion" | "collapsible" => accordion(family, comp),
        "breadcrumb" => breadcrumb(comp),
        "pagination" => pagination(comp),
        "table" | "data-table" => table(family, comp),
        "calendar" | "scheduler" => calendar(family, comp),
        "stepper" => stepper(comp),
        "sidebar" | "app-shell" | "navigation-menu" | "pill-nav" | "dock" | "toolbar"
        | "table-of-contents" | "workspace-switcher" => nav(family, comp),
        "mode-toggle" => mode_toggle(comp),
        "copy-button" => copy_button(comp),
        "split-button" | "button-group" | "fab" => buttonish(family, comp),
        "alert" | "banner" | "empty" => alert(family, comp),
        "metric" => metric(comp),
        "badge" | "chip" | "kbd" | "label" | "status-dot" => pill(family, comp),
        "separator" => {
            "<hr data-slot=\"separator\" style=\"border:0;border-top:1px solid var(--cronus-border);margin:0.5rem 0;\" />"
                .into()
        }
        "skeleton" => skeleton(),
        "spinner" => spinner(),
        "card" | "card-stack" | "glass-card" | "spotlight-card" | "tilt-card" | "flip-card"
        | "frame" | "aspect-ratio" => card(family, comp),
        "avatar" | "avatar-group" => avatar(family, comp),
        "description-list" => description_list(comp),
        "timeline" => timeline(comp),
        "kanban" => kanban(comp),
        "json-viewer" | "code-block" | "terminal" => codey(family, comp),
        "tree-view" => tree(comp),
        "video-player" => video(comp),
        "carousel" | "logo-carousel" => carousel(family, comp),
        "scroll-area" | "resizable" | "masonry" => scroll(family, comp),
        "toast" => toast(comp),
        _ => return None,
    };
    Some(html)
}

fn label_of(comp: &ComponentNode) -> String {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return esc(t);
            }
        }
    }
    esc(&comp.name)
}

fn item<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    comp.items
        .iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
}

fn texts(comp: &ComponentNode) -> Vec<String> {
    let mut out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if out.is_empty() {
        out.push(label_of(comp));
    }
    out
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn num(comp: &ComponentNode, default: u32) -> u32 {
    comp.props
        .get("value")
        .or_else(|| comp.props.get("progress"))
        .and_then(|s| s.parse().ok())
        .or_else(|| item(comp, "value").and_then(|s| s.parse().ok()))
        .unwrap_or(default)
}

fn checkbox(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!(
        "<label data-slot=\"checkbox\" style=\"{BASE}display:inline-flex;align-items:center;gap:0.5rem;font-size:0.875rem;cursor:pointer;\"{data}><input type=\"checkbox\" data-slot=\"checkbox-control\"{model} style=\"accent-color:var(--cronus-primary);width:1rem;height:1rem;\" /><span>{label}</span></label>",
        data = voodoo::data("{ checked: false }"),
        model = voodoo::model("checked"),
    )
}

fn switch(family: &str, comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!(
        "<label data-slot=\"{family}\" style=\"{BASE}display:inline-flex;align-items:center;gap:0.5rem;font-size:0.875rem;cursor:pointer;\"{data}><input type=\"checkbox\" role=\"switch\" data-slot=\"{family}-control\"{model} style=\"accent-color:var(--cronus-primary);width:2.25rem;height:1.15rem;\" /><span>{label}</span></label>",
        data = voodoo::data("{ on: false }"),
        model = voodoo::model("on"),
    )
}

fn radios(family: &str, comp: &ComponentNode) -> String {
    let name = esc(&comp.name);
    let opts = texts(comp);
    let buttons = opts
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let checked = if i == 0 { " checked" } else { "" };
            format!(
                "<label style=\"display:inline-flex;align-items:center;gap:0.35rem;padding:0.25rem 0.6rem;border:1px solid var(--cronus-border);border-radius:0.4rem;cursor:pointer;font-size:0.8125rem;\"><input type=\"radio\" name=\"{name}\" value=\"{t}\"{checked}{model} style=\"accent-color:var(--cronus-primary);\" />{t}</label>",
                model = voodoo::model("value"),
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"{family}\" role=\"radiogroup\" style=\"{BASE}display:flex;flex-wrap:wrap;gap:0.35rem;\"{data}>{buttons}</div>",
        data = voodoo::data("{ value: '' }"),
    )
}

fn slider(comp: &ComponentNode) -> String {
    let v = num(comp, 40);
    format!(
        "<label data-slot=\"slider\" style=\"{BASE}display:flex;flex-direction:column;gap:0.35rem;font-size:0.875rem;\"{data}><span style=\"color:var(--cronus-fg-secondary);\">{label} {shown}</span><input type=\"range\" min=\"0\" max=\"100\" value=\"{v}\" data-slot=\"slider-control\"{model}{bind} style=\"width:100%;accent-color:var(--cronus-primary);\" /></label>",
        label = label_of(comp),
        data = voodoo::data(&format!("{{ value: {v} }}")),
        model = voodoo::model("value"),
        bind = voodoo::bind("value", "value"),
        shown = voodoo::interp("value", &v.to_string()),
    )
}

fn progress(family: &str, comp: &ComponentNode) -> String {
    let v = num(comp, 40);
    format!(
        "<div data-slot=\"{family}\" style=\"{BASE}display:flex;flex-direction:column;gap:0.35rem;\"{data}><span style=\"font-size:0.75rem;color:var(--cronus-fg-secondary);\">{label} {shown}%</span><progress max=\"100\" value=\"{v}\"{bind} style=\"width:100%;height:0.5rem;accent-color:var(--cronus-primary);\"></progress></div>",
        label = label_of(comp),
        data = voodoo::data(&format!("{{ value: {v} }}")),
        bind = voodoo::bind("value", "value"),
        shown = voodoo::interp("value", &v.to_string()),
    )
}

fn rating(comp: &ComponentNode) -> String {
    let stars = (1..=5)
        .map(|i| {
            format!(
                "<label style=\"cursor:pointer;font-size:1.1rem;\"><input type=\"radio\" name=\"{name}\" value=\"{i}\" style=\"position:absolute;opacity:0;\"{model} />★</label>",
                name = esc(&comp.name),
                model = voodoo::model("value"),
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"rating\" role=\"radiogroup\" aria-label=\"{label}\" style=\"{BASE}display:inline-flex;gap:0.15rem;color:var(--cronus-primary);\"{data}>{stars}</div>",
        label = label_of(comp),
        data = voodoo::data("{ value: 0 }"),
    )
}

fn input(family: &str, comp: &ComponentNode, ty: &str) -> String {
    let label = label_of(comp);
    let ph = esc(item(comp, "text").unwrap_or(""));
    format!(
        "<label data-slot=\"{family}\" style=\"{BASE}display:flex;flex-direction:column;gap:0.35rem;font-size:0.875rem;\"{data}><span style=\"color:var(--cronus-fg-secondary);\">{label}</span><input data-slot=\"{family}-control\" type=\"{ty}\" placeholder=\"{ph}\"{model} style=\"{CTRL}\" /></label>",
        data = voodoo::data("{ value: '' }"),
        model = voodoo::model("value"),
    )
}

fn date_range(comp: &ComponentNode) -> String {
    format!(
        "<label data-slot=\"date-range-picker\" style=\"{BASE}display:flex;flex-direction:column;gap:0.35rem;font-size:0.875rem;\"><span style=\"color:var(--cronus-fg-secondary);\">{label}</span><span style=\"display:flex;gap:0.5rem;\"><input type=\"date\" style=\"{CTRL}flex:1;\" /><input type=\"date\" style=\"{CTRL}flex:1;\" /></span></label>",
        label = label_of(comp),
    )
}

fn textarea(family: &str, comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!(
        "<label data-slot=\"{family}\" style=\"{BASE}display:flex;flex-direction:column;gap:0.35rem;font-size:0.875rem;\"{data}><span style=\"color:var(--cronus-fg-secondary);\">{label}</span><textarea data-slot=\"{family}-control\" rows=\"4\"{model} style=\"{CTRL}height:auto;padding:0.6rem 0.75rem;resize:vertical;\"></textarea></label>",
        data = voodoo::data("{ value: '' }"),
        model = voodoo::model("value"),
    )
}

fn select(family: &str, comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let opts = texts(comp)
        .into_iter()
        .map(|t| format!("<option>{t}</option>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<label data-slot=\"{family}\" style=\"{BASE}display:flex;flex-direction:column;gap:0.35rem;font-size:0.875rem;\"{data}><span style=\"color:var(--cronus-fg-secondary);\">{label}</span><select data-slot=\"{family}-control\"{model} style=\"{CTRL}\">{opts}</select></label>",
        data = voodoo::data("{ value: '' }"),
        model = voodoo::model("value"),
    )
}

fn otp(comp: &ComponentNode) -> String {
    let inputs = (0..6)
        .map(|i| {
            format!(
                "<input data-slot=\"input-otp-slot\" inputmode=\"numeric\" maxlength=\"1\" aria-label=\"digit {n}\" style=\"{CTRL}width:2.5rem;text-align:center;font-variant-numeric:tabular-nums;\" />",
                n = i + 1,
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<fieldset data-slot=\"input-otp\" style=\"{BASE}display:flex;flex-direction:column;gap:0.35rem;border:0;padding:0;margin:0;\"><legend style=\"color:var(--cronus-fg-secondary);font-size:0.875rem;\">{label}</legend><div style=\"display:flex;gap:0.35rem;\">{inputs}</div></fieldset>",
        label = label_of(comp),
    )
}

fn field_form(family: &str, comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let fields = texts(comp)
        .into_iter()
        .map(|t| {
            format!(
                "<label style=\"display:flex;flex-direction:column;gap:0.25rem;font-size:0.875rem;\"><span style=\"color:var(--cronus-fg-secondary);\">{t}</span><input name=\"{t}\" style=\"{CTRL}\" /></label>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<form data-slot=\"{family}\" style=\"{BASE}{SURF}padding:1rem;display:flex;flex-direction:column;gap:0.75rem;\"><div style=\"font-weight:500;\">{label}</div>{fields}<button type=\"submit\" data-slot=\"button\" style=\"height:2.5rem;border-radius:0.5rem;border:0;background:var(--cronus-primary);color:var(--cronus-primary-foreground);cursor:pointer;\">Submit</button></form>"
    )
}

fn signature(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"signature-pad\" style=\"{BASE}{SURF}padding:0.75rem;\"><div style=\"font-size:0.875rem;color:var(--cronus-fg-secondary);margin-bottom:0.5rem;\">{label}</div><canvas width=\"320\" height=\"120\" style=\"width:100%;height:120px;background:var(--cronus-surface-inset,transparent);border-radius:0.4rem;\"></canvas></div>",
        label = label_of(comp),
    )
}

fn dialog(family: &str, comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let body = texts(comp)
        .into_iter()
        .skip(1)
        .map(|t| {
            format!("<p style=\"margin:0;color:var(--cronus-fg-secondary);font-size:0.875rem;\">{t}</p>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"{family}\" style=\"{BASE}\"><button type=\"button\" data-slot=\"button\" onclick=\"this.nextElementSibling.showModal()\" style=\"height:2.5rem;padding:0 1rem;border-radius:0.5rem;border:1px solid var(--cronus-border);background:var(--cronus-primary);color:var(--cronus-primary-foreground);cursor:pointer;\">Open {title}</button><dialog data-slot=\"{family}-content\" style=\"{SURF}padding:1.25rem;max-width:28rem;color:var(--cronus-fg);\"><form method=\"dialog\" style=\"display:flex;flex-direction:column;gap:0.75rem;\"><div style=\"font-weight:500;\">{title}</div>{body}<button type=\"submit\" value=\"close\" data-slot=\"button\" style=\"height:2.25rem;border-radius:0.5rem;border:1px solid var(--cronus-border);background:transparent;color:var(--cronus-fg);cursor:pointer;\">Close</button></form></dialog></div>"
    )
}

fn popover(family: &str, comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let body = texts(comp)
        .into_iter()
        .map(|t| format!("<div style=\"font-size:0.875rem;color:var(--cronus-fg-secondary);\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<details data-slot=\"{family}\" style=\"{BASE}position:relative;display:inline-block;\"><summary style=\"list-style:none;cursor:pointer;padding:0.4rem 0.7rem;border:1px solid var(--cronus-border);border-radius:0.4rem;\">{title}</summary><div style=\"{SURF}position:absolute;z-index:20;margin-top:0.35rem;padding:0.75rem;min-width:12rem;\">{body}</div></details>"
    )
}

fn tabs(family: &str, comp: &ComponentNode) -> String {
    let items = texts(comp);
    let data = voodoo::data("{ tab: 0 }");
    let buttons = items
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let selected = if i == 0 { "true" } else { "false" };
            let click = if voodoo::enabled() {
                voodoo::click(&format!("tab = {i}"))
            } else {
                format!(
                    " onclick=\"var r=this.closest('[data-slot]');r.querySelectorAll('[role=tab]').forEach(function(b,j){{b.setAttribute('aria-selected', String(j==={i}));}});r.querySelectorAll('[role=tabpanel]').forEach(function(p,j){{p.hidden=j!=={i};}});\""
                )
            };
            format!(
                "<button type=\"button\" role=\"tab\" aria-selected=\"{selected}\"{click} style=\"padding:0.4rem 0.75rem;border:0;border-bottom:2px solid {border};background:transparent;color:var(--cronus-fg);cursor:pointer;\">{t}</button>",
                border = if i == 0 {
                    "var(--cronus-primary)"
                } else {
                    "transparent"
                },
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let panels = items
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let hidden = if i == 0 { "" } else { " hidden" };
            let show = if voodoo::enabled() {
                voodoo::show(&format!("tab === {i}"))
            } else {
                String::new()
            };
            format!("<div role=\"tabpanel\"{hidden}{show} style=\"padding:0.75rem 0;font-size:0.875rem;\">{t}</div>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"{family}\" style=\"{BASE}\"{data}><div role=\"tablist\" style=\"display:flex;gap:0.15rem;border-bottom:1px solid var(--cronus-border);\">{buttons}</div>{panels}</div>"
    )
}

fn accordion(family: &str, comp: &ComponentNode) -> String {
    let items = texts(comp)
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            let open = if i == 0 { " open" } else { "" };
            format!(
                "<details{open} style=\"border-bottom:1px solid var(--cronus-border);padding:0.5rem 0;\"><summary style=\"cursor:pointer;font-size:0.875rem;\">{t}</summary><div style=\"padding:0.5rem 0;color:var(--cronus-fg-secondary);font-size:0.875rem;\">{t}</div></details>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"{family}\" style=\"{BASE}{SURF}padding:0.5rem 1rem;\">{items}</div>")
}

fn breadcrumb(comp: &ComponentNode) -> String {
    let crumbs = texts(comp)
        .into_iter()
        .map(|t| format!("<li style=\"display:inline;\"><a href=\"#\" style=\"color:var(--cronus-fg-secondary);text-decoration:none;\">{t}</a></li>"))
        .collect::<Vec<_>>()
        .join("<li aria-hidden=\"true\" style=\"display:inline;color:var(--cronus-fg-secondary);\"> / </li>");
    format!("<nav data-slot=\"breadcrumb\" aria-label=\"Breadcrumb\" style=\"{BASE}\"><ol style=\"list-style:none;padding:0;margin:0;display:flex;gap:0.35rem;font-size:0.8125rem;\">{crumbs}</ol></nav>")
}

fn pagination(comp: &ComponentNode) -> String {
    let _ = comp;
    format!(
        "<nav data-slot=\"pagination\" aria-label=\"Pagination\" style=\"{BASE}display:flex;gap:0.25rem;\"><button type=\"button\" style=\"{CTRL}width:2.5rem;\">‹</button><button type=\"button\" aria-current=\"page\" style=\"{CTRL}width:2.5rem;background:var(--cronus-primary);color:var(--cronus-primary-foreground);\">1</button><button type=\"button\" style=\"{CTRL}width:2.5rem;\">2</button><button type=\"button\" style=\"{CTRL}width:2.5rem;\">›</button></nav>"
    )
}

fn table(family: &str, comp: &ComponentNode) -> String {
    let cols = texts(comp);
    let head = cols
        .iter()
        .map(|c| {
            format!("<th style=\"text-align:left;padding:0.5rem 0.75rem;font-size:0.75rem;font-weight:500;color:var(--cronus-fg-secondary);border-bottom:1px solid var(--cronus-border);\">{c}</th>")
        })
        .collect::<Vec<_>>()
        .join("");
    let row = cols
        .iter()
        .map(|_| {
            "<td style=\"padding:0.6rem 0.75rem;font-size:0.875rem;border-bottom:1px solid var(--cronus-border);\">—</td>"
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"{family}\" style=\"{BASE}{SURF}overflow:auto;\"><table style=\"width:100%;border-collapse:collapse;\"><thead><tr>{head}</tr></thead><tbody><tr>{row}</tr></tbody></table></div>"
    )
}

fn calendar(family: &str, comp: &ComponentNode) -> String {
    let cells: String = (1..=28)
        .map(|d| {
            format!(
                "<button type=\"button\" style=\"height:2rem;border:0;background:transparent;color:var(--cronus-fg);border-radius:0.35rem;cursor:pointer;\">{d}</button>"
            )
        })
        .collect();
    format!(
        "<div data-slot=\"{family}\" style=\"{BASE}{SURF}padding:0.75rem;display:grid;grid-template-columns:repeat(7,1fr);gap:0.15rem;\"><div style=\"grid-column:1/-1;font-size:0.875rem;margin-bottom:0.35rem;\">{label}</div>{cells}</div>",
        label = label_of(comp),
    )
}

fn stepper(comp: &ComponentNode) -> String {
    let steps = texts(comp)
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            format!(
                "<li style=\"display:flex;align-items:center;gap:0.4rem;font-size:0.8125rem;\"><span style=\"width:1.4rem;height:1.4rem;border-radius:999px;display:inline-flex;align-items:center;justify-content:center;background:var(--cronus-primary);color:var(--cronus-primary-foreground);font-size:0.7rem;\">{n}</span>{t}</li>",
                n = i + 1,
            )
        })
        .collect::<Vec<_>>()
        .join("<li aria-hidden=\"true\" style=\"flex:1;height:1px;background:var(--cronus-border);\"></li>");
    format!("<ol data-slot=\"stepper\" style=\"{BASE}display:flex;align-items:center;gap:0.5rem;list-style:none;padding:0;margin:0;\">{steps}</ol>")
}

fn nav(family: &str, comp: &ComponentNode) -> String {
    let links = texts(comp)
        .iter()
        .map(|t| {
            format!("<a href=\"#\" style=\"color:var(--cronus-fg-secondary);text-decoration:none;padding:0.35rem 0.6rem;border-radius:0.4rem;\">{t}</a>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<nav data-slot=\"{family}\" style=\"{BASE}display:flex;flex-wrap:wrap;gap:0.25rem;align-items:center;\">{links}</nav>")
}

fn mode_toggle(comp: &ComponentNode) -> String {
    let _ = comp;
    format!(
        "<button type=\"button\" data-slot=\"mode-toggle\" onclick=\"document.documentElement.classList.toggle('dark')\" style=\"{BASE}{SURF}height:2.25rem;padding:0 0.75rem;cursor:pointer;\">Theme</button>"
    )
}

fn copy_button(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!(
        "<button type=\"button\" data-slot=\"copy-button\" onclick=\"navigator.clipboard.writeText(this.dataset.copy||this.textContent)\" data-copy=\"{label}\" style=\"{BASE}{SURF}height:2.25rem;padding:0 0.75rem;cursor:pointer;font-size:0.8125rem;\">Copy</button>"
    )
}

fn buttonish(family: &str, comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!(
        "<div data-slot=\"{family}\" style=\"{BASE}display:inline-flex;gap:0.25rem;\"><button type=\"button\" data-slot=\"button\" style=\"height:2.5rem;padding:0 1rem;border-radius:0.5rem;border:0;background:var(--cronus-primary);color:var(--cronus-primary-foreground);cursor:pointer;\">{label}</button></div>"
    )
}

fn alert(family: &str, comp: &ComponentNode) -> String {
    let rest = texts(comp)
        .into_iter()
        .map(|t| format!("<div>{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"{family}\" role=\"status\" style=\"{BASE}{SURF}padding:0.85rem 1rem;display:flex;flex-direction:column;gap:0.25rem;font-size:0.875rem;\">{rest}</div>"
    )
}

fn metric(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let value = item(comp, "value").unwrap_or("0");
    format!(
        "<section data-slot=\"metric\" style=\"{BASE}{SURF}padding:1rem;\"><div style=\"font-size:0.75rem;color:var(--cronus-fg-secondary);\">{title}</div><div style=\"font-size:1.5rem;letter-spacing:-0.02em;\">{value}</div></section>"
    )
}

fn pill(family: &str, comp: &ComponentNode) -> String {
    format!(
        "<span data-slot=\"{family}\" style=\"{BASE}{SURF}display:inline-flex;align-items:center;gap:0.35rem;padding:0.15rem 0.55rem;font-size:0.75rem;font-weight:500;\">{label}</span>",
        label = label_of(comp),
    )
}

fn skeleton() -> String {
    "<div data-slot=\"skeleton\" aria-hidden=\"true\" style=\"height:0.9rem;width:8rem;border-radius:0.35rem;background:var(--cronus-border);\"></div>".into()
}

fn spinner() -> String {
    "<div data-slot=\"spinner\" role=\"status\" aria-label=\"Loading\" style=\"width:1.25rem;height:1.25rem;border:2px solid var(--cronus-border);border-top-color:var(--cronus-primary);border-radius:999px;\"></div>".into()
}

fn card(family: &str, comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let rest = texts(comp)
        .into_iter()
        .skip(1)
        .map(|t| format!("<div style=\"color:var(--cronus-fg-secondary);font-size:0.875rem;\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<section data-slot=\"{family}\" style=\"{BASE}{SURF}padding:1rem;display:flex;flex-direction:column;gap:0.5rem;\"><div style=\"font-weight:500;\">{title}</div>{rest}</section>"
    )
}

fn avatar(family: &str, comp: &ComponentNode) -> String {
    let letter = label_of(comp)
        .chars()
        .next()
        .unwrap_or('A')
        .to_uppercase()
        .to_string();
    format!(
        "<div data-slot=\"{family}\" style=\"{BASE}width:2.25rem;height:2.25rem;border-radius:999px;background:var(--cronus-surface-overlay);border:1px solid var(--cronus-border);display:inline-flex;align-items:center;justify-content:center;font-size:0.75rem;font-weight:500;\">{letter}</div>"
    )
}

fn description_list(comp: &ComponentNode) -> String {
    let rows = texts(comp)
        .into_iter()
        .map(|t| format!("<div style=\"display:flex;justify-content:space-between;gap:1rem;font-size:0.875rem;\"><dt style=\"color:var(--cronus-fg-secondary);\">{t}</dt><dd style=\"margin:0;\">—</dd></div>"))
        .collect::<Vec<_>>()
        .join("");
    format!("<dl data-slot=\"description-list\" style=\"{BASE}{SURF}padding:1rem;display:flex;flex-direction:column;gap:0.5rem;\">{rows}</dl>")
}

fn timeline(comp: &ComponentNode) -> String {
    let items = texts(comp)
        .into_iter()
        .map(|t| {
            format!("<li style=\"padding-left:1rem;border-left:2px solid var(--cronus-border);padding-bottom:0.75rem;font-size:0.875rem;\">{t}</li>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<ol data-slot=\"timeline\" style=\"{BASE}list-style:none;padding:0;margin:0;\">{items}</ol>")
}

fn kanban(comp: &ComponentNode) -> String {
    let cols = texts(comp)
        .into_iter()
        .map(|t| {
            format!("<div style=\"{SURF}padding:0.75rem;min-width:10rem;\"><div style=\"font-size:0.75rem;color:var(--cronus-fg-secondary);margin-bottom:0.5rem;\">{t}</div><div style=\"{SURF}padding:0.5rem;font-size:0.8125rem;\">Item</div></div>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"kanban\" style=\"{BASE}display:flex;gap:0.75rem;overflow:auto;\">{cols}</div>")
}

fn codey(family: &str, comp: &ComponentNode) -> String {
    let body = texts(comp).join("\n");
    format!(
        "<pre data-slot=\"{family}\" style=\"{BASE}{SURF}padding:0.85rem 1rem;overflow:auto;font-size:0.8125rem;\"><code>{body}</code></pre>"
    )
}

fn tree(comp: &ComponentNode) -> String {
    let items = texts(comp)
        .into_iter()
        .map(|t| format!("<li>{t}</li>"))
        .collect::<Vec<_>>()
        .join("");
    format!("<ul data-slot=\"tree-view\" style=\"{BASE}{SURF}padding:0.75rem 1.25rem;font-size:0.875rem;\">{items}</ul>")
}

fn video(comp: &ComponentNode) -> String {
    format!(
        "<video data-slot=\"video-player\" controls style=\"{BASE}{SURF}width:100%;max-width:36rem;\"><p>{label}</p></video>",
        label = label_of(comp),
    )
}

fn carousel(family: &str, comp: &ComponentNode) -> String {
    let slides = texts(comp)
        .into_iter()
        .map(|t| {
            format!("<div style=\"{SURF}min-width:12rem;padding:1.5rem;text-align:center;\">{t}</div>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"{family}\" style=\"{BASE}display:flex;gap:0.75rem;overflow:auto;\">{slides}</div>")
}

fn scroll(family: &str, comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"{family}\" style=\"{BASE}{SURF}max-height:12rem;overflow:auto;padding:0.75rem;font-size:0.875rem;\">{label}</div>",
        label = label_of(comp),
    )
}

fn toast(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"toast\" role=\"status\" style=\"{BASE}{SURF}padding:0.75rem 1rem;font-size:0.875rem;\">{label}</div>",
        label = label_of(comp),
    )
}
