//! `/_action/*` and `/_form/*` (session and owner rules in `actions.rs`).

use super::*;

pub(super) async fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    // ── Action execution endpoint ──
    if method == Method::POST && path.starts_with("/_action/") {
        // SECURITY: session required; only AST-declared actions run (by
        // `action_id`), with the server's own instructions, owner-scoped.
        let action_access = access::Access::from_state(req.headers(), state.as_ref());
        let body_bytes = match http_guard::read_body(req).await {
            Ok(b) => b,
            Err(r) => return Ok(r),
        };
        let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));
        let (status, payload) = actions::handle_action(&state, &body, &action_access);
        let status = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_REQUEST);
        return Ok(json_response(status, payload));
    }

    // ── Form submission endpoint: POST creates, PATCH /_form/<section>/<id> edits ──
    if (method == Method::POST || method == Method::PATCH) && path.starts_with("/_form/") {
        // SECURITY: declared form section, session/owner rules from access.rs,
        // body filtered by authz::writable_body (see actions::handle_form).
        let form_access = access::Access::from_state(req.headers(), state.as_ref());
        let body_bytes = match http_guard::read_body(req).await {
            Ok(b) => b,
            Err(r) => return Ok(r),
        };
        let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));
        let (status, payload) = actions::handle_form(&state, &method, &path, &body, &form_access);
        let status = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_REQUEST);
        return Ok(json_response(status, payload));
    }

    Err(req)
}
