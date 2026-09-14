# CRONUS kernel documentation

## Current

| File | For | What it is |
|---|---|---|
| [../llms.txt](../llms.txt) | LLMs | Index in the llmstxt.org format |
| [../llms-full.txt](../llms-full.txt) | LLMs, authors | Canonical grammar, valid section/field types, example that builds clean, never-do list. Lists are checked against the code by `cargo test context_grammar` |
| [../LANGUAGE.md](../LANGUAGE.md) | Authors, kernel devs | Long-form language reference with security semantics |
| [../AGENTS.md](../AGENTS.md) | Kernel devs, agents | How the Rust kernel is laid out, security model, UI renderer rules, tests. `CLAUDE.md` is a symlink to it |
| [../VOODOO.md](../VOODOO.md) | Kernel devs | Full contract for the opt-in Voodoo.js runtime |
| [voodoo-llms.md](voodoo-llms.md) | LLMs | Compact Voodoo.js guide (was the root `llms.txt`) |
| [../CHANGELOG.md](../CHANGELOG.md) | Everyone | Changes and breaking changes |

Generated on demand:

- `cronus context --for-claude` — project summary, current `build --ai` errors, grammar and types
- `cronus build --ai` — JSON validation result

## Archive

Historical records. They describe the kernel at the time they were written and are not
kept up to date; trust the code and the current docs above.

- [archive/waves/](archive/waves/) — Wave 1a–1t reports (dedicated cronus-ui renderer ports and geometry parity evidence)
- [archive/planning/](archive/planning/) — 2026-04 plans, SDDs, session handoffs, the 2026-04-02 security audit and the `analysis-2026-04-10/` agent reports
