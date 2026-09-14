//! Dev tooling served before rate limiting: HMR version, block explorer,
//! Zeus observability, Hydra evolution. Gated by `http_guard::guard_internal`
//! in `handle_request` (404 in production).

use super::*;

pub(super) fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    // HMR version endpoint
    if path == "/.cronus/version" {
        return Ok(json_response(
            StatusCode::OK,
            json!({ "version": hmr::current_version() }),
        ));
    }

    // Block explorer
    if path == "/blocks" {
        let metrics = trust::all_metrics();
        let html = block_explorer::render_explorer(".", &metrics, &state.script_registry);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(html)))
            .unwrap());
    }

    // Zeus observability endpoints
    if path == "/zeus" {
        let html = zeus::render_dashboard(&state.zeus);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(html)))
            .unwrap());
    }
    if path == "/zeus/api" {
        let traces = state.zeus.last_n(100);
        let stats = state.zeus.stats();
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "stats": stats,
                "traces": traces,
            }),
        ));
    }
    if path == "/zeus/slow" {
        let slow = state.zeus.slow_traces(50.0); // >50ms
        return Ok(json_response(StatusCode::OK, json!({"traces": slow})));
    }
    if path == "/zeus/errors" {
        let errors = state.zeus.error_traces();
        return Ok(json_response(StatusCode::OK, json!({"traces": errors})));
    }

    // Hydra evolution endpoints
    if path == "/hydra" || path == "/api/hydra/registry" {
        let registry = hydra::registry::BlockRegistry::open(".cronus/block-registry.json");
        return Ok(json_response(StatusCode::OK, registry.to_json()));
    }
    if path == "/api/hydra/evolve" {
        let mut registry = hydra::registry::BlockRegistry::open(".cronus/block-registry.json");
        let report = hydra::evolve(&mut registry, &state.script_registry.scripts);
        return Ok(json_response(
            StatusCode::OK,
            serde_json::to_value(&report).unwrap_or(json!({"error":"serialize"})),
        ));
    }
    if path == "/api/hydra/candidates" {
        let candidates = hydra::extract::extract_candidates(&state.script_registry.scripts);
        let data: Vec<serde_json::Value> = candidates
            .iter()
            .map(|c| {
                json!({
                    "name": c.name,
                    "source": c.source_script,
                    "type": format!("{:?}", c.block_type),
                    "entity": c.entity,
                    "trust_score": format!("{:.3}", c.trust_score),
                    "executions": c.executions,
                    "promotable": c.promotable,
                    "reason": c.reason,
                })
            })
            .collect();
        return Ok(json_response(
            StatusCode::OK,
            json!({"candidates": data, "total": candidates.len()}),
        ));
    }

    Err(req)
}
