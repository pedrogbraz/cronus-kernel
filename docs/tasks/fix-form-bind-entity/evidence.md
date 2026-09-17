# Evidências

- Commit: `8de9bac` `fix(ui): form bind entity reaches data-cronus-entity`
- `harness check --tier full --task fix-form-bind-entity`: passed (`455d498df20e4f94b6e675f98230a33b`), including `kernel-critical`.
- Unit: `bind_entity_sets_data_cronus_entity`.
- HTTP: `protected_page_redirects_and_public_page_renders` asserts `data-cronus-entity="Note"` on `/dash`.
- Repro: live studio POST `/_form/form` with empty entity → 403 UNKNOWN_FORM; with `Project` → 201.
