# D5 — Internal + Archive + Top-Level AI SDDs

**Pass:** D5 (docs shallow inventory)
**Date:** 2026-04-10
**Scope:** `docs/internal/` (whole tree, read), `docs/_archive/` (shallow), 4 `SDD-AI*` + `SDD-INTERACTIVE-COURSES` at docs root.
**Excluded:** `docs/cronus-kernel/.cronus/` (already covered in other passes).

---

## 1. Executive Summary

`docs/internal/` holds **8 files / ~2,401 lines** of founder-level strategy, SDD methodology, governance, and release notes for the CRONUS language. Three of those documents (`CRONUS-v5.md`, `CRONUS-SDD-CODEX.md`, `LANGUAGE-REENGINEERING.md`) are overlapping versions of the same roadmap (v2 → SDD → v5), with v5 explicitly self-declared as "the single canonical plan. Replaces v2, SDD, v4." The other five files (change-policy, imperial-thesis, release-notes, rfc-template, stability-levels) are still operational governance and DO NOT overlap.

`docs/_archive/` holds **6 files / ~1,489 lines** of legacy user-facing reference content (API / DEPLOY / ENTITIES / EXAMPLES / REFERENCE), all superseded per the `_archive/README.md` disclaimer. Dates all 2026-03-30. Shallow inventory only.

The **4 AI SDDs at docs root** (SDD-AI-CONFIG-TAB, SDD-AI-OFFERS-ONECLICK, SDD-AI-SUPPORT-CONTEXT, SDD-INTERACTIVE-COURSES) do NOT describe CRONUS language features. They describe **Cooud platform features** (CooudTV, Sirius, Rigel, Dashboard, DeepSeek integration). They look misfiled in the CRONUS repo — none of the entities mentioned (ai_store_config, ai_memories, lesson_progress, InteractiveLesson.tsx) exist in `cronus-kernel/`. These are Cooud planning docs that ended up in the CRONUS docs root by accident.

No secrets or credentials found. No personal info beyond `authors = ["zedd"]` (already public in git history).

---

## 2. `internal/` Inventory Table

Directory total: **8 files, 2,401 lines**. All files modified 2026-03-30 (single-day writing session). Last mtime = `CRONUS-v5.md` at 21:51.

| File | Lines | Modified | Purpose (one line) | Status |
|------|------:|----------|-------------------|--------|
| `CRONUS-v5.md` | 507 | 21:51 | The canonical two-part plan: north-star architecture (5 levels, 12 principles, future binding/actions/permissions) + execution order (Phase 0-4). Self-declared as replacing v2, SDD, v4. | **LOAD-BEARING — current roadmap** |
| `CRONUS-SDD-CODEX.md` | 750 | 21:18 | Deep methodology of Spec-Driven Development: `.spec.toml` format, lifecycle, codegen pipeline, two-namespace keys, conformance test generation. | **LOAD-BEARING — pipeline spec** (kernel has `contracts.rs` + `contracts_generated.rs`, confirming it was built) |
| `LANGUAGE-REENGINEERING.md` | 458 | 21:18 | v3.0 reengineering plan merging v2 + Codex SDD refinements. Pre-v5 synthesis. | **SUPERSEDED by CRONUS-v5.md** (v5 explicitly replaces it) |
| `CRONUS-IMPERIAL-THESIS.md` | 321 | 21:35 | Founder thesis: CRONUS as semantic OS for agentic era; 4-level architecture (core/stdlib/patterns/adapters); 10 non-negotiables; 5 SDD invariants. | **LOAD-BEARING — foundational doctrine** |
| `rfc-template.md` | 154 | 20:47 | RFC format + lifecycle (draft → review → accepted → implemented). Used to propose new primitives. | **ACTIVE governance** |
| `release-notes.md` | 118 | 20:48 | v0.1.0 release record: 51 sections, 14 top-level nodes, 16 field types, known limitations, no-automated-tests gap. | **ACTIVE (single release)** |
| `stability-levels.md` | 114 | 20:47 | 4 stability levels (stable/experimental/internal/deprecated) + promotion/deprecation processes + current matrix. | **ACTIVE governance** |
| `change-policy.md` | 77 | 20:46 | 6-question checklist before merging language changes + approver matrix + PR checklist template. | **ACTIVE governance** |

### 2a. Load-bearing vs stale breakdown

**Load-bearing (describe current or recently-built architecture):**
- `CRONUS-v5.md` — single canonical plan, referenced as destination for all phases
- `CRONUS-SDD-CODEX.md` — pipeline methodology; `cronus-kernel/src/contracts_generated.rs` exists, so the codegen pipeline was (at least partially) implemented
- `CRONUS-IMPERIAL-THESIS.md` — principles are cited as doctrinal source
- `change-policy.md`, `stability-levels.md`, `rfc-template.md` — still the operational rules for PRs

**Stale / superseded:**
- `LANGUAGE-REENGINEERING.md` — explicitly superseded by CRONUS-v5 per v5's own header ("Replaces v2, SDD, v4"). Kept for historical context but nothing should reference it as current. **Candidate for move to `_archive/`**.

**Single-release historical:**
- `release-notes.md` — only has v0.1.0. Will become load-bearing again once v0.2.0 ships.

### 2b. Overlap between the three big plans

```
LANGUAGE-REENGINEERING.md (v3.0, 458L)  →  absorbed the v2 plan + Codex SDD
CRONUS-SDD-CODEX.md       (750L)        →  the methodology piece
CRONUS-v5.md              (507L)        →  the "single canonical plan" that replaces v2, SDD, v4
```

v5 has Part I (architecture) + Part II (execution). It cites the SDD Codex in lineage but does not physically duplicate the TOML format details — those live in CRONUS-SDD-CODEX.md. So the two are **complementary**, not duplicates. LANGUAGE-REENGINEERING.md is the one truly subsumed.

---

## 3. `_archive/` Inventory (shallow)

Directory total: **6 files, 1,489 lines**. All content dated 2026-03-30, README added 2026-04-07 marking everything as deprecated.

| File | Lines | Category | One-line purpose |
|------|------:|----------|------------------|
| `README.md` | 4 | Marker | "Deprecated documentation. Superseded by docs/ guides and reference. Do NOT reference these files." |
| `REFERENCE.md` | 685 | Superseded SDD | Old full reference manual: top-level nodes, field types, sections — superseded by current `docs/reference/` |
| `API.md` | 186 | Superseded feature doc | Old REST/GraphQL auto-generation doc |
| `DEPLOY.md` | 185 | Superseded feature doc | Old `cronus deploy` artifact generation doc |
| `EXAMPLES.md` | 181 | Old examples | Landing/dashboard/etc. example `.cronus` snippets |
| `ENTITIES.md` | 148 | Old feature doc | Entity syntax + SQLite/GraphQL generation |

**Categorization:** Zero historical sessions, zero old roadmaps (those live in current `internal/`), zero deprecated features. All 5 content files are **superseded user-facing reference docs** — the previous generation of the public `docs/` tree, frozen on 2026-03-30.

**Estimated archived lines:** ~1,485 content lines (excluding the 4-line README marker). Well-contained archive, no bloat.

**Recommendation:** Leave as-is. README properly warns readers. Small enough that pruning is not worth the risk.

---

## 4. Four Top-Level AI SDDs — Status Verdict

All four are dated **2026-04-08** (one day's planning session, 11 days old as of 2026-04-10). Authors: zedd. All use the Cooud SDD template (Spec / Affected Services / Risk Level / State Transitions / Rollback / Test Plan / Validation Checklist).

| File | Lines | Date | Purpose | Status | Describes kernel code? |
|------|------:|------|---------|--------|------------------------|
| `SDD-AI-CONFIG-TAB.md` | 94 | 2026-04-08 03:04 | Add "IA" tab on Cooud Dashboard support page where producer sets AI agent name + avatar URL. 2 new columns on `staging.ai_store_config`. | **IMPLEMENTING** (self-declared), **STALE-IN-CRONUS** | **No** — Cooud Dashboard / Orbit / CooudTV feature. Not CRONUS. |
| `SDD-AI-OFFERS-ONECLICK.md` | 57 | 2026-04-08 05:17 | IA during CooudTV chat can create offers and send a one-click purchase button using saved Stripe card. | **IMPLEMENTING** (self-declared), **STALE-IN-CRONUS** | **No** — CooudTV + Sirius (billing) + Stripe. Not CRONUS. |
| `SDD-AI-SUPPORT-CONTEXT.md` | 251 | 2026-04-08 02:53 | Upgrade `/api/ai/chat` on CooudTV to inject full context (course structure, ticket history, AI memories, store config) before DeepSeek call. Creates `staging.ai_memories` + `staging.ai_store_config`. | **IMPLEMENTING** (self-declared), **STALE-IN-CRONUS** | **No** — CooudTV + Rigel + Sirius + DeepSeek integration. Not CRONUS. |
| `SDD-INTERACTIVE-COURSES.md` | 307 | 2026-04-08 08:24 | AI-generated interactive course template engine: 5 block types (text/quiz/checklist/exercise/certificate), 5 modules × 3 lessons, `staging.lesson_progress` table, anti-chargeback strategies. | **DRAFT** (self-declared) | **No** — CooudTV (InteractiveLesson.tsx React) + Rigel (courses) + DeepSeek. Not CRONUS. |

### Verdict

**All 4 files are misfiled.** They are Cooud platform planning docs that belong in `/home/zedd/Documentos/cooud/` (or wherever Cooud's SDD tree lives), not in `/home/zedd/Documentos/CRONUS/docs/`. Grep of `cronus-kernel/` for any of the entities (`ai_store_config`, `ai_memories`, `lesson_progress`, `InteractiveLesson`, `AIConfigTab`) returned **zero matches in source code** — the only hit was inside `cronus-kernel/.cronus/DOMINATION-PLAN.md` (a meta-planning doc).

None of them are load-bearing for the CRONUS language/kernel. Three are actively being implemented — but in Cooud, not here.

---

## 5. Archival Recommendations

### Move to `_archive/` inside `docs/internal/`
1. **`LANGUAGE-REENGINEERING.md`** (458L) — explicitly superseded by v5 per v5's own header. Keep accessible for historical lineage, but it should not be read as current.

### Move out of the CRONUS repo entirely
2. **The 4 AI SDDs** (94 + 57 + 251 + 307 = **709 lines**) — relocate to `cooud/docs/` or similar. They pollute the CRONUS docs root with Cooud-specific planning and make it look like CRONUS has an AI support tab feature when it doesn't.

   Suggested destination: `/home/zedd/Documentos/cooud/docs/sdd/ai/` (or wherever Cooud SDDs live). Preserve git history via `git mv`.

### Leave as-is
- Everything in `docs/_archive/` — already marked deprecated, small footprint, no risk.
- `release-notes.md` — will fill up naturally on v0.2.0.
- `change-policy.md`, `stability-levels.md`, `rfc-template.md`, `CRONUS-IMPERIAL-THESIS.md`, `CRONUS-SDD-CODEX.md`, `CRONUS-v5.md` — all load-bearing.

---

## 6. Anything Concerning

### Secrets / credentials
**None found.** No API keys, no DB connection strings with passwords, no tokens. The DDL snippets in `SDD-AI-SUPPORT-CONTEXT.md` reference env var names (`DEEPSEEK_API_KEY`, `SIRIUS_DB_URL`, `RIGEL_DB_URL`) but never include values. Safe to commit.

### Personal info
- `authors = ["zedd"]` throughout — already public via git commit history. Not a leak.
- No real customer data, no store IDs beyond one example `01KCAD6` in log samples — looks like a test/dev store.

### Duplicated content
- `LANGUAGE-REENGINEERING.md` and `CRONUS-v5.md` both contain Phase 1-3 roadmaps. v5 explicitly supersedes it, so the duplication is by-design-temporarily but should be resolved by archiving LANGUAGE-REENGINEERING.md.
- `CRONUS-SDD-CODEX.md` and `CRONUS-v5.md` discuss the same spec lifecycle but from different angles (methodology vs execution plan). **Not duplicated content, complementary.**
- The 4 AI SDDs share large template boilerplate (Affected Services checkboxes, Risk Level blocks, Rollback Strategy). Normal for a templated SDD format.

### Misplaced files (biggest finding)
- **The 4 `SDD-AI*` / `SDD-INTERACTIVE-COURSES` files describe Cooud features, not CRONUS features**, and sit in the CRONUS repo docs root. They reference `staging.*` Postgres tables, CooudTV Next.js routes, DeepSeek, Sirius, Rigel, Orbit — none of which exist in `cronus-kernel/`. This is the only real "concerning" item: they create the false impression that CRONUS has an AI support subsystem.

### Other observations
- All 8 `internal/` docs were written in a single day (2026-03-30), all by the same author. Healthy for a coherent founding doctrine, but implies none have been battle-tested against real PRs yet (no revisions in the git log for this branch).
- `release-notes.md` honestly lists "No automated tests" and "No LSP" as known limitations. Good discipline, but means conformance claims in `CRONUS-v5.md` Phase 1 exit criteria ("50+ conformance tests passing") are **future**, not **present**.
- `CRONUS-SDD-CODEX.md` line 750 ends with `> "A spec is a promise. Code is the keeping of it. Tests are the proof."` — this exact quote reappears in `LANGUAGE-REENGINEERING.md` attributed to "Codex refinement, 2026-03-30", confirming the chronological ordering.

---

## Line count totals

| Bucket | Files | Lines |
|--------|------:|------:|
| `internal/` | 8 | 2,401 |
| `_archive/` | 6 | 1,489 |
| 4 AI SDDs at docs root | 4 | 709 |
| **D5 scope total** | **18** | **4,599** |

(`wc -l` summed to 4,597 — two-line difference is trailing newlines.)

---

> *D5 pass complete. No changes made to the repository. Report is read-only inventory.*
