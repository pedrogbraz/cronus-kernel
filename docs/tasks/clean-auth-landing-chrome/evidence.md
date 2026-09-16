# Evidências

- Commit: `f5de9e2` `fix(ui): native login forms and drop landing CDN chrome`
- `harness check --tier full --task clean-auth-landing-chrome`: passed (`4387b5ae1fd24efca4dfad6e69b6b8eb`), including `kernel-critical`.
- Unit: `login_page_is_a_native_form_without_script`, `login_page_shows_escaped_error`, `landing_layout_has_no_google_fonts_or_create_modal`, `json_from_urlencoded_decodes_plus_and_percent`.
- HTTP: `protected_page_redirects_and_public_page_renders` (GET /login has no script), `form_login_redirects_with_cookie_and_logout_form_clears_it`.
- Limits: `dashboard.rs` Google Fonts links and `CRONUS_ANIMATIONS_JS` on sidebar layouts are unchanged.
