# V2 — Section Type Dispatcher Verification

**Scope**: `cronus-kernel/src/ui/` — `mod.rs`, `page.rs`, `section_*.rs`, `component.rs`, `util.rs`.
**Method**: Read the actual Rust match arms in `render_section()`; count, categorize, and verify real rendering behavior.

---

## 1. The Dispatch Function

Function: `render_section(section, accent, theme, bound_data) -> String` in `/home/zedd/Documentos/CRONUS/cronus-kernel/src/ui/mod.rs` (lines 592–845).

Before dispatch, it runs:
1. **Visibility check** (`section.visibility` conditional, lines 594–617).
2. **Contract validation** (`crate::contracts::validate_section`, lines 619–653) — strict mode may short-circuit and render a red error `<div>`.
3. **Template override** (lines 656–671) — if the section carries a `template` block (field or config), it bypasses the entire match and calls `render_template()` instead.
4. **Alias resolution** (lines 674–676):
   ```rust
   let resolved_type = crate::contracts::ContractRegistry::resolve_alias(&section.section_type)
       .unwrap_or(section.section_type.as_str());
   ```

Then the big match (lines 677–759):

```rust
let section_html = match resolved_type {
    "hero" => section_hero::render_hero(section, accent, theme),
    "features" => section_features::render_features(section, accent, theme),
    "pricing" => section_misc::render_pricing(section, accent),
    "cta" => section_misc::render_cta(section, accent, theme),
    "faq" => section_misc::render_faq(section, accent),
    "stats" => section_misc::render_stats(section, accent),
    "trusted" => section_misc::render_trusted(section),
    "topbar" => section_misc::render_topbar(section, theme),
    "checkout" => section_misc::render_checkout_section(section),
    "testimonial" => section_misc::render_testimonial(section, theme),
    "footer" => section_misc::render_footer(section, theme),
    "page-header" => section_misc::render_page_header_section(section),
    "stat-cards" => section_kpi::render_stat_cards(section, bound_data),
    "product-grid" => section_extra::render_product_grid_section(section),
    "promo" => section_misc::render_promo(section),
    "info-bar" => section_misc::render_info_bar(section),
    "bento" => section_extra::render_bento(section, accent),
    "features-split" => section_features::render_features_split(section, accent),
    "team-list" => section_extra::render_team_list(section),
    "status-card" => section_extra::render_status_card(section),
    "policies" => section_extra::render_policies(section),
    "activity-table" => section_extra::render_activity_table(section),
    "edge" => section_extra::render_edge(section, accent),
    "sidebar" => section_extra::render_sidebar(section),
    "form" => section_form::render_form_section(section, bound_data),
    "card" | "live-keys" | "test-keys" | "webhooks" => section_extra::render_card_section(section),
    "links" | "quick-links" => section_extra::render_links_section(section),
    "tabs" => crate::tabs::render_tabs(section),
    "accordion" => crate::feedback::render_accordion(section),
    "breadcrumb" => crate::navigation::render_breadcrumb(section),
    "alert" => crate::feedback::render_alert(section),
    "chart" => section_chart::render_chart_section(section, bound_data),
    "modal" => section_extra::render_modal_section(section),
    "sheet" => section_extra::render_sheet_section(section),
    "skeleton" | "loading" => section_extra::render_skeleton_section(section),
    "empty" => section_extra::render_empty_section(section),
    "error" => section_extra::render_error_section(section),
    "not-found" | "404" => section_extra::render_not_found_section(section),
    "kpi" => {
        if theme == "dark" || theme == "obsidian" {
            section_kpi::render_kpi_dashboard_dark(section, bound_data)
        } else {
            section_kpi::render_kpi_section(section, bound_data)
        }
    }
    "timeline" => section_extra::render_timeline_section(section, bound_data),
    "progress" => section_extra::render_progress_section(section, bound_data),
    "command" => crate::command_palette::render_command_palette(section),
    "table" => { /* data_table::render_data_table[_dark] */ }
    "pagination" => crate::data_table::render_pagination(section),
    "filters" => crate::data_table::render_filters_toolbar(section),
    "dropdown" => crate::overlays::render_dropdown(section),
    "toast" => crate::overlays::render_toast(section),
    "notifications" => crate::overlays::render_notification_center(section),
    "kanban" => crate::board::render_kanban(section, bound_data),
    "dark-mode" => crate::board::render_dark_mode_toggle(section),
    "layout" => { /* layout_system::render_{column_,}layout(_section) */ }
    _ => section_extra::render_generic_section(section, accent),
};
```

### Raw arm count

- **Distinct match arms (lines)**: 52 (including the `_` default).
- **Distinct section type strings** across all arms: 58.
- **Arms sharing a renderer via `|` aliases**: 5 such arms account for 11 strings routing to 5 functions (see §4).

---

## 2. Section Type Catalog

**Note on alias resolution**: `ContractRegistry::resolve_alias()` (contracts.rs:415) rewrites certain type strings BEFORE the match runs. Several arms in `mod.rs` are therefore **unreachable dead code** in normal builds — they are shadowed by earlier alias rewrites. Marked `DEAD(alias→X)` in the table.

Hardcoded fallback alias map (contracts.rs:424–441):
```
stats        → kpi
stat-cards   → kpi
status-card  → card
activity-table → table
team-list    → table
policies     → form
live-keys    → card
test-keys    → card
webhooks     → card
quick-links  → links
promo        → card
info-bar     → alert
edge         → features
bento        → features
features-split → features
product-grid → card
```

| # | Section type | Renderer (file / fn) | Category | Notes |
|---|---|---|---|---|
| 1 | `hero` | section_hero.rs / render_hero | marketing | |
| 2 | `features` | section_features.rs / render_features | marketing | |
| 3 | `pricing` | section_misc.rs / render_pricing | marketing | |
| 4 | `cta` | section_misc.rs / render_cta | marketing | |
| 5 | `faq` | section_misc.rs / render_faq | marketing | |
| 6 | `stats` | section_misc.rs / render_stats | data | DEAD(alias→kpi) |
| 7 | `trusted` | section_misc.rs / render_trusted | marketing | |
| 8 | `topbar` | section_misc.rs / render_topbar | nav | |
| 9 | `checkout` | section_misc.rs / render_checkout_section | form | |
| 10 | `testimonial` | section_misc.rs / render_testimonial | marketing | |
| 11 | `footer` | section_misc.rs / render_footer | nav | |
| 12 | `page-header` | section_misc.rs / render_page_header_section | layout | |
| 13 | `stat-cards` | section_kpi.rs / render_stat_cards | data | DEAD(alias→kpi) |
| 14 | `product-grid` | section_extra.rs / render_product_grid_section | card | DEAD(alias→card) |
| 15 | `promo` | section_misc.rs / render_promo | marketing | DEAD(alias→card) |
| 16 | `info-bar` | section_misc.rs / render_info_bar | feedback | DEAD(alias→alert) |
| 17 | `bento` | section_extra.rs / render_bento | marketing | DEAD(alias→features) |
| 18 | `features-split` | section_features.rs / render_features_split | marketing | DEAD(alias→features) |
| 19 | `team-list` | section_extra.rs / render_team_list | data | DEAD(alias→table) |
| 20 | `status-card` | section_extra.rs / render_status_card | card | DEAD(alias→card) |
| 21 | `policies` | section_extra.rs / render_policies | form | DEAD(alias→form) |
| 22 | `activity-table` | section_extra.rs / render_activity_table | data | DEAD(alias→table) |
| 23 | `edge` | section_extra.rs / render_edge | marketing | DEAD(alias→features) |
| 24 | `sidebar` | section_extra.rs / render_sidebar | nav | |
| 25 | `form` | section_form.rs / render_form_section | form | **uses bound_data** |
| 26 | `card` | section_extra.rs / render_card_section | card | |
| 27 | `live-keys` | section_extra.rs / render_card_section | card | alias→card via `\|` |
| 28 | `test-keys` | section_extra.rs / render_card_section | card | alias→card via `\|` |
| 29 | `webhooks` | section_extra.rs / render_card_section | card | alias→card via `\|` |
| 30 | `links` | section_extra.rs / render_links_section | nav | |
| 31 | `quick-links` | section_extra.rs / render_links_section | nav | alias→links via `\|` (also alias map→links) |
| 32 | `tabs` | crate::tabs / render_tabs | layout | out of ui/ |
| 33 | `accordion` | crate::feedback / render_accordion | feedback | out of ui/ |
| 34 | `breadcrumb` | crate::navigation / render_breadcrumb | nav | out of ui/ |
| 35 | `alert` | crate::feedback / render_alert | feedback | out of ui/ — real banner |
| 36 | `chart` | section_chart.rs / render_chart_section | data | **uses bound_data** |
| 37 | `modal` | section_extra.rs / render_modal_section | overlay | |
| 38 | `sheet` | section_extra.rs / render_sheet_section | overlay | |
| 39 | `skeleton` | section_extra.rs / render_skeleton_section | feedback | |
| 40 | `loading` | section_extra.rs / render_skeleton_section | feedback | alias→skeleton via `\|` |
| 41 | `empty` | section_extra.rs / render_empty_section | feedback | |
| 42 | `error` | section_extra.rs / render_error_section | feedback | |
| 43 | `not-found` | section_extra.rs / render_not_found_section | feedback | |
| 44 | `404` | section_extra.rs / render_not_found_section | feedback | alias→not-found via `\|` |
| 45 | `kpi` | section_kpi.rs / render_kpi_section or render_kpi_dashboard_dark | data | **uses bound_data**; theme-branched |
| 46 | `timeline` | section_extra.rs / render_timeline_section | data | **uses bound_data** |
| 47 | `progress` | section_extra.rs / render_progress_section | data | **uses bound_data** |
| 48 | `command` | crate::command_palette / render_command_palette | overlay | out of ui/ |
| 49 | `table` | crate::data_table / render_data_table[_dark] | data | **uses bound_data**; theme-branched |
| 50 | `pagination` | crate::data_table / render_pagination | data | out of ui/ |
| 51 | `filters` | crate::data_table / render_filters_toolbar | data | out of ui/ |
| 52 | `dropdown` | crate::overlays / render_dropdown | overlay | out of ui/ |
| 53 | `toast` | crate::overlays / render_toast | feedback | out of ui/ |
| 54 | `notifications` | crate::overlays / render_notification_center | feedback | out of ui/ |
| 55 | `kanban` | crate::board / render_kanban | data | **uses bound_data**; out of ui/ |
| 56 | `dark-mode` | crate::board / render_dark_mode_toggle | control | out of ui/ |
| 57 | `layout` | crate::layout_system / render_layout_section or render_column_layout | layout | out of ui/; branched on `style` config |
| 58 | `_` (default) | section_extra.rs / render_generic_section | fallback | catches unknown types |

### Counts summary

- **Total distinct section-type strings accepted**: **58** (not 51).
- **Unique renderer functions invoked**: 47 (multiple strings share renderers).
- **Reachable arms after alias rewriting**: 58 − 14 DEAD = **44 reachable** string keys, mapping to:
  - 14 alias-rewrites that collapse into `kpi`, `card`, `table`, `form`, `features`, `links`, `alert` (canonical arms).
  - 5 inline `|` aliases (`live-keys`, `test-keys`, `webhooks`, `loading`, `404`, `quick-links`).
  - `command` also goes to `render_command_palette` exclusively.
- **Categorization** (by canonical target after alias resolution):
  - marketing: `hero`, `features` (+bento/edge/features-split aliases), `pricing`, `cta`, `faq`, `testimonial`, `trusted` → 7 canonicals
  - data: `kpi` (+stats/stat-cards aliases), `chart`, `table` (+activity-table/team-list), `timeline`, `progress`, `kanban`, `pagination`, `filters` → 8 canonicals
  - nav: `topbar`, `footer`, `sidebar`, `links` (+quick-links), `breadcrumb` → 5 canonicals
  - feedback: `alert` (+info-bar), `accordion`, `skeleton`/`loading`, `empty`, `error`, `not-found`/`404`, `toast`, `notifications` → 8 canonicals
  - layout: `page-header`, `tabs`, `layout` → 3 canonicals
  - form: `form` (+policies), `checkout` → 2 canonicals
  - card: `card` (+status-card/live-keys/test-keys/webhooks/promo/product-grid) → 1 canonical
  - overlay: `modal`, `sheet`, `dropdown`, `command` → 4 canonicals
  - control: `dark-mode` → 1 canonical
  - fallback: `_` → `render_generic_section`

**Canonical section types (post-alias)**: 39 distinct canonicals.

---

## 3. Fallback Behavior for Unknown Section Types

`_ => section_extra::render_generic_section(section, accent)` (mod.rs:758) is the only default arm.

- **`chart`** does NOT fall to the generic card — it has its own arm at line 709 and calls `section_chart::render_chart_section`, which renders real SVG charts (see §5).
- **`alert`** does NOT fall to the generic card either — it has its own arm at line 708 and calls `crate::feedback::render_alert`, which renders a real Tailwind-styled alert banner (see §6).

**Strict mode** (`STRICT_MODE` atomic) short-circuits BEFORE the match runs and returns a red `<div>` listing validation issues for unknown types if the contracts system reports them (mod.rs:644–653). Without strict mode, unknown types silently hit `render_generic_section`.

---

## 4. Sections Sharing a Renderer (Inline `|` Aliases)

Only five arms use `|` patterns, and they all route multiple strings to one function:

| Match line | Strings | Function |
|---|---|---|
| mod.rs:703 | `card`, `live-keys`, `test-keys`, `webhooks` | `section_extra::render_card_section` |
| mod.rs:704 | `links`, `quick-links` | `section_extra::render_links_section` |
| mod.rs:712 | `skeleton`, `loading` | `section_extra::render_skeleton_section` |
| mod.rs:715 | `not-found`, `404` | `section_extra::render_not_found_section` |

Additionally, the external `ContractRegistry::resolve_alias` performs *pre-match* rewrites (see §2 alias map). Those are not inline `|` aliases but achieve the same collapsing.

---

## 5. Chart Section Verification — REAL RENDERING

`section_chart.rs::render_chart_section` is **real**. It:

1. Pulls rows from `bound_data` (Rows variant) or falls back to static `section.items` (lines 11–37).
2. **Empty-data branch** (lines 39–149): renders a dark dashboard placeholder with real SVG (area type) or DOM bars (bar type). The area path is hand-coded Bézier points:
   ```rust
   let points = [(0,160),(30,140),(60,120),(90,135),(120,100), ... ];
   // C{cx},{prev_y} {cx},{y} {x},{y}
   path_line.push_str(&format!(" C{},{} {},{} {},{}", cx, prev.1, cx, y, x, y));
   ```
   and SVG output:
   ```rust
   <svg viewBox="0 0 600 200" preserveAspectRatio="none" ...>
     <defs><linearGradient id="area-grad" ... /><linearGradient id="line-grad" .../></defs>
     <path d="{area}" fill="url(#area-grad)" ... />
     <path d="{line}" fill="none" stroke="url(#line-grad)" ... />
     ...
   </svg>
   ```
3. **With-data branch** (lines 151–156): dispatches on `config.type`:
   - `"area" | "line"` → `render_chart_line()` (SVG polyline + fill gradient, lines 197–256)
   - `"donut"` → `render_chart_donut()` (SVG circles + stroke-dashoffset segments, lines 258–318)
   - else → `render_chart_bar()` (div bars with percent heights, lines 159–195)

   Example from `render_chart_line`:
   ```rust
   <svg viewBox="0 0 {width} {height}" style="width:100%;height:200px">
     <defs><linearGradient id="lineFill" .../></defs>
     <polyline points="{fill_pts}" fill="url(#lineFill)" .../>
     <polyline points="{polyline_pts}" fill="none" stroke="#000" stroke-width="2.5" .../>
     {circles}
   </svg>
   ```

**Verdict**: Chart produces real SVG data-viz markup (polylines, paths, circles, gradients). It is NOT a card fallback.

---

## 6. Alert Section Verification — REAL STYLED BANNER

`crate::feedback::render_alert` (feedback.rs:5–57) produces a real Tailwind-styled alert banner with icon + title + subtitle + close button:

```rust
let (bg, border, text, icon_color, icon_name) = match style {
    "success" => ("bg-emerald-50", "border-emerald-200", "text-emerald-800", "text-emerald-600", "check_circle"),
    "warning" => ("bg-amber-50", "border-amber-200", "text-amber-800", "text-amber-600", "warning"),
    "error"   => ("bg-red-50", "border-red-200", "text-red-800", "text-red-600", "error"),
    _         => ("bg-blue-50", "border-blue-200", "text-blue-800", "text-blue-600", "info"),
};
// ...
r#"<div class="cronus-alert flex items-start gap-3 p-4 rounded-xl border {bg} {border} {text}" role="alert">"#
```

With Material Symbols icon, title, subtitle, and an `onclick="this.closest('.cronus-alert').remove()"` close button. Properly tone-aware by `config.style` (info/success/warning/error).

**Verdict**: Alert is a real banner component, not a generic card.

---

## 7. Binding Integration (`resolve_binding` usage)

Only the following arms in `mod.rs` pass `bound_data` through to their renderers:

| Section type | Renderer | Uses `bound_data` |
|---|---|---|
| `form` | section_form::render_form_section | yes |
| `chart` | section_chart::render_chart_section | yes |
| `kpi` | section_kpi::render_kpi_{section,dashboard_dark} | yes |
| `stat-cards` (DEAD) | section_kpi::render_stat_cards | yes |
| `timeline` | section_extra::render_timeline_section | yes |
| `progress` | section_extra::render_progress_section | yes |
| `table` | data_table::render_data_table[_dark] | yes |
| `kanban` | board::render_kanban | yes |

`grep "bound_data" src/ui/` confirms the files touching bound data: `mod.rs`, `page.rs`, `section_kpi.rs`, `section_extra.rs`, `section_chart.rs`, `section_form.rs`.

**Marketing/nav/feedback sections do NOT receive bound_data** — this matches the architectural intent. `hero`, `features`, `pricing`, `cta`, `footer`, `alert`, `accordion`, `breadcrumb` etc. ignore `bound_data` entirely.

Outside the match, `mod.rs` wraps the result (lines 762–781) with data attributes whenever `bound_data` is non-empty:
```rust
"<div data-entity=\"{}\" data-bound-rows=\"{}\">{}</div>"
// or
"<div data-entity=\"{}\" data-bound-count=\"{}\">{}</div>"
```
So ALL sections with a binding get `data-entity` + `data-bound-rows|data-bound-count` wrappers regardless of whether their renderer uses the data — the outer wrapper is universal, but only 8 renderers *consume* the data.

---

## 8. Component System (`ui/component.rs`)

`component.rs` does **NOT** parse the `component Name(param: type) { state ... template "..." }` form. It is a layout-driven dispatcher for `ComponentNode` AST nodes produced by the parser:

```rust
pub fn render_component(comp: &ComponentNode) -> String {
    let layout = comp.layout.as_deref().unwrap_or("stack");
    let style = comp.style.as_deref().unwrap_or("");
    match layout {
        "inline" => render_inline_component(comp, style),    // Button, Badge
        "stack"  => render_stack_component(comp, style),     // StatCard, Empty, Card, Alert
        "grid"   => render_grid_component(comp, style),      // PricingGrid
        "table"  => render_table_component(comp, style),
        "hero"   => render_hero_component(comp, style),
        "modal"  => render_modal_component(comp, style),
        "sidebar"=> render_sidebar_component(comp, style),
        "tabs"   => render_tabs_component(comp, style),
        "menu"   => render_menu_component(comp, style),
        _        => render_stack_component(comp, style),
    }
}
```

`render_stack_component` branches on `style` substring:
- `metric`/`stat` → `components::stat_card`
- `empty` → `components::empty_state`
- `alert` → `components::alert`
- `command` → custom card with keyboard shortcut rows
- else → generic card via `components::card`

The component system is a **preset-library dispatcher** keyed on the parsed `layout` + `style` attributes, not a parameterized template engine. There is **no parameter-passing (`param: type`)**, **no `state {}` block handling**, and **no embedded `template "..."` string** within `component.rs`. Searching the file for `state` and `template` finds only CSS `grid-template-columns` matches and the `empty_state` function name — no component-local state or templating.

Module also contains `render_light_app_page` (light theme Payouts page) and `render_components_page` (iterates `render_component` over a slice).

---

## 9. Live (SSE) Support

`live true` on a section's binding **is** wired up. Lines 784–821 of `mod.rs`:

```rust
let is_live = section.binding.as_ref().map(|b| b.live).unwrap_or(false);
let output = if is_live {
    let entity = section.binding.as_ref().map(|b| b.entity.as_str()).unwrap_or("");
    let live_id = format!("live_{}", entity.to_lowercase());
    format!(
        r#"<div id="{live_id}" data-live-entity="{entity}">{output}</div>
<script>
(function(){{
  var el=document.getElementById('{live_id}');
  if(!el)return;
  var es=new EventSource('/api/sse');
  es.addEventListener('data_change',function(e){{
    try{{
      var d=JSON.parse(e.data);
      if(d.entity==='{entity}'){{
        if(window.__cronusNavigate){{
          window.__cronusNavigate(location.href,false);
        }}else{{
          location.reload();
        }}
      }}
    }}catch(err){{}}
  }});
  es.onerror=function(){{
    es.close();
    setTimeout(function(){{
      var script=document.createElement('script');
      script.textContent='('+arguments.callee.caller.toString()+')()';
    }},3000);
  }};
}})();
</script>"#,
        ...
    )
} else { output };
```

**Behavior**:
- When `section.binding.live == true`, the rendered section is wrapped in `<div id="live_{entity}" data-live-entity="{entity}">` plus an inline `<script>` that opens an `EventSource('/api/sse')`.
- On a `data_change` SSE event whose JSON payload matches the entity, it calls `window.__cronusNavigate(location.href, false)` (SPA re-render) or falls back to `location.reload()`.
- The `onerror` reconnect block attempts reconnection after 3 s, though its implementation using `arguments.callee.caller.toString()` is buggy (it relies on a non-strict-mode-only feature and doesn't actually execute the new script — only sets `textContent`). Reconnect is effectively broken but the initial wiring is real.
- Key: the SSE refresh strategy is **full page / full SPA re-render**, not targeted DOM patching. It is real but coarse.

No usages of `data-sse` attribute were found. The pattern is `data-live-entity` + `EventSource('/api/sse')`.

**Verdict**: `live true` IS wired up in the HTML output. The SSE stream endpoint is `/api/sse`, events are filtered by `entity`, and reloads are triggered via the CRONUS SPA router (or fallback `location.reload`).

---

## 10. Grand Count vs. Claims

| Source | Claim | Reality |
|---|---|---|
| AGENTS.md | "51-way match" | Actual arms: **52** (51 explicit + 1 `_` default). Actual distinct strings: **58**. |
| Docs homepage | "51 section types" | **58** distinct type strings accepted by dispatcher. **39** canonical types after alias collapse. |
| Docs `/components` index | "45 base + 10 aliases" (55 total) | Reality: **39 canonical + 19 aliases = 58**. Neither the base count (45) nor the alias count (10) matches. |

### Why the numbers differ
- The dispatcher accepts **14** strings that alias resolution silently rewrites to canonicals — those arms are dead code kept for backwards compat but still contribute string keys visible to users/parsers.
- Five inline `|` arms bundle **11** extra strings onto **5** canonical renderers.
- The default `_` arm catches anything else via `render_generic_section`, so there is no hard cap — undocumented names silently render a generic card.
- AGENTS.md's "51-way" is close to the line count of explicit arms but doesn't distinguish strings vs. arms vs. canonicals, and misses the `_` default.

### Authoritative numbers
- **Explicit match arms**: 51 (plus `_` default → 52 total).
- **Distinct section type strings handled**: 58.
- **Distinct renderer functions invoked**: ~47.
- **Canonical section types (post-alias collapse)**: 39.
- **Dead-code arms (shadowed by alias-map)**: 14.
- **Default fallback**: yes → `render_generic_section` (generic marketing-ish card).

---

## Appendix: Renderers residing outside `src/ui/`

The dispatcher delegates these types to other modules — outside the scope of this file-set verification but visible from `mod.rs`:

- `crate::tabs::render_tabs`
- `crate::feedback::{render_accordion, render_alert}` (inspected in §6)
- `crate::navigation::render_breadcrumb`
- `crate::command_palette::render_command_palette`
- `crate::data_table::{render_data_table, render_data_table_dark, render_pagination, render_filters_toolbar}`
- `crate::overlays::{render_dropdown, render_toast, render_notification_center}`
- `crate::board::{render_kanban, render_dark_mode_toggle}`
- `crate::layout_system::{render_layout_section, render_column_layout}`

These exist, but this verification did not audit their internals beyond confirming the dispatch path.
