//! Dedicated Plan renderer (AI suite). DOM matches React `Plan`: a Card
//! (`<div data-slot="card">`) around the Collapsible root
//! `<div data-state data-slot="plan">` > `<div data-slot="plan-header">` (the
//! CardHeader as a flex row: `<div data-slot="plan-title">` from the `title`
//! item, `<div data-slot="plan-description">` from `description:` — the two
//! share a slotless `<div>` — and the `<button data-slot="plan-trigger"
//! data-variant="ghost" aria-label="Toggle plan">` chevrons) >
//! `<div data-slot="collapsible-content">` > `<div data-slot="plan-content">`
//! with one `<p>` per `text` line. `streaming:true` renders the title and
//! description as `text-shimmer` like React's `isStreaming`.
//!
//! Zero JS: the trigger button is decorative inside a `<label>` after a
//! visually hidden checkbox that carries the open state (`open:true`, React's
//! `defaultOpen`; closed by default); CSS hides the content until `:checked`.
//! `toggle:"…"` renames the trigger's accessible name.

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, instance_id, item, truthy};
use crate::parser::ComponentNode;

const CHEVRONS: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"chevrons-up-down\"><path d=\"m7 15 5 5 5-5\"></path><path d=\"m7 9 5-5 5 5\"></path></svg>";
const TOGGLE: &str = "Toggle plan";
/// React `TextShimmer` default `spread` (px per UTF-16 unit).
const SPREAD_PER_CHAR: usize = 2;

fn shimmer(text: &str, streaming: bool) -> String {
    if streaming {
        format!(
            "<p data-slot=\"text-shimmer\" data-spread=\"{}\">{}</p>",
            text.encode_utf16().count() * SPREAD_PER_CHAR,
            esc(text)
        )
    } else {
        esc(text)
    }
}

pub fn render(comp: &ComponentNode) -> String {
    let open = ["open", "defaultOpen", "default-open"]
        .iter()
        .find_map(|k| attr(comp, k))
        .is_some_and(truthy);
    let state = if open { "open" } else { "closed" };
    let checked = if open { " checked" } else { "" };
    let streaming = attr(comp, "streaming")
        .or_else(|| attr(comp, "isStreaming"))
        .is_some_and(truthy);
    let title = item(comp, "title")
        .or_else(|| item(comp, "label"))
        .filter(|t| !t.is_empty())
        .unwrap_or(&comp.name);
    let title_html = format!(
        "<div data-slot=\"plan-title\">{}</div>",
        shimmer(title, streaming)
    );
    let heading = match attr_nonempty(comp, "description") {
        Some(d) => format!(
            "<div>{title_html}<div data-slot=\"plan-description\">{}</div></div>",
            shimmer(d, streaming)
        ),
        None => title_html,
    };
    let toggle = attr_nonempty(comp, "toggle")
        .map(esc)
        .unwrap_or_else(|| TOGGLE.into());
    let id = instance_id(comp, "plan");
    let body: String = comp
        .items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "title" | "label") && !i.text.is_empty())
        .map(|i| format!("<p>{}</p>", esc(&i.text)))
        .collect();
    format!(
        "<div data-slot=\"card\"><div data-state=\"{state}\" data-slot=\"plan\"><div data-slot=\"plan-header\">{heading}<label><input type=\"checkbox\" id=\"{id}\" aria-label=\"{toggle}\" aria-controls=\"{id}-content\"{checked}><button type=\"button\" data-slot=\"plan-trigger\" data-variant=\"ghost\" data-state=\"{state}\" aria-expanded=\"{open}\" aria-label=\"{toggle}\" tabindex=\"-1\" aria-hidden=\"true\">{CHEVRONS}</button></label></div><div id=\"{id}-content\" data-state=\"{state}\" data-slot=\"collapsible-content\"><div data-slot=\"plan-content\">{body}</div></div></div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use crate::parser::ComponentItemNode;

    fn text(t: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: t.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn docs() -> ComponentNode {
        let mut c = stub("plan", "Ship the chat surface");
        c.items[0].item_type = "title".into();
        c.props.insert("open".into(), "true".into());
        c.items
            .push(text("Wire Conversation, Message, and PromptInput."));
        c
    }

    #[test]
    fn docs_example_is_card_header_trigger_and_content() {
        reset_instance_ids();
        assert_eq!(
            render(&docs()),
            format!("<div data-slot=\"card\"><div data-state=\"open\" data-slot=\"plan\"><div data-slot=\"plan-header\"><div data-slot=\"plan-title\">Ship the chat surface</div><label><input type=\"checkbox\" id=\"cui-plan-plan\" aria-label=\"Toggle plan\" aria-controls=\"cui-plan-plan-content\" checked><button type=\"button\" data-slot=\"plan-trigger\" data-variant=\"ghost\" data-state=\"open\" aria-expanded=\"true\" aria-label=\"Toggle plan\" tabindex=\"-1\" aria-hidden=\"true\">{CHEVRONS}</button></label></div><div id=\"cui-plan-plan-content\" data-state=\"open\" data-slot=\"collapsible-content\"><div data-slot=\"plan-content\"><p>Wire Conversation, Message, and PromptInput.</p></div></div></div></div>")
        );
    }

    #[test]
    fn closed_streaming_description_and_escaping() {
        reset_instance_ids();
        let mut c = stub("plan", "<b>\"Plan\"</b>");
        c.props.insert("streaming".into(), "true".into());
        c.props
            .insert("description".into(), "Soon <i>now</i>".into());
        c.props.insert("toggle".into(), "Expand".into());
        let html = render(&c);
        assert!(html.contains("<div data-state=\"closed\" data-slot=\"plan\">"));
        assert!(html.contains("<div data-slot=\"plan-header\"><div><div data-slot=\"plan-title\"><p data-slot=\"text-shimmer\" data-spread=\"26\">&lt;b&gt;&quot;Plan&quot;&lt;/b&gt;</p></div><div data-slot=\"plan-description\"><p data-slot=\"text-shimmer\" data-spread=\"30\">Soon &lt;i&gt;now&lt;/i&gt;</p></div></div><label><input type=\"checkbox\" id=\"cui-plan-plan\" aria-label=\"Expand\" aria-controls=\"cui-plan-plan-content\"><button"));
        assert!(html.contains("aria-expanded=\"false\""));
        assert!(html.contains("<div data-slot=\"plan-content\"></div>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<details"));
    }

    #[test]
    fn chrome_hides_content_until_checked() {
        let css = include_str!("cronus_ui_css/plan.css");
        assert!(css.contains("[data-slot=\"plan\"]:not(:has(> [data-slot=\"plan-header\"] > label > input:checked)) > [data-slot=\"collapsible-content\"]"));
        assert!(
            css.contains("[data-slot=\"card\"]:has(> [data-slot=\"plan\"]) { box-shadow: none; }")
        );
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(dedicated_fn_name("plan"), Some("cronus_ui_plan::render"));
        assert_eq!(
            renderer_kind("plan"),
            RendererKind::Dedicated("cronus_ui_plan::render")
        );
        reset_instance_ids();
        let c = stub("plan", "Plan");
        let a = crate::cronus_ui_widgets::render(&c).unwrap();
        reset_instance_ids();
        assert_eq!(a, render(&c));
    }
}
