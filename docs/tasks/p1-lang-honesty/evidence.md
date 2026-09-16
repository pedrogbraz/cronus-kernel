# Evidências

- Commit: `f43687b` `feat(lang): diagnose duplicate fields, hollow components, and unused import aliases`
- `harness check --tier quick --task p1-lang-honesty`: passed (`git-diff-check`, `kernel-quick`). Run `b55c3a6db8fc4823b2ffd19a4a54a494`.
- Parser tests: FIELD_005 at line 3; RESOLVE_001 for `use Missing`; kit `use Card` ok; LANG_002 for params/state/template; PARSE_001 for English `on update when`; COMPOSE_003 for `import Tasks from`.
- `mcp_error_codes_match_language_md_table` included in the quick cargo filter via check.py (LANGUAGE.md / error_codes.rs changed).
- Limits: selective import is still not implemented (alias is an error). Component reactivity is still not implemented (honest fail). Security P1 (`/_files`, HMAC) is out of this task.
