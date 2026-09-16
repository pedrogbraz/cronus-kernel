//! Login and register page generators extracted from main.rs.

use super::state::AppState;

/// Post-login href: `auth { redirect "/" }`, else `/` if that page exists, else `/dashboard`, else first authed page.
pub(crate) fn post_login_paths(state: &AppState) -> (String, String) {
    let has = |r: &str| state.pages.iter().any(|p| p.route == r);
    let home = if let Some(ref r) = state.auth_redirect {
        r.clone()
    } else if has("/") {
        "/".to_string()
    } else if has("/dashboard") {
        "/dashboard".to_string()
    } else {
        state
            .pages
            .iter()
            .find(|p| p.requires.is_some())
            .map(|p| p.route.clone())
            .unwrap_or_else(|| "/".to_string())
    };
    let admin = if has("/admin") {
        "/admin".to_string()
    } else {
        home.clone()
    };
    (home, admin)
}

fn logo_letter(app: &str) -> String {
    app.chars().next().unwrap_or('C').to_uppercase().to_string()
}

fn auth_error_html(error: Option<&str>) -> String {
    match error.map(str::trim).filter(|s| !s.is_empty()) {
        Some(msg) => format!(
            r#"<p class="err" role="alert">{}</p>"#,
            crate::cronus_ui_kit::esc(msg)
        ),
        None => String::new(),
    }
}

fn auth_shell(title: &str, logo: &str, heading: &str, sub: &str, form: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en" data-cronus-auth>
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:system-ui,-apple-system,sans-serif;-webkit-font-smoothing:antialiased;background:#0a0a0a;color:#fafafa;min-height:100vh;display:flex;align-items:center;justify-content:center}}
input{{width:100%;padding:10px 14px;font-size:14px;background:#171717;border:1px solid #262626;border-radius:10px;color:#fafafa;outline:none}}
input:focus{{border-color:#525252}}
.btn{{width:100%;padding:10px 14px;font-size:14px;font-weight:600;border:none;border-radius:10px;background:#fafafa;color:#0a0a0a;cursor:pointer}}
.btn:hover{{opacity:0.9}}
.err{{padding:10px 14px;border-radius:10px;font-size:13px;text-align:center;background:#450a0a;color:#fca5a5;border:1px solid #7f1d1d}}
a{{color:#fafafa;font-weight:600;text-decoration:none}}
.wrap{{width:100%;max-width:400px;padding:32px}}
.brand{{text-align:center;margin-bottom:32px}}
.mark{{width:48px;height:48px;background:#fafafa;border-radius:12px;display:flex;align-items:center;justify-content:center;margin:0 auto 16px;color:#0a0a0a;font-weight:700;font-size:20px}}
h1{{font-size:24px;font-weight:700;margin:0 0 8px}}
.sub{{font-size:14px;color:#a3a3a3}}
form{{display:flex;flex-direction:column;gap:16px}}
.alt{{text-align:center;font-size:14px;color:#a3a3a3}}
</style>
</head>
<body>
<div class="wrap">
  <div class="brand">
    <div class="mark">{logo}</div>
    <h1>{heading}</h1>
    <p class="sub">{sub}</p>
  </div>
  {form}
</div>
</body></html>"##,
        title = crate::cronus_ui_kit::esc(title),
        logo = crate::cronus_ui_kit::esc(logo),
        heading = crate::cronus_ui_kit::esc(heading),
        sub = crate::cronus_ui_kit::esc(sub),
        form = form
    )
}

pub(crate) fn generate_login_page(state: &AppState, error: Option<&str>) -> String {
    let app = &state.app.name;
    let form = format!(
        r##"<form method="post" action="/login">
    <input name="email" type="email" autocomplete="username" placeholder="Email" required />
    <input name="password" type="password" autocomplete="current-password" placeholder="Password" required />
    {err}
    <button type="submit" class="btn">Sign In</button>
    <p class="alt">Don't have an account? <a href="/register">Register</a></p>
  </form>"##,
        err = auth_error_html(error)
    );
    auth_shell(
        &format!("Login — {app}"),
        &logo_letter(app),
        "Welcome back",
        "Sign in to your account",
        &form,
    )
}

pub(crate) fn generate_register_page(state: &AppState, error: Option<&str>) -> String {
    let app = &state.app.name;
    let form = format!(
        r##"<form method="post" action="/register">
    <input name="name" type="text" autocomplete="name" placeholder="Full name" required />
    <input name="email" type="email" autocomplete="username" placeholder="Email" required />
    <input name="password" type="password" autocomplete="new-password" placeholder="Password (15+ characters)" required minlength="15" />
    {err}
    <button type="submit" class="btn">Sign Up</button>
    <p class="alt">Already have an account? <a href="/login">Sign in</a></p>
  </form>"##,
        err = auth_error_html(error)
    );
    auth_shell(
        &format!("Register — {app}"),
        &logo_letter(app),
        "Create your account",
        "Get started for free",
        &form,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_security_tests::state_from;

    const APP: &str = r#"app "SaaS Starter" { port 5175 }
auth { entity User login email + password session jwt roles [member] redirect "/dashboard" }
entity User { name string email email! password string! sensitive role string }
page "/dashboard" type:custom requires:auth {}
"#;

    #[test]
    fn login_page_is_a_native_form_without_script() {
        let html = generate_login_page(&state_from(APP), None);
        assert!(html.contains(r#"<form method="post" action="/login">"#));
        assert!(html.contains(r#"autocomplete="username""#));
        assert!(!html.contains("<script"));
        assert!(!html.contains("localStorage"));
        assert!(!html.contains("fonts.googleapis"));
        assert!(!html.contains("saved_accounts"));
    }

    #[test]
    fn login_page_shows_escaped_error() {
        let html = generate_login_page(&state_from(APP), Some("<xss>"));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("&lt;xss&gt;"));
        assert!(!html.contains("<xss>"));
    }
}
