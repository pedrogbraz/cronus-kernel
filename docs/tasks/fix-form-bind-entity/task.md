# fix-form-bind-entity

Risco: critical

## Pedido

No saas, Save em `/projects/new` mostra toast vermelho "Not allowed". O HTML do form não tem `data-cronus-entity` (só `bind Project`). O runtime POST `/_form/form` com `entity:""` → `UNKNOWN_FORM`.

Escopo: `section_form.rs` lê a entidade do bind como `actions::section_entity`. Teste de HTML + POST `/_form/form`. CHANGELOG. Sem crates.

## Análise

Fatos (reproduzido no studio):
- `<form id="cronus-form" action="#" … data-cronus-form data-cronus-section="form">` sem `data-cronus-entity`.
- POST com `entity:""` → 403 UNKNOWN_FORM "Not allowed".
- POST com `entity:"Project"` → 201 e toast "Project created".
- `render_form_section` só lê `section.config["entity"]`. `find_declared_form` já usa bind.

Decisão: a entidade do form é `config["entity"]` senão `binding.entity`.

## Plano

1. `section_form.rs`: entidade do bind.
2. Teste: HTML contém `data-cronus-entity="Project"`; POST autenticado cria.
3. CHANGELOG.
4. harness quick + full (critical).

## Revisão

Não muda authz. Só o atributo que o cliente já deveria mandar. Adequado.

## Validação

- Form bind-only emite `data-cronus-entity`.
- POST `/_form/form` `{entity:Project,data:{name}}` 2xx com sessão.

## Ajustes

Nenhum ainda.

## Entrega

Critérios de aceitação:
- A form with bind Project and no config entity emits data-cronus-entity=Project
- POST /_form/form with that entity as a signed-in user creates a Project (not UNKNOWN_FORM)
