# HANDOFF — linguagem Cronus, kernel, e biblioteca de componentes

> Contexto para a **próxima sessão**: portar `@cronus-ui/ui` (React/Next em `cooud-ui/`) para a linguagem `.cronus`, com o mesmo visual (tokens, motion, a11y) — **sem JSX na fonte**.
>
> Verificado contra o parser e o renderer em **2026-09**. Quando este arquivo discordar de `README.md`, `generate-cronus/SKILL.md` ou templates `cronus new admin`, **confie neste arquivo e no Rust**. `LANGUAGE.md` está parcialmente desatualizado (sintaxe de `bind`/`where`/`group`).
>
> Compacto Voodoo: `llms.txt`. Contrato Voodoo: `VOODOO.md`. Kernel: `AGENTS.md`.

---

## 0. Veredito (leia antes de portar)

**Não dá para copiar o componente React “exatamente” para dentro de um `.cronus`.**

A linguagem **não é um segundo React**. `.cronus` declara *o que existe* (`component Save style:button+primary+md`). O **kernel Rust** emite o HTML, o CSS (`--cronus-*`) e o motion. Framer/`motion`, CVA, Radix, `"use client"` e CSS-in-JS **não entram na fonte**.

O que já existe:

| Camada | Onde | Papel |
|---|---|---|
| Design system React | `/Users/pedrogbraz/projects/cooud/cooud-ui` (`@cronus-ui/ui`, ~212 slugs) | spec visual: CVA, tokens, motion, Radix |
| Tokens | `cooud-ui/packages/tokens` **e** cópia vendored `cronus-kernel/src/cronus_ui_tokens.css` | mesma paleta `--cronus-*` |
| Kernel widgets | `src/cronus_ui_widgets.rs` — **173 families** | `style:<family>` já *nomeia* o widget |
| Ports nativos (pixel CONTRACT) | `PORTED_FAMILIES` — **12** | button, badge, input, label, textarea, checkbox, switch, spinner, separator, kbd, toggle, progress |
| Stubs genéricos | os outros ~161 | HTML mínimo `data-slot`; **não** é o React animado |
| Auditoria de paridade | `cooud-ui/packages/audit` | fixtures React × `.cronus` (`emit-cronus-fixture.ts`) |

**Caminho certo:** React continua a spec. Cada família ganha um **renderer Rust** (HTML nativo + tokens + CSS de motion no kernel). O catálogo `.cronus` é só declaração. Paridade visual = kernel + `packages/audit`, não `template "<div className=…"`.

Se alguém colar JSX/`template`/`style_block` no `.cronus`, a linguagem quebra. Já limpamos os demos disso.

---

## 1. O que a linguagem *é*

Uma `.cronus` **é o produto**. `cronus run` (binário Rust `cronus-lang`, ~7MB release) faz:

```
.cronus  →  parser  →  AST
                 ├─ SQLite (entity → tabela, money = INTEGER centavos)
                 ├─ REST  (/api/{entity}s, JWT)
                 ├─ HTML  (seções + widgets)
                 └─ opcional: script Voodoo no HTML já emitido
```

Não há bundler, não há `node_modules` no app. `stack react + tailwind` no `app { }` é **rótulo** — não carrega React.

Três faixas, zero overlap no default:

| Faixa | Arquivo | Roda | Dono de |
|---|---|---|---|
| Cronus | `*.cronus` | compile-time + SSR | app, entity, page, bind, auth, style |
| Kernel | `src/*.rs` | servidor | SQL, HTTP, HTML, tokens, motion |
| Voodoo | CDN, **só se opt-in** | browser | `v-model` / `@click` no DOM emitido |
| ScriptCronus | `*.scriptcronus` | servidor | `on Lead.create`, schedule, endpoint |

---

## 2. End to end — um `GET /`

Código vivo: `src/main.rs` `cmd_run` → `handle_request` → `handle_request_inner`.  
`src/server/router.rs` e `src/server/api.rs` são **mortos** (não compilam). Não editar.

### Boot

1. Acha `*.cronus` no cwd. Um arquivo → `parse_with_imports`. Vários → `parse_directory` (compose).
2. AST: `App`, `Entity`, `Page`, `Style`, `Api`, `Component`, `Auth`, `Layout`, `Define`, `Webhook`.
3. `use Name` na page:
   - se existe `define Name { section … }` → **injeta seções**
   - senão o nome fica em `page.components` → **widget** depois do body
4. `style { preset aurora }` → `theme::set_preset`.
5. SQLite: `CronusDB::migrate`. Colunas auto: `id`, `_owner_id`, `created_at`, `updated_at`.
6. Listener Hyper. HMR no mesmo processo.

### Request

1. Path normalizado. Voodoo **task-local** (`voodoo::scope`) — nunca `AtomicBool` global.
2. Match `page.route` (suporta `:id`).
3. Cookie `cronus_token` / Bearer → JWT → `owner_id`.
4. Cada `section` com `bind` → `binding::resolve_binding` (filtra `_owner_id` salvo `scope:public` ou entity `shared`).
5. `ui::render_section` → HTML.
6. Envelope:
   - seções marketing (`hero`, `features`, `pricing`, `cta`, `faq`, `footer`, `topbar`, `trusted`, `testimonial`) → landing, sem sidebar
   - `layout Main { }` + `requires:auth` → sidebar declarativa
   - senão → `<main>` centrado, **sem** chrome Cooud (Produtos/Afiliados)
7. `html_response`: tokens, animações, HMR, e se opt-in o `<script voodoojs@0.13.0>`.

### Form

`section form entity:Lead` → `POST /api/leads` → insert + `_owner_id` → `on Lead.create` no `.scriptcronus`.

Auth que o kernel **já serve** (ganha de `page "/login"`):

| Path | Função |
|---|---|
| `GET /login` | HTML gerado |
| `GET /register` `/signup` | HTML gerado |
| `GET /logout` | 302 `/login` |
| `POST /api/auth/login` | JWT |
| `POST /api/auth/signup` | signup |
| `GET /api/auth/me` | user |

Pós-login (`auth_pages.rs::post_login_paths`): `auth { redirect "/" }` se existir; senão `/` se a page existe; senão `/dashboard`; senão primeira page com `requires:`. Admin: `/admin` se existir.

---

## 3. Gramática REAL (o que o parser aceita hoje)

Tokenizer keywords (`src/parser/tokenizer.rs`):  
`app entity api page style service section import compose use merge on worker component middleware env test webhook constitution must never transition deploy`

**Não** são keywords (Identifier): `auth`, `layout`, `define`, `tailwind_config`.

HTTP methods: **GET POST PATCH PUT DELETE**. HEAD/OPTIONS = erro de parse.

### App

```cronus
app "Aurora" {
  stack voodoo
  port 4747
  database sqlite "./data.db"
  constitution {
    must "prices in centavos"
    never "expor token de pagamento"
  }
}
```

`theme` **dentro de `app { }` é ignorado**. Tema vai em `style { theme dark }`. Default port 5175.

### Entity

```cronus
entity Lead {
  name    string! searchable
  email   email!  unique
  company string
  plan    enum ["starter", "studio"] default:"starter"
  amount  money! min:0
  parent  -> Customer
}
```

16 tipos: `string text email url slug phone number money percentage boolean date ulid json enum ip` + `-> Relation`. Tipo desconhecido vira **string** em silêncio.

Modificadores na **mesma linha**: `!`/`required`, `unique`, `sensitive`, `optional`, `searchable`, `index`, `featured`, `formatted`, `default:`, `min:`, `max:`, `match:"regex"`.

Dois campos na mesma linha funcionam: `author string! note text!`. `email email! unique` também (email é tipo **e** nome).

P041 (nome de entity/field/enum): `SELECT DROP INSERT DELETE UPDATE TABLE FROM WHERE OR AND UNION ALTER CREATE INDEX EXEC EXECUTE INTO VALUES SET NULL TRUE FALSE`.  
`body` e `content` **não** são reservados.  
Migrate **pula** field names `id`, `created_at`, `createdAt`, `updated_at`, `updatedAt`.

Money = **INTEGER centavos**. `2990` = R$29,90.

### Bind (parser ≠ LANGUAGE.md)

```cronus
bind Lead { query all }
bind Lead { query all where status eq "won" order created_at desc limit 20 }
bind Lead { query one where id eq route.id }
bind Lead { query count }
bind Lead { query count scope:public }
bind Order { aggregate sum(total) group created_at by:month }
```

`QueryType` só: **All | One | Count**.  
`where` usa **espaços**: `eq "paid"`, **não** `eq:"paid"` (isso é um token ColonPair e o parser não parte).  
Ops: `eq ne gt gte lt lte contains starts_with`. Sem `ends_with` / `in:[…]`.  
`scope:public` → não filtra `_owner_id` (KPI de landing / catálogo compartilhado).

### Auth / style / layout

```cronus
auth {
  entity User
  login email + password
  session jwt expires:24h
  roles [admin, user]
  redirect "/"
}

style {
  theme dark
  preset aurora          # aurora|neutral|midnight|sunset|emerald
  runtime voodoo
  accent-hex "#0ea5e9"
  font "Inter"
  radius 14px
}

layout Main {
  sidebar {
    brand "Acme"
    "Dashboard" -> "/" icon:dashboard
    divider
    "Admin" -> "/admin" icon:admin_panel_settings
  }
  topbar {
    search placeholder:"Search…"
  }
}
```

`brand` só vale **dentro de `sidebar { }`**. Sem `layout Main`, não há sidebar.

Voodoo liga com **qualquer um**: `stack voodoo` **ou** `style { runtime voodoo }`.

### Page / section / component / use

```cronus
component Save layout:inline style:button+primary+md { label "Save" }
component Email layout:stack style:input { label "Email"  text "you@cooud.app" }

page "/" type:custom {
  title "Home"
  use Save
  use Email
  section hero {
    badge "Language"
    title "One file is the product"
    subtitle "Kernel emits HTML."
    cta "Start" -> "/" primary
  }
}

page "/kit" type:components { title "Kit" }
```

`type:` default = `custom`.  
`type:components` = dump **todos** os `component` do app.  
`use Save` = só aquele.  
Landing é **inferida** (tem `hero`/`features`/…), não é um type.

Seções que o dispatcher realmente pinta: `hero features pricing cta faq trusted topbar checkout testimonial footer page-header sidebar form card links tabs accordion breadcrumb alert chart modal sheet skeleton empty error kpi timeline progress command table pagination filters dropdown toast notifications kanban dark-mode layout`.  
Aliases (`stats`→`kpi`, `data-table` como **family** de widget, etc.).

Família de widget = **primeiro segmento** de `style`: `button+primary+md` → `button`.  
`style:primary` **sem** `button+` = Button Obsidian legado. Não sequestrar.

### Import

```cronus
import Auth from "./auth.cronus"
```

Não `import "path"`. Só `.cronus`.

### O que o parser aceita mas você **não deve escrever**

`template "…"`, `style_block`, `tailwind_config` — lixo de `cronus dump`.  
`component Counter { state x  template "<button @click>" }` — parseia, renderer **ignora** reatividade.

### `.scriptcronus` (arquivo separado)

```
on Lead.create {
  log "lead created"
}
```

Não existe bloco `scriptcronus { }` no parser da linguagem.

---

## 4. Como um componente nasce (React → Cronus)

### No `.cronus` (catálogo)

```cronus
component PrimaryButton layout:inline style:button+primary+md {
  label "Save"
}
```

Itens oficiais: `label text title subtitle value item tab columns field` …  
Não há gramática para `@click` ou `{ n }`. Interação = renderer.

### No kernel (onde o pixel vive)

Ordem de dispatch (`ui/component.rs` → `cronus_ui_widgets::render`):

1. `PORTED_FAMILIES` → módulo dedicado (`cronus_ui_input.rs`, …) — paridade CONTRACT
2. `cronus_ui_interact.rs` → HTML nativo (`<dialog>`, `<input type=checkbox>`, tabs com `onclick`+`hidden`)
3. stub genérico (`data-slot="<family>"`)
4. `None` → layout legado (`inline`/`stack`/…)

**Tabs:** só `hidden` + `onclick`. Misturar `v-show` + `hidden` apaga todos os painéis.

**Hero cartão:** só se o `.cronus` tiver `card_brand` / `card_number` / `card_holder`. Senão coluna única.

**Motion:** `src/animations.rs`, `data-animate`, `.stagger` nos KPI, `prefers-reduced-motion` no CSS do kernel. Framer no React **não** atravessa. Para o widget “quicar igual”, a animação vai no **CSS/HTML do renderer**, não no `.cronus`.

### Arquivos para **adicionar/aprimorar** uma família

| Arquivo | Quando |
|---|---|
| `scripts/gen_cronus_ui_widgets.py` | novo slug em `FAMILIES` |
| `src/cronus_ui_widgets.rs` | `FAMILIES`, `PORTED_FAMILIES`, `dedicated_render` |
| `src/cronus_ui_interact.rs` | HTML nativo + Voodoo gated |
| `src/cronus_ui_<family>.rs` **novo** | port CONTRACT (padrão input/badge) |
| `src/main.rs` | `mod cronus_ui_<family>;` |
| `src/cronus_ui.rs` | chrome CVA / `[data-slot]` |
| `src/cronus_ui_tokens.css` | **não** editar paleta à mão — re-vendor de `@cronus-ui/tokens` |
| `src/voodoo.rs` | só se precisar `v-model` |
| `src/animations.rs` | só se precisar classe de motion nova |
| `cooud-ui/packages/audit/fixtures/<family>/` | fixture de paridade |
| `demos/cronus-ui/widgets.cronus` | `use` no catálogo |

Checklist ported: módulo dedicado + `PORTED_FAMILIES` + arm em `dedicated_render` para o interact **nunca** rodar nessa família.

Testes: `cargo test cronus_ui` · `cargo test voodoo`. Manter: 173 families, sem `zinc-`/`amber-`, `style:primary` → None, sem `v-data` com runtime off.

---

## 5. Biblioteca React que vamos espelhar

Path: **`/Users/pedrogbraz/projects/cooud/cooud-ui`** (monorepo `cronus-ui-monorepo` v0.7.6).

| Path | Pacote |
|---|---|
| `packages/ui` | `@cronus-ui/ui` — um arquivo por família, CVA, `data-slot`, Radix |
| `packages/tokens` | `@cronus-ui/tokens` — aurora/neutral/midnight/sunset/emerald × light/dark |
| `packages/theme` | provider |
| `packages/audit` | dual preview React × kernel |
| `CONTRACT.md` | lei do componente React (tokens semânticos, sem `bg-zinc-900`) |

~212 slugs no docs; ~173 já têm **nome** no kernel; **12** têm renderer CONTRACT. O resto é stub.

Slugs React **fora** de `FAMILIES` (precisam de family nova se quiser 1:1): AI (`conversation`, `prompt-input`, `reasoning`, …), `number-flow`, `globe-3d`, `bouncy-accordion`, `token-swap`, etc.

Estilo React: Tailwind v4 semântico + CVA + `motion` v12. Overlay: Radix + vaul + sonner. **Sem CSS modules.**

Kernel já injeta a mesma `tokens.css`. O gap é **estrutura HTML + motion**, não a paleta.

Playbook de uma família (ex. Dialog):

1. Abrir `cooud-ui/packages/ui/src/components/dialog.tsx` + CONTRACT.
2. Decidir HTML nativo (`<dialog>`) — já existe em `cronus_ui_interact.rs`.
3. Subir para `PORTED_FAMILIES` + módulo dedicado se o visual CVA/motion ainda não bate.
4. Fixture em `packages/audit/fixtures/dialog/`.
5. No `.cronus`: `component Confirm layout:stack style:dialog { label "Delete project"  text "…" }` + `use Confirm`.
6. **Não** traduzir `framer-motion` para o arquivo `.cronus`.

---

## 6. Exemplos vivos neste workspace

| App | Path | O que prova |
|---|---|---|
| Landing | `landing/app.cronus` (cwd `cooud/landing`, port 5310) | hero/features/pricing/faq/form, sem HTML na fonte |
| Widgets | `cronus-kernel/demos/cronus-ui/widgets.cronus` | `use Save` / `use Email` / tabs nativos |
| Button | `demos/cronus-ui/button.cronus` | `style:button+…` **sem** Voodoo |
| Aurora template | `templates/cronus-ui.cronus` | `cronus new aurora` |
| Stitch / NovaPay / Nova Core | `demos/stitch-dashboard`, `demos/saas-billing`, `examples/nova-core` | `layout Main` + bind, puro Cronus |

CLI: `cronus run` no diretório do `.cronus`. `cronus parse arquivo.cronus`. `cronus new aurora`.

---

## 7. NUNCA (próxima sessão)

1. HTML / JSX / CSS / TS / `@click` / `{ count }` **dentro** de `.cronus`.
2. `template "…"`, `style_block`, `tailwind_config`.
3. Importar `.tsx` / `.html` / `.css` (import só `.cronus`).
4. `style:primary` sequestrar o Button legado.
5. Forçar `preset aurora` em app que não pediu.
6. Paleta `zinc-900` / `bg-neutral-` no kernel ou na fonte.
7. Editar `src/server/router.rs` / `api.rs`.
8. `AtomicBool` global para Voodoo.
9. `where status eq:"paid"` — use `eq "paid"`.
10. `query sum` — use `query count` ou `aggregate sum(x) group …`.
11. Animar “no Cronus” com Framer. Animar no **renderer**.
12. Inventar cartão ULTIMA no hero sem `card_brand`.

---

## 8. Mentiras comuns na documentação antiga

| Escrito em SKILL / LANGUAGE.md | Realidade |
|---|---|
| `where status eq:"paid"` | `where status eq "paid"` |
| `group_by:created_at interval:month` | `group created_at by:month` |
| `aggregate sum field:total` | `aggregate sum(total)` + `group` |
| `layout { brand "X" sidebar {` | `sidebar { brand "X"` |
| `import "path"` | `import Alias from "path"` |
| `app { theme dark }` | ignorado; use `style { theme dark }` |
| `stack react + tailwind` liga React | não liga |
| 51 section types | ~39 canônicas + aliases + families |
| `component { state; template @click }` | parseia; **não** é widget reativo |

---

## 9. Prompt para colar na próxima sessão

```
Leia cronus-kernel/.agents/HANDOFF-LANGUAGE.md, llms.txt, VOODOO.md, AGENTS.md.
Biblioteca React: cooud-ui/packages/ui (CONTRACT.md). Tokens já vendored no kernel.

Tarefa: portar família <NOME> de @cronus-ui/ui para o kernel (HTML nativo +
--cronus-* + motion no renderer). Catálogo em .cronus só declara
component X style:<family>+<variant>. Sem JSX/template/CSS na fonte.

Não quebrar style:primary legado. Testes: cargo test cronus_ui && cargo test voodoo.
Paridade: cooud-ui/packages/audit.
```

Troque `<NOME>` por `dialog`, `tabs`, `data-table`, `animated-button`, etc.

---

## 10. Mapa rápido de pastas

```
cronus-kernel/
  src/parser/          gramática
  src/binding.rs       bind → SQL
  src/database.rs      migrate SQLite
  src/main.rs          cmd_run + HTTP vivo
  src/ui/              seções (hero, kpi, form, layout)
  src/cronus_ui*.rs    tokens + 173 families + interact
  src/voodoo.rs        opt-in runtime
  src/animations.rs    motion
  demos/cronus-ui/     catálogo use
  templates/cronus-ui.cronus
  .agents/HANDOFF-LANGUAGE.md   ← este arquivo

cooud-ui/
  packages/ui/src/components/   spec React
  packages/tokens/              paleta
  packages/audit/               paridade
  CONTRACT.md
```
