# Sessão CRONUS kernel — contrato para qualquer IA

Cole o bloco **Prompt** no início de uma sessão nova (Grok, Claude, Codex, Cursor, ChatGPT, …). O resto deste arquivo é o manual. Se o agente já estiver no repositório, peça só: *leia `.harness/SESSION.md` e cumpra o ritual de abertura*.

---

## Prompt (cole isto)

```
Você está no repositório cronus-kernel (linguagem .cronus + binário Rust `cronus`).

ANTES de qualquer código, plano longo ou `cargo test` inventado:

1. Leia, nesta ordem, sem pular:
   - `.harness/SESSION.md` (este contrato — cumpra tudo)
   - `.harness/CONTEXT.md` (o que ler e o que não ler)
   - `.harness/project.json` (checks reais)
   - o bloco harness + “Kernel overlay” no fim de `AGENTS.md`
2. Rode `harness doctor` no checkout principal (comando abaixo). Se `harness` não existir no PATH, use:
   python3 /Users/pedrogbraz/projects/dev-harness/bin/harness -C . doctor
3. Só então atenda o pedido.

Pedido de implementação / correção / P0–P2 / “vamos fazer X”:
- abra (ou retome) uma tarefa do harness
- uma worktree por tarefa (`harness start`)
- `quick` enquanto constrói; `full` + review no commit atual antes de integrar
- risk `critical` se tocar auth, cookies, CSRF, SQL, webhooks, GraphQL, `/_files`, sessões
- não reduza testes para ficar verde
- não faça push
- não adicione crates

Pedido só de explicação / mapa / nota: não abra tarefa; leia CONTEXT.md e responda.

Nunca: `cargo test --lib`; dois filtros posicionais (`cargo test a b`); `cronus test` como CI; pular o harness “porque a suíte é grande”.
```

---

## Ritual de abertura (obrigatório em pedido de código)

No checkout **principal**, branch base `main`:

```sh
# PATH, ou o binário do kit:
harness doctor
# equivalente:
python3 /Users/pedrogbraz/projects/dev-harness/bin/harness -C . doctor
```

Interprete o doctor:

| Saída | Ação |
|---|---|
| pré-requisitos ok, árvore limpa | pode `task new` / `start` |
| árvore limpa: False | não `start`/`integrate`; commit, stash ou continue na tarefa já ativa |
| pendência de `checks.full` | não entregue; o `project.json` deste repo já tem full |
| harness não encontrado | use o `python3 …/dev-harness/bin/harness -C .` |

Depois leia só o que o pedido exige (tabela em `.harness/CONTEXT.md`). Não carregue `src/` inteiro, `LANGUAGE.md` inteiro, nem `docs/archive/`.

---

## O que é o harness aqui

Kit pessoal **dev-harness 1.0.0** (o mesmo do Cooud). Transforma pedido em entrega **verificada**, com evidência amarrada ao conteúdo Git.

Não é sandbox. Não faz push. Não isola banco/portas (o kernel usa SQLite por app; worktrees não compartilham `target/` se o helper rodar).

Camadas:

| Camada | Onde | Função |
|---|---|---|
| Kit | `~/…/dev-harness` ou `python3 …/bin/harness` | ciclo, lock, worktree, evidência |
| Projeto | `.harness/project.json` | `quick` / `full` / `critical` deste kernel |
| Instruções | `AGENTS.md` (bloco `<!-- dev-harness -->` + overlay) | regras estáveis |
| Tarefa | `docs/tasks/<id>/{task,state,evidence}.md` | plano versionado |
| Estado operacional | `.git/dev-harness/` (não versionar) | locks, logs, `summary.json` |

Checks deste repo (não invente argv):

- **quick** → `python3 scripts/harness/check.py quick`  
  fmt + clippy `-D clippy::correctness` + testes **do diff** (`--bin cronus`, um filtro, `CARGO_TARGET_DIR=$PWD/target`). `Cargo.toml` / `main.rs` / >8 módulos → suíte inteira.
- **full** → o contrato da CI: fmt, clippy correctness, `cargo test --locked`, `--no-default-features --all-targets`, binário release ≤ 12 MiB.
- **critical** → full + filtro `session` / `access` / `auth` / `http_guard` / `webhook` / `files` / GraphQL HTTP.

`quick` **nunca** autoriza integrar. `full` + review do **commit atual** autorizam. Tarefa `critical` exige o check `critical` (o harness acrescenta sozinho se o risco no `task.md` for `critical`).

---

## Ciclo — implementação (cumpra nesta ordem)

Planejamento: **pedido → análise → plano → revisão → validação → ajustes → entrega**  
Código: **contexto → plano → worktree → construção → review → testes → integrate**

### 1. Planejar (checkout principal, `main`)

```sh
harness task new <id-curto> \
  --goal "uma frase" \
  --scope "arquivos / superfície" \
  --accept "critério observável" \
  --accept "outro critério" \
  --risk normal
```

`<id>`: `[a-z0-9][a-z0-9-]`, até 64 chars. Exemplos: `p1-indexes`, `csrf-xfh`, `hmr-reload`.

`--risk`:

| Valor | Quando |
|---|---|
| `light` | docs, comentário, CSS sem lógica |
| `normal` | feature / bug de comportamento / parser / UI |
| `critical` | auth, JWT, cookies, CSRF, SQL, owner scope, webhooks, GraphQL, `/_files`, sessões |

Abra `docs/tasks/<id>/task.md` e **substitua todo `PREENCHER`**. Sete seções: Pedido, Análise, Plano, Revisão, Validação, Ajustes, Entrega. Sem placeholder o `ready` recusa.

Atualize `state.md`: estado, decisões, próximo passo.

```sh
git add docs/tasks/<id>
git commit -m "docs: plan <id>"
harness task ready <id>
harness start <id>
```

`start` cria worktree `task/<id>` (fora do repo, em `../.worktrees/…`). **Uma tarefa de escrita ativa por projeto.** Entre no caminho que o comando imprimir.

### 2. Construir (dentro da worktree)

- Leia o arquivo + vizinho. Confie no código se discordar de docs.
- Não adicione crates. Rotas novas em `src/routes/`, não em `main.rs`.
- Linguagem: `LANGUAGE.md` + `llms-full.txt` + `CHANGELOG.md` na mesma entrega.
- Teste de regressão no **mesmo arquivo** (`#[cfg(test)]`). Contrato de CLI em `tests/mcp_cli.rs` / `tests/build_cli.rs`.
- Depois de um incremento útil:

```sh
harness check --tier quick --task <id>
```

Atualize `state.md` em marcos (não a cada ferramenta).

Quando o comportamento estiver completo:

```sh
git add <arquivos-revisados>
git commit -m "…"
harness task review <id> --summary "O que foi revisado no diff: critérios, erros, o que não entrou."
harness check --tier full --task <id>
```

Review exige árvore **limpa** e summary ≥ 12 caracteres. Full depois de mudar código invalida full antigo — rode de novo.

Preencha `evidence.md` com: commit SHA, tier, como achar o run (`harness resume <id>`), o que passou, limites. Não cole logs no chat.

### 3. Integrar (de volta ao checkout principal)

```sh
harness integrate <id>
harness finish <id>
```

- Só fast-forward local em `main`. **Não push.**
- Se `main` avançou: o harness faz worktree candidata, merge, `full` de novo. Conflito deixa `main` intacta.
- `finish --remove-worktree` só se não houver nem ignorados (`.env`, `*.db`). Sem `--force`.

Retomar: `harness resume <id>`. Pausar (árvore limpa): `harness task pause <id> --reason "…"`. Abandonar sem apagar: `harness task cancel <id> --reason "…"`.

---

## Pedido só de leitura

Explicar arquitetura, nota 0–10, “onde está X”: **não** abra tarefa. Leia `.harness/CONTEXT.md` e o mínimo de código. Não rode `full` “por garantia”.

Se no meio da conversa o usuário pedir para implementar, **aí** abra o ciclo acima, mesmo no mesmo chat.

---

## Regras que evitam falha de teste (obrigatórias)

O helper `scripts/harness/check.py` já aplica isto. Não contorne com cargo cru, salvo inspeção pontual **além** do harness:

1. Sempre `--locked`.
2. Não existe `[lib]` → nunca `--lib`.
3. Um filtro posicional: `cargo test --locked --bin cronus -- 'session::'`  
   Errado: `cargo test session parser` ou `cargo test cronus_ui voodoo`.
4. Integração: `cargo test --locked --test mcp_cli` (arquivo em `tests/`).
5. Worktree: `CARGO_TARGET_DIR` = `$PWD/target` (o script força). Sem isso o binário da outra árvore responde `mcp_cli`/`build_cli`.
6. `cronus test` = HTTP do **app**. Não é CI do kernel.
7. `cronus audit visual` sai 2 (Playwright no cooud-ui). Fora do `full`.
8. Suíte ~1800 testes em ~7s **quente**. Não pule `full` porque “são muitos testes”. O caro é clippy `--all-targets` e release LTO — por isso estão no `full`, não no `quick`.
9. Ferramenta ausente ou teste não rodado = **pendência**, não skip.
10. Não enfraqueça clippy, size 12 MiB, nem `--no-default-features`.

---

## O que ler (não despeje o repo)

Detalhe: `.harness/CONTEXT.md`. Resumo:

| Pedido | Ler |
|---|---|
| App `.cronus` | `llms-full.txt` |
| Kernel Rust | `AGENTS.md` + arquivo + vizinho |
| Gramática / sections / tipos | blocos marked em `llms-full.txt` + seção de `LANGUAGE.md` |
| Authz / HTTP | `AGENTS.md` security + `access.rs` / `session.rs` / `http_guard.rs` |
| Família UI | `AGENTS.md` UI rules + **um** `cronus_ui_<família>.rs` |
| CLI / MCP | `docs/MCP.md` |

`docs/archive/` é histórico. Se `LANGUAGE.md` discordar do `src/`, o código ganha; depois corrija o doc na mesma tarefa.

---

## Comandos do kit (referência)

```
harness doctor
harness task new <id> --goal … --scope … --accept … [--risk light|normal|critical]
harness task ready <id>
harness start <id>
harness check --tier quick|full [--task <id>]
harness task review <id> --summary "…"
harness integrate <id>
harness finish <id> [--remove-worktree]
harness resume <id>
harness task pause <id> --reason "…"
harness task cancel <id> --reason "…"
```

Sempre pode prefixar: `python3 /Users/pedrogbraz/projects/dev-harness/bin/harness -C <repo> …`

`-C` seleciona o repo sem `cd`. Worktree: rode `harness -C .` **dentro** dela para `check`/`review`.

---

## Proibido

- Começar a editar kernel sem ritual de abertura (pedido de código).
- Dois escritores / duas worktrees de escrita no mesmo projeto.
- Integrar sem review + `full` atuais.
- Push, deploy, `--force` em worktree, apagar branch pelo harness.
- Adicionar crate em `Cargo.toml`.
- Tratar `docs/archive/` ou HANDOFF antigo como verdade.
- Inventar `tests/conformance/` ou `specs/`.
- Declarar “pronto” com check que falhou, timeout, ou árvore que mudou depois do full.

---

## Definição de pronto

Uma entrega de código só é pronta quando:

1. Critérios de aceite em `task.md` demonstrados em `evidence.md`.
2. `harness task review` no commit entregue.
3. `harness check --tier full` (e `critical` se o risco for critical) **passed** nesse conteúdo.
4. `harness integrate` na `main` local, se o usuário pediu integrar. Senão: diga o caminho da worktree e o que falta (`integrate`).
5. Limitações explícitas (o que não foi feito).

Se algo bloquear: registre em `state.md`, não invente bypass.
