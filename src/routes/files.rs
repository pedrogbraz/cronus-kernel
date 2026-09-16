//! `GET /_files/<name>` — stored `file` field payloads.

use super::*;

pub(super) async fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        ..
    } = ctx;
    if *method != Method::GET || !path.starts_with("/_files/") {
        return Err(req);
    }
    let access = access::Access::from_state(req.headers(), state.as_ref());
    if access.viewer.is_none() {
        return Ok(json_response(
            StatusCode::UNAUTHORIZED,
            authz::error_body("UNAUTHORIZED", "Authentication required"),
        ));
    }
    let name = path.trim_start_matches("/_files/");
    let stored = format!("/_files/{name}");
    if !crate::files::viewer_can_read(&state.db, &state.entities, &access, &stored) {
        return Ok(json_response(
            StatusCode::NOT_FOUND,
            authz::error_body("NOT_FOUND", "Not found"),
        ));
    }
    let dir = crate::files::dir_for(&state.db_path);
    let Some((mime, bytes)) = crate::files::read_stored(&dir, name) else {
        return Ok(json_response(
            StatusCode::NOT_FOUND,
            authz::error_body("NOT_FOUND", "Not found"),
        ));
    };
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", mime)
        .header("x-content-type-options", "nosniff")
        .header("cache-control", "private, max-age=31536000")
        .body(Full::new(Bytes::from(bytes)))
        .unwrap_or_else(|_| {
            json_response(
                StatusCode::NOT_FOUND,
                authz::error_body("NOT_FOUND", "Not found"),
            )
        }))
}
