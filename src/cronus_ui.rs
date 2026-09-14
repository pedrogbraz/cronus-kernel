//! Cronus UI token + Button slice.
//!
//! `.cronus` files already declare `style { preset aurora; accent-hex "..." }`
//! and `component Button layout:inline style:button+primary+md`. Without this
//! module the kernel paints Obsidian/amber hardcodes, so a complete `.cronus`
//! catalog can never look like cronus-ui.
//!
//! Values copied from `@cronus-ui/tokens` (aurora / neutral Ã— light / dark).
//! Components use ONLY `var(--cronus-*)` â€” no palette scales (`zinc-900`).

pub fn is_named_preset(preset: &str) -> bool {
    matches!(
        preset.trim(),
        "aurora" | "neutral" | "midnight" | "sunset" | "emerald"
    )
}

/// `style { theme … }` value as a color mode: `"light"`, `"system"` or
/// `"dark"` (the default, also for unknown values).
pub fn normalize_mode(mode: &str) -> &'static str {
    match mode.trim().to_ascii_lowercase().as_str() {
        "light" => "light",
        "system" => "system",
        _ => "dark",
    }
}

/// Tokens + every family, unlayered (the pre-split output). Layouts emit
/// `cronus_ui_css::page_stylesheet_for` instead, which ships only the families
/// a page renders. `light`/`dark` are selected by `data-cronus-mode`; `system`
/// adds the preset's other-mode tokens under `prefers-color-scheme`.
pub fn token_css(preset: &str, mode: &str) -> String {
    crate::cronus_ui_css::full_stylesheet_for(preset, mode)
}

/// Audit preflight + vendored tokens + every family, layered. The audit
/// document uses `cronus_ui_css::audit_stylesheet(widget_html)`; theme/mode are
/// selected by `data-cronus-theme` / `data-cronus-mode` on `<html>`.
pub fn audit_stylesheet() -> String {
    crate::cronus_ui_css::full_audit_stylesheet()
}

pub fn vendored_tokens_css() -> &'static str {
    crate::cronus_ui_css::TOKENS_CSS
}

/// Theme chrome + every family's component CSS, in cascade order
/// (`src/cronus_ui_css/`, ordered by its MANIFEST).
pub fn component_chrome_css() -> &'static str {
    crate::cronus_ui_css::full_components()
}

/// Button matching cronus-ui CONTRACT: semantic tokens, variants, sizes,
/// `data-slot`, `data-variant`, focus-visible, href â†’ `<a>`.
/// `danger` is accepted as an alias of `destructive` (legacy kernel name).
pub fn button(label: &str, variant: &str, size: &str, href: Option<&str>) -> String {
    button_ex(label, variant, size, href, false)
}

pub fn button_ex(
    label: &str,
    variant: &str,
    size: &str,
    href: Option<&str>,
    disabled: bool,
) -> String {
    let variant = match variant {
        "danger" | "destructive" => "destructive",
        "secondary" => "secondary",
        "outline" => "outline",
        "ghost" => "ghost",
        "link" => "link",
        _ => "primary",
    };
    let tag = if href.is_some() { "a" } else { "button" };
    let href_attr = href
        .map(|h| format!(" href=\"{}\"", crate::cronus_ui_kit::safe_url(h)))
        .unwrap_or_default();
    let type_attr = if href.is_none() {
        " type=\"button\""
    } else {
        ""
    };
    let disabled_attr = if disabled && href.is_none() {
        " disabled"
    } else {
        ""
    };
    format!(
        "<{tag}{href_attr}{type_attr}{disabled_attr} data-slot=\"button\" data-variant=\"{variant}\" data-size=\"{size}\" class=\"cui-btn\">{label}</{tag}>",
        size = crate::cronus_ui_kit::esc(size),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aurora_dark_emits_semantic_vars() {
        let css = token_css("aurora", "dark");
        assert!(css.contains("--cronus-primary:"));
        assert!(css.contains("--cronus-surface-base:"));
        assert!(css.contains("--cronus-fg:"));
        assert!(css.contains("--cronus-error:"));
        assert!(!css.contains("zinc-900"));
        assert!(css.contains(":root, [data-cronus-theme=\"aurora\"]"));
    }

    #[test]
    fn legacy_preset_does_not_force_aurora_root() {
        let css = token_css("legacy", "dark");
        assert!(css.contains("--cronus-primary: var(--primary"));
        assert!(!css.contains("oklch(0.685 0.169 237.3)"));
        assert!(!css.contains("color-scheme: dark"));
    }

    #[test]
    fn button_contract() {
        let html = button("Save", "primary", "md", None);
        assert!(html.contains("data-slot=\"button\""));
        assert!(html.contains("data-variant=\"primary\""));
        assert!(html.contains("data-size=\"md\""));
        assert!(html.contains("class=\"cui-btn\""));
        assert!(html.contains("type=\"button\""));
        assert!(!html.contains("uppercase"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("amber-"));
        assert!(html.starts_with("<button"));
        let css = token_css("aurora", "dark");
        assert!(css.contains("background: var(--cronus-primary)"));
        assert!(css.contains("height: 2.5rem"));
    }

    #[test]
    fn button_href_is_anchor() {
        let html = button("Docs", "link", "md", Some("/docs"));
        assert!(html.starts_with("<a "));
        assert!(html.contains("href=\"/docs\""));
        assert!(html.contains("data-variant=\"link\""));
        assert!(!html.contains("type="));
        let css = token_css("aurora", "dark");
        assert!(css.contains("color: var(--cronus-primary-text)"));
    }

    #[test]
    fn button_destructive_alias_danger() {
        let a = button("Delete", "destructive", "md", None);
        let b = button("Delete", "danger", "md", None);
        assert!(a.contains("data-variant=\"destructive\""));
        assert!(b.contains("data-variant=\"destructive\""));
        let css = token_css("aurora", "dark");
        assert!(css.contains("color-mix(in oklch, var(--cronus-error), black 30%)"));
        assert!(css.contains("color: #fff"));
    }

    #[test]
    fn button_sizes() {
        let sm = button("A", "primary", "sm", None);
        let icon = button("A", "primary", "icon", None);
        assert!(sm.contains("data-size=\"sm\""));
        assert!(icon.contains("data-size=\"icon\""));
        let css = token_css("legacy", "dark");
        assert!(css.contains("[data-size=\"sm\"] { height: 2rem"));
        assert!(css.contains("[data-size=\"icon\"] { height: 2.25rem"));
    }

    #[test]
    fn aurora_tokens_match_vendored_css() {
        let css = token_css("aurora", "dark");
        assert!(css.contains("oklch(0.685 0.169 237.3)"));
        assert!(css.contains("--cronus-font-display"));
        assert!(css.contains("--cronus-shadow-glow"));
        assert!(css.contains("--cronus-chart-1"));
        assert!(css.contains("letter-spacing: -0.03em"));
    }

    #[test]
    fn button_disabled() {
        let html = button_ex("Nope", "primary", "md", None, true);
        assert!(html.contains(" disabled"));
        assert!(!html.contains("style="));
        let css = component_chrome_css();
        assert!(
            css.contains("[data-slot=\"button\"]:disabled { opacity: 0.5; pointer-events: none; }")
        );
    }

    #[test]
    fn button_wave1t_geometry_matches_react() {
        // React primary/ghost/destructive: no border; text-sm -> 1.25rem line-height.
        let css = component_chrome_css();
        assert!(css.contains("line-height: 1.25rem; cursor: pointer; text-decoration: none;\n  outline: none; border: 0 solid transparent;"));
        assert!(css.contains("font-size: 0.75rem; line-height: 1rem; }"));
        assert!(css.contains("font-size: 1rem; line-height: 1.5rem; }"));
        assert!(css.contains("border-width: 1px; border-color: var(--cronus-border);"));
        let input = &css[css
            .find("[data-slot=\"input\"] { line-height")
            .expect("input lh")..];
        assert!(input.starts_with("[data-slot=\"input\"] { line-height: 1.25rem; }"));
        assert!(css.contains("[data-slot=\"textarea\"] { line-height: 1.25rem; }"));
        assert!(css.contains("[data-slot=\"toggle\"] {\n  line-height: 1.25rem;"));
        assert!(css
            .contains("font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-secondary);"));
    }

    #[test]
    fn tokens_include_focus_visible_ring() {
        let css = token_css("legacy", "dark");
        assert!(css.contains(":focus-visible"));
        assert!(css.contains("--cronus-ring"));
        assert!(css.contains("prefers-reduced-motion"));
    }

    #[test]
    fn component_chrome_has_no_tailwind_palette() {
        let css = audit_stylesheet();
        assert!(!css.contains("--tw-"));
        assert!(!css.contains("@tailwind"));
        assert!(!css.contains("zinc-900"));
        assert!(!css.contains("bg-zinc-"));
    }

    /// The React audit reference renders with Tailwind v4 preflight: form
    /// controls inherit font/line-height (else 13.33px / `normal`), everything
    /// is border-box with zeroed margins. The audit document must match, and the
    /// preflight must precede the chrome so component rules still win.
    #[test]
    fn audit_stylesheet_starts_with_tailwind_preflight() {
        use crate::cronus_ui_css::{AUDIT_CSS, LAYER_ORDER};
        let css = audit_stylesheet();
        assert!(css.starts_with(&format!(
            "{LAYER_ORDER}\n@layer cronus.tokens {{\n{AUDIT_CSS}"
        )));
        assert!(css.find(AUDIT_CSS).unwrap() < css.find("[data-slot=\"button\"]").unwrap());
        assert!(
            AUDIT_CSS.contains("box-sizing: border-box; margin: 0; padding: 0; border: 0 solid;")
        );
        assert!(AUDIT_CSS.contains("font: inherit; font-feature-settings: inherit;"));
        assert!(AUDIT_CSS.contains("letter-spacing: inherit; color: inherit; border-radius: 0; background-color: transparent;"));
        assert!(AUDIT_CSS.contains("ol, ul, menu { list-style: none; }"));
        assert_eq!(
            AUDIT_CSS.matches('{').count(),
            AUDIT_CSS.matches('}').count()
        );
        assert!(!component_chrome_css().contains(AUDIT_CSS));
    }

    #[test]
    fn component_chrome_braces_are_balanced() {
        let css = component_chrome_css();
        let open = css.chars().filter(|&c| c == '{').count();
        let close = css.chars().filter(|&c| c == '}').count();
        assert!(
            close >= open,
            "unclosed CSS rules: {open} open vs {close} close"
        );
        assert!(
            close - open <= 8,
            "too many extra CSS closes: {open} vs {close}"
        );
        assert!(css.contains("[popover]"));
        assert!(css.contains("cronus-pop-in"));
    }

    fn rule<'a>(css: &'a str, selector: &str) -> &'a str {
        let start = css
            .find(&format!("{selector} {{"))
            .unwrap_or_else(|| panic!("missing rule {selector}"));
        let body = &css[start..];
        &body[..body.find('}').expect("rule end")]
    }

    /// fa: React `aria-invalid:ring-2 aria-invalid:ring-error/30` on Input/Textarea.
    #[test]
    fn invalid_input_and_textarea_paint_error_ring_like_react() {
        let css = component_chrome_css();
        for slot in ["input", "textarea"] {
            let r = rule(
                css,
                &format!("[data-slot=\"{slot}\"][aria-invalid=\"true\"]"),
            );
            assert!(r.contains("border-color: var(--cronus-error);"), "{slot}");
            assert!(
                r.contains(
                    "box-shadow: 0 0 0 2px color-mix(in oklab, var(--cronus-error) 30%, transparent);"
                ),
                "{slot}"
            );
        }
    }

    /// fa: RadioGroupItem is `shadow-xs`; soft Chip is `border-transparent`;
    /// the Lightbox `<img>` has no placeholder shadow.
    #[test]
    fn radio_chip_lightbox_image_chrome_matches_react() {
        let css = component_chrome_css();
        assert!(rule(css, "[data-slot=\"radio-group-item\"]")
            .contains("box-shadow: var(--cronus-shadow-xs, none);"));
        let chip = rule(css, "[data-slot=\"chip\"]");
        assert!(chip.contains("border: 1px solid transparent;"));
        assert!(!chip.contains("var(--cronus-border)"));
        assert!(!rule(css, "[data-slot=\"lightbox-image\"]").contains("box-shadow"));
    }

    /// fa: JS-only controls stay `disabled` but keep React's undimmed idle look
    /// where React's control is enabled (scheduler nav, invite Cancel, split
    /// chevron unless the whole group is disabled).
    #[test]
    fn js_only_disabled_buttons_not_dimmed_where_react_is_enabled() {
        let css = component_chrome_css();
        assert!(css
            .contains("[data-slot=\"scheduler\"] [data-slot=\"button\"]:disabled { opacity: 1; }"));
        assert!(css.contains(
            "[data-slot=\"invite-dialog\"] [data-slot=\"button\"]:disabled { opacity: 1; }"
        ));
        assert!(css.contains("[data-slot=\"split-button\"]:not([data-disabled]) > [data-slot=\"button\"][aria-haspopup]:disabled { opacity: 1; }"));
        assert!(css.contains(
            "[data-slot=\"split-button\"][data-disabled=\"\"] { opacity: 0.5; pointer-events: none; }"
        ));
    }
}
