//! Per-client rate limits for `/api/*` (stricter for login/signup).

use super::*;

pub(super) fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    // ── Rate limiting (API endpoints only) ──
    if path.starts_with("/api/") {
        // Socket peer IP; X-Forwarded-For only when the peer is in CRONUS_TRUSTED_PROXIES
        let client_ip = http_guard::client_ip_from_headers(*remote_addr, req.headers()).to_string();

        let is_auth = path.starts_with("/api/auth/login") || path.starts_with("/api/auth/signup");

        let check_result = if is_auth {
            // Stricter limit for auth endpoints: 10 req/60s
            let auth_key = format!("auth:{}", client_ip);
            state.auth_rate_limiter.check(&auth_key)
        } else {
            // Standard limit: 100 req/60s
            state.rate_limiter.check(&client_ip)
        };

        if let Err(retry_after) = check_result {
            return Ok(http_guard::too_many_requests(retry_after));
        }
    }

    Err(req)
}
