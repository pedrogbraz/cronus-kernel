//! Dedicated GoalCard renderer. DOM matches React:
//! `<article data-slot="goal-card" data-status aria-labelledby>` holding the
//! title row (`h3` + optional ghost `icon-sm` delete button), an optional
//! description `<p>`, the progress row (`[data-slot="progress"]` + `NN%`) and
//! the meta row: status chip (`goalStatusVariants`, a plain `<span>` in React,
//! so the tone rides on the article's `data-status`), a secondary
//! `[data-slot="badge"]` with the lucide `check-circle-2` glyph and
//! `done/total steps`, the calendar date and the `View goal` button.
//!
//! Inputs: `title "…"` (or `label`), `description:"…"`, `progress:75`
//! (derived from the steps when absent), `status:not_started|in_progress|
//! completed|at_risk` (prop or style segment `goal-card+in_progress`; also
//! derived from progress like React), `due:"2025-12-08"` (ISO dates print as
//! `Dec 8, 2025`, anything else prints as written), `item "Design"
//! completed:true` steps, `action "View goal" -> "/goals/mvp"` (an anchor; a
//! plain `action "View goal"` is the JS-only button, rendered disabled) and
//! `deletable:true` for the hover-revealed delete button (JS-only, disabled).

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, item, label_of, safe_url, truthy};
use crate::parser::ComponentNode;

const STATUSES: &[&str] = &["not_started", "in_progress", "completed", "at_risk"];

pub fn render(comp: &ComponentNode) -> String {
    let title = item(comp, "title")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| label_of(comp));
    let steps: Vec<(String, bool)> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .map(|i| {
            let done = i
                .config
                .get("completed")
                .or_else(|| i.config.get("done"))
                .map(|v| truthy(v))
                .unwrap_or(false);
            (esc(&i.text), done)
        })
        .collect();
    let total = steps.len();
    let done = steps.iter().filter(|(_, d)| *d).count();
    let progress = attr_nonempty(comp, "progress")
        .and_then(|p| p.trim().trim_end_matches('%').parse::<f64>().ok())
        .map(|p| p.clamp(0.0, 100.0).round() as u32)
        .unwrap_or_else(|| {
            if total > 0 {
                ((done as f64 / total as f64) * 100.0).round() as u32
            } else {
                0
            }
        });
    let status = status_of(comp).unwrap_or(match progress {
        0 => "not_started",
        100 => "completed",
        _ => "in_progress",
    });
    let status_label = match status {
        "in_progress" => "In progress",
        "completed" => "Completed",
        "at_risk" => "At risk",
        _ => "Not started",
    };
    let id = crate::cronus_ui_kit::instance_id(comp, "goal-card-title");
    let delete = if flag(comp, "deletable") {
        format!(
            "<button type=\"button\" data-slot=\"button\" data-variant=\"ghost\" data-size=\"icon-sm\" aria-label=\"Delete goal\" disabled>{}</button>",
            crate::cronus_ui_icons::svg_or_empty("more-vertical")
        )
    } else {
        String::new()
    };
    let description = attr_nonempty(comp, "description")
        .map(|d| format!("<p>{}</p>", esc(d)))
        .unwrap_or_default();
    let bar_state = if progress >= 100 {
        "complete"
    } else {
        "loading"
    };
    let progress_html = format!(
        "<div data-slot=\"progress\" role=\"progressbar\" aria-valuenow=\"{progress}\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-valuetext=\"{progress}%\" data-state=\"{bar_state}\" data-value=\"{progress}\" data-max=\"100\" aria-label=\"Goal progress\"><div data-state=\"{bar_state}\" data-value=\"{progress}\" data-max=\"100\"></div></div>"
    );
    let steps_html = if total > 0 {
        format!(
            "<span data-slot=\"badge\" data-variant=\"secondary\">{}{done}/{total} steps</span>",
            crate::cronus_ui_icons::svg_or_empty("check-circle-2")
        )
    } else {
        String::new()
    };
    let date_html = attr_nonempty(comp, "due")
        .or_else(|| attr_nonempty(comp, "due-date"))
        .or_else(|| attr_nonempty(comp, "created"))
        .map(|d| {
            format!(
                "<span>{}{}</span>",
                crate::cronus_ui_icons::svg_or_empty("calendar"),
                esc(&format_date(d))
            )
        })
        .unwrap_or_default();
    let view_html = comp
        .items
        .iter()
        .find(|i| i.item_type == "action" && !i.text.is_empty())
        .map(|a| {
            let text = esc(&a.text);
            let glyph = crate::cronus_ui_icons::svg_or_empty("arrow-right");
            match a.link.as_deref().filter(|l| !l.trim().is_empty()) {
                Some(href) => format!(
                    "<a href=\"{}\" data-slot=\"button\" data-variant=\"primary\" data-size=\"sm\">{text}{glyph}</a>",
                    safe_url(href)
                ),
                None => format!(
                    "<button type=\"button\" data-slot=\"button\" data-variant=\"primary\" data-size=\"sm\" disabled>{text}{glyph}</button>"
                ),
            }
        })
        .unwrap_or_default();
    format!(
        "<article data-slot=\"goal-card\" data-status=\"{status}\" aria-labelledby=\"{id}\"><div><h3 id=\"{id}\">{title}</h3>{delete}</div>{description}<div>{progress_html}<span>{progress}%</span></div><div><span class=\"goal-status\">{status_label}</span>{steps_html}{date_html}{view_html}</div></article>"
    )
}

fn status_of(comp: &ComponentNode) -> Option<&'static str> {
    let raw = attr_nonempty(comp, "status")
        .map(|s| s.trim().replace('-', "_"))
        .or_else(|| {
            comp.style
                .as_deref()
                .unwrap_or("")
                .split('+')
                .skip(1)
                .map(|s| s.trim().replace('-', "_"))
                .find(|s| STATUSES.contains(&s.as_str()))
        })?;
    STATUSES.iter().copied().find(|s| *s == raw)
}

/// `YYYY-MM-DD` → `Mon D, YYYY` (React's `en-US` short-month format, UTC);
/// any other value is printed as written.
fn format_date(raw: &str) -> String {
    let t = raw.trim();
    let parts: Vec<&str> = t.split('-').collect();
    if parts.len() == 3 && parts[0].len() == 4 {
        if let (Ok(y), Ok(m), Ok(d)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>(),
        ) {
            const MONTHS: [&str; 12] = [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ];
            if (1..=12).contains(&m) && (1..=31).contains(&d) {
                return format!("{} {d}, {y}", MONTHS[(m - 1) as usize]);
            }
        }
    }
    t.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use crate::parser::ComponentItemNode;
    use std::collections::HashMap;

    fn goal(title: &str) -> ComponentNode {
        reset_instance_ids();
        let mut c = stub("goal-card", title);
        c.items[0].item_type = "title".into();
        c
    }

    fn step(c: &mut ComponentNode, text: &str, done: bool) {
        let mut config = HashMap::new();
        if done {
            config.insert("completed".into(), "true".into());
        }
        c.items.push(ComponentItemNode {
            item_type: "item".into(),
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
    fn docs_progress_example_dom() {
        let mut c = goal("Launch MVP by end of quarter");
        c.style = Some("goal-card+in_progress".into());
        c.props.insert("progress".into(), "75".into());
        c.props.insert("due".into(), "2025-12-08".into());
        step(&mut c, "Design", true);
        step(&mut c, "Develop", true);
        step(&mut c, "Ship", false);
        step(&mut c, "Announce", false);
        let html = render(&c);
        assert!(html.starts_with("<article data-slot=\"goal-card\" data-status=\"in_progress\" aria-labelledby=\"cui-goal-card-goal-card-title\"><div><h3 id=\"cui-goal-card-goal-card-title\">Launch MVP by end of quarter</h3></div><div><div data-slot=\"progress\" role=\"progressbar\" aria-valuenow=\"75\""));
        assert!(html.contains("data-value=\"75\" data-max=\"100\" aria-label=\"Goal progress\"><div data-state=\"loading\" data-value=\"75\" data-max=\"100\"></div></div><span>75%</span></div>"));
        assert!(html.contains("<span class=\"goal-status\">In progress</span><span data-slot=\"badge\" data-variant=\"secondary\"><svg"));
        assert!(html.contains("data-icon=\"check-circle-2\""));
        assert!(html.contains("</svg>2/4 steps</span><span><svg"));
        assert!(html.contains("data-icon=\"calendar\""));
        assert!(html.contains("</svg>Dec 8, 2025</span></div></article>"));
        assert!(!html.contains("data-slot=\"button\""));
        reject_js(&html);
    }

    #[test]
    fn every_status_renders_its_chip() {
        for (status, label) in [
            ("not_started", "Not started"),
            ("in_progress", "In progress"),
            ("completed", "Completed"),
            ("at_risk", "At risk"),
        ] {
            let mut c = goal("Goal");
            c.props.insert("status".into(), status.into());
            let html = render(&c);
            assert!(
                html.contains(&format!("data-status=\"{status}\"")),
                "{status}"
            );
            assert!(html.contains(&format!("<span class=\"goal-status\">{label}</span>")));
            let mut c = goal("Goal");
            c.style = Some(format!("goal-card+{}", status.replace('_', "-")));
            assert!(render(&c).contains(&format!("data-status=\"{status}\"")));
        }
    }

    #[test]
    fn status_and_progress_derive_like_react() {
        let mut c = goal("Goal");
        step(&mut c, "One", true);
        step(&mut c, "Two", true);
        step(&mut c, "Three", false);
        let html = render(&c);
        assert!(html.contains("aria-valuenow=\"67\""));
        assert!(html.contains("data-status=\"in_progress\""));
        let mut c = goal("Goal");
        c.props.insert("progress".into(), "100".into());
        let html = render(&c);
        assert!(html.contains("data-status=\"completed\""));
        assert!(html.contains("data-state=\"complete\""));
        let html = render(&goal("Goal"));
        assert!(html.contains("data-status=\"not_started\""));
        assert!(html.contains("aria-valuenow=\"0\""));
        assert!(!html.contains("steps</span>"));
        let mut c = goal("Goal");
        c.props.insert("progress".into(), "140".into());
        assert!(render(&c).contains("aria-valuenow=\"100\""));
    }

    #[test]
    fn description_delete_and_view_controls() {
        let mut c = goal("Goal");
        c.props
            .insert("description".into(), "Ship the <b>thing</b>".into());
        c.props.insert("deletable".into(), "true".into());
        c.items.push(ComponentItemNode {
            item_type: "action".into(),
            text: "View goal".into(),
            link: Some("/goals/mvp".into()),
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&c);
        assert!(html.contains("</div><p>Ship the &lt;b&gt;thing&lt;/b&gt;</p><div>"));
        assert!(html.contains("<button type=\"button\" data-slot=\"button\" data-variant=\"ghost\" data-size=\"icon-sm\" aria-label=\"Delete goal\" disabled><svg"));
        assert!(html.contains("<a href=\"/goals/mvp\" data-slot=\"button\" data-variant=\"primary\" data-size=\"sm\">View goal<svg"));
        assert!(html.contains("data-icon=\"arrow-right\""));
        c.items.last_mut().unwrap().link = Some("javascript:alert(1)".into());
        assert!(render(&c).contains("href=\"#\""));
        c.items.last_mut().unwrap().link = None;
        assert!(render(&c).contains("data-size=\"sm\" disabled>View goal<svg"));
    }

    #[test]
    fn escapes_hostile_text_and_dates() {
        let mut c = goal("<b>x</b> \"q\"");
        c.props.insert("due".into(), "Q4 \"soon\"".into());
        let html = render(&c);
        assert!(html.contains(
            "<h3 id=\"cui-goal-card-goal-card-title\">&lt;b&gt;x&lt;/b&gt; &quot;q&quot;</h3>"
        ));
        assert!(html.contains("</svg>Q4 &quot;soon&quot;</span>"));
        reject_js(&html);
        assert_eq!(format_date("2026-01-31"), "Jan 31, 2026");
        assert_eq!(format_date("2026-13-01"), "2026-13-01");
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/goal-card.css");
        assert!(css.contains("[data-slot=\"goal-card\"] {"));
        assert!(css.contains(
            "border-radius: var(--cronus-radius-3xl, calc(var(--cronus-radius, 14px) + 14px))"
        ));
        assert!(css.contains("padding: 1.25rem"));
        assert!(
            css.contains("[data-slot=\"goal-card\"][data-status=\"at_risk\"] > div > .goal-status")
        );
        assert!(
            css.contains("[data-slot=\"goal-card\"] [data-slot=\"progress\"] { height: 0.625rem")
        );
        assert!(!css.contains("#"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        let c = goal("Goal");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), {
            reset_instance_ids();
            render(&c)
        });
        assert_eq!(
            dedicated_fn_name("goal-card"),
            Some("cronus_ui_goal_card::render")
        );
        assert_eq!(
            renderer_kind("goal-card"),
            RendererKind::Dedicated("cronus_ui_goal_card::render")
        );
    }
}
