//! Dedicated TodoItem renderer. DOM matches React:
//! `<article data-slot="todo-item" data-completed data-selected data-priority
//! aria-labelledby>` = the `Checkbox` (same zero-JS shape as the checkbox
//! family: visually hidden `<input type="checkbox">` inside a `<label>` before
//! the decorative `<button data-slot="checkbox" role="checkbox">`) + a body
//! with the `h3` title, the optional description `<p>` and the meta row: due
//! date (lucide `calendar`, tone overdue / today / upcoming), project chip
//! (lucide `folder`), label chips, priority chip (`todoPriorityVariants`,
//! lucide `flag`) and a secondary `[data-slot="badge"]` with lucide
//! `square-check` and `done/total subtasks`.
//!
//! Toggling the checkbox is the one interaction React has (`onToggleComplete`
//! flips `completed`); CSS mirrors it with `:has(input:checked)`, so the row
//! dims and strikes through without JS.
//!
//! Inputs: `title "…"` (or `label`), `text "…"` description, `priority:high|
//! medium|low|none` (prop or style segment `todo-item+high`), `completed:true`,
//! `selected:true`, `due:"today"` (React prints a relative date — `today`,
//! `tomorrow`, `yesterday`, `in 3 days`, `2 days ago`; the tone is inferred
//! from the words, or forced with `due-tone:overdue|today|upcoming`),
//! `project:"Cronus UI"`, `badge "Documentation"` label chips and `item
//! "Outline" completed:true` subtasks. The `flag` and `square-check` glyphs are
//! not in the vendored lucide set, so their paths are inlined here.

use crate::cronus_ui_kit::{attr_nonempty, choice, esc, flag, item, label_of, truthy};
use crate::parser::ComponentNode;

const PRIORITIES: &[&str] = &["high", "medium", "low", "none"];

const FLAG_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"flag\"><path d=\"M4 22V4a1 1 0 0 1 .4-.8A6 6 0 0 1 8 2c3 0 5 2 7.333 2q2 0 3.067-.8A1 1 0 0 1 20 4v10a1 1 0 0 1-.4.8A6 6 0 0 1 16 16c-3 0-5-2-8-2a6 6 0 0 0-4 1.528\"/></svg>";
const SQUARE_CHECK_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"square-check\"><rect width=\"18\" height=\"18\" x=\"3\" y=\"3\" rx=\"2\"/><path d=\"m9 12 2 2 4-4\"/></svg>";
const CHECK_INDICATOR: &str = "<span data-state=\"checked\"><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M20 6 9 17l-5-5\"></path></svg></span>";

pub fn render(comp: &ComponentNode) -> String {
    let title = item(comp, "title")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| label_of(comp));
    // Component-level only: `item "Outline" completed:true` is a subtask flag.
    let completed = comp.props.get("completed").is_some_and(|v| truthy(v));
    let selected = flag(comp, "selected");
    let priority = choice(comp, "priority", PRIORITIES).unwrap_or("none");
    let id = crate::cronus_ui_kit::instance_id(comp, "todo-item-title");
    let box_id = crate::cronus_ui_kit::instance_id(comp, "todo-item-check");
    let (state, aria, checked) = if completed {
        ("checked", "true", " checked")
    } else {
        ("unchecked", "false", "")
    };
    let checkbox = format!(
        "<label data-control=\"checkbox\"><input type=\"checkbox\" id=\"{box_id}\" aria-label=\"Mark complete\"{checked}><button type=\"button\" role=\"checkbox\" aria-checked=\"{aria}\" data-state=\"{state}\" value=\"on\" data-slot=\"checkbox\" tabindex=\"-1\" aria-hidden=\"true\">{CHECK_INDICATOR}</button></label>"
    );
    let description = item(comp, "text")
        .filter(|t| !t.is_empty())
        .map(|t| format!("<p>{}</p>", esc(t)))
        .unwrap_or_default();
    let mut meta = String::new();
    if let Some(due) = attr_nonempty(comp, "due").or_else(|| attr_nonempty(comp, "due-date")) {
        let tone = attr_nonempty(comp, "due-tone")
            .map(str::trim)
            .filter(|t| matches!(*t, "overdue" | "today" | "upcoming"))
            .unwrap_or_else(|| due_tone(due));
        meta.push_str(&format!(
            "<span data-tone=\"{tone}\">{}{}</span>",
            crate::cronus_ui_icons::svg_or_empty("calendar"),
            esc(due.trim())
        ));
    }
    if let Some(project) = attr_nonempty(comp, "project") {
        meta.push_str(&format!(
            "<span class=\"todo-chip\">{}{}</span>",
            crate::cronus_ui_icons::svg_or_empty("folder"),
            esc(project)
        ));
    }
    for label in comp
        .items
        .iter()
        .filter(|i| i.item_type == "badge" && !i.text.is_empty())
    {
        meta.push_str(&format!(
            "<span class=\"todo-chip\">{}</span>",
            esc(&label.text)
        ));
    }
    if priority != "none" {
        meta.push_str(&format!(
            "<span class=\"todo-priority\">{FLAG_ICON}{priority}</span>"
        ));
    }
    let subtasks: Vec<bool> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .map(|i| {
            i.config
                .get("completed")
                .or_else(|| i.config.get("done"))
                .map(|v| truthy(v))
                .unwrap_or(false)
        })
        .collect();
    if !subtasks.is_empty() {
        let done = subtasks.iter().filter(|d| **d).count();
        meta.push_str(&format!(
            "<span data-slot=\"badge\" data-variant=\"secondary\">{SQUARE_CHECK_ICON}{done}/{} subtasks</span>",
            subtasks.len()
        ));
    }
    let meta = if meta.is_empty() {
        String::new()
    } else {
        format!("<div>{meta}</div>")
    };
    format!(
        "<article data-slot=\"todo-item\" data-completed=\"{completed}\" data-selected=\"{selected}\" data-priority=\"{priority}\" aria-labelledby=\"{id}\">{checkbox}<div><h3 id=\"{id}\">{title}</h3>{description}{meta}</div></article>"
    )
}

/// React derives the tone from the due date against `now`; the kernel reads
/// the relative words the docs print (`yesterday` / `… ago` → overdue,
/// `today` / `now` → today, everything else → upcoming).
fn due_tone(due: &str) -> &'static str {
    let d = due.trim().to_ascii_lowercase();
    if d.contains("ago")
        || d.contains("yesterday")
        || d.contains("overdue")
        || d.starts_with("last ")
    {
        "overdue"
    } else if d == "today" || d == "now" || d.contains("hour") || d.contains("minute") {
        "today"
    } else {
        "upcoming"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use crate::parser::ComponentItemNode;
    use std::collections::HashMap;

    fn todo(title: &str) -> ComponentNode {
        reset_instance_ids();
        let mut c = stub("todo-item", title);
        c.items[0].item_type = "title".into();
        c
    }

    fn push(c: &mut ComponentNode, kind: &str, text: &str, done: Option<bool>) {
        let mut config = HashMap::new();
        if let Some(d) = done {
            config.insert("completed".into(), d.to_string());
        }
        c.items.push(ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config,
        });
    }

    fn reject_js(html: &str) {
        for bad in ["<script", "style=", "onclick", "onmouse", "<canvas"] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn docs_interactive_example_dom() {
        let mut c = todo("Complete project documentation");
        c.style = Some("todo-item+high".into());
        c.props.insert("due".into(), "today".into());
        c.props.insert("project".into(), "Cronus UI".into());
        push(
            &mut c,
            "text",
            "Write comprehensive docs for the new API",
            None,
        );
        push(&mut c, "badge", "Documentation", None);
        push(&mut c, "badge", "API", None);
        push(&mut c, "item", "Outline", Some(true));
        push(&mut c, "item", "Draft", Some(false));
        let html = render(&c);
        assert!(html.starts_with("<article data-slot=\"todo-item\" data-completed=\"false\" data-selected=\"false\" data-priority=\"high\" aria-labelledby=\"cui-todo-item-todo-item-title\"><label data-control=\"checkbox\"><input type=\"checkbox\" id=\"cui-todo-item-todo-item-check\" aria-label=\"Mark complete\"><button type=\"button\" role=\"checkbox\" aria-checked=\"false\" data-state=\"unchecked\" value=\"on\" data-slot=\"checkbox\" tabindex=\"-1\" aria-hidden=\"true\"><span data-state=\"checked\"><svg"));
        assert!(html.contains("</label><div><h3 id=\"cui-todo-item-todo-item-title\">Complete project documentation</h3><p>Write comprehensive docs for the new API</p><div><span data-tone=\"today\"><svg"));
        assert!(html.contains("</svg>today</span><span class=\"todo-chip\"><svg"));
        assert!(html.contains("data-icon=\"folder\""));
        assert!(html.contains("</svg>Cronus UI</span><span class=\"todo-chip\">Documentation</span><span class=\"todo-chip\">API</span><span class=\"todo-priority\"><svg"));
        assert!(html.contains("data-icon=\"flag\""));
        assert!(html.contains(
            "</svg>high</span><span data-slot=\"badge\" data-variant=\"secondary\"><svg"
        ));
        assert!(html.contains("data-icon=\"square-check\""));
        assert!(html.ends_with("</svg>1/2 subtasks</span></div></div></article>"));
        reject_js(&html);
    }

    #[test]
    fn every_priority_renders_and_none_hides_the_chip() {
        for p in ["high", "medium", "low"] {
            let mut c = todo("Task");
            c.props.insert("priority".into(), p.into());
            let html = render(&c);
            assert!(html.contains(&format!("data-priority=\"{p}\"")));
            assert!(html.contains(&format!("</svg>{p}</span>")));
        }
        let mut c = todo("Task");
        c.props.insert("priority".into(), "none".into());
        let html = render(&c);
        assert!(html.contains("data-priority=\"none\""));
        assert!(!html.contains("todo-priority"));
        assert!(
            html.ends_with("<h3 id=\"cui-todo-item-todo-item-title\">Task</h3></div></article>")
        );
        let html = render(&todo("Task"));
        assert!(html.contains("data-priority=\"none\""));
    }

    #[test]
    fn completed_and_selected_variants() {
        let mut c = todo("Task");
        c.props.insert("completed".into(), "true".into());
        c.props.insert("selected".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("data-completed=\"true\" data-selected=\"true\""));
        assert!(html.contains("aria-label=\"Mark complete\" checked><button type=\"button\" role=\"checkbox\" aria-checked=\"true\" data-state=\"checked\""));
    }

    #[test]
    fn due_tone_follows_the_words_or_the_override() {
        for (due, tone) in [
            ("today", "today"),
            ("in 3 hours", "today"),
            ("tomorrow", "upcoming"),
            ("in 2 days", "upcoming"),
            ("yesterday", "overdue"),
            ("3 days ago", "overdue"),
        ] {
            let mut c = todo("Task");
            c.props.insert("due".into(), due.into());
            assert!(
                render(&c).contains(&format!("<span data-tone=\"{tone}\">")),
                "{due}"
            );
        }
        let mut c = todo("Task");
        c.props.insert("due".into(), "Dec 8".into());
        c.props.insert("due-tone".into(), "overdue".into());
        assert!(render(&c).contains("<span data-tone=\"overdue\">"));
    }

    #[test]
    fn escapes_hostile_input() {
        let mut c = todo("<b>x</b> \"q\"");
        c.props.insert("project".into(), "<i>p</i>".into());
        push(&mut c, "badge", "\"L\"", None);
        push(&mut c, "text", "<script>1</script>", None);
        let html = render(&c);
        assert!(html.contains(
            "<h3 id=\"cui-todo-item-todo-item-title\">&lt;b&gt;x&lt;/b&gt; &quot;q&quot;</h3>"
        ));
        assert!(html.contains("<p>&lt;script&gt;1&lt;/script&gt;</p>"));
        assert!(html.contains(
            "</svg>&lt;i&gt;p&lt;/i&gt;</span><span class=\"todo-chip\">&quot;L&quot;</span>"
        ));
        reject_js(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/todo-item.css");
        assert!(css.contains("[data-slot=\"todo-item\"] {"));
        assert!(css.contains("padding: 1rem"));
        assert!(css.contains("[data-slot=\"todo-item\"][data-selected=\"true\"]"));
        assert!(css.contains("[data-slot=\"todo-item\"]:has(> label > input:checked)"));
        assert!(css.contains("[data-slot=\"todo-item\"][data-priority=\"high\"] .todo-priority"));
        assert!(css.contains("text-decoration-line: line-through"));
        assert!(!css.contains("#"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        let c = todo("Task");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), {
            reset_instance_ids();
            render(&c)
        });
        assert_eq!(
            dedicated_fn_name("todo-item"),
            Some("cronus_ui_todo_item::render")
        );
        assert_eq!(
            renderer_kind("todo-item"),
            RendererKind::Dedicated("cronus_ui_todo_item::render")
        );
    }
}
