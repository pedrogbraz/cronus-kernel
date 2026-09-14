//! `.scriptcronus` endpoints and `/hooks/*` webhooks.

use super::*;

pub(super) async fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    // ── Script routes (.scriptcronus endpoints + webhooks) ──
    {
        let method_str = method.as_str();
        let has_endpoint = state
            .script_registry
            .get_endpoints()
            .iter()
            .any(|(_s, ep)| ep.method == method_str && ep.path == *path);
        // SECURITY: webhooks only match paths under /hooks/ prefix
        let has_webhook = !has_endpoint
            && method == Method::POST
            && path.starts_with("/hooks/")
            && !state.script_registry.get_webhook_handlers(&path).is_empty();

        if has_endpoint || has_webhook {
            let token = req
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string());
            let (user_id, role) = token
                .as_deref()
                .and_then(|t| auth::verify_token(t, &auth::default_secret()).ok())
                .map(|c| (c.sub.clone(), c.role.clone()))
                .unwrap_or_else(|| ("anonymous".into(), "public".into()));

            let body_bytes = match http_guard::read_body(req).await {
                Ok(b) => b,
                Err(r) => return Ok(r),
            };
            let body: Option<serde_json::Value> = serde_json::from_slice(&body_bytes).ok();

            if has_endpoint {
                for (script, ep) in state.script_registry.get_endpoints() {
                    if ep.method == method_str && ep.path == *path {
                        // SECURITY: endpoints require auth by default
                        // Use auth:public to explicitly allow unauthenticated access
                        let required_role = ep.auth.as_deref().unwrap_or("any");
                        if required_role != "public" {
                            // Must have valid token
                            if user_id == "anonymous" {
                                return Ok(json_response(
                                    StatusCode::UNAUTHORIZED,
                                    json!({"error": "authentication required"}),
                                ));
                            }
                            // Check role if specific role required
                            if required_role != "any" && role != required_role {
                                return Ok(json_response(
                                    StatusCode::FORBIDDEN,
                                    json!({"error": "insufficient role"}),
                                ));
                            }
                        }
                        let ctx = scripting::execute_endpoint(
                            ep,
                            &script.name,
                            &state.db,
                            &user_id,
                            &role,
                            body.as_ref(),
                            &std::collections::HashMap::new(),
                        );
                        if let Some(resp) = ctx.response {
                            // SECURITY: clamp status to safe range
                            let safe_status = resp.status.max(200).min(599);
                            let mut builder = Response::builder().status(safe_status);
                            // SECURITY: whitelist safe response headers — block Set-Cookie, Location, etc.
                            const ALLOWED_HEADERS: &[&str] = &[
                                "content-type",
                                "content-disposition",
                                "cache-control",
                                "x-request-id",
                                "x-total-count",
                            ];
                            for (k, v) in &resp.headers {
                                let k_lower = k.to_lowercase();
                                if ALLOWED_HEADERS.contains(&k_lower.as_str()) {
                                    // SECURITY: strip newlines to prevent header injection
                                    let safe_v = v.replace('\n', "").replace('\r', "");
                                    builder = builder.header(k.as_str(), safe_v.as_str());
                                }
                            }
                            if !resp
                                .headers
                                .keys()
                                .any(|k| k.to_lowercase() == "content-type")
                            {
                                builder = builder.header("Content-Type", "application/json");
                            }
                            return Ok(builder.body(Full::new(Bytes::from(resp.body))).unwrap());
                        }
                        return Ok(json_response(
                            StatusCode::OK,
                            json!({"ok": true, "logs": ctx.logs}),
                        ));
                    }
                }
            }
            // Webhook
            let body_val = body.unwrap_or(serde_json::Value::Null);
            let _ctx = scripting::execute_webhook(
                &state.script_registry,
                &path,
                &body_val,
                &state.db,
                &std::collections::HashMap::new(),
            );
            return Ok(json_response(StatusCode::OK, json!({"ok": true})));
        }
    }

    Err(req)
}
