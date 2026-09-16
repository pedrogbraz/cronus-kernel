# Contexto sob demanda — CRONUS kernel

Nova sessão / outra IA: cumpra `.harness/SESSION.md` antes de desenvolver.

Objetivo: gastar tokens no arquivo certo, não no repositório inteiro.
`src/` tem ~360 `.rs` e ~138k linhas. Não carregue isso.

## O que ler (nesta ordem)

| Tarefa | Ler | Não ler |
|---|---|---|
| App `.cronus` (não o kernel) | `llms-full.txt` | `AGENTS.md`, `src/` |
| Kernel Rust | `AGENTS.md` + o arquivo + o vizinho | `docs/archive/`, `LANGUAGE.md` inteiro |
| Gramática / tipos / sections | `llms-full.txt` (blocos machine-checked) + `LANGUAGE.md` da seção | archive, VOODOO se não for runtime |
| Código de erro `cronus build` | `LANGUAGE.md` §15.9 + `src/cli/mcp/error_codes.rs` | |
| Authz / HTTP / cookies | `AGENTS.md` security model + `src/access.rs` / `session.rs` / `http_guard.rs` | UI families |
| Família cronus-ui | `AGENTS.md` UI rules + `src/cronus_ui_<família>.rs` + CSS da família | outras 180 families |
| Voodoo runtime | `docs/voodoo-llms.md` depois `VOODOO.md` | |
| CLI / MCP | `docs/MCP.md` + `src/cli/<cmd>.rs` | |

`docs/archive/` é histórico. `LANGUAGE.md` §18 lista mentiras antigas; confie no código.

## Checks (não invente cargo)

O harness **nunca** usa shell. Comandos reais:

```
harness doctor
harness check --tier quick --task <id>     # fmt + clippy::correctness + testes do diff
harness check --tier full --task <id>      # contrato da CI (inclui --locked, size 12MB, no-default-features)
```

Tarefa `critical` (auth, cookies, SQL, webhooks, `/_files`, GraphQL) acrescenta o filtro de segurança.

Regras que evitam falha boba de teste:

- Sempre `--locked`. Sem `--lib` (não há `[lib]`).
- Um filtro posicional só: `cargo test --locked --bin cronus -- 'session::'` — nunca `cargo test a b`.
- Worktree: `CARGO_TARGET_DIR` é `$PWD/target` (o script força). Sem isso o `cronus` de outra árvore responde os testes de CLI.
- `cronus test` é o produto HTTP, **não** a CI do kernel.
- `cronus audit visual` sai 2 e aponta para cooud-ui — não está no `full`.

## Worktree

Uma tarefa de escrita = uma worktree (`harness start`). O kit serializa o CLI; editores não.
Integração: `harness integrate` só na base **local**. Sem push.

## Evidência

`docs/tasks/<id>/evidence.md` aponta o commit e o `summary.json` do run (`harness resume <id>`). Não cole logs inteiros no chat.
