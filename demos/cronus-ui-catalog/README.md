# Cronus UI catalog

Native `.cronus` shelf. Source is only `app.cronus`. The kernel emits HTML, `--cronus-*` tokens, and motion.

## Run

From the kernel repo:

```bash
cargo build
cd demos/cronus-ui-catalog
../../target/debug/cronus run
```

Open **IPv4**, not `localhost`:

- http://127.0.0.1:5311/ — landing
- http://127.0.0.1:5311/kit — every declared widget

The kernel binds IPv4 only. `localhost` often resolves to `::1` and the browser shows connection refused.

Edit `app.cronus` while `cronus run` is up — the process hot-reloads.

## Docs site

The Cronus UI site documents this catalog at `/language`:

```bash
cd cooud-ui
bun run www
# http://127.0.0.1:4747/language
```

That page proxies `/kit` at `/language/kit` so the iframe is same-origin.

## Rules

- No JSX, HTML, CSS, `template`, or `style_block` in `.cronus`
- Always `style:<family>+<variant>` (`button+primary`). `style:primary` alone is legacy Obsidian
- `page "/kit" type:components` dumps every `component` in the file
