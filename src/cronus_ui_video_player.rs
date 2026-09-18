//! Dedicated VideoPlayer renderer. DOM matches React idle (paused, 0:00):
//! `video-player` > `video-player-video` + `video-player-overlay-play` +
//! `video-player-controls` (play, time, seek, rate, mute, volume, fullscreen).
//!
//! `src:` / `poster:` feed the `<video>` (`muted:true`, `loop:true` too);
//! `captions:"…"` + `captions-lang:` + `captions-label:` add the docs'
//! default `<track kind="captions">` (an inline `data:text/vtt` stub is
//! accepted). `aspect` (`video` | `square` | `wide`, style segment or
//! `aspect:` prop) travels as a class (`a-square`, `a-wide`; React has no data
//! attribute), `max-width:xs` is the docs' centred `max-w-xs`. Every control
//! name is overridable like React's `labels`: `label-play`, `label-pause`,
//! `label-mute`, `label-unmute`, `label-seek`, `label-volume`, `label-settings`,
//! `label-fullscreen`, `label-exit-fullscreen`. The root is named by
//! `aria-label:` or the `label` item only.
//!
//! Zero JS in this renderer: custom controls are native buttons/ranges (idle
//! look, enabled unless the author set `disabled:`). Page runtime
//! (`cronus_ui_live.js`) drives play/pause, seek, mute, volume, rate, and
//! fullscreen. Time readout starts at React's `0:00 / 0:00`. Not catalog
//! `display()` SURF, not interact `video()` (`<video data-slot="video-player"
//! controls>`).

use crate::cronus_ui_kit::{attr_nonempty, choice, esc, flag, item, safe_url};
use crate::parser::ComponentNode;

const SVG_OPEN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\">";
const PLAY: &str = "<path d=\"M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z\"></path></svg>";
const VOLUME: &str = "<path d=\"M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z\"></path><path d=\"M16 9a5 5 0 0 1 0 6\"></path><path d=\"M19.364 18.364a9 9 0 0 0 0-12.728\"></path></svg>";
const VOLUME_X: &str = "<path d=\"M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z\"></path><line x1=\"22\" x2=\"16\" y1=\"9\" y2=\"15\"></line><line x1=\"16\" x2=\"22\" y1=\"9\" y2=\"15\"></line></svg>";
const MAXIMIZE: &str = "<path d=\"M8 3H5a2 2 0 0 0-2 2v3\"></path><path d=\"M21 8V5a2 2 0 0 0-2-2h-3\"></path><path d=\"M3 16v3a2 2 0 0 0 2 2h3\"></path><path d=\"M16 21h3a2 2 0 0 0 2-2v-3\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let aria = attr_nonempty(comp, "aria-label")
        .or_else(|| item(comp, "label").filter(|l| !l.is_empty()))
        .map(|l| format!(" aria-label=\"{}\"", esc(l)))
        .unwrap_or_default();
    let label = |key: &str, default: &str| {
        attr_nonempty(comp, key)
            .map(esc)
            .unwrap_or_else(|| default.to_string())
    };
    let play = label("label-play", "Play");
    let pause = label("label-pause", "Pause");
    let mute = label("label-mute", "Mute");
    let unmute = label("label-unmute", "Unmute");
    let seek = label("label-seek", "Seek");
    let volume = label("label-volume", "Volume");
    let settings = label("label-settings", "Playback speed");
    let fullscreen = label("label-fullscreen", "Fullscreen");
    let exit_fs = label("label-exit-fullscreen", "Exit fullscreen");
    let disabled = if flag(comp, "disabled") {
        " disabled"
    } else {
        ""
    };
    let muted = flag(comp, "muted");
    let mute_aria = if muted {
        unmute.as_str()
    } else {
        mute.as_str()
    };
    let mute_icon = if muted {
        format!("{SVG_OPEN}{VOLUME_X}")
    } else {
        format!("{SVG_OPEN}{VOLUME}")
    };
    let volume_value = if muted { "0" } else { "1" };
    let mut video_attrs = String::new();
    if let Some(src) = attr_nonempty(comp, "src") {
        video_attrs.push_str(&format!(" src=\"{}\"", safe_url(src)));
    }
    if let Some(poster) = attr_nonempty(comp, "poster") {
        video_attrs.push_str(&format!(" poster=\"{}\"", safe_url(poster)));
    }
    if flag(comp, "loop") {
        video_attrs.push_str(" loop");
    }
    if flag(comp, "muted") {
        video_attrs.push_str(" muted");
    }
    let track = attr_nonempty(comp, "captions")
        .map(|src| {
            let src = if src.starts_with("data:text/vtt,") {
                esc(src)
            } else {
                safe_url(src)
            };
            let lang = attr_nonempty(comp, "captions-lang")
                .map(|l| format!(" srclang=\"{}\"", esc(l)))
                .unwrap_or_default();
            let label = attr_nonempty(comp, "captions-label")
                .map(|l| format!(" label=\"{}\"", esc(l)))
                .unwrap_or_default();
            format!("<track kind=\"captions\" src=\"{src}\"{lang}{label} default>")
        })
        .unwrap_or_default();
    let mut classes: Vec<&str> = Vec::new();
    if let Some(aspect) = choice(comp, "aspect", &["square", "wide"]) {
        classes.push(if aspect == "square" {
            "a-square"
        } else {
            "a-wide"
        });
    }
    if let Some(mw) = crate::cronus_ui_card::max_width_class(comp) {
        classes.push(mw);
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    format!(
        "<div data-slot=\"video-player\"{class}{aria}><video data-slot=\"video-player-video\"{video_attrs} playsinline preload=\"metadata\">{track}</video><button type=\"button\" data-slot=\"video-player-overlay-play\" aria-label=\"{play}\"{disabled}>{SVG_OPEN}{PLAY}</button><div data-slot=\"video-player-controls\"><button type=\"button\" data-slot=\"video-player-play\" aria-label=\"{play}\" data-pause=\"{pause}\"{disabled}>{SVG_OPEN}{PLAY}</button><span dir=\"ltr\" data-slot=\"video-player-time\">0:00 / 0:00</span><input type=\"range\" data-slot=\"video-player-seek\" aria-label=\"{seek}\" aria-valuetext=\"0:00 / 0:00\" min=\"0\" max=\"0\" step=\"0.1\" value=\"0\"{disabled}><button type=\"button\" data-slot=\"video-player-rate\" aria-label=\"{settings}, 1x\" data-settings=\"{settings}\"{disabled}>1x</button><button type=\"button\" data-slot=\"video-player-mute\" aria-label=\"{mute_aria}\" data-mute=\"{mute}\" data-unmute=\"{unmute}\"{disabled}>{mute_icon}</button><input type=\"range\" data-slot=\"video-player-volume\" aria-label=\"{volume}\" min=\"0\" max=\"1\" step=\"0.05\" value=\"{volume_value}\"{disabled}><button type=\"button\" data-slot=\"video-player-fullscreen\" aria-label=\"{fullscreen}\" data-exit=\"{exit_fs}\"{disabled}>{SVG_OPEN}{MAXIMIZE}</button></div></div>"
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
                "<div data-slot=\"video-player\" aria-label=\"Launch video\"><video data-slot=\"video-player-video\" playsinline preload=\"metadata\"></video><button type=\"button\" data-slot=\"video-player-overlay-play\" aria-label=\"Play\">{SVG_OPEN}{PLAY}</button><div data-slot=\"video-player-controls\"><button type=\"button\" data-slot=\"video-player-play\" aria-label=\"Play\" data-pause=\"Pause\">{SVG_OPEN}{PLAY}</button><span dir=\"ltr\" data-slot=\"video-player-time\">0:00 / 0:00</span><input type=\"range\" data-slot=\"video-player-seek\" aria-label=\"Seek\" aria-valuetext=\"0:00 / 0:00\" min=\"0\" max=\"0\" step=\"0.1\" value=\"0\"><button type=\"button\" data-slot=\"video-player-rate\" aria-label=\"Playback speed, 1x\" data-settings=\"Playback speed\">1x</button><button type=\"button\" data-slot=\"video-player-mute\" aria-label=\"Mute\" data-mute=\"Mute\" data-unmute=\"Unmute\">{SVG_OPEN}{VOLUME}</button><input type=\"range\" data-slot=\"video-player-volume\" aria-label=\"Volume\" min=\"0\" max=\"1\" step=\"0.05\" value=\"1\"><button type=\"button\" data-slot=\"video-player-fullscreen\" aria-label=\"Fullscreen\" data-exit=\"Exit fullscreen\">{SVG_OPEN}{MAXIMIZE}</button></div></div>"
            )
        );
        assert!(!html.contains(">Play</button>"));
        reject_stub(&html);
    }

    #[test]
    fn controls_are_live_unless_author_disabled() {
        let html = render(&stub("video-player", "Demo"));
        assert_eq!(html.matches("<button").count(), 5);
        assert_eq!(html.matches("<input").count(), 2);
        assert_eq!(html.matches(" disabled").count(), 0);
        reject_stub(&html);
        let mut c = stub("video-player", "Demo");
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert_eq!(html.matches(" disabled").count(), 7);
        assert!(html.contains(
            "data-slot=\"video-player-play\" aria-label=\"Play\" data-pause=\"Pause\" disabled>"
        ));
        reject_stub(&html);
    }

    #[test]
    fn video_has_no_src_and_label_is_escaped() {
        let html = render(&stub("video-player", "https://example.com/<clip>.mp4"));
        assert!(!html.contains("src="));
        assert!(html.contains("aria-label=\"https://example.com/&lt;clip&gt;.mp4\""));
        reject_stub(&html);
    }

    /// Docs examples: `src` / `poster` / captions track, the `wide` and
    /// `square` aspects as classes, `muted` + `loop`, localized labels.
    #[test]
    fn src_poster_track_aspect_and_labels() {
        let mut c = stub("video-player", "");
        c.items.clear();
        c.props.insert(
            "src".into(),
            "https://media.w3.org/2010/05/sintel/trailer.mp4".into(),
        );
        c.props.insert(
            "poster".into(),
            "https://media.w3.org/2010/05/sintel/poster.png".into(),
        );
        c.props.insert(
            "captions".into(),
            "data:text/vtt,WEBVTT%0A%0A00:00.000%20--%3E%2000:04.000%0AWind".into(),
        );
        c.props.insert("captions-lang".into(), "en".into());
        c.props.insert("captions-label".into(), "English".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"video-player\"><video data-slot=\"video-player-video\" src=\"https://media.w3.org/2010/05/sintel/trailer.mp4\" poster=\"https://media.w3.org/2010/05/sintel/poster.png\" playsinline preload=\"metadata\"><track kind=\"captions\" src=\"data:text/vtt,WEBVTT%0A%0A00:00.000%20--%3E%2000:04.000%0AWind\" srclang=\"en\" label=\"English\" default></video>"));
        c.props
            .insert("captions".into(), "javascript:alert(1)".into());
        assert!(render(&c).contains("<track kind=\"captions\" src=\"#\""));
        let mut w = stub("video-player", "");
        w.items.clear();
        w.style = Some("video-player+wide".into());
        assert!(render(&w).starts_with("<div data-slot=\"video-player\" class=\"a-wide\">"));
        w.style = Some("video-player".into());
        w.props.insert("aspect".into(), "square".into());
        w.props.insert("max-width".into(), "xs".into());
        assert!(render(&w).starts_with("<div data-slot=\"video-player\" class=\"a-square mw-xs\">"));
        let mut l = stub("video-player", "");
        l.items.clear();
        l.props.insert("muted".into(), "true".into());
        l.props.insert("loop".into(), "true".into());
        l.props.insert("label-play".into(), "Reproduzir".into());
        l.props.insert("label-pause".into(), "Pausar".into());
        l.props.insert("label-mute".into(), "Silenciar".into());
        l.props.insert("label-unmute".into(), "Ativar som".into());
        l.props.insert("label-seek".into(), "Avançar".into());
        l.props.insert("label-settings".into(), "Velocidade".into());
        l.props
            .insert("label-fullscreen".into(), "Tela cheia".into());
        l.props
            .insert("label-exit-fullscreen".into(), "Sair da tela cheia".into());
        let html = render(&l);
        assert!(html.contains("<video data-slot=\"video-player-video\" loop muted playsinline"));
        assert!(html.contains("aria-label=\"Reproduzir\">"));
        assert!(html.contains("data-pause=\"Pausar\""));
        // muted:true idle = Unmute + VolumeX, slider at 0 (React).
        assert!(html.contains(
            "aria-label=\"Ativar som\" data-mute=\"Silenciar\" data-unmute=\"Ativar som\""
        ));
        assert!(html.contains(VOLUME_X));
        assert!(html.contains(
            "data-slot=\"video-player-volume\" aria-label=\"Volume\" min=\"0\" max=\"1\" step=\"0.05\" value=\"0\""
        ));
        assert!(html.contains("aria-label=\"Avançar\" aria-valuetext"));
        assert!(
            html.contains("aria-label=\"Velocidade, 1x\" data-settings=\"Velocidade\">1x</button>")
        );
        assert!(html.contains("aria-label=\"Tela cheia\" data-exit=\"Sair da tela cheia\">"));
        assert!(!html.contains("aria-label=\"Play\""));
        assert!(!html.contains(" disabled"));
        reject_stub(&html);
        let css = include_str!("cronus_ui_css/video-player.css");
        assert!(css.contains("[data-slot=\"video-player\"].a-square { aspect-ratio: 1 / 1; }"));
        assert!(css.contains("[data-slot=\"video-player\"].a-wide { aspect-ratio: 21 / 9; }"));
        assert!(css.contains(
            "[data-slot=\"video-player\"].mw-xs { max-width: 20rem; margin-inline: auto; }"
        ));
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
        let css = include_str!("cronus_ui_css/video-player.css");
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
        assert!(css.contains(
            "[data-slot=\"video-player\"][data-playing] [data-slot=\"video-player-overlay-play\"] { display: none; }"
        ));
        assert!(css.contains(
            "[data-slot=\"video-player\"][data-playing] [data-slot=\"video-player-controls\"] { opacity: 0; pointer-events: none; }"
        ));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn live_runtime_binds_custom_controls() {
        let js = include_str!("cronus_ui_live.js");
        assert!(js.contains("[data-slot=\"video-player-play\"]"));
        assert!(js.contains("[data-slot=\"video-player-overlay-play\"]"));
        assert!(js.contains("[data-slot=\"video-player-seek\"]"));
        assert!(js.contains("[data-slot=\"video-player-mute\"]"));
        assert!(js.contains("[data-slot=\"video-player-volume\"]"));
        assert!(js.contains("[data-slot=\"video-player-fullscreen\"]"));
        assert!(js.contains("[data-slot=\"video-player-rate\"]"));
        assert!(js.contains("requestFullscreen"));
        assert!(js.contains("currentTime"));
        assert!(js.contains("bindVideoPlayers"));
    }
}
