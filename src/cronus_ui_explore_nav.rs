//! Dedicated ExploreNav renderer (Skiper 75 Apple product navbar). DOM matches
//! React: `<nav data-slot="explore-nav" aria-label>` > the 32px bar (family
//! `h2`, the Explore / close toggle, the collapsed Buy pill) and the expanded
//! body: a snap-scrolling product carousel of `[data-slot="explore-nav-product"]`
//! tiles (image, name, `Currently Viewing` or the badge) with the previous /
//! next arrows, the selected product's heading + price + big Buy, and the
//! Overview disclosure with its chip links.
//!
//! Zero JS, three native states:
//! - expanded: a visually hidden checkbox in the Explore `<label>` (click,
//!   Space, focus ring); `:has(:checked)` grows the bar from 55px to auto
//!   (`interpolate-size` where supported), morphs Explore into the 42px X,
//!   fades the title and collapsed Buy out and animates the body in.
//! - selected product: a radio per tile (`explore-nav-product` is the
//!   decorative button after it); `:has(:nth-child(k) > input:checked)` shows
//!   the k-th heading / price and swaps the badge for `Currently Viewing`.
//! - overview: a checkbox (open by default) shows the chips, themselves a
//!   radio group whose checked chip is inverted.
//! Previous / next arrows are `data-explore-nav="prev|next"` `type="button"`;
//! page runtime scrolls `[data-slot="explore-nav-strip"]`. The strip also
//! scrolls natively.
//!
//! Inputs: `title "iPhone 17 Pro"`, `item "iPhone 17" image:"…" badge:"New"
//! price:"From $1099" price-note:"or $45.79/mo." selected:true` products,
//! `link "Highlights"` chips, `buy:"#buy"` (an anchor; without it Buy is the
//! JS-only button, disabled), `expanded:true`, `overview:false`.

use crate::cronus_ui_kit::{esc, flag, item, label_of, safe_url, truthy};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let title = item(comp, "title")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| label_of(comp));
    let expanded = flag(comp, "expanded") || flag(comp, "default-expanded");
    let overview_open = comp
        .props
        .get("overview")
        .map(|v| truthy(v))
        .unwrap_or(true);
    let toggle_id = crate::cronus_ui_kit::instance_id(comp, "explore-nav-toggle");
    let radio_name = crate::cronus_ui_kit::instance_id(comp, "explore-nav-product");
    let link_name = crate::cronus_ui_kit::instance_id(comp, "explore-nav-link");
    let overview_id = crate::cronus_ui_kit::instance_id(comp, "explore-nav-overview");
    let buy_href = comp
        .props
        .get("buy")
        .or_else(|| comp.props.get("buy-href"))
        .filter(|h| !h.trim().is_empty())
        .map(|h| safe_url(h));
    let buy = |class: &str| match &buy_href {
        Some(href) => format!("<a href=\"{href}\" class=\"{class}\">Buy</a>"),
        None => format!("<button type=\"button\" class=\"{class}\" disabled>Buy</button>"),
    };
    let products: Vec<&crate::parser::ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .collect();
    let selected = products
        .iter()
        .position(|p| p.config.get("selected").is_some_and(|v| truthy(v)))
        .unwrap_or(0);
    let mut tiles = String::new();
    let mut names = String::new();
    let mut prices = String::new();
    for (n, p) in products.iter().enumerate() {
        let name = esc(&p.text);
        let img = p
            .config
            .get("image")
            .or_else(|| p.config.get("src"))
            .or_else(|| p.link.as_ref())
            .map(|s| format!("<img src=\"{}\" alt=\"\">", safe_url(s)))
            .unwrap_or_default();
        let badge = p
            .config
            .get("badge")
            .filter(|b| !b.is_empty())
            .map(|b| format!("<p class=\"badge\">{}</p>", esc(b)))
            .unwrap_or_default();
        let (checked, current) = if n == selected {
            (" checked", " aria-current=\"true\"")
        } else {
            ("", "")
        };
        tiles.push_str(&format!(
            "<label><input type=\"radio\" name=\"{radio_name}\" aria-label=\"{name}\"{checked}><button type=\"button\" data-slot=\"explore-nav-product\"{current} tabindex=\"-1\" aria-hidden=\"true\">{img}<p>{name}</p><p class=\"viewing\">Currently Viewing</p>{badge}</button></label>"
        ));
        names.push_str(&format!("<h2>{name}</h2>"));
        let price = p
            .config
            .get("price")
            .filter(|v| !v.is_empty())
            .map(|v| {
                let note = p
                    .config
                    .get("price-note")
                    .or_else(|| p.config.get("note"))
                    .filter(|v| !v.is_empty())
                    .map(|v| format!("<p>{}</p>", esc(v)))
                    .unwrap_or_default();
                format!("<p>{}</p>{note}", esc(v))
            })
            .unwrap_or_default();
        prices.push_str(&format!("<div>{price}</div>"));
    }
    if products.is_empty() {
        names.push_str(&format!("<h2>{title}</h2>"));
        prices.push_str("<div></div>");
    }
    let links: String = comp
        .items
        .iter()
        .filter(|i| (i.item_type == "link" || i.item_type == "tab") && !i.text.is_empty())
        .enumerate()
        .map(|(n, l)| {
            let checked = if n == 0 { " checked" } else { "" };
            format!(
                "<label><input type=\"radio\" name=\"{link_name}\"{checked}><span>{}</span></label>",
                esc(&l.text)
            )
        })
        .collect();
    let links_html = if links.is_empty() {
        String::new()
    } else {
        format!("<div class=\"links\">{links}</div>")
    };
    let expanded_attr = if expanded { " checked" } else { "" };
    let aria_expanded = if expanded { "true" } else { "false" };
    let overview_attr = if overview_open { " checked" } else { "" };
    let aria_overview = if overview_open { "true" } else { "false" };
    let plus = crate::cronus_ui_icons::svg_or_empty("plus");
    let left = crate::cronus_ui_icons::svg_or_empty("chevron-left");
    let right = crate::cronus_ui_icons::svg_or_empty("chevron-right");
    let down = crate::cronus_ui_icons::svg_or_empty("chevron-down");
    let buy_small = buy("buy");
    let buy_large = buy("buy");
    format!(
        "<nav data-slot=\"explore-nav\" aria-label=\"{title}\"><div class=\"bar\"><div><h2>{title}</h2></div><div><label><input type=\"checkbox\" id=\"{toggle_id}\" aria-label=\"Explore\"{expanded_attr}><button type=\"button\" aria-expanded=\"{aria_expanded}\" aria-label=\"Explore\" tabindex=\"-1\" aria-hidden=\"true\"><span class=\"explore\">Explore</span><span class=\"close\">{plus}</span></button></label><span class=\"buy-collapsed\">{buy_small}</span></div></div><div class=\"body\"><div class=\"carousel\"><div class=\"scroller\" data-slot=\"explore-nav-strip\">{tiles}</div><button type=\"button\" data-explore-nav=\"prev\" aria-label=\"Previous slide\">{left}</button><button type=\"button\" data-explore-nav=\"next\" aria-label=\"Next slide\">{right}</button></div><div class=\"selected\"><div class=\"names\">{names}</div><div class=\"buy-row\"><div class=\"prices\">{prices}</div>{buy_large}</div></div><div class=\"overview\"><label><input type=\"checkbox\" id=\"{overview_id}\" aria-label=\"Overview\"{overview_attr}><button type=\"button\" aria-expanded=\"{aria_overview}\" tabindex=\"-1\" aria-hidden=\"true\">Overview{down}</button></label>{links_html}</div></div></nav>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use crate::parser::ComponentItemNode;
    use std::collections::HashMap;

    fn nav() -> ComponentNode {
        reset_instance_ids();
        let mut c = stub("explore-nav", "iPhone 17 Pro");
        c.items[0].item_type = "title".into();
        c.props.insert("buy".into(), "#buy".into());
        c
    }

    fn product(c: &mut ComponentNode, name: &str, cfg: &[(&str, &str)]) {
        let mut config = HashMap::new();
        for (k, v) in cfg {
            config.insert((*k).into(), (*v).into());
        }
        c.items.push(ComponentItemNode {
            item_type: "item".into(),
            text: name.into(),
            link: None,
            tone: None,
            config,
        });
    }

    fn link(c: &mut ComponentNode, label: &str) {
        c.items.push(ComponentItemNode {
            item_type: "link".into(),
            text: label.into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
    }

    fn reject_js(html: &str) {
        for bad in [
            "<script", "style=", "onclick", "onmouse", "<canvas", "<details",
        ] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn docs_family_example_dom() {
        let mut c = nav();
        product(
            &mut c,
            "iPhone 17 Pro",
            &[
                (
                    "image",
                    "https://skiper-ui.com/images/oct25Coll/iphone17/1.png",
                ),
                ("price", "From $1099"),
                ("price-note", "or $45.79/mo. for 24 mo."),
            ],
        );
        product(
            &mut c,
            "iPhone 17",
            &[
                (
                    "image",
                    "https://skiper-ui.com/images/oct25Coll/iphone17/2.png",
                ),
                ("badge", "New"),
            ],
        );
        link(&mut c, "Highlights");
        link(&mut c, "Performance");
        let html = render(&c);
        assert!(html.starts_with("<nav data-slot=\"explore-nav\" aria-label=\"iPhone 17 Pro\"><div class=\"bar\"><div><h2>iPhone 17 Pro</h2></div><div><label><input type=\"checkbox\" id=\"cui-explore-nav-explore-nav-toggle\" aria-label=\"Explore\"><button type=\"button\" aria-expanded=\"false\" aria-label=\"Explore\" tabindex=\"-1\" aria-hidden=\"true\"><span class=\"explore\">Explore</span><span class=\"close\"><svg"));
        assert!(html.contains("</span></button></label><span class=\"buy-collapsed\"><a href=\"#buy\" class=\"buy\">Buy</a></span></div></div><div class=\"body\"><div class=\"carousel\"><div class=\"scroller\" data-slot=\"explore-nav-strip\"><label><input type=\"radio\" name=\"cui-explore-nav-explore-nav-product\" aria-label=\"iPhone 17 Pro\" checked><button type=\"button\" data-slot=\"explore-nav-product\" aria-current=\"true\" tabindex=\"-1\" aria-hidden=\"true\"><img src=\"https://skiper-ui.com/images/oct25Coll/iphone17/1.png\" alt=\"\"><p>iPhone 17 Pro</p><p class=\"viewing\">Currently Viewing</p></button></label><label><input type=\"radio\" name=\"cui-explore-nav-explore-nav-product\" aria-label=\"iPhone 17\"><button type=\"button\" data-slot=\"explore-nav-product\" tabindex=\"-1\" aria-hidden=\"true\"><img src=\"https://skiper-ui.com/images/oct25Coll/iphone17/2.png\" alt=\"\"><p>iPhone 17</p><p class=\"viewing\">Currently Viewing</p><p class=\"badge\">New</p></button></label></div><button type=\"button\" data-explore-nav=\"prev\" aria-label=\"Previous slide\"><svg"));
        assert!(html.contains("data-icon=\"chevron-left\""));
        assert!(html.contains("data-explore-nav=\"next\" aria-label=\"Next slide\""));
        assert!(!html.contains("Previous slide\" disabled"));
        assert!(!html.contains("Next slide\" disabled"));
        assert!(html.contains("<div class=\"selected\"><div class=\"names\"><h2>iPhone 17 Pro</h2><h2>iPhone 17</h2></div><div class=\"buy-row\"><div class=\"prices\"><div><p>From $1099</p><p>or $45.79/mo. for 24 mo.</p></div><div></div></div><a href=\"#buy\" class=\"buy\">Buy</a></div></div><div class=\"overview\"><label><input type=\"checkbox\" id=\"cui-explore-nav-explore-nav-overview\" aria-label=\"Overview\" checked><button type=\"button\" aria-expanded=\"true\" tabindex=\"-1\" aria-hidden=\"true\">Overview<svg"));
        assert!(html.ends_with("</svg></button></label><div class=\"links\"><label><input type=\"radio\" name=\"cui-explore-nav-explore-nav-link\" checked><span>Highlights</span></label><label><input type=\"radio\" name=\"cui-explore-nav-explore-nav-link\"><span>Performance</span></label></div></div></div></nav>"));
        reject_js(&html);
    }

    #[test]
    fn expanded_selected_and_overview_states() {
        let mut c = nav();
        c.props.insert("expanded".into(), "true".into());
        c.props.insert("overview".into(), "false".into());
        product(&mut c, "A", &[]);
        product(&mut c, "B", &[("selected", "true")]);
        let html = render(&c);
        assert!(html.contains(
            "aria-label=\"Explore\" checked><button type=\"button\" aria-expanded=\"true\""
        ));
        assert!(html.contains(
            "aria-label=\"A\"><button type=\"button\" data-slot=\"explore-nav-product\" tabindex"
        ));
        assert!(html.contains("aria-label=\"B\" checked><button type=\"button\" data-slot=\"explore-nav-product\" aria-current=\"true\""));
        assert!(html
            .contains("aria-label=\"Overview\"><button type=\"button\" aria-expanded=\"false\""));
        assert!(!html.contains("class=\"links\""));
    }

    #[test]
    fn buy_without_href_is_disabled_and_title_fallback() {
        let mut c = nav();
        c.props.remove("buy");
        let html = render(&c);
        assert_eq!(
            html.matches("<button type=\"button\" class=\"buy\" disabled>Buy</button>")
                .count(),
            2
        );
        assert!(html.contains("<div class=\"names\"><h2>iPhone 17 Pro</h2></div>"));
        c.props.insert("buy".into(), "javascript:alert(1)".into());
        assert!(render(&c).contains("<a href=\"#\" class=\"buy\">Buy</a>"));
    }

    #[test]
    fn escapes_hostile_input() {
        let mut c = nav();
        c.items[0].text = "<b>x</b> \"q\"".into();
        product(
            &mut c,
            "<i>p</i>",
            &[
                ("image", "javascript:alert(1)"),
                ("badge", "\"N\""),
                ("price", "<s>$1</s>"),
            ],
        );
        link(&mut c, "<u>l</u>");
        let html = render(&c);
        assert!(html.contains("aria-label=\"&lt;b&gt;x&lt;/b&gt; &quot;q&quot;\""));
        assert!(html.contains("<img src=\"#\" alt=\"\"><p>&lt;i&gt;p&lt;/i&gt;</p>"));
        assert!(html.contains("<p class=\"badge\">&quot;N&quot;</p>"));
        assert!(html.contains("<div><p>&lt;s&gt;$1&lt;/s&gt;</p></div>"));
        assert!(html.contains("<span>&lt;u&gt;l&lt;/u&gt;</span>"));
        reject_js(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/explore-nav.css");
        assert!(css.contains("[data-slot=\"explore-nav\"] {"));
        assert!(css.contains("max-width: 64rem"));
        assert!(css.contains("height: 55px"));
        assert!(css.contains("[data-slot=\"explore-nav\"]:has(> .bar input:checked)"));
        assert!(css.contains("--cui-explore-nav-buy"));
        assert!(css.contains(
            ".scroller > label:nth-child(12) > input:checked) .names > h2:nth-child(12)"
        ));
        assert!(css.contains("[data-slot=\"explore-nav-product\"] {"));
        assert!(css.contains("var(--cronus-spring-soft)"));
        assert!(css.contains("scroll-snap-type: x mandatory"));
    }

    #[test]
    fn registered_as_dedicated() {
        let c = nav();
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), {
            reset_instance_ids();
            render(&c)
        });
        assert_eq!(
            dedicated_fn_name("explore-nav"),
            Some("cronus_ui_explore_nav::render")
        );
        assert_eq!(
            renderer_kind("explore-nav"),
            RendererKind::Dedicated("cronus_ui_explore_nav::render")
        );
    }
}
