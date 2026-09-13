//! Dedicated Calendar renderer. Static month grid with `data-slot="calendar"`.
//! Days 1–28 in a `<table>` — no JS. Not interact `calendar()` SURF grid /
//! date input.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let caption = label_of(comp);
    let mut body = String::new();
    let mut day = 1;
    for _week in 0..4 {
        body.push_str("<tr>");
        for _dow in 0..7 {
            body.push_str(&format!("<td>{day}</td>"));
            day += 1;
        }
        body.push_str("</tr>");
    }
    format!(
        "<div data-slot=\"calendar\"><table><caption>{caption}</caption><thead><tr><th>Mo</th><th>Tu</th><th>We</th><th>Th</th><th>Fr</th><th>Sa</th><th>Su</th></tr></thead><tbody>{body}</tbody></table></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("showModal"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<input"));
        assert!(!html.contains("type=\"date\""));
        assert!(!html.contains("grid-template-columns:repeat(7,1fr)"));
    }

    #[test]
    fn static_month_grid_with_calendar_slot() {
        let html = render(&stub("calendar", "March"));
        assert!(html.starts_with("<div data-slot=\"calendar\">"));
        assert!(html.contains("<table>"));
        assert!(html.contains("<caption>March</caption>"));
        assert!(html.contains("<th>Mo</th>"));
        assert!(html.contains("<td>1</td>"));
        assert!(html.contains("<td>28</td>"));
        assert!(!html.contains("<td>29</td>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"calendar\"><table><caption>March</caption><thead><tr><th>Mo</th><th>Tu</th><th>We</th><th>Th</th><th>Fr</th><th>Sa</th><th>Su</th></tr></thead><tbody><tr><td>1</td><td>2</td><td>3</td><td>4</td><td>5</td><td>6</td><td>7</td></tr><tr><td>8</td><td>9</td><td>10</td><td>11</td><td>12</td><td>13</td><td>14</td></tr><tr><td>15</td><td>16</td><td>17</td><td>18</td><td>19</td><td>20</td><td>21</td></tr><tr><td>22</td><td>23</td><td>24</td><td>25</td><td>26</td><td>27</td><td>28</td></tr></tbody></table></div>"
        );
    }

    #[test]
    fn skips_interact_surf_grid() {
        let c = stub("calendar", "March");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("calendar", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("data-slot=\"calendar\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("grid-template-columns:repeat(7,1fr)"));
        assert!(interact.contains("<button type=\"button\""));
        assert!(!html.contains("<button"));
        assert!(html.contains("data-slot=\"calendar\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("calendar", "March"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"calendar\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"calendar\"]"));
        assert!(css.contains("border-collapse: collapse"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
