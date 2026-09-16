# fix-static-route-beats-param

Risco: normal

## Pedido

No app saas real, `GET /projects/new` pinta a página de detalhe (`id = "new"`) com o botão Delete, não o form de criar. O banner de `cronus run` aponta para `localhost` enquanto o bind é `127.0.0.1` (`::1` recusa ligação).

Escopo: matching de rotas em `access` + `routes/pages`, teste HTTP, template saas (ordem), banner em `cli/run.rs`. Sem crates.

## Análise

Fatos:
- `pages.rs` faz `.find(|p| route_pattern_matches)`. A primeira página que casa ganha.
- `route_pattern_matches("/projects/:id", "/projects/new")` é true (`:id` aceita qualquer segmento não vazio).
- Em `templates/saas.cronus`, `/projects/:id` está **antes** de `/projects/new`.
- Resultado observado: main = "Project Delete project". KPI/tabela no dashboard estão corretos quando há rows.
- Banner: `http://localhost:{port}`; o servidor escuta `127.0.0.1`. `curl [::1]:5175` falha.

Decisões:
- Entre vários matches, ganha o padrão com menos parâmetros; empate → o exact string. Exact `path == pattern` sempre ganha.
- Usar essa escolha em `pages.rs` (página e `requires:`).
- Reordenar o template (estático antes do param) para o dump/ler humano.
- Banner usa o IP de bind (`127.0.0.1`), não a palavra localhost.

## Plano

1. `best_matching_route(patterns, path)` em `access.rs` + testes unitários.
2. `pages.rs` usa-o em vez de `.find`.
3. HTTP: app com `/projects/:id` e `/projects/new`; GET `/projects/new` contém o form, não Delete.
4. `saas.cronus`: `/projects/new` antes de `:id`.
5. Banner `cli/run.rs`.
6. CHANGELOG + AGENTS known gaps se mencionar rotas.
7. harness quick, depois full.

## Revisão

Não reescreve login/register JS, CSP, nem o runtime de 14 scripts na landing. Adequado ao bug que o utilizador vê ao clicar "New project".

## Validação

- `route_patterns_prefer_static_over_param`
- HTTP `projects_new_is_the_form_not_a_detail_id`

## Ajustes

Nenhum ainda.

## Entrega

Critérios de aceitação:
- GET /projects/new matches the static page, not /projects/:id
- GET /projects/<ulid> still matches the detail page
- cronus run banner prints the bind address 127.0.0.1, not localhost
