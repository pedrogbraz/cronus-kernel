# Cronus UI catalog

Every cronus-ui family declared in `.cronus`, one file per family, with every
example of the React docs: variants, sizes, states and motion. Source is only
`.cronus`: no JSX, no imports. The kernel emits the HTML, the `--cronus-*`
tokens and the CSS motion.

## Run

From the kernel repo:

```bash
cargo build --locked --bin cronus
cd demos/cronus-ui-catalog
../../target/debug/cronus run
```

Open **IPv4**, not `localhost` (the kernel binds `127.0.0.1`; `localhost` may
resolve to `::1`):

- http://127.0.0.1:5311/ — landing
- http://127.0.0.1:5311/kit: every family, grouped like the React docs
- http://127.0.0.1:5311/button: one family: each docs example with its
  specimens and the `.cronus` that declared them (`/toggle`, `/select`, etc.)

Edit any `.cronus` while `cronus run` is up; the process hot-reloads.

## Layout

```
app.cronus        app, theme (neutral, dark) and the / and /kit pages
<family>.cronus   page "/<family>" type:components family:<family> + components
```

A family file mirrors the docs page of that component:

```cronus
page "/button" type:components family:button {
  title "Button"
  description:"Clickable action with variants, sizes and asChild."
}

component Primary layout:inline style:button+primary group:"Variants" note:"Six visual styles" { label "Primary" }
component Icon layout:inline style:button+icon icon:settings group:"Sizes" { label "Settings" }
component ReadDocs layout:inline style:button icon-end:arrow-right group:"As link" { label "Read the docs" -> "/docs" }
```

- `style:<family>+<variant>+<size>` carries the React variant props.
- `group:"..."` is the docs example the specimen belongs to (declaration order);
  `note:"..."` its description. Both are page props; renderers never read them.
- `icon:` / `icon-end:` and `item ... icon:` take lucide ids (`arrow-right`).
- Overlays declare `trigger:"..."` so they render closed and open natively.

## React vs `.cronus`

| React | `.cronus` |
|---|---|
| `<Button variant="outline" size="sm"><Heart />Like</Button>` | `component Like style:button+outline+sm icon:heart { label "Like" }` |
| `<Toggle variant="outline" pressed>Italic</Toggle>` | `component Italic style:toggle+outline pressed:true { label "Italic" }` |
| `<ToggleGroup type="multiple"></ToggleGroup>` | `component Formats style:toggle-group type:multiple { item "Bold" icon:bold }` |
| `<Select><SelectGroup><SelectLabel>Europe` | `item "Ireland" group:"Europe"` |
| `<Slider value={[20, 80]} />` | `component Price style:slider value:"20,80" { label "Price range" }` |
| `<Chip variant="soft" color="success">` | `item "Soft" variant:soft color:success` |
| `<Fab actions={[{ icon: <Pencil />, label: "Write a note" }]} />` | `action "Write a note" icon:pencil` |
| `<Sheet><SheetTrigger>Open</SheetTrigger>` | `component Filters style:sheet trigger:"Open" { title "Filters" }` |

## Rules

- No JSX, HTML, CSS, `template` or `style_block` in `.cronus`
- Always `style:<family>+<variant>` (`button+primary`); `style:primary` alone is the legacy Obsidian button
- Validate with `../../target/debug/cronus build --ai` (must print `"valid": true`)
