# A5 — CRONUS Monorepo: Sibling Projects Inventory

Scope: `/home/zedd/Documentos/CRONUS/` — all siblings of `cronus-kernel/`.
Method: metadata only (package.json, Cargo.toml, tauri.conf.json, git state, mtimes). No code changes, no builds.
Date: 2026-04-10

---

## 1. Executive Summary

- **Not a real monorepo.** No root `package.json`, no `pnpm-workspace.yaml`, no `turbo.json`, no Cargo workspace. The CRONUS root is ONE git repo containing ~70 top-level directories plus scattered docs; `daemon/` and `dashboard/` have their own nested `.git/` dirs (embedded repos) that the root repo also tracks loosely.
- **The core shipping artifact is `cronus-kernel/` (Rust binary).** Everything else is either (a) tooling around the `.cronus` language, (b) an old UI experiment, or (c) unrelated cooud archives (`repos/cooud-archives/*`).
- **Two parallel "observer" stacks are being built at once:** the old `daemon/` (Elysia + Bun, spawns Claude/Codex processes) and the new `cronus-nexus-v2/` Observer Engine (Bun + SQLite + knowledge graph). They overlap in purpose and both live on disk simultaneously with no clear deprecation.
- **Massive git hygiene debt.** Root repo: 170 deletions, 450 untracked, 58 modifications. Nested `dashboard/` repo: 254 deletions, 19 untracked, 12 modified — the deletions are a reorg where `dashboard/src/` was flattened from a nested `dashboard/dashboard/*` layout.
- **Multiple versioned siblings with no migration complete:** `dashboard/` vs `dashboard-v2/`, `cronus-nexus/` vs `cronus-nexus-v2/`, `docs/site/` vs `docs/site-v2/`. In every pair, the v1 is still wired up and the v2 is less complete; current work continues on v1 in two of three cases.

---

## 2. Project Inventory

Only in-scope siblings are listed in detail. The `CRONUS/` root also contains ~55 other directories (landings, archives, test scratch, `repos/cooud-archives/*`) — grouped at the bottom.

### 2.1 `cronus-kernel/` — the Rust compiler/runtime
- **What**: The `.cronus` declarative language compiler + HTTP server runtime. A 7MB single Rust binary that parses `app.cronus`, renders HTML/JS, serves pages, handles auth + DB + actions.
- **Stack**: Rust 2021, `tokio`, `hyper`, `rusqlite`, `argon2`, `jsonwebtoken`, `scraper`. Release profile: opt-level=z, LTO, strip, panic=abort. Package name `cronus-lang`, bin `cronus`.
- **Status**: ACTIVE — latest commit `2235667 feat: dump 8 production open-source apps to .cronus templates`. Modified files in root-repo status (13 files in kernel).
- **Relationship**: **The root producer.** Every other sibling is upstream tooling (language extension, grammar, docs) or downstream consumer (nexus desktop shell, daemon pattern extractor references `cronus-kernel-*` IDs).
- **Git**: Dirty via root repo (13 kernel files modified, no separate `.git`).
- **Builds?**: Assumed OK — no obvious breakage in Cargo.toml. `src/ui.rs.bak` hints at an in-progress UI refactor.
- **Last change**: 2026-04-09/10 (src/ mtimes).

### 2.2 `daemon/` — Bun/Elysia orchestrator (has own `.git`)
- **What**: HTTP daemon on port 4800. Orchestrates spawning `claude`/`codex`/`openai` CLI processes via `src/spawner/`, exposes Elysia routes (`agents`, `chat`, `chat-smart`, `execution`, `flow-studio`, `user-projects`), runs Hydra auth, injections system (affiliate-tracker, auth, coupon-engine, data-persistence, queue-processing, review-system, notifications, ai-content, integrations, loyalty-program, agents-orchestration). Includes `src/kernel/`, `src/pipeline/`, `src/dsl/`, `src/memory/`, `src/watchers/`, `src/neurons-server.ts`.
- **Stack**: Bun + TypeScript, Elysia 1.4, Drizzle ORM 0.45, postgres 3.4, puppeteer-core + stealth plugin, amqplib, zod. Docker + docker-compose present.
- **Status**: ACTIVE — heavy modifications across routes, spawner, hydra, injections, kernel. Latest commit `3e56563 feat(kernel): Feature Factory — block hierarchy, entity model, composition rules, DSL, pipeline` (own repo).
- **Relationship**: **Consumer of `cronus-kernel`** (pattern extractor references kernel template names) but otherwise an independent TS runtime. Dashboard/dashboard-v2 speak to it via proxy (`/api → http://localhost:4800`).
- **Git**: Own `.git`. 85 untracked, 54 modified. Dirty.
- **Builds?**: `bun run build` = `bunx tsc --noEmit`. No build output check was run; imports look consistent. `users.db` is an empty file (0 bytes) which may break persistence.
- **Claude spawner confirmed**: `daemon/src/spawner/claude.ts` extracts streaming events from Claude CLI (type assistant/tool_use/tool_result), broadcasts via WS. Sibling files: `claude-cli-proxy.ts`, `openai.ts`, `openai-auth.ts`, `openai-tools.ts`, `process-pool.ts`.
- **Last change**: 2026-04-09 14:59.

### 2.3 `dashboard/` — React UI for daemon (has own `.git`)
- **What**: Vite + React 19 SPA on port 4803 (proxies `/api` to daemon at 4800). Pages include `CodeV2`, `Hydra`, `BrainHealth`, `BrainLogs`, `BrainUI`, `Documentation`, `GalaxyView3D`, `HydraAgents`, `NeuronsPage`, `OfficeGame`, `Terminal`, `ThinkPredict`, `UsersPage`, `WorkspacesTable`. Has `CodexChat`, flow studio (`@xyflow/react`), three.js.
- **Stack**: Vite 8, React 19.2, Tailwind v4, @tanstack/react-query, framer-motion, three + @react-three/fiber + drei + postprocessing, @xyflow/react. Also carries a merged copy of daemon backend (`src/api/`, `src/daemon.ts`, `src/hydra/`, `src/injections/`, `src/kernel/`, `src/spawner/`) — pkg name is `cronus-compiler-engine`, so this directory is being repurposed into a combined frontend+backend.
- **Status**: ACTIVE, but in the middle of a BIG reorg. Latest commit `45219e6 feat: Feature Compiler Engine + Voice Chat + Studio UX overhaul`.
- **Relationship**: Consumer of daemon (HTTP proxy), consumer of cronus-kernel (dumps viewer). Now ALSO absorbing daemon code.
- **Git**: Own `.git`. 254 deletions, 19 untracked, 12 modified. The 254 `D` entries are all from a previous `dashboard/dashboard/*` nested layout being flattened to `dashboard/src/*`. Also has `.git-backup/` directory — user cloned the `.git` as a safety net. Dangerous state.
- **Builds?**: Likely broken — the merge of daemon backend into a Vite React project means Vite will try to bundle node modules (`postgres`, `drizzle-orm`, `elysia`, `amqplib`, `puppeteer-core`). `package.json` lists all these as deps but no SSR/node-only build pipeline is visible. Expect lots of Vite errors.
- **Last change**: 2026-04-09 14:43.

### 2.4 `dashboard-v2/` — abandoned Next.js attempt
- **What**: Earlier attempt at a second dashboard UI. Has a stale `.next/` build dir (Next.js) but `package.json` scripts say `vite` — clear evidence of a failed Next→Vite migration, never finished.
- **Stack (claimed)**: Vite 8, React 19.2, Tailwind v4, @xyflow, framer-motion, lucide, class-variance-authority. Minimal src/ (`app/`, `components/`, `lib/`, `styles/`).
- **Status**: STALE / borderline ABANDONED. Src mtime 2026-04-07 (untouched since); no commits in its own history (part of root repo). Only 2 files modified in root git status.
- **Relationship**: None active — was to be a `dashboard` replacement. Superseded by work happening INSIDE `dashboard/`.
- **Git**: No own `.git`, tracked by root repo. Nearly clean (2 M files).
- **Builds?**: Unknown. Stale `.next/` will confuse anyone running it fresh.
- **Last change**: 2026-04-07 17:05.

### 2.5 `cronus-nexus/` — Tauri desktop shell v1
- **What**: Desktop app wrapping the CRONUS kernel/daemon. Tauri 2 with `portable-pty` (spawns terminal), Monaco editor, React 19 + zustand + tailwind v4. CSP allows `localhost:4800` + `localhost:4810` — connects to daemon. Has SecurityPanel component.
- **Stack**: Tauri 2 (Rust: tauri-plugin-shell/fs/dialog, portable-pty 0.8), Vite 6, React 19.1, @monaco-editor/react, zustand, tailwind v4, react-markdown.
- **Status**: STALE. Last src-tauri lib.rs mtime 2026-04-07; App.tsx mtime 2026-04-03. Modified files in root git status (6 files) but no commits flowing.
- **Relationship**: Desktop wrapper around daemon (port 4800). Currently the "live" nexus the user has on disk with working builds.
- **Git**: Root repo (6 M files).
- **Builds?**: Likely OK — had working `dist/` output (mar 29). CSP and Tauri config are sane.
- **Last change**: 2026-04-07.

### 2.6 `cronus-nexus-v2/` — Observer Engine (new design)
- **What**: Per `ARCHITECTURE.md`, a two-layer design: (1) a headless "CRONUS Observer Engine" daemon that runs 24/7 capturing Claude sessions, git commits, file changes, server probes, feeding a SQLite knowledge graph with embeddings + correlator + brain bridge; (2) an optional Nexus UI that consumes it. Exposes HTTP on :4802. src/ has `api/`, `brain-bridge/`, `core/`, `graph/`, `ingestion/`, `rag/`, `remote/`, `router/`, `ui/`. Package name `cronus-nexus-v2`, runs via `bun run src/index.ts`.
- **Stack**: Bun + TypeScript, `@xenova/transformers` (embeddings), `better-sqlite3`, `uuid`. `src-tauri/` directory exists but contains ONLY `tauri.conf.json` — Rust side not scaffolded yet.
- **Status**: WIP. Heavy activity — src/index.ts mtime 2026-04-10 01:59, vault/, data/, scripts/ all recent. Has SDD.md, ARCHITECTURE.md, SESSION-HANDOFF.md.
- **Relationship**: Intended REPLACEMENT for `cronus-nexus/` AND complement to `daemon/` — overlaps with both. Could eventually consume cronus-kernel for Knowledge Graph hydration.
- **Git**: Root repo, mostly untracked (new dir).
- **Builds?**: No build step; `bun run` the TS directly. Unknown if `src/index.ts` is complete.
- **Last change**: 2026-04-10 01:59.

### 2.7 `cronus-vscode/` — VS Code language extension
- **What**: Syntax highlighting, snippets (18), bracket matching for `.cronus` and `.scriptcronus` files. Publisher `cronus-lang`. No `main` entry point — pure declarative contribution.
- **Stack**: VS Code extension manifest only. TextMate grammar in `syntaxes/`, snippets JSON in `snippets/`, language-configuration.json files.
- **Status**: ACTIVE — freshly created/updated 2026-04-09. Clean.
- **Relationship**: Tooling for cronus-kernel language. Independent.
- **Git**: Root repo, untracked (new).
- **Builds?**: N/A (no build step).
- **Last change**: 2026-04-09 23:12.

### 2.8 `tree-sitter-cronus/` — tree-sitter grammar
- **What**: 8.2KB `grammar.js` defining the `.cronus` grammar for tree-sitter. Queries in `queries/`, generated C parser in `src/`.
- **Stack**: tree-sitter-cli 0.22, nan bindings for Node.
- **Status**: WIP — fresh (created 2026-04-09 23:00-23:01). Not wired into cronus-vscode yet (which still uses a TextMate grammar).
- **Relationship**: Future replacement for TextMate grammar in cronus-vscode; also could be used by tree-sitter-based tools (nvim, Zed, etc.).
- **Git**: Root repo, untracked.
- **Builds?**: Needs `tree-sitter generate` before `src/` is usable.
- **Last change**: 2026-04-09 23:01.

### 2.9 `docs/` — documentation (NOT a docs site)
- **What**: Two parallel things in one folder:
  1. Markdown authoring content: `getting-started/`, `guides/`, `reference/`, `_archive/`, `internal/`, `actions.md`, `language-reference.md`, `sections.md`, etc. Plus SDDs (`SDD-AI-*`, `SDD-INTERACTIVE-COURSES.md`, `DOCS-SPEC.md`).
  2. `docs/site/` and `docs/site-v2/` — these are **NOT VitePress/Astro/mdBook**. They are CRONUS apps. Each contains an `app.cronus` file + `data.db` + `.db-shm/.db-wal`. The docs site is self-hosted by the cronus-kernel binary running the `app.cronus`. `site-v2/app.cronus` defines "CRONUS Design System Documentation" on port 4900.
- **Stack**: `.cronus` language (compiled by kernel). No Node/Vite/Astro.
- **Status**: site-v2 ACTIVE (ast-snapshot.json modified 2026-04); site (v1) STALE, backup files accumulating in site-v2 (`app.cronus.bak`, `app.cronus.working-backup`, `backup/`).
- **Relationship**: **Dogfooding** — the docs are written in the very language the kernel ships. If kernel breaks, docs site breaks.
- **Git**: Root repo. Many _legacy deletions, some M.
- **Last change**: 2026-04-08/10.

### 2.10 `cronus-schema.json`
- **What**: 55KB JSON Schema (draft 2020-12) titled "CRONUS Language Schema", **auto-generated from 64 `.spec.toml` contracts** (per its header). Documents every section (`action`, `api`, `page`, `entity`, etc.) with `layer`, `stability`, `intent`, `structural_keys`, `config_keys`, `entity_binding`, `aliases`.
- **Purpose**: IDE tooling / JSON validation / contract reference. Feeds cronus-vscode suggestions, doc generators, and any static validator for `.cronus` files.
- **Status**: ACTIVE — modified in root git status. Generated at `2026-04-08`.
- **Producer**: `cronus-kernel` (via a `contracts_generated.rs` or similar spec-toml pipeline — `src/contracts_generated.rs` exists in kernel).

### 2.11 Other siblings (not in scope — noted briefly)

Alive-looking: `hydra/` (SQLite brain dbs, modified), `landing-glo-21/` (user landing, 3001, modified), `cronus-pentest/` (security tooling), `database/`, `designsystem/`, `shared/`, `knowledge/`, `workspaces/`, `specs/`.

Archive/dead: `cronus-browser/`, `cronus-browser-electron/`, `cronus-cli/`, `cronus-landing/`, `cronus-supervisor/`, `cronus-visual-cli/`, `dashboard-v2/`, `landing/`, `landing-v2/`, `landing-new/`, `landing-glo-18/20/20-clone/`, `orbit-landing/`, `fusion-page/`, `design-hub/`, `deepseek-cli/`, `lab-v2/`, `estudos/`, `test-area/`, `test-cronus-lang/`, `test-output/`, `ir-test-clean/`, `multi-agent-test/`, `checkout-test/`, `ui-test/`, `vercel-clone/`, `vinext-scanner/`, `noir-minimal-temp/`, `examples/`, `demos/`, `outputs/`, `generated/`, `published-site/`, `stitch-pages/`, `agents/`, `security/`, `server-intelligence/`, `worktrees/`, `repos/` (contains `cooud-archives/*` — unrelated cooud repos).

---

## 3. Dependency Graph

```
                        cronus-kernel (Rust bin: cronus-lang)
                        │
          ┌─────────────┼────────────────┬─────────────────┐
          │             │                │                 │
     consumed by    consumed by      generates         dogfooded by
          │             │                │                 │
          ▼             ▼                ▼                 ▼
    cronus-nexus   daemon          cronus-schema.json   docs/site-v2
    (Tauri v1)     (Bun/Elysia)    (64 .spec.toml)      (app.cronus)
          │             │                │
          │             │                │
          │             │                └──> cronus-vscode   (IDE validation future)
          │             │                └──> tree-sitter-cronus (grammar future)
          │             │
          │             │──> consumed by ──> dashboard/ (vite/react, port 4803)
          │                                  │
          │                                  └──> also MERGED daemon source (in-progress absorb)
          │
          └──> obsolete once ──> cronus-nexus-v2 (Observer Engine, Bun+SQLite)
                                  │
                                  └──> overlaps with daemon (both capture Claude sessions)
```

### Edges in words:
- `cronus-kernel` produces the binary + `cronus-schema.json` (via `.spec.toml` contracts).
- `docs/site-v2` is `app.cronus` served BY the kernel binary — circular dogfood.
- `cronus-vscode` + `tree-sitter-cronus` are language-tooling sidecars for the kernel (not yet wired).
- `daemon` (port 4800) spawns Claude/Codex CLIs and has its own Hydra/injection stack — it references cronus kernel patterns by name but does NOT link the Rust crate.
- `dashboard` (port 4803) proxies `/api → :4800` and is currently absorbing daemon backend into its own `src/`.
- `cronus-nexus` (Tauri v1) wraps daemon in a desktop window via `localhost:4800` CSP.
- `cronus-nexus-v2` is a NEW observer engine that duplicates part of what daemon + nexus already do.

---

## 4. Git Hygiene

| Scope | Own .git? | M | D | ?? | Notes |
|---|---|---|---|---|---|
| `CRONUS/` (root) | yes | 58 | 170 | 450 | Covers kernel + nexus + nexus-v2 + docs + vscode + tree-sitter + landings + cooud-archives + ~70 dirs. |
| `daemon/` | yes (nested) | 54 | 0 | 85 | Feature Factory WIP, injections refactor. |
| `dashboard/` | yes (nested) | 12 | 254 | 19 | 254 deletions = reorg of nested `dashboard/dashboard/*` path. `.git-backup/` present. |

**Why it's dirty:**
- The 254 `D` in `dashboard/` are not real file losses — they're the old nested layout (`dashboard/dashboard/src/*`) that was flattened to `dashboard/src/*` without a commit. Until staged + committed, `git status` will stay loud.
- The 450 `??` in root are partly expected (node_modules? no — `.gitignore` is only 841 bytes so probably not comprehensive) and partly real new dirs (`cronus-vscode/`, `tree-sitter-cronus/`, `cronus-nexus-v2/data/`, etc.).
- The 170 `D` in root are spread across `cooud-test/`, `designsystem/design-test/`, `docs/_legacy/`, `docs/site-v2/*.cronus` moved to backup, `repos/cooud-archives/*` moved/renamed.
- `hydra/brain.db`, `knowledge.db`, `memory.db`, `users.db` show as modified — SQLite DBs are being tracked (should not be — add to .gitignore).
- Nested repos inside the root repo means root `git status` shows `m` lines (subrepo dirty marker) for `repos/cooud-archives/dashboard`, `luminus-ai`, `orbit`, `sagan` — those are submodule-ish pointers that will drift.

**Bottom line:** root repo is unsalvageable as-is without a targeted commit sprint that groups: (a) kernel changes, (b) new sibling creations (vscode/tree-sitter/nexus-v2), (c) deletion cleanup, (d) `.gitignore` additions for `*.db`, `node_modules/`, `.git-backup/`, `dist/`, `.next/`, `target/`.

---

## 5. Dashboard vs Dashboard-v2 — Verdict

**Keep `dashboard/`. Archive or delete `dashboard-v2/`.**

Evidence:
- `dashboard/` has current commits (latest 2026-04-09, `45219e6 feat: Feature Compiler Engine + Voice Chat + Studio UX overhaul`), 19 new pages in untracked state, active work on Hydra/Brain/Flow/Terminal/Galaxy3D views, and the owner is actively merging daemon backend into it (renamed `cronus-compiler-engine`).
- `dashboard-v2/` has src mtime 2026-04-07, a stale `.next/` from an abandoned Next.js attempt, a package.json that now points at Vite but never migrated, minimal src tree (`app/`, `components/`, `lib/`, `styles/`), no recent commits, no references from daemon or nexus.
- Naming is also backwards: `dashboard-v2` is older than current `dashboard/` work — "v2" here marks a *failed* redo, not the new one.

**Caveat:** `dashboard/` is currently in a risky shape — it's absorbing daemon server code (postgres, elysia, amqplib, puppeteer-core) into a Vite React project. Vite cannot bundle these for the browser. Either split into `dashboard/client/` + `dashboard/server/`, or keep the merge but run the server side as a separate Bun entrypoint (the pkg has both `dev: vite` and `start: bun run src/index.ts start`). This is worth calling out in A1/A4.

**Before deleting dashboard-v2**, grep for any imports pointing at it (`@dashboard-v2`, `../dashboard-v2`) — none seen in this scan but not exhaustively searched.

---

## 6. Recommendations

**Immediate (cleanup, no risk):**
1. Archive `dashboard-v2/` → `_archive/dashboard-v2-abandoned-2026-04-07/`. Nothing references it, work moved on.
2. Commit the 254 `D` deletions in `dashboard/.git` as a single "reorg: flatten nested dashboard layout" commit. Delete `dashboard/.git-backup/`.
3. Add `*.db`, `*.db-shm`, `*.db-wal`, `node_modules/`, `.next/`, `dist/`, `target/`, `.git-backup/` to root `.gitignore`. Stop tracking `hydra/*.db`.
4. Decide on `cronus-nexus/` vs `cronus-nexus-v2/` — they target the same slot. If nexus-v2 Observer Engine is the future, put `cronus-nexus/` in `_archive/` or tag v0.1 and freeze. If nexus-v2 is a side experiment, move it under `cronus-nexus/experiments/observer-engine/`.
5. Decide if `daemon/` is deprecated by `cronus-nexus-v2/` Observer Engine. Both capture Claude sessions. Both have HTTP APIs. Running both 24/7 is waste. The SDD in `cronus-nexus-v2/SDD.md` and `SDD-SESSION-OBSERVER.md` at root should declare a winner.

**Medium (needs design decision):**
6. `dashboard/` absorbing `daemon/` into one project must be resolved. Either (a) split into two build targets inside `dashboard/` (vite frontend + bun backend), or (b) abort the merge and keep `daemon/` independent. As-is the Vite build will fail.
7. Wire `tree-sitter-cronus/` into `cronus-vscode/` — currently vscode uses only TextMate grammar. Tree-sitter would give accurate highlighting + folding + structural selection.
8. Decide docs stack. Current `docs/site-v2/app.cronus` is dogfood, which is cool but creates a chicken-and-egg (doc site breaks when kernel breaks). Worth having a minimal static mirror.
9. Untangle the nested `.git` dirs in `daemon/` and `dashboard/`. Either convert to proper git submodules (with pinned SHAs), or absorb into root (`git rm -rf --cached daemon/.git && git add daemon/`). The current "embedded repo with no submodule link" is Git's most-foot-gun state.

**Focus (what to keep active):**
- `cronus-kernel/` — the actual product.
- `cronus-vscode/` + `tree-sitter-cronus/` + `cronus-schema.json` — the language tooling trio.
- ONE of: `daemon/` OR `cronus-nexus-v2/` (pick).
- ONE of: `cronus-nexus/` (Tauri shell) OR something else (pick).
- `dashboard/` — UI for whichever of the above wins.
- `docs/site-v2/` — dogfooded docs site.

**Archive candidates:** `dashboard-v2/`, `cronus-browser/`, `cronus-browser-electron/`, `cronus-cli/`, `cronus-visual-cli/`, `cronus-supervisor/`, `cronus-landing/`, all `landing-*` variants, `deepseek-cli/`, `lab-v2/`, `multi-agent-test/`, `checkout-test/`, `ui-test/`, `vercel-clone/`, `vinext-scanner/`, `noir-minimal-temp/`, and most of `repos/cooud-archives/*` (those are unrelated cooud repos that leaked into this tree).

---

## Appendix — Key File Paths

- Kernel manifest: `/home/zedd/Documentos/CRONUS/cronus-kernel/Cargo.toml`
- Kernel source root: `/home/zedd/Documentos/CRONUS/cronus-kernel/src/`
- Kernel contracts generator output: `/home/zedd/Documentos/CRONUS/cronus-kernel/src/contracts_generated.rs`
- Language schema: `/home/zedd/Documentos/CRONUS/cronus-schema.json`
- Daemon pkg: `/home/zedd/Documentos/CRONUS/daemon/package.json`
- Daemon Claude spawner: `/home/zedd/Documentos/CRONUS/daemon/src/spawner/claude.ts`
- Daemon injections: `/home/zedd/Documentos/CRONUS/daemon/src/injections/`
- Dashboard pkg (now `cronus-compiler-engine`): `/home/zedd/Documentos/CRONUS/dashboard/package.json`
- Dashboard vite config: `/home/zedd/Documentos/CRONUS/dashboard/vite.config.ts`
- Dashboard-v2 (abandoned): `/home/zedd/Documentos/CRONUS/dashboard-v2/`
- Nexus v1 Tauri conf: `/home/zedd/Documentos/CRONUS/cronus-nexus/src-tauri/tauri.conf.json`
- Nexus v1 Rust: `/home/zedd/Documentos/CRONUS/cronus-nexus/src-tauri/src/lib.rs`
- Nexus v2 architecture: `/home/zedd/Documentos/CRONUS/cronus-nexus-v2/ARCHITECTURE.md`
- Nexus v2 SDD: `/home/zedd/Documentos/CRONUS/cronus-nexus-v2/SDD.md`
- Nexus v2 entry: `/home/zedd/Documentos/CRONUS/cronus-nexus-v2/src/index.ts`
- VS Code extension: `/home/zedd/Documentos/CRONUS/cronus-vscode/package.json`
- Tree-sitter grammar: `/home/zedd/Documentos/CRONUS/tree-sitter-cronus/grammar.js`
- Docs (markdown): `/home/zedd/Documentos/CRONUS/docs/`
- Docs site (cronus app, v2 active): `/home/zedd/Documentos/CRONUS/docs/site-v2/app.cronus`
- Root SDDs of interest: `/home/zedd/Documentos/CRONUS/SDD-SESSION-OBSERVER.md`, `SDD-CRONUS-EVOLUTION.md`, `SDD-CRONUS-SDD-PROTOCOL.md`
