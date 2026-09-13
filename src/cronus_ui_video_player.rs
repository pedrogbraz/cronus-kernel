//! Dedicated VideoPlayer renderer. DOM matches React idle:
//! `<div data-slot="video-player"><video data-slot="video-player-video">`
//! (no src required) plus `video-player-controls` with a play button.
//! Zero JS. Not catalog `display()` SURF `<section>` without a video
//! element, not interact `video()` (`<video data-slot="video-player">`).

use crate::parser::ComponentNode;

pub fn render(_comp: &ComponentNode) -> String {
    "<div data-slot=\"video-player\"><video data-slot=\"video-player-video\" playsinline preload=\"metadata\"></video><div data-slot=\"video-player-controls\"><button type=\"button\" data-slot=\"video-player-play\" aria-label=\"Play\">Play</button></div></div>"
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";

    fn reject_stub(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("<pre"));
        assert!(!html.starts_with("<video"));
        assert!(!html.contains("<video data-slot=\"video-player\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("addEventListener"));
        assert!(!html.contains(".play("));
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("display("));
        assert!(!html.contains("video("));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_div_with_video_and_play_controls() {
        let html = render(&stub("video-player", "Demo"));
        assert!(html.starts_with("<div data-slot=\"video-player\">"));
        assert!(html.contains("<video data-slot=\"video-player-video\""));
        assert!(html.contains("</video>"));
        assert!(!html.contains(" src="));
        assert!(html.contains("<div data-slot=\"video-player-controls\">"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"video-player-play\" aria-label=\"Play\">Play</button>"
        ));
        reject_stub(&html);
        assert_eq!(
            html,
            "<div data-slot=\"video-player\"><video data-slot=\"video-player-video\" playsinline preload=\"metadata\"></video><div data-slot=\"video-player-controls\"><button type=\"button\" data-slot=\"video-player-play\" aria-label=\"Play\">Play</button></div></div>"
        );
    }

    #[test]
    fn video_has_no_src() {
        let html = render(&stub("video-player", "https://example.com/clip.mp4"));
        assert!(html.contains("<video data-slot=\"video-player-video\""));
        assert!(!html.contains("src="));
        assert!(!html.contains("example.com"));
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_video_and_display_surf() {
        let c = stub("video-player", "Demo");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("video-player", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<video data-slot=\"video-player\""));
        assert!(interact.contains("style="));
        assert!(interact.contains(" controls"));
        assert!(!interact.contains("data-slot=\"video-player-video\""));
        assert!(!interact.contains("data-slot=\"video-player-controls\""));
        assert!(html.contains("data-slot=\"video-player-video\""));
        assert!(html.contains("data-slot=\"video-player-controls\""));
        assert!(html.contains("<video"));
        assert!(!html.contains("<section"));
        assert!(!html.contains(DISPLAY_SURF));
        reject_stub(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("video-player", "Demo"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"video-player-video\""));
            assert!(html.contains("data-slot=\"video-player-play\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"video-player\"]"));
        assert!(css.contains("[data-slot=\"video-player-video\"]"));
        assert!(css.contains("[data-slot=\"video-player-controls\"]"));
        assert!(css.contains("[data-slot=\"video-player-play\"]"));
        assert!(css.contains("aspect-ratio: 16 / 9"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-surface-base)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
