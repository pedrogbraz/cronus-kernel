use crate::parser::SectionNode;

/// Renders a breadcrumb navigation from a SectionNode.
///
/// Each item becomes a crumb. Items with a "link" value render as anchors;
/// the last item renders as the current-page label (no link, bold).
pub fn render_breadcrumb(section: &SectionNode) -> String {
    let items = &section.items;
    if items.is_empty() {
        return String::new();
    }

    let mut html = String::from(
        "<nav aria-label=\"Breadcrumb\">\
         <ol style=\"display:flex;flex-wrap:wrap;align-items:center;gap:8px;\
         list-style:none;margin:0;padding:0;font-size:14px;\">",
    );

    let last_idx = items.len() - 1;

    for (i, item) in items.iter().enumerate() {
        let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let link = item.get("link").map(|s| s.as_str()).unwrap_or("");
        let is_last = i == last_idx;

        html.push_str("<li style=\"display:flex;align-items:center;gap:8px;\">");

        if is_last || link.is_empty() {
            // Current page — no link, bold
            html.push_str(&format!(
                "<span style=\"color:#171717;font-weight:500;\" aria-current=\"page\">{}</span>",
                label
            ));
        } else {
            // Linked crumb
            html.push_str(&format!(
                "<a href=\"{}\" style=\"color:#737373;text-decoration:none;\" \
                 onmouseover=\"this.style.color='#404040'\" \
                 onmouseout=\"this.style.color='#737373'\">{}</a>",
                link, label
            ));
        }

        // Separator after all items except the last
        if !is_last {
            html.push_str(
                "<span style=\"color:#a3a3a3;font-size:14px;line-height:1;\" aria-hidden=\"true\">\
                 <svg width=\"14\" height=\"14\" viewBox=\"0 0 24 24\" fill=\"none\" \
                 stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" \
                 stroke-linejoin=\"round\"><polyline points=\"9 18 15 12 9 6\"></polyline></svg>\
                 </span>",
            );
        }

        html.push_str("</li>");
    }

    html.push_str("</ol></nav>");
    html
}
