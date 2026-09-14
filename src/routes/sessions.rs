//! `/api/auth/*`: sessions live in `session.rs`; login backoff in `http_guard`.

use super::*;

pub(super) async fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    // Auth routes
    if path.starts_with("/api/auth/") {
        let headers = req.headers().clone();
        let body_bytes = if method == Method::POST {
            match http_guard::read_body(req).await {
                Ok(b) => b,
                Err(r) => return Ok(r),
            }
        } else {
            Default::default()
        };
        let outcome = session::handle_auth(&state, &method, &path, &query, &headers, &body_bytes);
        if let Some(ref account) = outcome.login_account {
            http_guard::login_account_record(account, outcome.response.status());
        }
        return Ok(outcome.response);
    }

    Err(req)
}
