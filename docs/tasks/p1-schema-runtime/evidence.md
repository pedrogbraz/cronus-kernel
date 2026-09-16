# Evidências

- Commit: `b9cd91a` `feat(runtime): alter existing tables, timeout handlers, and cap GraphQL cost`
- `harness check --tier quick`: passed (`fffd2123dc734e5ebf13a2987ce53648`).
- Tests: `migrate_adds_new_columns_on_existing_tables`, `slow_handler_becomes_504`, `selection_deeper_than_max_is_cost_exceeded`.
- CLI: `parser::load_cwd()` = `parse_directory(".")` in seed, deploy, test, parse (path argument still `parse_source_at`).
- Limits: ALTER does not add NOT NULL/UNIQUE. Timeout aborts the join handle; in-flight SQLite work may still finish until abort. GraphQL cap does not fix N+1.
