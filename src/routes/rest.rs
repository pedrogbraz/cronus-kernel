//! REST `/api/<entity>` via `api_crud::handle_api`, tracked in the brain.

use super::*;

pub(super) async fn route(req: Request<Incoming>, ctx: &Ctx, start: std::time::Instant) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    // API routes: /api/...
    if path.starts_with("/api/") {
        // SECURITY: read the session BEFORE consuming the request body.
        let api_access = access::Access::from_state(req.headers(), state.as_ref());

        let body_bytes = match http_guard::read_body(req).await {
            Ok(b) => b,
            Err(r) => return Ok(r),
        };
        let body: Option<serde_json::Value> = serde_json::from_slice(&body_bytes).ok();

        let resp = api_crud::handle_api(&state, &method, &path, &query, body.as_ref(), &api_access);
        // Track request in brain
        if let Some(ref brain) = state.brain {
            let duration = start.elapsed().as_millis() as u64;
            let status = resp.status().as_u16();
            brain.track_request(method.as_str(), &path, status, duration);
        }
        return Ok(resp);
    }

    Err(req)
}
