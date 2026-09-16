# p1-sec-files-hooks-role

Risco: critical

## Pedido

Fechar os três buracos P1 de segurança: `GET /_files/*` hoje é URL de capability sem sessão; `POST /hooks/*` inbound não verifica HMAC (o outbound já assina); o JWT carrega `role` por até 24h e a demotion no User não vale até o token expirar.

Escopo: `files` route + helper, inbound HMAC em `webhook.rs` + `routes/scripts.rs`, `Access` live role a partir do SQLite, testes HTTP em `http_dispatch_tests.rs`, CHANGELOG e modelo em AGENTS.md. Sem crates. Sem TLS inbound.

## Análise

Fatos:
- `routes/files.rs` serve qualquer nome seguro em disco. `files.rs` já valida o nome (`..` / `/` recusados).
- Outbound (`webhook.rs`) já emite `X-Cronus-Timestamp` e `X-Cronus-Signature: sha256=<hex>` com o mesmo secret de `.cronus/webhook.key` / `CRONUS_WEBHOOK_SECRET`.
- `viewer_from_headers` copia `claims.role` do JWT. `http_guard::request_role` usa o mesmo para rotas internas.
- `unmatched_webhook_path_falls_through_to_404` POSTa `{}` sem assinatura e espera 404.

Decisões:
- `/_files`: 401 sem sessão; 404 se nenhuma linha visível (owner/admin/shared) contém o path num campo `file`; 200 + `X-Content-Type-Options: nosniff` se visível. Anônimo nunca lê arquivo, mesmo em entity `shared` (P1 pediu sessão).
- `/hooks`: todo `POST /hooks/*` exige HMAC + timestamp com skew ≤ 300s (anti-replay). Path desconhecido depois da assinatura continua 404.
- Role: depois de verificar o JWT, ler a row do auth entity por `id`. Sem row → sem sessão. `role` não vazio no banco substitui o JWT.

## Plano

1. `files::viewer_can_read` — percorre entities com campo `file`, `find_one` com scope de `read_scope`.
2. `routes/files.rs` — sessão via `Access::from_state`, 401/404/200 + nosniff.
3. `webhook::verify_inbound(headers, body)` — timestamp, assinatura, compare constante; testes unitários (válido, velho, forjado, ausente).
4. `routes/scripts.rs` — `POST /hooks/` lê o body, verifica, depois despacha ou 404.
5. `Access::from_state` + `apply_live_role`; trocar call sites HTTP que têm `AppState`. `guard_internal` usa role viva.
6. Testes HTTP: files 401/404/200; hooks 401/404; role stale.
7. CHANGELOG + AGENTS.md security bullets.
8. `harness check` quick, depois full **e** critical.

## Revisão

HMAC reutiliza o MAC outbound — um contrato só. Live role no Access cobre REST/GraphQL/forms/SSE/páginas se todos os call sites passarem por `from_state`. Rotas internas (`/zeus`) também, via `guard_internal`. Não implementa HTTPS webhook (P2, crate TLS). Adequado e proporcional ao risco.

## Validação

- Testes unitários de HMAC inbound e de `viewer_can_read`.
- `http_dispatch_tests` no dispatcher real.
- `harness check --tier full` e o check `critical` (risk=critical).

## Ajustes

Nenhum ainda.

## Entrega

Critérios de aceitação:
- GET /_files without a session is 401; a signed-in user who cannot see a row holding that path gets 404; the owner or admin gets 200 with nosniff
- POST /hooks/* without a valid X-Cronus-Signature is 401; a valid HMAC on an unknown path is 404; a valid HMAC on a registered webhook runs the handler
- A JWT with a stale role is not admin after the User row is demoted; a promoted User row is admin without a new token
