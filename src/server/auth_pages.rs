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
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:'Inter',system-ui,-apple-system,sans-serif;-webkit-font-smoothing:antialiased;background:#0a0a0a;color:#fafafa;min-height:100vh;display:flex;align-items:center;justify-content:center}}
input{{width:100%;padding:10px 14px;font-size:14px;background:#171717;border:1px solid #262626;border-radius:10px;color:#fafafa;outline:none;transition:border-color 0.15s}}
input:focus{{border-color:#525252}}
input[type=checkbox]{{width:16px;height:16px;accent-color:#fafafa;cursor:pointer}}
.btn{{width:100%;padding:10px 14px;font-size:14px;font-weight:600;border:none;border-radius:10px;background:#fafafa;color:#0a0a0a;cursor:pointer;transition:opacity 0.15s}}
.btn:hover{{opacity:0.9}}
.account-card{{display:flex;align-items:center;gap:12px;padding:12px 16px;border:1px solid #262626;border-radius:12px;background:#171717;cursor:pointer;transition:all 0.15s;width:100%}}
.account-card:hover{{border-color:#525252;background:#1f1f1f}}
.account-avatar{{width:40px;height:40px;border-radius:10px;background:#262626;display:flex;align-items:center;justify-content:center;font-weight:700;font-size:16px;color:#fafafa;flex-shrink:0}}
.account-info{{flex:1;text-align:left;min-width:0}}
.account-name{{font-size:14px;font-weight:600;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}}
.account-email{{font-size:12px;color:#a3a3a3;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}}
.account-remove{{color:#525252;font-size:18px;padding:4px;border-radius:6px;transition:color 0.15s;flex-shrink:0;display:flex;align-items:center}}
.account-remove:hover{{color:#ef4444}}
.material-symbols-outlined{{font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24;font-size:20px;display:inline-block;line-height:1;vertical-align:middle}}
</style>
</head>
<body>
<div style="width:100%;max-width:400px;padding:32px">
  <div style="text-align:center;margin-bottom:32px">
    <div style="width:48px;height:48px;background:#fafafa;border-radius:12px;display:flex;align-items:center;justify-content:center;margin:0 auto 16px">
      <span style="color:#0a0a0a;font-weight:700;font-size:20px">{logo}</span>
    </div>
    <h1 style="font-size:24px;font-weight:700;margin:0 0 8px" id="page-title">Welcome back</h1>
    <p style="font-size:14px;color:#a3a3a3" id="page-sub">Sign in to your account</p>
  </div>

  <!-- Saved accounts view -->
  <div id="accountsView" style="display:none;flex-direction:column;gap:8px">
    <div id="accountsList" style="display:flex;flex-direction:column;gap:8px"></div>
    <div style="margin-top:8px">
      <button onclick="showFullLogin()" style="width:100%;padding:10px 14px;font-size:14px;font-weight:500;border:1px solid #262626;border-radius:10px;background:transparent;color:#a3a3a3;cursor:pointer;transition:all 0.15s;display:flex;align-items:center;justify-content:center;gap:8px" onmouseover="this.style.borderColor='#525252';this.style.color='#fafafa'" onmouseout="this.style.borderColor='#262626';this.style.color='#a3a3a3'">
        <span class="material-symbols-outlined" style="font-size:18px">add</span>
        Use another account
      </button>
    </div>
    <p style="text-align:center;font-size:14px;color:#a3a3a3;margin-top:8px">Don't have an account? <a href="/register" style="color:#fafafa;font-weight:600;text-decoration:none">Register</a></p>
  </div>

  <!-- Password-only view (after clicking a saved account) -->
  <form id="quickLogin" style="display:none;flex-direction:column;gap:16px">
    <div class="account-card" style="cursor:default;border-color:#525252">
      <div class="account-avatar" id="quick-avatar">?</div>
      <div class="account-info">
        <div class="account-name" id="quick-name"></div>
        <div class="account-email" id="quick-email"></div>
      </div>
    </div>
    <input name="password" type="password" placeholder="Password" required autofocus />
    <input name="email" type="hidden" id="quick-email-input" />
    <label style="display:flex;align-items:center;gap:8px;cursor:pointer;font-size:14px;color:#a3a3a3">
      <input name="remember" type="checkbox" checked />
      Remember me
    </label>
    <div id="quick-error" style="display:none;padding:10px 14px;border-radius:10px;font-size:13px;text-align:center;background:#450a0a;color:#fca5a5;border:1px solid #7f1d1d"></div>
    <button type="submit" class="btn">Sign In</button>
    <button type="button" onclick="showAccounts()" style="width:100%;padding:8px;font-size:13px;background:transparent;border:none;color:#a3a3a3;cursor:pointer">
      ← Back to accounts
    </button>
  </form>

  <!-- Full login form -->
  <form id="loginForm" style="display:flex;flex-direction:column;gap:16px">
    <input name="email" type="email" placeholder="Email" required />
    <input name="password" type="password" placeholder="Password" required />
    <label style="display:flex;align-items:center;gap:8px;cursor:pointer;font-size:14px;color:#a3a3a3">
      <input name="remember" type="checkbox" />
      Remember me
    </label>
    <div id="error" style="display:none;padding:10px 14px;border-radius:10px;font-size:13px;text-align:center;background:#450a0a;color:#fca5a5;border:1px solid #7f1d1d"></div>
    <button type="submit" class="btn">Sign In</button>
    <p style="text-align:center;font-size:14px;color:#a3a3a3">Don't have an account? <a href="/register" style="color:#fafafa;font-weight:600;text-decoration:none">Register</a></p>
  </form>
</div>
<script>
// Saved accounts management
function getSavedAccounts() {{
  try {{ return JSON.parse(localStorage.getItem('saved_accounts') || '[]'); }} catch(e) {{ return []; }}
}}
function saveAccount(user) {{
  var accounts = getSavedAccounts();
  // Remove existing with same email
  accounts = accounts.filter(function(a) {{ return a.email !== user.email; }});
  // Add to front
  accounts.unshift({{ name: user.name, email: user.email, role: user.role, initial: (user.name||'?').charAt(0).toUpperCase() }});
  // Max 5 accounts
  if (accounts.length > 5) accounts = accounts.slice(0, 5);
  localStorage.setItem('saved_accounts', JSON.stringify(accounts));
}}
function removeAccount(email) {{
  var accounts = getSavedAccounts().filter(function(a) {{ return a.email !== email; }});
  localStorage.setItem('saved_accounts', JSON.stringify(accounts));
  renderAccounts();
}}

function renderAccounts() {{
  var accounts = getSavedAccounts();
  if (accounts.length === 0) {{
    showFullLogin();
    return;
  }}
  var list = document.getElementById('accountsList');
  list.innerHTML = '';
  accounts.forEach(function(acc) {{
    var card = document.createElement('div');
    card.className = 'account-card';
    card.onclick = function() {{ selectAccount(acc.email); }};
    card.innerHTML = '<div class="account-avatar">' + acc.initial + '</div>' +
      '<div class="account-info"><div class="account-name">' + acc.name + '</div><div class="account-email">' + acc.email + '</div></div>' +
      '<div class="account-remove" onclick="event.stopPropagation();removeAccount(\'' + acc.email + '\')" title="Remove"><span class="material-symbols-outlined" style="font-size:16px">close</span></div>';
    list.appendChild(card);
  }});
  document.getElementById('accountsView').style.display = 'flex';
  document.getElementById('loginForm').style.display = 'none';
  document.getElementById('quickLogin').style.display = 'none';
  document.getElementById('page-title').textContent = 'Choose an account';
  document.getElementById('page-sub').textContent = 'Sign in to {app}';
}}

function selectAccount(email) {{
  var acc = getSavedAccounts().find(function(a) {{ return a.email === email; }});
  if (!acc) return;
  document.getElementById('quick-avatar').textContent = acc.initial;
  document.getElementById('quick-name').textContent = acc.name;
  document.getElementById('quick-email').textContent = acc.email;
  document.getElementById('quick-email-input').value = acc.email;
  document.getElementById('accountsView').style.display = 'none';
  document.getElementById('loginForm').style.display = 'none';
  document.getElementById('quickLogin').style.display = 'flex';
  document.getElementById('page-title').textContent = 'Welcome back';
  document.getElementById('page-sub').textContent = acc.name;
  document.getElementById('quick-error').style.display = 'none';
  // Focus password
  setTimeout(function() {{ document.querySelector('#quickLogin input[type=password]').focus(); }}, 100);
}}

function showAccounts() {{
  renderAccounts();
}}

function showFullLogin() {{
  document.getElementById('accountsView').style.display = 'none';
  document.getElementById('quickLogin').style.display = 'none';
  document.getElementById('loginForm').style.display = 'flex';
  document.getElementById('page-title').textContent = 'Welcome back';
  document.getElementById('page-sub').textContent = 'Sign in to your account';
}}

// Login handler
async function doLogin(email, password, remember, errorEl, btn) {{
  btn.disabled = true; btn.textContent = 'Loading...';
  try {{
    var res = await fetch('/api/auth/login', {{ method: 'POST', headers: {{'Content-Type': 'application/json'}}, body: JSON.stringify({{ email: email, password: password, remember: remember }}) }});
    var json = await res.json();
    if (json.token) {{
      localStorage.setItem('token', json.token);
      localStorage.setItem('user', JSON.stringify(json.user || {{}}));
      saveAccount(json.user);
      var role = json.user && json.user.role;
      window.location.href = (role === 'admin') ? '/admin' : '/';
    }} else {{
      errorEl.style.display = 'block';
      errorEl.textContent = json.error || 'Invalid credentials';
      btn.disabled = false; btn.textContent = 'Sign In';
    }}
  }} catch(err) {{
    errorEl.style.display = 'block';
    errorEl.textContent = 'Connection failed';
    btn.disabled = false; btn.textContent = 'Sign In';
  }}
}}

// Full login form
document.getElementById('loginForm').addEventListener('submit', function(e) {{
  e.preventDefault();
  var fd = Object.fromEntries(new FormData(e.target));
  doLogin(fd.email, fd.password, !!fd.remember, document.getElementById('error'), e.target.querySelector('button[type=submit]'));
}});

// Quick login form
document.getElementById('quickLogin').addEventListener('submit', function(e) {{
  e.preventDefault();
  var fd = Object.fromEntries(new FormData(e.target));
  doLogin(fd.email, fd.password, !!fd.remember, document.getElementById('quick-error'), e.target.querySelector('button[type=submit]'));
}});

// Init: show saved accounts or full login
renderAccounts();
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
// Save account on register too
function getSavedAccounts() {{
  try {{ return JSON.parse(localStorage.getItem('saved_accounts') || '[]'); }} catch(e) {{ return []; }}
}}
function saveAccount(user) {{
  var accounts = getSavedAccounts().filter(function(a) {{ return a.email !== user.email; }});
  accounts.unshift({{ name: user.name, email: user.email, role: user.role, initial: (user.name||'?').charAt(0).toUpperCase() }});
  if (accounts.length > 5) accounts = accounts.slice(0, 5);
  localStorage.setItem('saved_accounts', JSON.stringify(accounts));
}}
document.getElementById('registerForm').addEventListener('submit', async (e) => {{
  e.preventDefault();
  const btn = e.target.querySelector('button');
  btn.disabled = true; btn.textContent = 'Loading...';
  const data = Object.fromEntries(new FormData(e.target));
  try {{
    const res = await fetch('/api/auth/signup', {{ method: 'POST', headers: {{'Content-Type': 'application/json'}}, body: JSON.stringify(data) }});
    const json = await res.json();
    if (json.token) {{
      localStorage.setItem('token', json.token);
      localStorage.setItem('user', JSON.stringify(json.user || {{}}));
      saveAccount(json.user);
      const role = json.user && json.user.role;
      window.location.href = (role === 'admin') ? '/admin' : '/';
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
