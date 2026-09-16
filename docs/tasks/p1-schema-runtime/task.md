# p1-schema-runtime

Risco: normal

## Pedido

Fechar o P1 de schema/runtime: `CREATE TABLE IF NOT EXISTS` não adiciona colunas novas; o handler HTTP não tem timeout de execução; GraphQL aninhado não tem teto; `seed`/`deploy`/`test`/`parse` leem um único arquivo e ignoram o load graph de `cronus run`.

Escopo: `database.rs` migrate, `http_guard.rs` isolate_panics, `graphql.rs` execute, CLI seed/deploy/test/parse + helper de load, CHANGELOG/AGENTS. Sem crates. Sem TLS.

## Análise

Fatos:
- `migrate_tables` só `CREATE TABLE IF NOT EXISTS`. Campos novos em DB existente não existem. Índices `IF NOT EXISTS` já correm.
- `isolate_panics` faz spawn; header read timeout existe; o future do handler pode correr para sempre.
- `attach_nested` é recursivo sem depth/node cap. N+1 aninhado já é gap conhecido; o teto evita explosão.
- `cmd_seed`/`deploy`/`test`/`parse` usam `find_cronus_file` + `parser::parse`. `run` usa `parse_source_at` ou `parse_directory`.

Decisões:
- ALTER ADD COLUMN só o tipo SQLite (sem NOT NULL/UNIQUE em tabela existente — linhas velhas não têm valor).
- Timeout: `CRONUS_HANDLER_TIMEOUT_SECS` default 30, 504 `HANDLER_TIMEOUT`.
- GraphQL: profundidade máx. 8, 200 nós de seleção; erro `COST_EXCEEDED` antes dos resolvers.
- Load: `parser::load_cwd()` = `parse_directory(".")` (mesmo grafo que `run` multi-file). `parse <file>` continua no path passado via `parse_source_at`.

## Plano

1. Após CREATE TABLE, `PRAGMA table_info`; ADD COLUMN para campos ausentes; teste migrate duas vezes.
2. `handler_timeout_from` + `tokio::time::timeout` em `isolate_panics`; teste com sleep.
3. `selection_cost` em `execute_graphql`; testes de query rasa vs profunda.
4. `load_cwd` em `compose.rs`; seed/deploy/test/parse usam; parse com argumento usa `parse_source_at`. Teste de load_cwd com dois arquivos.
5. CHANGELOG + AGENTS.md known gaps.

## Revisão

ALTER sem NOT NULL é o único caminho seguro no SQLite. Timeout 30s não quebra SSR normal. Cost cap é honesto (não resolve N+1, só limita). Load graph unificado evita seed cego. Adequado.

## Validação

Testes no mesmo arquivo + `harness check --tier full`.

## Ajustes

Nenhum ainda.

## Entrega

Critérios de aceitação:
- A second migrate adds a new entity field as a column on an existing SQLite table
- A handler that exceeds CRONUS_HANDLER_TIMEOUT_SECS returns 504 without taking down the server
- A GraphQL selection deeper than 8 or with more than 200 nodes returns a cost error and does not run resolvers
- seed, deploy, test, and parse load multi-file apps the same way as cronus run
