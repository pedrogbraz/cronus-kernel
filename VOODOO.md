# VOODOO.md — Cronus × Voodoo.js

> **Read this before emitting Voodoo, writing JSX, or “adding interactivity” to a `.cronus` app.**
> Compact ingest for Claude/Grok: **`llms.txt`** at kernel root (paste that first).
> Language spec: `LANGUAGE.md`. Kernel rules: `AGENTS.md`. This file is the full contract for the **opt-in HTML runtime**.
>
> Verified against `src/voodoo.rs`, `src/cronus_ui_interact.rs`, `src/cronus_ui_widgets.rs` (2026-09).

---

## 1. The law

**Cronus is the language. Voodoo is the runtime of the HTML the kernel emits.**

| Layer | Owns | Lives in |
|---|---|---|
| **Cronus** | `app`, `entity`, `page`, `section`, `component`, `style`, `bind`, `auth`, tokens, widgets | `.cronus` source |
| **Kernel** | parse → AST → SQL/API/HTML | `cronus-kernel` Rust |
| **Voodoo.js** | reactivity on the **already-rendered** DOM (`v-data`, `v-model`, `@click`, `{ expr }`) | one `<script>` tag in the **output** HTML |

`.cronus` is never HTML, never JSX, never CSS. `cronus run` is what produces HTML. Voodoo starts **after** that.

Upstream: [kwy404/Voodoo.js](https://github.com/kwy404/Voodoo.js) · docs: https://kwy404.github.io/Voodoo.js/docs/

---

## 2. Opt-in (either is enough)

```cronus
app "Cooud" {
  stack voodoo
  port 4747
}

style {
  theme dark
  preset aurora
  runtime voodoo
}
```

Parser already accepts both:

- `app { stack voodoo }` → `AppNode.stack` contains `"voodoo"` (case-insensitive)
- `style { runtime voodoo }` → `StyleNode.config["runtime"] == "voodoo"` (case-insensitive)

`src/voodoo.rs::wanted()` is the single predicate. Default is **off**. Obsidian demos, `style:primary`, `--foreground` must keep working with zero Voodoo on the page.

Pinned CDN (full build, injected only when opted in):

```
https://cdn.jsdelivr.net/npm/voodoojs@0.13.0/dist/voodoo.full.min.js
```

Tag shape (kernel, not authoring):

```html
<script src="https://cdn.jsdelivr.net/npm/voodoojs@0.13.0/dist/voodoo.full.min.js"
        data-cronus-runtime="voodoo" defer></script>
```

CSP already allows `https://cdn.jsdelivr.net` (`src/security.rs`). Injection is idempotent (`data-cronus-runtime="voodoo"` or `voodoojs@` already present → skip). Happens in `html_response` (`src/server/response.rs`) **inside** a tokio task-local scope set in `main.rs::handle_request`. Do not use a process-wide `AtomicBool` — `.await` would leak the flag onto another request.

---

## 3. Never do this

These are the failures this contract exists to prevent.

1. **Do not write JSX / HTML / CSS in a `.cronus` file.** Not in `template`, not in `style_block`, not in a string that the parser will treat as Cronus. The conversion pack (`kwy404/cooud-cronus`) and Zedd’s rule: *apenas `.cronus` por trás*.
2. **Do not make Voodoo the language.** `component Counter { template "<button @click>…"` is still **not implemented** (`LANGUAGE.md` §6). Do not “finish” it by stuffing JSX into the parser.
3. **Do not emit `{ count }` unless the runtime is on.** Without the script, the user sees the literal characters `{ count }`. Always `voodoo::interp("count", "0")`.
4. **Do not emit `v-data` / `v-model` / `@click` / `v-show` / `:attr` unless `voodoo::enabled()`.** Use the helpers. Hand-writing the attributes bypasses the gate.
5. **Do not replace legacy Button.** `style:primary` (no `button+`) stays Obsidian (`uppercase`, `--foreground`). Cronus-ui Button only when the first `style` segment is `button`.
6. **Do not force `preset aurora` / `data-cronus-theme` on pages that did not ask.** `token_css("legacy")` only emits `--cronus-*` fallback aliases.
7. **Do not vendor a second runtime** (Alpine, petite-vue, React, HTMX) for the same job. Voodoo is the opted-in interaction layer.
8. **Do not edit `src/server/router.rs` or `src/server/api.rs`.** Dead code, never compiled (`AGENTS.md`).

---

## 4. How to author a `.cronus` app (the only public API)

Agents writing **apps** (not the kernel) stay in Cronus syntax.

```cronus
app "Billing" {
  stack voodoo
  port 4747
}

style {
  theme dark
  preset aurora
  runtime voodoo
  accent-hex "#0ea5e9"
}

component Email layout:stack style:input {
  label "Email"
  text "you@cooud.app"
}

component Agree layout:inline style:checkbox {
  label "I agree"
}

component Volume layout:stack style:slider {
  label "Volume"
  value "40"
}

component Nav layout:stack style:tabs {
  tab "Overview"
  tab "Usage"
  tab "Settings"
}

page "/" type:custom {
  section stats {
    title "Settings"
  }
}
```

Rules for this side:

- Family = first `style` segment: `style:input`, `style:button+primary+md`, `style:dialog`.
- Copy lives in official item types: `label`, `text`, `title`, `value`, `item`, `tab`, `columns`, …
- Native HTML is what the kernel emits. With Voodoo on, those controls also get `v-model` / `v-data`.
- Demo: `demos/cronus-ui/widgets.cronus`. Parse check: `cargo run -- parse demos/cronus-ui/widgets.cronus`.

There is **no** Cronus syntax for `@click="count++"` or `{ count }` today. If a widget needs a live value, extend **the kernel renderer** (next section), not the `.cronus` grammar.

---

## 5. How to extend the kernel (agents editing Rust)

### 5.1 Helpers — always use these

`src/voodoo.rs` (the only module allowed to know Voodoo attribute names):

| Helper | Off | On |
|---|---|---|
| `wanted(stack, style)` | predicate | — |
| `enabled()` | `false` | `true` (task-local) |
| `with_enabled(on, f)` | sync tests | — |
| `scope(on, fut)` | async request | wraps `handle_request_inner` |
| `data("{ n: 0 }")` | `""` | ` v-data="{ n: 0 }"` |
| `model("n")` | `""` | ` v-model="n"` |
| `click("n++")` | `""` | ` @click="n++"` |
| `show("n > 0")` | `""` | ` v-show="n > 0"` |
| `bind("value", "n")` | `""` | ` :value="n"` |
| `interp("n", "0")` | `"0"` | `{ n }` |
| `inject_into_html(html)` | identity | script before `</head>` |
| `SCRIPT_SRC` | pinned CDN | do not un-pin without a reason |

In `format!`, a Voodoo interpolation is `{{ n }}` in the format string (Rust escape) **or** `voodoo::interp(...)`. Never `format!("{ n }")` — that is a Rust placeholder.

### 5.2 Where HTML is born

```
component style:input
        │
        ▼
ui/component.rs::render_component
        │
        ├─ cronus_ui_widgets::render          family = first style segment
        │         │
        │         ├─ cronus_ui_interact::render   native controls + gated Voodoo attrs
        │         └─ generic pill/field/overlay/chart/fx fallback
        │
        └─ legacy layout dispatchers          style:primary, style:metric, …
                                              (interact/widgets returned None)
```

Unknown family → `None` → legacy path. `style:primary` must keep returning `None` from `cronus_ui_widgets::render`.

### 5.3 Adding a widget behaviour

1. Prefer **native HTML that works with Voodoo off** (`<input type=checkbox>`, `<dialog>`, `<details>`, `<progress>`, `<table>`, tablist + `onclick`).
2. Add Voodoo attrs only through the helpers.
3. Put the renderer in `src/cronus_ui_interact.rs` (not the generated match in `cronus_ui_widgets.rs`).
4. If you regenerate widgets: `scripts/gen_cronus_ui_widgets.py` must keep the `cronus_ui_interact::render` call at the top of `render()`.
5. Tokens: only `var(--cronus-*)`. No palette scales (`zinc-900`, `bg-neutral-`). See `docs/adr/0001` in cronus-ui if you need the why.
6. `data-slot="<family>"` on the root. Focus-visible stays real.
7. Tests in the same module or in `cronus_ui_widgets::tests`:
   - off: no `v-data`, no `{ value }`, real tag still present
   - on: `with_enabled(true, \|\| { … })` sees `v-data` / `v-model`
   - `style:primary` still `None`
   - no `zinc-`

### 5.4 What interact already emits (Voodoo-gated)

| Family (style slug) | Native tag | Voodoo when on |
|---|---|---|
| `checkbox` | `<input type=checkbox>` | `v-data="{ checked: false }"` + `v-model="checked"` |
| `switch`, `toggle` | `role=switch` | `v-model="on"` |
| `radio-group`, `toggle-group`, `segmented-control` | `role=radiogroup` | `v-model="value"` |
| `slider` | `type=range` | `v-model="value"` + `{ value }` |
| `progress`, `usage-meter`, `scroll-progress` | `<progress>` | `:value="value"` + `{ value }` |
| `rating` | radio stars | `v-model="value"` |
| `input` + typed variants | `<input type=…>` | `v-model="value"` |
| `textarea`, `rich-text-editor` | `<textarea>` | `v-model="value"` |
| `select`, `combobox`, `autocomplete`, `multi-select`, `tags-input` | `<select>` | `v-model="value"` |
| `dialog`, `alert-dialog`, `sheet`, `drawer`, … | `<dialog>` + `showModal()` | native first (works off) |
| `tabs`, `code-tabs`, `expandable-tabs` | `role=tablist` | `v-data="{ tab: 0 }"`, `@click`, `v-show` |
| `accordion`, `collapsible` | `<details>` | native (no JS required) |
| `table`, `data-table` | `<table>` | — |
| `form`, `field` | `<form>` | native submit |

Charts / FX families stay in the generic `chart()` / `fx()` fallbacks (`cronus_ui_widgets.rs`). Button stays `cronus_ui::button_ex` (CONTRACT), not interact.

---

## 6. Voodoo surface that exists in the CDN (do not paste into `.cronus`)

The full build is loaded. That means the **emitted HTML** may use anything in Voodoo 0.13. Agents extending **kernel HTML** may use these. Agents writing **`.cronus` may not type them**.

State / DOM: `v-data`, `v-model`, `v-if` / `v-else-if` / `v-else`, `v-show`, `v-for`, `:class`, `:style`, `:attr`, `@click`, `{ expr }` (single brace).

HTTP (declarative, on the emitted node): `v-get`, `v-post`, `v-put`, `v-patch`, `v-delete`, `v-resource`, `v-submit`, `v-confirm`, `v-toast-success`.

JS API on the page: `V.reactive`, `V.http`, `V.store`, `V.component`, `V.router`. Cronus still owns routing, auth, entities, and money-as-centavos. Do not invent a second router in Voodoo for a Cronus app.

Interpolation: `{ name }` is Voodoo. `{{ name }}` is also accepted by Voodoo. Cronus kernel always emits the single-brace form via `interp()`.

Docs: https://kwy404.github.io/Voodoo.js/docs/referencia/directives.html

---

## 7. Dual theme (do not reopen)

```
legacy path (default)          cronus-ui path (opt-in)
-------------------------      --------------------------------
style:primary                  style:button+primary+md
Obsidian / --foreground        --cronus-* from preset aurora|neutral|midnight|sunset|emerald
no Voodoo script               Voodoo script only if stack/runtime voodoo
demos/saas-billing, stitch-*   demos/cronus-ui/button.cronus, widgets.cronus
```

`token_css("legacy" | "obsidian" | "default" | "")` → fallback aliases only.
A named preset also applies on `:root` **for that page**, because the author wrote `preset …`.

---

## 8. File map

```
llms.txt                        compact LLM ingest (paste this first)
VOODOO.md                       this contract
LANGUAGE.md                     §6 mentions the runtime; this file is the full contract
AGENTS.md                       kernel rules; points here

src/voodoo.rs                   opt-in flag, helpers, CDN, inject, tests
src/cronus_ui_interact.rs       native HTML + gated attrs
src/cronus_ui_widgets.rs        173 families; calls interact first
src/cronus_ui.rs                tokens + CONTRACT Button
src/ui/component.rs             dispatch: widgets then legacy layouts
src/main.rs                     mod voodoo; scope() around handle_request_inner
src/server/response.rs          inject_into_html in html_response
scripts/gen_cronus_ui_widgets.py  keep the interact call if you regenerate
demos/cronus-ui/widgets.cronus  opt-in example
demos/cronus-ui/button.cronus   tokens + Button, no Voodoo required
```

---

## 9. Tests an agent must not break

```
cargo test cronus_ui
cargo test voodoo
```

Minimum:

- `registers_173_unique_families`
- `every_family_renders_slot_without_palette_scales`
- `legacy_primary_style_is_not_hijacked`
- `interactive_families_emit_real_controls` (real tags, **no** `v-data` off)
- `voodoo_attrs_only_when_runtime_on`
- `voodoo_off_does_not_inject_script`
- `parser_stack_and_style_runtime`
- `components::legacy_button_tests::legacy_button_unchanged`
- `ui::layout::tests::render_layout_declarative_has_no_hardcoded_brand_colors`

Full suite: `cargo test`. One known environmental fail on machines that are not Zedd’s Linux box: `dump::detect::tests::test_hero_extraction_developer_landing` reads `/home/zedd/Downloads/…`. Unrelated to Voodoo. Do not “fix” it by deleting dump tests.

---

## 10. Recipe: new live control

You want a cronus-ui `counter` that increments.

**Wrong:** add JSX to `.cronus` or a `template` string with `@click`.

**Right:**

1. `.cronus` (authoring):

```cronus
app "X" { stack voodoo }
component Hits layout:inline style:button+primary {
  label "Hits"
}
```

2. Kernel (`cronus_ui_interact.rs` or a dedicated arm): emit native `<button>` that works off, and when `enabled()` wrap with `voodoo::data("{ n: 0 }")` + `voodoo::click("n++")` + `voodoo::interp("n", "0")`.
3. Test off (static label, no `{ n }`) and on (`v-data`, `{ n }`).
4. `cargo test cronus_ui voodoo` equivalent filters, then `cargo test`.

Cronus still owns the entity/API if the increment must persist: `@click` updates the DOM; a `v-post` / Cronus action persists. Do not store canonical money or auth state only in `v-data`.
