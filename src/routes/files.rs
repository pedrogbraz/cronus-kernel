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
    let name = path.trim_start_matches("/_files/");
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
        .header("cache-control", "private, max-age=31536000")
        .body(Full::new(Bytes::from(bytes)))
        .unwrap_or_else(|_| {
            json_response(
                StatusCode::NOT_FOUND,
                authz::error_body("NOT_FOUND", "Not found"),
            )
        }))
}
