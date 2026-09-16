# Evidências

- Commit: `7e851da` `fix(security): gate /_files, HMAC inbound /hooks, and live User role`
- `harness check --tier quick --task p1-sec-files-hooks-role`: passed (`eeb3d6f144a147d397c5d5494722df4e`).
- Unit: `viewer_can_read_requires_a_visible_file_row`, `inbound_hmac_accepts_fresh_signature_and_rejects_the_rest`, `live_role_overrides_jwt_when_row_exists`.
- HTTP: `files_require_session_and_a_visible_row` (401/404/200+nosniff), `unmatched_webhook_path_requires_hmac_then_404`, `live_role_from_user_row_beats_stale_jwt` (demote 403, promote 200).
- Limits: deleted User row still keeps JWT (tests mint tokens without rows). No TLS inbound.
