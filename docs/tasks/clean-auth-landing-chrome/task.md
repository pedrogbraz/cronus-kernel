# clean-auth-landing-chrome

Risco: critical

## Pedido

A camada que o utilizador vê no saas ainda é amadora: login/register em `fetch` + `localStorage` de contas; landing com Google Fonts, Tailwind CDN, Chart.js e o blob JS do `create-modal` (tema claro, `TeamMember`); Sign Out em `onclick`+`fetch`. `POST /login` hoje devolve o HTML do GET.

Escopo: `auth_pages.rs`, `routes/sessions.rs` (POST form), `pages.rs` GET login error, landing + Sign Out em `layout.rs`, `theme.rs` `font_links`. Testes HTTP. CHANGELOG/AGENTS. Sem crates. Não reescreve `dashboard.rs` nem o runtime de ações nas páginas com sidebar.

## Análise

Fatos:
- `generate_login_page` / `generate_register_page`: JS, `saved_accounts`, `fetch /api/auth/*`, fontes Google, `onmouseover`.
- `pages.rs` trata GET e POST `/login` iguais (HTML).
- `render_layout_landing_ex` injeta Google Fonts, `cdn.tailwindcss.com`, Chart.js, `CRONUS_ANIMATIONS_JS` (`create-modal`, `#e5e7eb`).
- Sidebar: `onclick="fetch('/api/auth/logout'…"`.
- CSS local `CRONUS_TAILWIND` já existe no landing; o CDN é duplicado.

Decisões:
- Login/register: `<form method="post">` nativo, `autocomplete`, sem `<script>`. Erro no HTML (POST falhou) ou `?error=` no GET.
- `POST /login` e `POST /register` em `sessions.rs`: form-urlencoded → JSON para `handle_auth`; 2xx → 302 + Set-Cookie; senão HTML com erro.
- `POST /logout`: limpa cookie, 302 `/login`. GET `/logout` inalterado (não limpa).
- Sign Out: `<form method="post" action="/logout">`.
- Landing: sem Google Fonts, Chart.js, CDN Tailwind, `CRONUS_ANIMATIONS_JS`. `font-family: system-ui`.
- `theme::font_links()` deixa de apontar para fonts.googleapis.com.
- JSON `/api/auth/login` e `/api/auth/signup` mantêm-se.

## Plano

1. Reescrever `auth_pages.rs` (login + register).
2. `parse_urlencoded` + POST `/login` `/register` `/signup` `/logout` em `sessions.rs`.
3. GET `/login` lê `error` da query.
4. Landing: cortar CDNs e `anim_js`. Teste `landing_layout_*`.
5. Sign Out form no layout declarativo.
6. HTTP: form login 302+cookie; landing sem google/chart/create-modal; POST logout.
7. CHANGELOG + AGENTS.
8. harness quick, full (critical).

## Revisão

CSRF já corre antes de `sessions`. Form same-origin manda Origin. Não implementa WebAuthn nem remove HMR/SSE em dev. Adequado.

## Validação

- `GET /login` sem `<script` e sem `localStorage`.
- `POST /login` form → 302 e `Set-Cookie`.
- `render_layout_landing` sem `fonts.googleapis` / `chart.js` / `create-modal`.
- `POST /logout` limpa cookie.

## Ajustes

Nenhum ainda.

## Entrega

Critérios de aceitação:
- GET /login is a POST form to /login with no script and no localStorage
- POST /login with form body sets the cookie and redirects; failed login re-renders the form with an error
- Public landing HTML has no fonts.googleapis, chart.js, or create-modal
- Sidebar Sign Out is a POST form to /logout that clears the cookie
