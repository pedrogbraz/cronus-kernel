# p2-sec-logout-timing-cors

Risco: critical

## Pedido

Fechar o P2 de segurança do mapa original: GET `/logout` é CSRF-logout; login em e-mail inexistente não corre Argon2; signup revela e-mail já registado; SQLite/audit não são 0600; não há HSTS; denylist IPv6 de webhook falta 6to4/benchmark; `CRONUS_CORS_ORIGIN=*` é ecoado no JSON; `is_strong_password` (min 8) está morto ao lado da política de 15 caracteres.

Escopo: session + pages logout, login/signup, `CronusDB::open` / `AuditTrail::open`, `security_headers` HSTS, `webhook::is_blocked_ip`, `cors_origin`, apagar `is_strong_password`. Testes no mesmo arquivo. CHANGELOG/AGENTS/LANGUAGE. Sem crates. Sem TLS client de webhook.

## Análise

Fatos:
- `POST /api/auth/logout` já limpa o cookie. `GET /logout` em `routes/pages.rs` faz 302 `/login` **e** `Set-Cookie` Max-Age=0. O layout já faz logout por POST. `http_dispatch_tests` afirma o GET CSRF.
- Login `Ok(None)` devolve 401 `"invalid credentials"` sem `verify_password`. Hash Argon2id só corre quando a row existe.
- Signup duplicado: 400 `"email already registered"`. Insert fail: 500 `"could not create account"`.
- `Connection::open` em `database.rs` / `audit.rs` não ajusta modo. Chaves JWT já são 0600.
- `security_headers()` não inclui HSTS. `--prod` já existe em `http_guard::policy().mode`.
- `is_blocked_ip` IPv6 cobre loopback/ULA/link-local/doc (`2001:db8`). Não cobre `2002::/16` (6to4) nem `2001:2::/48` (benchmarking).
- SSE recusa `CRONUS_CORS_ORIGIN=*`. `json_response` ecoa o valor, incluindo `*`.
- `security.rs::is_strong_password` (len≥8) não tem callers; `auth::validate_new_password` exige 15.

Decisões:
- GET `/logout` → 302 `/login` **sem** Set-Cookie. POST `/api/auth/logout` inalterado.
- Login sem row: `verify_password` contra um hash dummy (`OnceLock` de `hash_password("timing-dummy")`), depois 401 igual.
- Signup duplicado: 400 `"could not create account"` (mesmo texto do insert, sem 500).
- Unix: após abrir o ficheiro SQLite, `chmod 0600` no path e nos `-wal`/`-shm` se existirem. No-op noutros OS.
- HSTS `max-age=31536000; includeSubDomains` só quando `RunMode::Production`. Pedidos HTTP em dev não enviam. X-Forwarded-Proto sozinho não basta (não há peer neste helper).
- Bloquear `seg[0]==0x2002` e `seg[0]==0x2001 && seg[1]==0x0002`.
- `cors_allow_origin`: vazio, `same-origin` e `*` → nenhum `Access-Control-Allow-Origin`.
- Apagar `is_strong_password`; não reexportar o mínimo de 8.

## Plano

1. `pages.rs` GET `/logout` sem cookie; atualizar `http_dispatch_tests`.
2. Dummy Argon2 + signup genérico em `session.rs` + testes unitários.
3. `tighten_file_mode` em database + audit; teste unix.
4. HSTS em `security_headers` se production; teste com policy.
5. IPv6 denylist + casos no teste existente.
6. `cors_allow_origin` + teste `*` não ecoa.
7. Remover `is_strong_password`.
8. CHANGELOG, AGENTS security, LANGUAGE §11 `/logout`.
9. harness `quick`, depois `full` **e** `critical`.

## Revisão

GET `/logout` deixa de ser logout: um GET cross-site já não limpa a sessão. Quem tinha bookmark em `/logout` só vai ao login, cookie intacto — o botão da UI já usa POST. Dummy Argon2 não iguala o tempo de `find_by_field` (isso fica). HSTS só em `--prod`. Sem crate TLS. Adequado ao P2.

## Validação

- `http_dispatch_tests`: GET `/logout` 302, Set-Cookie ausente ou sem Max-Age=0.
- `session::` login dummy + signup genérico.
- `webhook::` 6to4/benchmark blocked.
- `database`/`audit` mode 0600 (unix).
- `security_headers` contém HSTS só em production.

## Ajustes

Nenhum ainda.

## Entrega

Critérios de aceitação:
- GET /logout redirects to /login without clearing the session cookie; POST /api/auth/logout still clears it
- Unknown-email login still runs Argon2 verify; signup duplicate email does not say already registered
- SQLite and audit files are mode 0600 on unix; --prod responses send Strict-Transport-Security
- 2002::/16 and 2001:2::/48 are blocked webhook targets; CRONUS_CORS_ORIGIN=* does not set ACAO; is_strong_password is gone
