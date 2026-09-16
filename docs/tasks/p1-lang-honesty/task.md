# p1-lang-honesty

Risco: normal

## Pedido

Tornar honesta a superfície de linguagem que ainda parseia e não avisa: campos duplicados na entity, `page { use X }` sem define/component, `component` com params/state/template (sem runtime), alias de `import` ignorado, e o `templates/ai-prompt.md` ensinando blocos `LANG_001`.

Escopo: `src/parser/` (mod, compose, diagnostic), `src/cli/mcp/error_codes.rs`, `LANGUAGE.md` §15.9, `CHANGELOG.md`, `templates/ai-prompt.md`, `llms-full.txt` (never-do). Sem crates. Sem auth/HTTP (isso é P1 security, outra tarefa).

## Análise

Fatos (código, 2026-09-16):
- `parse_entity` empurra fields sem checar nome repetido (`src/parser/mod.rs`). Dumps em `templates/ecosystem/` exploram isso.
- `expand_defines` deixa nomes que não são `define` em `page.components` (kit). `use` de um nome que também não é `component` não gera erro (`compose.rs` splice_defines).
- `ComponentNode` guarda `params`, `state`, `template`; `src/ui/component.rs` só usa layout+style+items. Silêncio.
- `ImportNode.alias` é preenchido e o loader não filtra por alias (`parse_import`).
- `templates/ai-prompt.md` ainda mostra `service`, `worker`, `middleware`, `on order.created`, `required` em vez de `!`, camelCase.
- `on update when status changed to paid` não é a gramática; a real é `on update status { when paid { } }` (já documentado no P0). Esta tarefa só adiciona um hint no parse se o inglês aparecer, sem nova gramática.
- Códigos novos precisam ir em LANGUAGE.md §15.9 e `error_codes.rs` (`cargo test mcp_error_codes`).

Dependências: `context_grammar` não lista error codes. `parse_diagnostics` já agrega `parser.diagnostics`.

Riscos: dumps `templates/ecosystem/` e `demos/vinext-dumps` podem passar a falhar `every_template_and_demo_parses` se campos duplicados virarem erro no parse (hoje só parse, não `build --ai`). O teste de parse-all não pode quebrar o crate: dumps que duplicam campos devem continuar parseando com diagnóstico recuperável, ou o teste usa `parse` que falha no primeiro fatal. Preferir diagnóstico recuperável (como TYPE_001) para duplicate fields, para `parse_collect` seguir.

## Plano

1. `FIELD_005` — segundo campo com o mesmo nome na entity: diagnostic recuperável no token do segundo, first kept. Teste no parser.
2. Após `expand_defines`, `RESOLVE_001` (ou o mesmo código com mensagem de `use`) se `page.components` restante não bate com nenhum `component` name. Teste compose/parser.
3. `LANG_002` se `params` não vazio, `state` não vazio, ou `template` Some no component. Hint: use `define` + `page { use }` ou kit `style:family`. Teste parser.
4. `COMPOSE_003` se `import Alias from "file"` e o alias não é vazio — hoje o alias nunca é usado; diagnosticar como unused (não quebrar `import "file"` sem alias). Teste compose.
5. Hint em `parse_effect_block` quando vier `when` + `changed` estilo inglês.
6. Reescrever `templates/ai-prompt.md` para a gramática canônica (`!`, sem service/worker/middleware, money centavos, `auth:jwt`).
7. LANGUAGE.md §15.9 + error_codes.rs + CHANGELOG + never-do em llms-full.txt.
8. `harness check --tier quick` no diff; `full` antes de integrar.

## Revisão

O plano não implementa componentes reativos nem import seletivo de verdade — só honestidade. Alias unused em vez de selective import evita fingir uma feature. Dumps parse-only: diagnóstico recuperável não zera o AST. Não mistura `/_files` nem HMAC. Adequado para um ciclo de harness.

## Validação

- Testes `#[test]` no parser/compose para cada código novo.
- `cargo test mcp_error_codes` e `context_grammar` se llms-full mudar.
- Abrir um `.cronus` mínimo com cada caso e `cronus build --ai` (via teste de `parse_diagnostics` / `validate_source_ai`).

## Ajustes

Nenhum ainda; a implementação segue este plano. Se um dump first-party quebrar o teste de parse-all, o diagnóstico permanece e o teste de parse-all usa o caminho que coleta diagnostics (já existe `parse_collect`).

## Entrega

Critérios de aceitação:
- Duplicate field names in one entity fail build with a stable code and the second field span
- page use Missing fails when Missing is neither define nor component
- component params, state, or template is a build error with a fix hint, not a silent no-op
- unused import alias is diagnosed; ai-prompt.md does not teach service, worker, or middleware

Demonstrar cada critério e registrar limitações em evidence.md. Etapas não exigem aprovações humanas adicionais quando a autorização já existe.
