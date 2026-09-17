//! Kit pages for `page type:components`.
//!
//! Two views, both driven only by what the `.cronus` files declare:
//!
//! - **Overview** (`page "/kit" type:components`): one card per family that
//!   has a family page, grouped like the React docs sidebar, showing the
//!   family's first specimen and linking to its page. Without family pages
//!   the legacy grouped catalog (`component::render_components_page`) renders.
//! - **Family page** (`page "/button" type:components family:button`): the
//!   docs layout of one family. Every `component` whose style family matches
//!   is a specimen; `group:"Variants"` on a component names the example it
//!   belongs to (declaration order, first appearance wins), `note:"…"` is that
//!   example's description. Each example shows its specimens in a preview
//!   frame followed by the `.cronus` source that declared them.
//!
//! No JS. Chrome CSS lives in `cronus_ui_css/catalog.css`.

use crate::cronus_ui_kit::esc;
use crate::parser::{ComponentNode, PageNode};

/// Component props that describe the page, not the widget.
const PAGE_PROPS: &[&str] = &["group", "note"];

/// Family page of a `type:components` page, when it names a `family:`.
pub fn family_of_page(page: &PageNode) -> Option<&str> {
    page.config
        .get("family")
        .map(String::as_str)
        .filter(|f| !f.is_empty())
}

/// Every `type:components` page that names a family, in declaration order.
fn family_pages(pages: &[PageNode]) -> Vec<&PageNode> {
    pages
        .iter()
        .filter(|p| p.page_type == "components" && family_of_page(p).is_some())
        .collect()
}

fn specimens<'a>(family: &str, comps: &'a [ComponentNode]) -> Vec<&'a ComponentNode> {
    comps
        .iter()
        .filter(|c| crate::cronus_ui_widgets::family_of(c) == Some(family))
        .collect()
}

fn page_title(page: &PageNode, family: &str) -> String {
    page.title
        .clone()
        .unwrap_or_else(|| super::component::kit_family_title(family))
}

fn page_lead(page: &PageNode) -> String {
    page.config
        .get("description")
        .map(|d| esc(d))
        .unwrap_or_default()
}

/// Kit group (sidebar category) of a family, from the docs order.
fn group_of(family: &str) -> Option<(&'static str, &'static str)> {
    super::component::KIT_GROUPS
        .iter()
        .find(|(_, _, families)| families.contains(&family))
        .map(|(slug, title, _)| (*slug, *title))
}

fn anchor(title: &str) -> String {
    let mut id = String::new();
    let mut dash = false;
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            if dash && !id.is_empty() {
                id.push('-');
            }
            id.push(c.to_ascii_lowercase());
            dash = false;
        } else {
            dash = true;
        }
    }
    if id.is_empty() {
        "example".into()
    } else {
        id
    }
}

struct Example<'a> {
    title: String,
    note: String,
    comps: Vec<&'a ComponentNode>,
}

fn examples<'a>(comps: &[&'a ComponentNode]) -> Vec<Example<'a>> {
    let mut out: Vec<Example<'a>> = Vec::new();
    for comp in comps {
        let title = comp
            .props
            .get("group")
            .cloned()
            .filter(|g| !g.trim().is_empty())
            .unwrap_or_else(|| "Default".into());
        let note = comp.props.get("note").cloned().unwrap_or_default();
        match out.iter_mut().find(|e| e.title == title) {
            Some(e) => {
                if e.note.is_empty() {
                    e.note = note;
                }
                e.comps.push(comp);
            }
            None => out.push(Example {
                title,
                note,
                comps: vec![comp],
            }),
        }
    }
    out
}

/// The `.cronus` declaration of a component, rebuilt from the AST (page-only
/// props dropped, props sorted, items in order).
pub fn cronus_source(comp: &ComponentNode) -> String {
    let mut head = format!("component {}", comp.name);
    if let Some(layout) = comp.layout.as_deref().filter(|l| !l.is_empty()) {
        head.push_str(&format!(" layout:{layout}"));
    }
    if let Some(style) = comp.style.as_deref().filter(|s| !s.is_empty()) {
        head.push_str(&format!(" style:{style}"));
    }
    let mut props: Vec<(&String, &String)> = comp
        .props
        .iter()
        .filter(|(k, _)| !PAGE_PROPS.contains(&k.as_str()))
        .collect();
    props.sort();
    for (k, v) in props {
        head.push_str(&format!(" {k}:{}", cronus_value(v)));
    }
    if comp.items.is_empty() {
        return format!("{head} {{}}");
    }
    let mut body = String::new();
    for item in &comp.items {
        body.push_str("\n  ");
        body.push_str(&item.item_type);
        if !item.text.is_empty() {
            body.push_str(&format!(" {}", cronus_string(&item.text)));
        }
        if let Some(link) = item.link.as_deref() {
            body.push_str(&format!(" -> {}", cronus_string(link)));
        }
        if let Some(tone) = item.tone.as_deref() {
            body.push_str(&format!(" tone:{tone}"));
        }
        let mut cfg: Vec<(&String, &String)> = item.config.iter().collect();
        cfg.sort();
        for (k, v) in cfg {
            body.push_str(&format!(" {k}:{}", cronus_value(v)));
        }
    }
    format!("{head} {{{body}\n}}")
}

fn cronus_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn cronus_value(v: &str) -> String {
    let bare = !v.is_empty()
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '#' | '%'));
    if bare {
        v.to_string()
    } else {
        cronus_string(v)
    }
}

fn render_example(family: &str, example: &Example<'_>) -> String {
    crate::cronus_ui_kit::reset_instance_ids();
    let id = anchor(&example.title);
    let widgets = example
        .comps
        .iter()
        .map(|c| super::component::render_component(c))
        .collect::<Vec<_>>()
        .join("\n");
    let source = example
        .comps
        .iter()
        .map(|c| esc(&cronus_source(c)))
        .collect::<Vec<_>>()
        .join("\n\n");
    let note = if example.note.is_empty() {
        String::new()
    } else {
        format!(
            "<p data-slot=\"catalog-example-note\">{}</p>",
            esc(&example.note)
        )
    };
    let wide = if super::component::kit_is_wide(family) {
        " data-wide=\"true\""
    } else {
        ""
    };
    format!(
        r#"<section id="{id}" data-slot="catalog-example" data-family="{family}">
  <h2>{title}</h2>
  {note}
  <div data-slot="catalog-frame" data-force-motion{wide}>{widgets}</div>
  <details data-slot="catalog-source">
    <summary><span>.cronus</span></summary>
    <pre><code>{source}</code></pre>
  </details>
</section>"#,
        family = esc(family),
        title = esc(&example.title),
    )
}

/// Docs page of one family.
pub fn render_family_page(page: &PageNode, pages: &[PageNode], comps: &[ComponentNode]) -> String {
    let family = family_of_page(page).unwrap_or("component");
    let title = esc(&page_title(page, family));
    let lead = page_lead(page);
    let specimens = specimens(family, comps);
    let examples = examples(&specimens);
    let group = group_of(family);
    let eyebrow = group.map(|(_, t)| t).unwrap_or("Components");
    let lead_html = if lead.is_empty() {
        String::new()
    } else {
        format!("<p data-slot=\"catalog-lead\">{lead}</p>")
    };
    let toc = examples
        .iter()
        .map(|e| format!("<a href=\"#{}\">{}</a>", anchor(&e.title), esc(&e.title)))
        .collect::<Vec<_>>()
        .join("");
    let body = examples
        .iter()
        .map(|e| render_example(family, e))
        .collect::<Vec<_>>()
        .join("\n");
    let (prev, next) = neighbours(page, pages);
    let pager = match (prev, next) {
        (None, None) => String::new(),
        (prev, next) => {
            let prev = prev
                .map(|p| {
                    format!(
                        "<a href=\"{}\" rel=\"prev\"><small>Previous</small>{}</a>",
                        esc(&p.route),
                        esc(&page_title(p, family_of_page(p).unwrap_or("")))
                    )
                })
                .unwrap_or_else(|| "<span></span>".into());
            let next = next
                .map(|p| {
                    format!(
                        "<a href=\"{}\" rel=\"next\"><small>Next</small>{}</a>",
                        esc(&p.route),
                        esc(&page_title(p, family_of_page(p).unwrap_or("")))
                    )
                })
                .unwrap_or_else(|| "<span></span>".into());
            format!("<nav data-slot=\"catalog-pager\" aria-label=\"Families\">{prev}{next}</nav>")
        }
    };
    let count = specimens.len();
    format!(
        r#"<div data-slot="catalog" data-view="family" data-family="{family}">
  <header data-slot="catalog-header">
    <nav data-slot="catalog-crumbs" aria-label="Breadcrumb"><a href="/kit">Kit</a><span aria-hidden="true">/</span><span>{eyebrow}</span></nav>
    <div data-slot="catalog-title-row">
      <h1>{title}</h1>
      <div data-slot="catalog-stats">
        <span><strong>{examples_n}</strong> examples</span>
        <span><strong>{count}</strong> specimens</span>
      </div>
    </div>
    {lead_html}
  </header>
  <nav data-slot="catalog-toc" aria-label="On this page">{toc}</nav>
{body}
{pager}
</div>"#,
        family = esc(family),
        eyebrow = esc(eyebrow),
        examples_n = examples.len(),
    )
}

fn neighbours<'a>(
    page: &PageNode,
    pages: &'a [PageNode],
) -> (Option<&'a PageNode>, Option<&'a PageNode>) {
    let ordered = ordered_family_pages(pages);
    let idx = ordered.iter().position(|p| p.route == page.route);
    match idx {
        Some(i) => (
            if i > 0 { Some(ordered[i - 1]) } else { None },
            ordered.get(i + 1).copied(),
        ),
        None => (None, None),
    }
}

/// Family pages in docs order: by kit group, then by declaration.
fn ordered_family_pages(pages: &[PageNode]) -> Vec<&PageNode> {
    let mut out: Vec<&PageNode> = Vec::new();
    let all = family_pages(pages);
    for (_, _, families) in super::component::KIT_GROUPS {
        for family in *families {
            if let Some(p) = all.iter().find(|p| family_of_page(p) == Some(family)) {
                out.push(p);
            }
        }
    }
    for p in all {
        if !out.iter().any(|o| o.route == p.route) {
            out.push(p);
        }
    }
    out
}

/// Overview of every family that has a page. `None` when no family pages
/// exist, so the caller keeps the legacy grouped catalog.
pub fn render_overview(pages: &[PageNode], comps: &[ComponentNode]) -> Option<String> {
    let all = family_pages(pages);
    if all.is_empty() {
        return None;
    }
    let mut sections = String::new();
    let mut nav = String::new();
    let mut family_count = 0;
    let mut used: Vec<&str> = Vec::new();
    let mut groups: Vec<(&str, &str, Vec<&PageNode>)> = super::component::KIT_GROUPS
        .iter()
        .map(|(slug, title, families)| {
            let members: Vec<&PageNode> = families
                .iter()
                .filter_map(|f| all.iter().find(|p| family_of_page(p) == Some(f)).copied())
                .collect();
            (*slug, *title, members)
        })
        .collect();
    for (_, _, members) in &groups {
        for p in members {
            used.push(family_of_page(p).unwrap_or(""));
        }
    }
    let rest: Vec<&PageNode> = all
        .iter()
        .filter(|p| !used.contains(&family_of_page(p).unwrap_or("")))
        .copied()
        .collect();
    if !rest.is_empty() {
        groups.push(("other", "Other", rest));
    }
    for (slug, title, members) in &groups {
        if members.is_empty() {
            continue;
        }
        nav.push_str(&format!("<a href=\"#{slug}\">{title}</a>"));
        let mut cards = String::new();
        for p in members {
            let family = family_of_page(p).unwrap_or("");
            family_count += 1;
            let specimen = specimens(family, comps)
                .first()
                .map(|c| {
                    crate::cronus_ui_kit::reset_instance_ids();
                    super::component::render_component(c)
                })
                .unwrap_or_default();
            let lead = page_lead(p);
            let lead_html = if lead.is_empty() {
                String::new()
            } else {
                format!("<p>{lead}</p>")
            };
            let wide = if super::component::kit_is_wide(family) {
                " data-wide=\"true\""
            } else {
                ""
            };
            cards.push_str(&format!(
                r#"<article data-slot="catalog-card" data-family="{family}"{wide}>
  <div data-slot="catalog-card-canvas" data-force-motion inert>{specimen}</div>
  <div data-slot="catalog-card-body"><h3><a href="{route}">{title}</a></h3>{lead_html}</div>
</article>
"#,
                route = esc(&p.route),
                family = esc(family),
                title = esc(&page_title(p, family)),
            ));
        }
        sections.push_str(&format!(
            r#"<section id="{slug}" data-slot="catalog-section" data-group="{slug}">
  <h2>{title}</h2>
  <div data-slot="catalog-cards">
{cards}  </div>
</section>
"#
        ));
    }
    let group_count = groups.iter().filter(|(_, _, m)| !m.is_empty()).count();
    Some(format!(
        r#"<div data-slot="catalog" data-view="overview">
  <header data-slot="catalog-header">
    <p data-slot="catalog-eyebrow">Language</p>
    <div data-slot="catalog-title-row">
      <h1>Kit</h1>
      <div data-slot="catalog-stats">
        <span><strong>{family_count}</strong> families</span>
        <span><strong>{group_count}</strong> groups</span>
      </div>
    </div>
    <p data-slot="catalog-lead">Every family declared in .cronus. The kernel emits the HTML, tokens and motion: the same components as the React catalog, without JSX. Open a family for all of its variants.</p>
    <a href="/" data-slot="catalog-home">Home</a>
  </header>
  <nav data-slot="catalog-toc" aria-label="Kit groups">{nav}</nav>
{sections}</div>"#
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ComponentItemNode, Span};
    use std::collections::HashMap;

    fn comp(name: &str, style: &str, group: &str, label: &str) -> ComponentNode {
        let mut props = HashMap::new();
        if !group.is_empty() {
            props.insert("group".to_string(), group.to_string());
        }
        ComponentNode {
            name: name.into(),
            layout: Some("inline".into()),
            style: Some(style.into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: label.into(),
                link: None,
                tone: None,
                config: Default::default(),
            }],
            props,
            params: vec![],
            template: None,
            sections: vec![],
            state: vec![],
            tests: vec![],
            binding: None,
            span: Span::default(),
        }
    }

    fn page(route: &str, family: &str, title: &str) -> PageNode {
        let mut config = HashMap::new();
        config.insert("family".to_string(), family.to_string());
        config.insert("description".to_string(), "Lead text.".to_string());
        PageNode {
            route: route.into(),
            page_type: "components".into(),
            entity: None,
            title: Some(title.into()),
            sections: vec![],
            config,
            components: vec![],
            requires: None,
            doc: None,
            span: Span::default(),
        }
    }

    #[test]
    fn source_round_trips_props_and_items() {
        let mut c = comp("Save", "button+primary+md", "Variants", "Save");
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("note".into(), "hidden".into());
        assert_eq!(
            cronus_source(&c),
            "component Save layout:inline style:button+primary+md disabled:true {\n  label \"Save\"\n}"
        );
    }

    #[test]
    fn source_quotes_values_with_spaces() {
        let mut c = comp("Hint", "tooltip", "", "Hint");
        c.props
            .insert("description".into(), "Kernel emits \"chrome\"".into());
        assert!(cronus_source(&c).contains("description:\"Kernel emits \\\"chrome\\\"\""));
    }

    #[test]
    fn family_page_groups_examples_in_declaration_order() {
        let comps = vec![
            comp("Primary", "button+primary", "Variants", "Primary"),
            comp("Small", "button+sm", "Sizes", "Small"),
            comp("Ghost", "button+ghost", "Variants", "Ghost"),
            comp("Tag", "badge", "Default", "New"),
        ];
        let pages = vec![
            page("/button", "button", "Button"),
            page("/badge", "badge", "Badge"),
        ];
        let html = render_family_page(&pages[0], &pages, &comps);
        let variants = html.find("id=\"variants\"").unwrap();
        let sizes = html.find("id=\"sizes\"").unwrap();
        assert!(variants < sizes);
        assert!(html.contains("<strong>2</strong> examples"));
        assert!(html.contains("<strong>3</strong> specimens"));
        assert!(!html.contains("data-slot=\"badge\""));
        assert!(html.contains("Lead text."));
        assert!(html.contains("rel=\"next\""));
        assert!(html.contains("component Primary layout:inline style:button+primary {"));
    }

    #[test]
    fn overview_lists_families_with_pages_only() {
        let comps = vec![
            comp("Primary", "button+primary", "", "Primary"),
            comp("Tag", "badge", "", "New"),
            comp("Face", "avatar", "", "AL"),
        ];
        let pages = vec![
            page("/badge", "badge", "Badge"),
            page("/button", "button", "Button"),
        ];
        let html = render_overview(&pages, &comps).unwrap();
        assert!(html.contains("<h3><a href=\"/button\">Button</a></h3>"));
        assert!(html.contains("href=\"/badge\""));
        assert!(!html.contains("data-family=\"avatar\""));
        assert!(html.find("id=\"buttons\"").unwrap() < html.find("id=\"data-display\"").unwrap());
        assert!(render_overview(&[], &comps).is_none());
    }

    #[test]
    fn anchors_are_slugs() {
        assert_eq!(anchor("With icons"), "with-icons");
        assert_eq!(anchor("As link (asChild)"), "as-link-aschild");
        assert_eq!(anchor("!!!"), "example");
    }
}
