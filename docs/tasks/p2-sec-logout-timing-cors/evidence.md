# Evidências

- Commit: `98a1024` `fix(security): stop CSRF logout, dummy Argon2, HSTS, tight CORS`
- `harness check --tier full --task p2-sec-logout-timing-cors`: passed (`69b12ae69666400ebec64f3448400e77`), including `kernel-critical`.
- Unit: `unknown_email_login_is_invalid_credentials`, `duplicate_signup_does_not_reveal_the_email`, `dummy_password_hash_is_argon2id`, `open_sets_owner_only_mode`, `production_headers_include_hsts`, `wildcard_cors_origin_is_refused`, `blocks_loopback_private_link_local_and_metadata`.
- HTTP: `protected_page_redirects_and_public_page_renders` (GET `/logout` 302 without Max-Age=0).
- Limits: dummy Argon2 does not hide `find_by_field` time. HSTS only in `--prod`. No TLS webhook client. Duplicate signup is 400 vs insert-fail 500.
