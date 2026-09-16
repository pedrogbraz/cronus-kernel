//! Per-client rate limits for `/api/*`, GraphQL POST, `/_form`, `/_action`
//! (stricter for login/signup and inbound `/hooks`).

use super::*;

pub(super) fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    let is_auth = path.starts_with("/api/auth/login")
        || path.starts_with("/api/auth/signup")
        || path.starts_with("/hooks");
    let is_std = path.starts_with("/api/")
        || (*path == "/graphql" && *method != Method::GET)
        || path.starts_with("/_form")
        || path.starts_with("/_action");

    if is_auth || is_std {
        // Socket peer IP; X-Forwarded-For only when the peer is in CRONUS_TRUSTED_PROXIES
        let client_ip = http_guard::client_ip_from_headers(*remote_addr, req.headers()).to_string();

        let check_result = if is_auth {
            let auth_key = format!("auth:{}", client_ip);
            state.auth_rate_limiter.check(&auth_key)
        } else {
            state.rate_limiter.check(&client_ip)
        };

        if let Err(retry_after) = check_result {
            return Ok(http_guard::too_many_requests(retry_after));
        }
    }

    Err(req)
}
