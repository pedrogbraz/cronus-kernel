//! Audit widget trigger/results and the hash-chained audit trail.

use super::*;

pub(super) async fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    // Audit endpoint — triggers browser scan and stores results
    if path == "/api/audit/trigger" && method == hyper::Method::GET {
        // Return a page that auto-scans and posts results back
        let trigger_html = r#"<!DOCTYPE html><html><head><script>
        fetch('/').then(r=>r.text()).then(html=>{
            var iframe=document.createElement('iframe');
            iframe.style.cssText='position:fixed;top:0;left:0;width:100vw;height:100vh;border:none;z-index:1';
            document.body.appendChild(iframe);
            iframe.srcdoc=html;
            iframe.onload=function(){
                var w=iframe.contentWindow;
                // Wait for audit to auto-scan
                var check=setInterval(function(){
                    if(w.__CRONUS_DUMP_AUDIT && w.__CRONUS_DUMP_AUDIT.results()){
                        clearInterval(check);
                        var r=w.__CRONUS_DUMP_AUDIT.results();
                        fetch('/api/audit/results',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(r)});
                        document.title='AUDIT DONE: '+r.fidelity+'%';
                    }
                },500);
            };
        });
        </script></head><body style="margin:0;background:#0e0e0e;color:#e2e2e2;font-family:Inter,sans-serif">
        <div style="display:flex;align-items:center;justify-content:center;height:100vh">Running audit...</div>
        </body></html>"#;
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(trigger_html)))
            .unwrap());
    }

    // Audit results storage (posted by the audit widget)
    if path == "/api/audit/results" && method == hyper::Method::POST {
        // Dev only (http_guard 404s it in production). Project dir, never /tmp.
        let body_bytes = match http_guard::read_body(req.into_body()).await {
            Ok(b) => b,
            Err(r) => return Ok(r),
        };
        let Ok(val) = serde_json::from_slice::<Value>(&body_bytes) else {
            return Ok(json_response(
                StatusCode::BAD_REQUEST,
                authz::error_body("BAD_REQUEST", "Expected JSON body"),
            ));
        };
        {
            let saved = std::fs::create_dir_all(".cronus").and_then(|_| {
                std::fs::write(http_guard::AUDIT_WIDGET_RESULTS_PATH, val.to_string())
            });
            if let Err(e) = saved {
                eprintln!("  \x1b[33m[AUDIT]\x1b[0m could not save results: {}", e);
                return Ok(json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    authz::error_body("INTERNAL", "Could not save audit results"),
                ));
            }
            eprintln!(
                "\n  \x1b[36m[AUDIT]\x1b[0m Results saved to {}",
                http_guard::AUDIT_WIDGET_RESULTS_PATH
            );
            {
                let fidelity = val.get("fidelity").and_then(|v| v.as_i64()).unwrap_or(0);
                let missing = val
                    .get("missingItems")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let extra = val
                    .get("extraItems")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let dot = if fidelity >= 90 {
                    "🟢"
                } else if fidelity >= 60 {
                    "🟡"
                } else {
                    "🔴"
                };
                eprintln!(
                    "  {} Fidelity: {}% | Missing: {} | Extra: {}",
                    dot, fidelity, missing, extra
                );
            }
        }
        return Ok(json_response(StatusCode::OK, json!({"ok": true})));
    }

    // Audit results read (for CLI/agent access)
    if path == "/api/audit/results" && method == hyper::Method::GET {
        let results = std::fs::read_to_string(http_guard::AUDIT_WIDGET_RESULTS_PATH)
            .unwrap_or_else(|_| "{}".into());
        let val: Value =
            serde_json::from_str(&results).unwrap_or(json!({"error": "no audit results yet"}));
        return Ok(json_response(StatusCode::OK, val));
    }

    // Audit trail endpoints -- tamper-proof hash-chained log
    if path == "/api/audit/trail/verify" && method == Method::GET {
        match state.audit_trail.verify() {
            Ok(result) => return Ok(json_response(StatusCode::OK, result)),
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m audit trail verify: {}", e);
                return Ok(json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    authz::error_body("INTERNAL", "Audit trail unavailable"),
                ));
            }
        }
    }
    if (path == "/api/audit/trail" || path.starts_with("/api/audit/trail?"))
        && method == Method::GET
    {
        let query_str = req.uri().query().unwrap_or("");
        let limit: usize = query_str
            .split('&')
            .find(|p| p.starts_with("limit="))
            .and_then(|p| p.strip_prefix("limit="))
            .and_then(|v| v.parse().ok())
            .unwrap_or(50);
        let entity_filter: Option<String> = query_str
            .split('&')
            .find(|p| p.starts_with("entity="))
            .and_then(|p| p.strip_prefix("entity="))
            .map(|v| v.to_string());
        match state
            .audit_trail
            .query_filtered(limit, entity_filter.as_deref())
        {
            Ok(mut entries) => {
                http_guard::redact_audit_entries(&mut entries, &state.entities);
                return Ok(json_response(StatusCode::OK, entries));
            }
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m audit trail query: {}", e);
                return Ok(json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    authz::error_body("INTERNAL", "Audit trail unavailable"),
                ));
            }
        }
    }

    Err(req)
}
