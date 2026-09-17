//! Dedicated Toggle renderer. DOM matches React/Radix Toggle: a `<button
//! data-slot="toggle" data-state="on|off" aria-pressed>` with the label and an
//! optional lucide glyph (`icon:`). React emits no `data-variant` / `data-size`
//! (the audit checks that), so `toggleVariants` — `outline` and `sm|md|lg` —
//! travel as classes (`v-outline`, `s-sm`) from the style (`toggle+outline+sm`)
//! or `variant:` / `size:` props.
//!
//! Zero JS: the button sits in a `<label>` after a visually hidden checkbox
//! that carries the pressed state (click, Space, focus ring); `pressed:true`
//! checks it. CSS paints the button after `:checked` as on. The button keeps
//! React's slot and look but is decorative (`aria-hidden`, `tabindex="-1"`,
//! `pointer-events: none`). Not interact `switch()`
//! (`<label data-slot="toggle"><input type="checkbox">`).

use crate::cronus_ui_kit::{attr_nonempty, choice, esc, flag, instance_id, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = item(comp, "label")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| esc(&comp.name));
    let on = pressed(comp);
    let (state, aria, checked) = if on {
        ("on", "true", " checked")
    } else {
        ("off", "false", "")
    };
    let mut class = String::new();
    if choice(comp, "variant", &["outline"]).is_some() {
        class.push_str(" v-outline");
    }
    if let Some(size) = choice(comp, "size", &["sm", "lg"]) {
        class.push_str(&format!(" s-{size}"));
    }
    let class = if class.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", class.trim())
    };
    let icon = attr_nonempty(comp, "icon")
        .map(|i| crate::cronus_ui_icons::svg_or_empty(i))
        .unwrap_or_default();
    let icon_only = flag(comp, "icon-only");
    let text = if icon_only {
        String::new()
    } else {
        label.clone()
    };
    let name = attr_nonempty(comp, "aria-label")
        .map(esc)
        .unwrap_or_else(|| label.clone());
    let disabled = flag(comp, "disabled");
    let input_disabled = if disabled { " disabled" } else { "" };
    let dimmed = if disabled { " data-disabled=\"\"" } else { "" };
    let id = instance_id(comp, "toggle");
    format!(
        "<label><input type=\"checkbox\" id=\"{id}\" aria-label=\"{name}\"{checked}{input_disabled}><button type=\"button\" data-slot=\"toggle\"{class} data-state=\"{state}\" aria-pressed=\"{aria}\" tabindex=\"-1\" aria-hidden=\"true\"{dimmed}>{icon}{text}</button></label>"
    )
}

fn pressed(comp: &ComponentNode) -> bool {
    if let Some(v) = comp.props.get("pressed").or_else(|| comp.props.get("on")) {
        return is_true(v);
    }
    if comp.items.iter().any(|i| {
        i.config.get("pressed").map(|s| is_true(s)).unwrap_or(false)
            || i.config.get("on").map(|s| is_true(s)).unwrap_or(false)
    }) {
        return true;
    }
    comp.style
        .as_deref()
        .unwrap_or("")
        .split('+')
        .any(|part| part.trim() == "on")
}

fn is_true(raw: &str) -> bool {
    matches!(raw, "true" | "on" | "1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};

    fn toggle(label: &str) -> ComponentNode {
        reset_instance_ids();
        stub("toggle", label)
    }

    #[test]
    fn off_is_button_after_unchecked_checkbox() {
        let html = render(&toggle("Bold"));
        assert!(html.starts_with("<label><input type=\"checkbox\" id=\"cui-toggle-toggle\" aria-label=\"Bold\"><button type=\"button\" data-slot=\"toggle\" data-state=\"off\" aria-pressed=\"false\" tabindex=\"-1\" aria-hidden=\"true\">Bold</button></label>"));
        assert!(!html.contains("data-variant"));
        assert!(!html.contains("data-size"));
        assert!(!html.contains("toggle-control"));
    }

    #[test]
    fn pressed_checks_the_input() {
        let mut c = toggle("Bold");
        c.props.insert("pressed".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" checked>"));
        assert!(html.contains("data-state=\"on\" aria-pressed=\"true\""));
    }

    #[test]
    fn style_segments_become_classes_and_icon_glyph() {
        let mut c = toggle("Italic");
        c.style = Some("toggle+outline+sm".into());
        c.props.insert("icon".into(), "italic".into());
        let html = render(&c);
        assert!(html.contains("class=\"v-outline s-sm\""));
        assert!(html.contains("data-icon=\"italic\""));
        assert!(html.contains("</svg>Italic</button>"));
        c.props.insert("icon-only".into(), "true".into());
        c.props.insert("aria-label".into(), "Bold (small)".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Bold (small)\""));
        assert!(html.contains("</svg></button>"));
    }

    #[test]
    fn disabled_dims_button_and_disables_input() {
        let mut c = toggle("Bold");
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" disabled><button"));
        assert!(html.contains("data-disabled=\"\""));
    }

    #[test]
    fn registered_as_dedicated() {
        let c = toggle("Bold");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), {
            reset_instance_ids();
            render(&c)
        });
        assert_eq!(
            dedicated_fn_name("toggle"),
            Some("cronus_ui_toggle::render")
        );
        assert_eq!(
            renderer_kind("toggle"),
            RendererKind::Dedicated("cronus_ui_toggle::render")
        );
    }
}
