//! Trust engine and brain diagnostics (`/api/trust`, `/api/brain/*`).

use super::*;

pub(super) fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    // Trust engine endpoint
    if path == "/api/trust" || path == "/trust" {
        let metrics = trust::all_metrics();
        let trust_data: Vec<serde_json::Value> = metrics
            .iter()
            .map(|m| {
                let evidence = m.to_evidence();
                let gates = trust::TrustGates::new_clean();
                let profile = trust::TrustProfile::from_evidence(&evidence, gates);
                json!({
                    "block_id": m.block_id,
                    "executions": m.executions,
                    "errors": m.errors,
                    "avg_latency_ms": format!("{:.2}", m.avg_latency_ms()),
                    "trust_score": format!("{:.3}", profile.score()),
                    "status": format!("{:?}", profile.status()),
                    "promotable": profile.promotable(),
                })
            })
            .collect();
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "blocks": trust_data,
                "total_tracked": metrics.len(),
            }),
        ));
    }

    // Brain stats endpoint
    if path == "/api/brain/stats" {
        if let Some(ref brain) = state.brain {
            return Ok(json_response(StatusCode::OK, brain.stats()));
        }
        return Ok(json_response(
            StatusCode::OK,
            json!({"status": "brain not initialized"}),
        ));
    }
    if path == "/api/brain/suggest" {
        if let Some(ref brain) = state.brain {
            let suggestions = brain.suggest("");
            return Ok(json_response(
                StatusCode::OK,
                json!({"suggestions": suggestions}),
            ));
        }
        return Ok(json_response(StatusCode::OK, json!({"suggestions": []})));
    }

    Err(req)
}
