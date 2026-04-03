#![allow(dead_code, unused_imports)]
//! Login and register page generators extracted from main.rs.

use super::state::AppState;

pub(crate) fn generate_login_page(state: &AppState) -> String {
    let app_name = &state.app.name;
    let logo_letter = app_name.chars().next().unwrap_or('C').to_uppercase().to_string();
    format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Login — {app}</title>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:'Inter',system-ui,-apple-system,sans-serif;-webkit-font-smoothing:antialiased;background:#0a0a0a;color:#fafafa;min-height:100vh;display:flex;align-items:center;justify-content:center}}
input{{width:100%;padding:10px 14px;font-size:14px;background:#171717;border:1px solid #262626;border-radius:10px;color:#fafafa;outline:none;transition:border-color 0.15s}}
input:focus{{border-color:#525252}}
.btn{{width:100%;padding:10px 14px;font-size:14px;font-weight:600;border:none;border-radius:10px;background:#fafafa;color:#0a0a0a;cursor:pointer;transition:opacity 0.15s}}
.btn:hover{{opacity:0.9}}
</style>
</head>
<body>
<div style="width:100%;max-width:400px;padding:32px">
  <div style="text-align:center;margin-bottom:32px">
    <div style="width:48px;height:48px;background:#fafafa;border-radius:12px;display:flex;align-items:center;justify-content:center;margin:0 auto 16px">
      <span style="color:#0a0a0a;font-weight:700;font-size:20px">{logo}</span>
    </div>
    <h1 style="font-size:24px;font-weight:700;margin:0 0 8px">Welcome back</h1>
    <p style="font-size:14px;color:#a3a3a3">Sign in to your account</p>
  </div>
  <form id="loginForm" style="display:flex;flex-direction:column;gap:16px">
    <input name="email" type="email" placeholder="Email" required />
    <input name="password" type="password" placeholder="Password" required />
    <div id="error" style="display:none;padding:10px 14px;border-radius:10px;font-size:13px;text-align:center;background:#450a0a;color:#fca5a5;border:1px solid #7f1d1d"></div>
    <button type="submit" class="btn">Sign In</button>
    <p style="text-align:center;font-size:14px;color:#a3a3a3">Don't have an account? <a href="/register" style="color:#fafafa;font-weight:600;text-decoration:none">Register</a></p>
  </form>
</div>
<script>
document.getElementById('loginForm').addEventListener('submit', async (e) => {{
  e.preventDefault();
  const btn = e.target.querySelector('button');
  btn.disabled = true; btn.textContent = 'Loading...';
  const data = Object.fromEntries(new FormData(e.target));
  try {{
    const res = await fetch('/api/auth/login', {{ method: 'POST', headers: {{'Content-Type': 'application/json'}}, body: JSON.stringify(data) }});
    const json = await res.json();
    if (json.token) {{
      // cookie set by server via Set-Cookie header (HttpOnly + Secure + SameSite=Strict)
      localStorage.setItem('token', json.token);
      localStorage.setItem('user', JSON.stringify(json.user || {{}}));
      window.location.href = '/';
    }} else {{
      const err = document.getElementById('error');
      err.style.display = 'block';
      err.textContent = json.error || 'Invalid credentials';
      btn.disabled = false; btn.textContent = 'Sign In';
    }}
  }} catch(err) {{
    const el = document.getElementById('error');
    el.style.display = 'block';
    el.textContent = 'Connection failed';
    btn.disabled = false; btn.textContent = 'Sign In';
  }}
}});
</script>
</body></html>"##, app = app_name, logo = logo_letter)
}

pub(crate) fn generate_register_page(state: &AppState) -> String {
    let app_name = &state.app.name;
    let logo_letter = app_name.chars().next().unwrap_or('C').to_uppercase().to_string();
    format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Register — {app}</title>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:'Inter',system-ui,-apple-system,sans-serif;-webkit-font-smoothing:antialiased;background:#0a0a0a;color:#fafafa;min-height:100vh;display:flex;align-items:center;justify-content:center}}
input{{width:100%;padding:10px 14px;font-size:14px;background:#171717;border:1px solid #262626;border-radius:10px;color:#fafafa;outline:none;transition:border-color 0.15s}}
input:focus{{border-color:#525252}}
.btn{{width:100%;padding:10px 14px;font-size:14px;font-weight:600;border:none;border-radius:10px;background:#fafafa;color:#0a0a0a;cursor:pointer;transition:opacity 0.15s}}
.btn:hover{{opacity:0.9}}
</style>
</head>
<body>
<div style="width:100%;max-width:400px;padding:32px">
  <div style="text-align:center;margin-bottom:32px">
    <div style="width:48px;height:48px;background:#fafafa;border-radius:12px;display:flex;align-items:center;justify-content:center;margin:0 auto 16px">
      <span style="color:#0a0a0a;font-weight:700;font-size:20px">{logo}</span>
    </div>
    <h1 style="font-size:24px;font-weight:700;margin:0 0 8px">Create your account</h1>
    <p style="font-size:14px;color:#a3a3a3">Get started for free</p>
  </div>
  <form id="registerForm" style="display:flex;flex-direction:column;gap:16px">
    <input name="name" type="text" placeholder="Full name" required />
    <input name="email" type="email" placeholder="Email" required />
    <input name="password" type="password" placeholder="Password" required minlength="6" />
    <div id="error" style="display:none;padding:10px 14px;border-radius:10px;font-size:13px;text-align:center;background:#450a0a;color:#fca5a5;border:1px solid #7f1d1d"></div>
    <button type="submit" class="btn">Sign Up</button>
    <p style="text-align:center;font-size:14px;color:#a3a3a3">Already have an account? <a href="/login" style="color:#fafafa;font-weight:600;text-decoration:none">Sign in</a></p>
  </form>
</div>
<script>
document.getElementById('registerForm').addEventListener('submit', async (e) => {{
  e.preventDefault();
  const btn = e.target.querySelector('button');
  btn.disabled = true; btn.textContent = 'Loading...';
  const data = Object.fromEntries(new FormData(e.target));
  try {{
    const res = await fetch('/api/auth/signup', {{ method: 'POST', headers: {{'Content-Type': 'application/json'}}, body: JSON.stringify(data) }});
    const json = await res.json();
    if (json.token) {{
      // cookie set by server via Set-Cookie header (HttpOnly + Secure + SameSite=Strict)
      localStorage.setItem('token', json.token);
      localStorage.setItem('user', JSON.stringify(json.user || {{}}));
      window.location.href = '/';
    }} else {{
      const err = document.getElementById('error');
      err.style.display = 'block';
      err.textContent = json.error || 'Registration failed';
      btn.disabled = false; btn.textContent = 'Sign Up';
    }}
  }} catch(err) {{
    const el = document.getElementById('error');
    el.style.display = 'block';
    el.textContent = 'Connection failed';
    btn.disabled = false; btn.textContent = 'Sign Up';
  }}
}});
</script>
</body></html>"##, app = app_name, logo = logo_letter)
}
