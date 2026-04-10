# A2 — Documentation Inventory (cronus-kernel)

> Generated: 2026-04-10
> Scope: `cronus-kernel/.cronus/`, `cronus-kernel/docs/`, `cronus-kernel/AGENTS.md`, `cronus-kernel/README.md`, `.agents/skills/generate-cronus/`
> Purpose: separate CURRENT from SUPERSEDED / HISTORICAL / REDUNDANT docs, verify the NEXTGEN roadmap, identify contradictions.

---

## 1. Executive Summary — Top 5 Findings

1. **The single load-bearing roadmap is `SDD-CRONUS-NEXTGEN.md` (2026-04-09)**. It names 4 phases. Of those, **Phase 2 is partially implemented, Phase 1 / 3 / 4 have zero code in `src/`** (no `RenderStrategy` enum, no `MiddlewareNode`, no `cache_ttl`, no `agent` / `workflow` block). The NextGen roadmap is therefore real-but-unstarted on its critical first phase.

2. **`SESSION-HANDOFF-2026-04-10.md` is the only current session document**. All prior `SESSION-*.md` files (04-01 evening, 04-01 night, 04-02, 04-02-FULL) and `HANDOFF-2026-04-02.md` are historical snapshots from 8+ days ago and should be archived.

3. **"cronus.test" is a local-dev `.test` TLD**, not a test file or doc section. It is wired via `/etc/hosts` + nginx to serve multiple kernel demos on one machine (see §2). There is no `cronus.test` file anywhere in the tree.

4. **Massive SDD redundancy from 2026-04-02**: `SDD-SPEC-DRIVEN-LANGUAGE.md` and `SDD-SPEC-DRIVEN-LANGUAGE-v2.md` both exist, both marked PROPOSED, both dated the same day. `SDD-CRONUS-LANGUAGE-V2.md` (2026-04-07) appears to be the third evolution of the same idea. None of them are referenced by `SESSION-HANDOFF-2026-04-10.md`, which points only at NEXTGEN.

5. **The canonical live docs are now outside `.cronus/`**: `cronus-kernel/AGENTS.md` (+ `CLAUDE.md` symlink), `cronus-kernel/README.md`, `cronus-kernel/docs/LANGUAGE-REFERENCE.md`, `cronus-kernel/docs/scriptcronus.md`, and `.agents/skills/generate-cronus/SKILL.md`. The `.cronus/` directory is now effectively a planning / history archive.

---

## 2. "cronus.test" — What It Actually Is

**Answer: it is a local-development DNS suffix (`.test` TLD) used to give each kernel demo its own hostname on 127.0.0.1.** It is NOT a file, NOT a test runner target, NOT a doc section.

### Evidence

`SESSION-2026-04-08.md:138-149` (in `/home/zedd/Documentos/CRONUS/` — parent of kernel):

```
# /etc/hosts
127.0.0.1 cronus.test docs.cronus.test novapay.cronus.test demo.cronus.test design.cronus.test original.cronus.test

# /etc/nginx/conf.d/cronus.conf
docs.cronus.test     -> localhost:4900
novapay.cronus.test  -> localhost:5220
cronus.test          -> localhost:5175
demo.cronus.test     -> localhost:5210
design.cronus.test   -> localhost:4901
original.cronus.test -> localhost:4902
```

`SESSION-2026-04-08-FINAL.md:305-308` confirms the table: design.cronus.test (4901), docs.cronus.test (4900), original.cronus.test (4902), novapay.cronus.test (5220).

Code reference: `cronus-browser-electron/src/audit.js:138` whitelists `*.cronus.test` and `*.cronus.local` as local-dev origins.

### What is NOT the answer

- No files matching `*cronus.test*` exist anywhere under `/home/zedd/Documentos/CRONUS/` (verified via `find`).
- `src/testing.rs` exists but is the internal test / assertion module, unrelated to the hostname.
- The `.test` TLD is a reserved RFC2606 name ensuring it never collides with real DNS — correct choice for dev.

### Caveats

- The nginx config lives on the host machine (`/etc/nginx/conf.d/cronus.conf`), not in the repo — none of the 4 `SESSION-2026-04-0X.md` files that describe it actually check it in. If the machine is rebuilt, this setup disappears with no automation.
- This is documented ONLY in the parent `CRONUS/` folder session notes, NOT in `cronus-kernel/.cronus/` or `cronus-kernel/README.md`. That is a missing-doc gap (see §7).

---

## 3. Document Inventory Table

Legend: CURRENT = load-bearing for ongoing work, SUPERSEDED = newer doc replaces it, HISTORICAL = session snapshot, REDUNDANT = duplicates another doc, REFERENCE = cheatsheet / stable user doc.

### 3.1 `.cronus/` planning directory

| # | File | Date (mtime) | Purpose | Status | Verdict |
|---|------|--------------|---------|--------|---------|
| 1 | `SDD-CRONUS-NEXTGEN.md` | 2026-04-09 | 4-phase roadmap to surpass Next.js/VINEXT | **CURRENT** | Keep — only live roadmap |
| 2 | `SESSION-HANDOFF-2026-04-10.md` | 2026-04-10 | Latest session handoff, links NEXTGEN phases | **CURRENT** | Keep — primary context for next session |
| 3 | `SDD-CRONUS-LANGUAGE-V2.md` | 2026-04-07 | "Blueprint for #1 language" fundacional | REDUNDANT w/ NEXTGEN | Archive; NEXTGEN supersedes |
| 4 | `AGENT-CONTEXT.md` | 2026-04-03 | Master context for new agent session | SUPERSEDED by `../AGENTS.md` | Archive |
| 5 | `BRIEF.md` | 2026-04-02 | Session-start brief (150 tests, 65k LOC) | HISTORICAL (stats stale: today 203 tests, 74k LOC) | Archive |
| 6 | `CONTEXT.md` | 2026-04-02 | NOVA CORE demo entity definitions | HISTORICAL (demo replaced by `demos/landing-test/`) | Archive |
| 7 | `DOMINATION-PLAN.md` | 2026-04-02 | "Plano de Dominação" — competitive analysis | SUPERSEDED by NEXTGEN §5 comparison table | Archive |
| 8 | `HANDOFF-2026-04-02.md` | 2026-04-02 | Session handoff 8 days old | HISTORICAL | Archive |
| 9 | `ORCHESTRATION.md` | 2026-04-03 | "4 Claude Code sessions in parallel" protocol | HISTORICAL (multi-session orchestration is not active) | Archive |
| 10 | `SDD-CODE-QUALITY.md` | 2026-04-02 | Rust-grade quality SDD, PROPOSED | SUPERSEDED (quality work now tracked in NEXTGEN phases) | Archive |
| 11 | `SDD-DEBUG-SYSTEM.md` | 2026-04-02 | Integrated debug system SDD, PROPOSED | HISTORICAL (no NEXTGEN ref) | Archive (or move to `ideas/`) |
| 12 | `SDD-EVOLUTION-ARCHITECTURE.md` | 2026-04-02 | Evolution/security/quality, PROPOSED | PARTIAL — Hydra exists in `src/hydra/`; SDD itself historical | Archive, note Hydra shipped |
| 13 | `SDD-LANGUAGE-CONSTITUTION.md` | 2026-04-02 | Constitution rules SDD, PROPOSED | SHIPPED — `src/constitution_check.rs` exists; SDD historical | Archive as "shipped SDD" |
| 14 | `SDD-QUALITY-BLOCK1.md` | 2026-04-03 | Error handling / CronusError, IN PROGRESS | HISTORICAL (error.rs shipped; status not updated) | Archive |
| 15 | `SDD-SEMANTIC-OPTIMIZATION.md` | 2026-04-02 | AI-first semantic SDD, PROPOSED | HISTORICAL | Archive |
| 16 | `SDD-SPEC-DRIVEN-LANGUAGE.md` | 2026-04-02 | Spec-driven language v1, PROPOSED | REDUNDANT — see v2 below | Archive |
| 17 | `SDD-SPEC-DRIVEN-LANGUAGE-v2.md` | 2026-04-02 | Deep review of v1 | REDUNDANT w/ NEXTGEN | Archive |
| 18 | `SDD-THEME-SYSTEM.md` | 2026-04-03 | Accent-driven theme derivation | PARTIAL — `src/theme.rs` exists; SDD historical | Archive |
| 19 | `SDD-ZERO-HARDCODE-ENFORCEMENT.md` | 2026-04-02 | Hardcode lint enforcement | SHIPPED — `src/hardcode_lint.rs` + lint rules | Archive as "shipped SDD" |
| 20 | `SECURITY-AUDIT-2026-04-02.md` | 2026-04-01 | 12 vulnerabilities found | HISTORICAL — session 2026-04-02 already fixed them | Archive |
| 21 | `SESSION-2026-04-01-EVENING.md` | 2026-04-01 | Session snapshot | HISTORICAL | Archive |
| 22 | `SESSION-2026-04-01-NIGHT.md` | 2026-04-01 | Session snapshot | HISTORICAL | Archive |
| 23 | `SESSION-2026-04-02.md` | 2026-04-01 | Session snapshot (security hardening) | HISTORICAL | Archive |
| 24 | `SESSION-2026-04-02-FULL.md` | 2026-04-02 | Biggest-ever session summary | HISTORICAL | Archive |
| 25 | `ast-snapshot.json` | 2026-04-02 | AST snapshot data file (not a doc) | REFERENCE (used by `ast_diff.rs`?) | Keep, verify usage |

### 3.2 Kernel-root user-facing docs

| File | Date | Purpose | Status |
|------|------|---------|--------|
| `cronus-kernel/README.md` | 2026-04-07 | Public-facing kernel README | CURRENT (280 lines) |
| `cronus-kernel/AGENTS.md` | 2026-04-09 | AI agent cheatsheet (165 lines) | CURRENT |
| `cronus-kernel/CLAUDE.md` | — | Symlink -> `AGENTS.md` | CURRENT |
| `cronus-kernel/CHANGELOG.md` | 2026-04-03 | Changelog | STALE (last entry pre-NEXTGEN) |
| `cronus-kernel/docs/LANGUAGE-REFERENCE.md` | 2026-04-07 | Full language reference (758 lines) | CURRENT (reference) |
| `cronus-kernel/docs/scriptcronus.md` | 2026-04-07 | ScriptCronus VM / 7 namespaces (527 lines) | CURRENT (reference) |
| `.agents/skills/generate-cronus/SKILL.md` | 2026-04-09 | Agent Skill for generating .cronus (369 lines) | CURRENT |
| `.agents/skills/generate-cronus/references/common-mistakes.md` | 2026-04-09 | 10 AI anti-patterns | CURRENT |
| `.agents/skills/generate-cronus/references/section-types.md` | 2026-04-09 | 51 section types with syntax | CURRENT |

### 3.3 Status counts

- CURRENT / load-bearing: **10** files (2 in `.cronus/`, 8 outside)
- REFERENCE docs (stable, user-facing): **2** (`LANGUAGE-REFERENCE.md`, `scriptcronus.md`)
- HISTORICAL (session snapshots): **7**
- SUPERSEDED / REDUNDANT (older SDDs replaced by NEXTGEN): **10**
- STALE (not updated but still in root): **1** (`CHANGELOG.md`)

---

## 4. Current Roadmap State — `SDD-CRONUS-NEXTGEN.md`

### 4.1 The 4 phases as stated in the SDD

| Phase | Name | Estimated | Features |
|-------|------|-----------|----------|
| **1** | FOUNDATIONS | 1 week | (a) render strategy per section (`render:static/island/stream/dynamic`) with `RenderStrategy` enum + `cache_ttl` on `SectionNode`; (b) `middleware` block with `MiddlewareNode` + `MiddlewareAction` enum; (c) declarative caching (`cache:5m`, cache.invalidate in ScriptCronus) |
| **2** | MODERN WEB | 1 week | (a) View Transitions API (meta tag + CSS `::view-transition-*`); (b) Speculation Rules / prefetch (`prefetch:eager/hover`); (c) Image optimization (`/_cronus/image` endpoint); (d) Font optimization (preconnect + @font-face) |
| **3** | SERVER POWER | 1 week | (a) first-class `action submit/delete` block; (b) ScriptCronus `function` server-callable block; (c) streaming SSR for `render:stream` via chunked transfer |
| **4** | EDGE & AI | 1 week | (a) `cronus build --target cloudflare` Workers target; (b) `agent` block for Cloudflare Agents SDK; (c) `workflow` block for durable execution |

Stated total: 4 weeks, ~95 new tests, zero breaking changes.

### 4.2 Code evidence of progress per phase

| Phase | Feature | Evidence in `src/` | Status |
|-------|---------|--------------------|--------|
| 1a | `RenderStrategy` enum, `render_strategy` field, `render:static\|island\|stream` parser | `grep -r "RenderStrategy\|render:static\|render_strategy"` -> **0 hits** | **NOT STARTED** |
| 1b | `MiddlewareNode`, `MiddlewareAction`, `middleware {}` parser | `grep -r "MiddlewareNode\|MiddlewareAction"` -> **0 hits** | **NOT STARTED** |
| 1c | `cache_ttl`, `cache:5m` parse, cache map | `grep -r "cache_ttl\|cache:5m\|cache:forever"` -> **0 hits** | **NOT STARTED** |
| 2a | View Transitions meta + CSS | `src/render.rs`, `src/ui/mod.rs`, `src/ui/layout.rs`, `src/ui/dashboard.rs` | **DONE** (handoff §2 confirms) |
| 2b | `prefetch:eager/hover`, speculation rules, data-prefetch | `src/render.rs` | **PARTIAL** (only `render.rs` hit; parser does not yet read a `prefetch:` modifier per handoff §4.5 "partially done") |
| 2c | `/_cronus/image` endpoint, srcset generation | No hits | **NOT STARTED** |
| 2d | font preconnect, @font-face generation | No hits | **NOT STARTED** |
| 3a-c | `action` block first-class, server `function`, streaming SSR | No hits for `render_section_stream` etc. | **NOT STARTED** |
| 4a-c | cloudflare target, `agent`, `workflow` | No hits | **NOT STARTED** |

**Also delivered this session (not on the NEXTGEN phase list but credited in the handoff):**
- Next.js → `.cronus` dump (`src/dump/nextjs.rs`, ~750 LOC) — **DONE**
- `nav` keyword optional in sidebar (`src/parser/mod.rs:841`) — **DONE**
- Dashboard type dispatch to `render_custom` (fix) — **DONE**
- Auth+layout router guard — **DONE**
- Removal of hardcoded `#CC0000` brand leaks — **DONE**

### 4.3 Does the 2026-04-10 handoff match NEXTGEN's Phase 0 or Phase 1?

**Answer: neither, strictly. NEXTGEN has no Phase 0.** The handoff delivered:

- A chunk of **Phase 2** (view transitions + hover prefetch).
- A large body of pre-NEXTGEN work (Next.js dump, layout rewrite, brand-color removal) that the SDD does not enumerate but which is a prerequisite for Phase 1.

The handoff's own "Next features" section (§Known Issues/TODO) correctly points at Phase 1 (Foundations) as the next work: render strategy, middleware, cache. So the handoff's self-positioning is: "we shipped a slice of Phase 2 out of order; Phase 1 is the real next step." That is consistent with the evidence.

---

## 5. Contradictions & Outdated Claims

### 5.1 Line counts / test counts (minor, fixable)

| Source | Claim | Truth |
|--------|-------|-------|
| `.cronus/BRIEF.md` (2026-04-02) | "150/150 tests passing, 65,400 LOC, 111 files" | Handoff 2026-04-10: **203 tests, ~74,130 LOC, 128 files** |
| `.cronus/AGENT-CONTEXT.md` (2026-04-03) | "6.5MB Rust binary, 150 lines = SaaS" | Handoff: **7MB binary, 50 lines = SaaS** |
| `.cronus/DOMINATION-PLAN.md` (2026-04-02) | "87 tests, 2800 line parser" | Out of date |

These are stale stats, not bugs, but they would mislead any agent reading the older doc first.

### 5.2 "Language spec" duplication

Three docs describe "the language" from overlapping angles:

- `SDD-SPEC-DRIVEN-LANGUAGE.md` (v1, 2026-04-02, PROPOSED)
- `SDD-SPEC-DRIVEN-LANGUAGE-v2.md` (2026-04-02, PROPOSED, "deep review of v1")
- `SDD-CRONUS-LANGUAGE-V2.md` (2026-04-07, "fundacional")

None of them match the actual shipped syntax in `AGENTS.md` (2026-04-09) or `docs/LANGUAGE-REFERENCE.md` (2026-04-07). An agent reading `SDD-SPEC-DRIVEN-LANGUAGE-v2.md` and writing to its syntax will produce non-parsable `.cronus` code.

**Contradiction flag:** `SDD-CRONUS-LANGUAGE-V2.md` is titled "v2" but is dated AFTER `SDD-SPEC-DRIVEN-LANGUAGE-v2.md`, so "v2" refers to different things in different files. The naming itself is ambiguous.

### 5.3 Dashboard type dispatch (now resolved, but old docs still say otherwise)

- `SDD-LANGUAGE-CONSTITUTION.md` and several older session notes reference `type:dashboard` as rendering a "rich default Cooud-style template".
- `SESSION-HANDOFF-2026-04-10.md §5` explicitly reverses this: `type:dashboard -> render_custom`, user sections always respected.

Any doc that says `type:dashboard` provides a built-in template is now wrong.

### 5.4 Brand color leak (now resolved, but not retracted)

Older sessions reference `#CC0000` (Cooud red) injections. `SESSION-HANDOFF-2026-04-10.md §2` confirms these were removed. Stale docs that still show red-injected examples should be archived or marked.

### 5.5 Orchestration protocol

`ORCHESTRATION.md` (2026-04-03) describes "4 Claude Code sessions working in parallel, each owns specific files". The 2026-04-10 handoff is a single-session document and makes no reference to multi-session orchestration. **The 4-session protocol is de-facto dead.** Keeping `ORCHESTRATION.md` in the active planning dir risks misleading a fresh agent into thinking parallel sessions are the current workflow.

### 5.6 CLAUDE.md symlink

`cronus-kernel/CLAUDE.md -> AGENTS.md`. This is intentional per the handoff, but it means editing one file rewrites both. Any tool that writes `CLAUDE.md` assuming it's independent will clobber `AGENTS.md`. Worth noting in a hazards list.

---

## 6. Archival Recommendations

Create `cronus-kernel/.cronus/archive/` and move these files there, grouped by sub-folder for clarity:

### `.cronus/archive/sessions/`
- `HANDOFF-2026-04-02.md`
- `SESSION-2026-04-01-EVENING.md`
- `SESSION-2026-04-01-NIGHT.md`
- `SESSION-2026-04-02.md`
- `SESSION-2026-04-02-FULL.md`
- `SECURITY-AUDIT-2026-04-02.md` (audit is a point-in-time snapshot, vulns already fixed)

### `.cronus/archive/superseded-sdds/`
- `SDD-SPEC-DRIVEN-LANGUAGE.md`
- `SDD-SPEC-DRIVEN-LANGUAGE-v2.md`
- `SDD-CRONUS-LANGUAGE-V2.md`
- `SDD-SEMANTIC-OPTIMIZATION.md`
- `SDD-CODE-QUALITY.md`
- `SDD-QUALITY-BLOCK1.md`
- `DOMINATION-PLAN.md`

### `.cronus/archive/shipped-sdds/` (SDDs whose feature is already merged)
- `SDD-LANGUAGE-CONSTITUTION.md` (see `src/constitution_check.rs`)
- `SDD-ZERO-HARDCODE-ENFORCEMENT.md` (see `src/hardcode_lint.rs`)
- `SDD-EVOLUTION-ARCHITECTURE.md` (see `src/hydra/`)
- `SDD-THEME-SYSTEM.md` (see `src/theme.rs`)
- `SDD-DEBUG-SYSTEM.md` (partial — check what shipped before archiving)

### `.cronus/archive/stale-context/`
- `AGENT-CONTEXT.md` (replaced by `../AGENTS.md`)
- `BRIEF.md` (stale stats)
- `CONTEXT.md` (NOVA CORE demo context, demo is gone)
- `ORCHESTRATION.md` (multi-session protocol abandoned)

### Stay in `.cronus/` (active)
- `SDD-CRONUS-NEXTGEN.md` — the only live roadmap
- `SESSION-HANDOFF-2026-04-10.md` — latest handoff, read-at-session-start
- `analysis-2026-04-10/` — this analysis folder
- `ast-snapshot.json` — data file, confirm usage before touching

**Net effect**: `.cronus/` drops from ~25 items to 4 items. A new agent opening the folder will see exactly what matters.

---

## 7. Missing Documentation (topics that should exist but don't)

1. **`cronus.test` local-dev setup** — no doc in `cronus-kernel/` explains how to wire `/etc/hosts` + nginx to serve multiple demos under `*.cronus.test`. The only mention is in parent-folder session notes that will never be found by someone cloning just `cronus-kernel`. **Recommendation**: add `cronus-kernel/docs/local-dev-hosts.md` with the `/etc/hosts` block and nginx config, or ship a `scripts/setup-local-dns.sh`.

2. **Phase 1 design gaps in NEXTGEN** — the SDD names the features but doesn't specify (a) how cache invalidation keys interact with ownership isolation, (b) what happens when a `render:island` section binds data that changes under a `middleware` redirect, (c) how `cache:forever` interacts with entity mutations. These need to be resolved before implementation.

3. **Deprecation notes** — no file lists "features removed" or "behavior changed". The `type:dashboard` change (built-in template -> render_custom delegation) is a breaking behavior change for any user app relying on it, mentioned only in a session handoff.

4. **VS Code extension + tree-sitter grammar docs** — both exist as sibling repos (`CRONUS/cronus-vscode/`, `CRONUS/tree-sitter-cronus/`) but `cronus-kernel/README.md` doesn't mention them, so a user cloning only the kernel will never know editor support exists.

5. **Current test inventory** — no doc maps the 203 tests to features. With Phase 1 adding ~30 tests, tracking which feature each test covers will matter quickly.

6. **Cheatsheet for porting Next.js app** — `src/dump/nextjs.rs` exists and the handoff lists 8 successful ports, but there is no user-facing doc for `cronus dump --nextjs ./path`. The SKILL.md focuses on generation, not on migration from Next.js.

7. **Decision log for breaking changes** — repeatedly the kernel makes behavior changes (CC0000 removal, dashboard dispatch, nav keyword optional) with no ADR / CHANGELOG entry. `CHANGELOG.md` exists but was last touched 2026-04-03.

---

## End of report

Total documents inspected: 29 markdown files + 1 data file + 1 skills directory + kernel source tree verification.
