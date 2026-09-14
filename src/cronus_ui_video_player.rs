//! Dedicated VideoPlayer renderer. DOM matches React idle (paused, 0:00):
//! `video-player` > `video-player-video` + `video-player-overlay-play` +
//! `video-player-controls` (play, time, seek, rate, mute, volume, fullscreen).
//!
//! Zero JS: every custom control drives the media element from script, so
//! each one is rendered `disabled` (inert, announced as unavailable) at
//! React's idle geometry; the time readout is static `0:00 / 0:00`. No `src`
//! is emitted (the emitter does not carry one). Not catalog `display()` SURF,
//! not interact `video()` (`<video data-slot="video-player" controls>`).

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of};
use crate::parser::ComponentNode;

const SVG_OPEN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\">";
const PLAY: &str = "<path d=\"M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z\"></path></svg>";
const VOLUME: &str = "<path d=\"M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z\"></path><path d=\"M16 9a5 5 0 0 1 0 6\"></path><path d=\"M19.364 18.364a9 9 0 0 0 0-12.728\"></path></svg>";
const MAXIMIZE: &str = "<path d=\"M8 3H5a2 2 0 0 0-2 2v3\"></path><path d=\"M21 8V5a2 2 0 0 0-2-2h-3\"></path><path d=\"M3 16v3a2 2 0 0 0 2 2h3\"></path><path d=\"M16 21h3a2 2 0 0 0 2-2v-3\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let aria = attr_nonempty(comp, "aria-label")
        .map(esc)
        .unwrap_or_else(|| label_of(comp));
    format!(
        "<div data-slot=\"video-player\" aria-label=\"{aria}\"><video data-slot=\"video-player-video\" playsinline preload=\"metadata\"></video><button type=\"button\" data-slot=\"video-player-overlay-play\" aria-label=\"Play\" disabled>{SVG_OPEN}{PLAY}</button><div data-slot=\"video-player-controls\"><button type=\"button\" data-slot=\"video-player-play\" aria-label=\"Play\" disabled>{SVG_OPEN}{PLAY}</button><span dir=\"ltr\" data-slot=\"video-player-time\">0:00 / 0:00</span><input type=\"range\" data-slot=\"video-player-seek\" aria-label=\"Seek\" aria-valuetext=\"0:00 / 0:00\" min=\"0\" max=\"0\" step=\"0.1\" value=\"0\" disabled><button type=\"button\" data-slot=\"video-player-rate\" aria-label=\"Playback speed, 1x\" disabled>1x</button><button type=\"button\" data-slot=\"video-player-mute\" aria-label=\"Mute\" disabled>{SVG_OPEN}{VOLUME}</button><input type=\"range\" data-slot=\"video-player-volume\" aria-label=\"Volume\" min=\"0\" max=\"1\" step=\"0.05\" value=\"1\" disabled><button type=\"button\" data-slot=\"video-player-fullscreen\" aria-label=\"Fullscreen\" disabled>{SVG_OPEN}{MAXIMIZE}</button></div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";

    fn reject_stub(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("<nav"));
        assert!(!html.starts_with("<video"));
        assert!(!html.contains("<video data-slot=\"video-player\""));
        assert!(!html.contains(" controls"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("addEventListener"));
        assert!(!html.contains(".play("));
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn dom_matches_react_idle_player() {
        let html = render(&stub("video-player", "Launch video"));
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"video-player\" aria-label=\"Launch video\"><video data-slot=\"video-player-video\" playsinline preload=\"metadata\"></video><button type=\"button\" data-slot=\"video-player-overlay-play\" aria-label=\"Play\" disabled>{SVG_OPEN}{PLAY}</button><div data-slot=\"video-player-controls\"><button type=\"button\" data-slot=\"video-player-play\" aria-label=\"Play\" disabled>{SVG_OPEN}{PLAY}</button><span dir=\"ltr\" data-slot=\"video-player-time\">0:00 / 0:00</span><input type=\"range\" data-slot=\"video-player-seek\" aria-label=\"Seek\" aria-valuetext=\"0:00 / 0:00\" min=\"0\" max=\"0\" step=\"0.1\" value=\"0\" disabled><button type=\"button\" data-slot=\"video-player-rate\" aria-label=\"Playback speed, 1x\" disabled>1x</button><button type=\"button\" data-slot=\"video-player-mute\" aria-label=\"Mute\" disabled>{SVG_OPEN}{VOLUME}</button><input type=\"range\" data-slot=\"video-player-volume\" aria-label=\"Volume\" min=\"0\" max=\"1\" step=\"0.05\" value=\"1\" disabled><button type=\"button\" data-slot=\"video-player-fullscreen\" aria-label=\"Fullscreen\" disabled>{SVG_OPEN}{MAXIMIZE}</button></div></div>"
            )
        );
        assert!(!html.contains(">Play</button>"));
        reject_stub(&html);
    }

    #[test]
    fn every_control_is_inert() {
        let html = render(&stub("video-player", "Demo"));
        assert_eq!(html.matches("<button").count(), 5);
        assert_eq!(html.matches("<input").count(), 2);
        assert_eq!(html.matches(" disabled").count(), 7);
        reject_stub(&html);
    }

    #[test]
    fn video_has_no_src_and_label_is_escaped() {
        let html = render(&stub("video-player", "https://example.com/<clip>.mp4"));
        assert!(!html.contains("src="));
        assert!(html.contains("aria-label=\"https://example.com/&lt;clip&gt;.mp4\""));
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_video_and_display_surf() {
        let c = stub("video-player", "Demo");
        let html = render(&c);
        reject_stub(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("video-player", "Demo"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"video-player-play\""));
        });
    }

    #[test]
    fn chrome_is_token_only_and_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("overflow: hidden; border: 1px solid var(--cronus-border);\n  border-radius: var(--cronus-radius-xl);"));
        assert!(css.contains("aspect-ratio: 16 / 9"));
        assert!(css.contains("[data-slot=\"video-player-overlay-play\"] {"));
        assert!(css.contains("width: 3.5rem; height: 3.5rem;"));
        assert!(css.contains("[data-slot=\"video-player-time\"] {"));
        assert!(css.contains("[data-slot=\"video-player-seek\"] { min-width: 0; flex: 1; }"));
        assert!(
            css.contains("[data-slot=\"video-player-volume\"] { width: 4rem; flex-shrink: 0; }")
        );
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(!css.contains("zinc-"));
    }
}
