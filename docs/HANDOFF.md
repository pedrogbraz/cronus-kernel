# Handoff — cronus-kernel

Para o engenheiro que assume o kernel. Verdade no Git (`main` = `origin/main`), não neste arquivo se divergirem.

Repositório: `https://github.com/pedrogbraz/cronus-kernel`  
Checkout local típico: `/Users/pedrogbraz/projects/business/cronus-kernel`  
Branch de integração: **só `main`**.

---

## Ler primeiro

1. `LANGUAGE.md` — gramática e códigos de erro (a seção da tarefa, não o arquivo inteiro)
2. `CHANGELOG.md` → `[Unreleased]`
3. `AGENTS.md` — segurança, UI zero-JS, crate layout, harness
4. `.harness/SESSION.md` — ritual de tarefa (a partir de **Ritual de abertura**)

Sob demanda: `.harness/CONTEXT.md`, `.harness/project.json`, `docs/MCP.md`, `llms-full.txt` (apps `.cronus`, não o kernel).

Se doc e código discordarem, o código ganha; o doc entra na mesma entrega.

---

## O que é

Linguagem declarativa full-stack. Um crate `cronus-lang`, um `[[bin]]` `cronus`, sem `[lib]`, sem workspace. Lê `.cronus` e serve SQLite, REST, GraphQL, HTML SSR, cookie de sessão, SSE e audit em cadeia de hash.

- **Não adicione crates.**
- Rotas novas em `src/routes/`, não em `src/main.rs`.
- Autorização: `src/access.rs`. Sessão/CSRF: `src/session.rs`. Guardas: `src/http_guard.rs`.

---

## Como rodar

```bash
cd cronus-kernel
cargo build --locked --bin cronus
./target/debug/cronus new minha-app --template saas
cd minha-app && ../cronus-kernel/target/debug/cronus run
# escuta 127.0.0.1 (não assuma IPv6 em localhost)
```

Harness (kit em `~/projects/dev-harness`):

```bash
harness doctor
# se não estiver no PATH:
python3 /Users/pedrogbraz/projects/dev-harness/bin/harness -C . doctor
```

| Check | Função |
|---|---|
| `quick` | fmt + clippy correctness + testes do diff |
| `full` | contrato da CI + binário release ≤ 12 MiB |
| `critical` | full + auth/cookies/SQL/webhooks/GraphQL/`/_files` |

`quick` não autoriza integrar. Entrega = review do **commit atual** + `full` (`critical` se a tarefa for critical). Em worktree: `CARGO_TARGET_DIR=$PWD/target`. Nunca `cargo test --lib`; um filtro posicional só.

Pedido de código: `harness task new` → `task.md` completo → `ready` → `start` (worktree) → `quick` → commit → `review` → `full` → `integrate` no checkout principal. Risk `critical` se tocar auth, JWT, cookies, CSRF, SQL, webhooks, GraphQL, `/_files`, sessões.

---

## Demo

App de referência (não está no git do kernel):

- Path: `/Users/pedrogbraz/projects/business/cronus-apps/studio`
- URL: `http://127.0.0.1:5175`
- Conta: `pedro@studio.test` — senha com o Pedro (mínimo 15 caracteres)

Template: `templates/saas.cronus`.

---

## Já em `main`

P0–P2 de segurança/linguagem/schema/UI, mais correções de produto:

- Rotas estáticas ganham de `:param` (`/projects/new` não é `id=new`)
- Login/register são forms HTML (`POST /login`, `POST /register`); Sign Out é `POST /logout`
- Form com `bind Entity` emite `data-cronus-entity` (Save deixa de ser 403 `UNKNOWN_FORM`)
- Landing sem Google Fonts / Chart.js / blob `create-modal`
- `GET /logout` não limpa cookie; Argon2 dummy no login; SQLite 0600; HSTS em `--prod`; CORS `*` ignorado

Tarefas versionadas em `docs/tasks/<id>/{task,state,evidence}.md`.

---

## Pendências (próximo trabalho)

1. Landing ainda injeta runtime/HMR/SSE em dev (~11 scripts).
2. `src/ui/dashboard.rs` ainda puxa Google Fonts (templates mortos).
3. `section tabs` vs família `style:tabs` ainda ambíguo.
4. Contratos de section “thin” (validação vs renderer).
5. Playwright / `cronus audit visual` mora no repo **cronus-ui**, não no `full` do kernel.
6. Webhook HTTPS impossível (sem crate TLS). Só `http://`.

Gaps menores: `AGENTS.md` → Known gaps.

---

## Não regredir

Deny by default. Superfícies autenticadas pedem sessão. Owner scope no SQL. Cookie HttpOnly + CSRF. Senha nova ≥ 15 (Argon2id). Bind anônimo só com `scope:public`. Mudança nisto = `CHANGELOG.md` + teste HTTP.

UI dedicada: zero JS (`cronus_ui_output_gate.rs`). CSS em `src/cronus_ui_css/` + `MANIFEST`.

`cronus test` é HTTP do **app**, não CI do kernel.
