//! `/api/auth/*` and HTML form POST `/login` `/register` `/logout`.

use super::*;
use hyper::header::{HeaderValue, SET_COOKIE};

pub(super) async fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr: _,
    } = ctx;

    if path.starts_with("/api/auth/") {
        let headers = req.headers().clone();
        let body_bytes = if *method == Method::POST {
            match http_guard::read_body(req).await {
                Ok(b) => b,
                Err(r) => return Ok(r),
            }
        } else {
            Default::default()
        };
        let outcome = session::handle_auth(state, method, path, query, &headers, &body_bytes);
        if let Some(ref account) = outcome.login_account {
            http_guard::login_account_record(account, outcome.response.status());
        }
        return Ok(outcome.response);
    }

    if *method == Method::POST && matches!(path.as_str(), "/login" | "/register" | "/signup") {
        return Ok(form_auth(req, ctx).await);
    }

    if *method == Method::POST && path == "/logout" {
        let headers = req.headers().clone();
        let _ = http_guard::read_body(req).await;
        let secure = session::cookie_secure(http_guard::policy().mode, &headers);
        return Ok(Response::builder()
            .status(StatusCode::FOUND)
            .header("Location", "/login")
            .header(SET_COOKIE, session::clear_session_cookie(secure))
            .body(Full::new(Bytes::new()))
            .unwrap());
    }

    Err(req)
}

async fn form_auth(req: Request<Incoming>, ctx: &Ctx) -> Response<Full<Bytes>> {
    let headers = req.headers().clone();
    let body_bytes = match http_guard::read_body(req).await {
        Ok(b) => b,
        Err(r) => return r,
    };
    let json_body = if is_urlencoded(&headers) {
        session::json_from_urlencoded(&body_bytes)
            .to_string()
            .into_bytes()
    } else {
        body_bytes.to_vec()
    };
    let api_path = if ctx.path == "/login" {
        "/api/auth/login"
    } else {
        "/api/auth/signup"
    };
    let outcome = session::handle_auth(
        &ctx.state,
        &Method::POST,
        api_path,
        &ctx.query,
        &headers,
        &json_body,
    );
    if let Some(ref account) = outcome.login_account {
        http_guard::login_account_record(account, outcome.response.status());
    }
    let status = outcome.response.status();
    if status.is_success() {
        return redirect_with_session(&ctx.state, outcome.response);
    }
    let msg = error_message(status);
    let html = if ctx.path == "/login" {
        generate_login_page(&ctx.state, Some(msg))
    } else {
        generate_register_page(&ctx.state, Some(msg))
    };
    html_response(html)
}

fn is_urlencoded(headers: &hyper::HeaderMap) -> bool {
    headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            v.to_ascii_lowercase()
                .contains("application/x-www-form-urlencoded")
        })
        .unwrap_or(false)
}

fn error_message(status: StatusCode) -> &'static str {
    match status {
        StatusCode::UNAUTHORIZED => "invalid credentials",
        StatusCode::BAD_REQUEST => "could not create account",
        _ => "could not sign in",
    }
}

fn redirect_with_session(state: &AppState, api: Response<Full<Bytes>>) -> Response<Full<Bytes>> {
    let (home, _) = crate::server::auth_pages::post_login_paths(state);
    let dest = home;
    let mut builder = Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", dest);
    if let Some(cookie) = api.headers().get(SET_COOKIE) {
        if let Ok(v) = HeaderValue::from_bytes(cookie.as_bytes()) {
            builder = builder.header(SET_COOKIE, v);
        }
    }
    builder.body(Full::new(Bytes::new())).unwrap()
}
