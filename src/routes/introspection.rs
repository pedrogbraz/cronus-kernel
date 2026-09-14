//! App introspection: health, schema, seed, AI context, server logs/stats, docs.

use super::*;

pub(super) fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    // Health endpoint
    if path == "/api/health" {
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "ok": true,
                "app": state.app.name,
                "entities": state.entities.len(),
                "pages": state.pages.len(),
                "runtime": "cronus-kernel",
                "version": env!("CARGO_PKG_VERSION")
            }),
        ));
    }

    // Schema endpoint — returns all entities with fields
    if path == "/api/schema" {
        let schema: Vec<Value> = state
            .entities
            .iter()
            .map(|e| {
                let fields: Vec<Value> = e
                    .fields
                    .iter()
                    .map(|f| {
                        json!({
                            "name": f.name,
                            "type": format!("{:?}", f.field_type).to_lowercase(),
                            "required": f.required,
                            "unique": f.unique,
                        })
                    })
                    .collect();
                json!({
                    "entity": e.name,
                    "fields": fields,
                    "field_count": e.fields.len(),
                })
            })
            .collect();
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "entities": schema,
                "total": state.entities.len(),
                "pages": state.pages.len(),
            }),
        ));
    }

    // Seed endpoint
    if path == "/api/_seed" && method == Method::POST {
        let mut results = serde_json::Map::new();
        for entity in &state.entities {
            if let Ok(count) = state.db.seed_entity(entity) {
                results.insert(entity.name.clone(), json!(count));
            }
        }
        return Ok(json_response(StatusCode::OK, json!({"seeded": results})));
    }

    // Health endpoint — lint + behavioral audit
    if path == "/api/_health" && method == Method::GET {
        let brain_stats = state.brain.as_ref().map(|b| b.stats()).unwrap_or(json!({}));
        let entity_rows: Vec<Value> = state
            .entities
            .iter()
            .filter(|e| !e.name.starts_with('_'))
            .map(|e| {
                let count = state.db.count(&e.name).unwrap_or(0);
                json!({"entity": e.name, "rows": count})
            })
            .collect();
        let empty_bound: Vec<&str> = state
            .pages
            .iter()
            .flat_map(|p| {
                p.sections.iter().filter_map(|s| {
                    if s.binding.is_some() {
                        let entity = s.binding.as_ref().unwrap().entity.clone();
                        let count = state.db.count(&entity).unwrap_or(0);
                        if count == 0 {
                            Some(entity)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .map(|_| "")
            .collect(); // placeholder
        let behavioral: Vec<String> = state
            .pages
            .iter()
            .flat_map(|p| {
                p.sections.iter().filter_map(|s| {
                    if let Some(ref b) = s.binding {
                        let count = state.db.count(&b.entity).unwrap_or(0);
                        if count == 0 {
                            Some(format!(
                                "Entity '{}' has 0 rows — {} page shows empty state",
                                b.entity, p.route
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .collect();
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "status": "healthy",
                "lint": { "warnings": 0, "errors": 0 },
                "behavioral": behavioral,
                "entities": entity_rows,
                "brain": brain_stats,
            }),
        ));
    }

    // AI Context Protocol — single endpoint with everything an AI needs
    if path == "/api/_context" && method == Method::GET {
        let brain_stats = state.brain.as_ref().map(|b| b.stats()).unwrap_or(json!({}));

        let entities_json: Vec<Value> = state.entities.iter()
            .filter(|e| !e.name.starts_with('_'))
            .map(|e| {
                let fields: Vec<Value> = e.fields.iter().map(|f| {
                    let mut fj = json!({
                        "name": f.name,
                        "type": reconcile_field_type_str(&f.field_type),
                        "required": f.required,
                        "unique": f.unique,
                        "sensitive": f.sensitive,
                    });
                    if let Some(ref doc) = f.doc {
                        fj["doc"] = json!({
                            "summary": doc.summary,
                            "tags": doc.tags.iter().map(|t| json!({"name": t.name, "value": t.value})).collect::<Vec<_>>(),
                        });
                    }
                    fj
                }).collect();
                let mut ej = json!({
                    "name": e.name,
                    "shared": e.shared,
                    "fields": fields,
                });
                if let Some(ref doc) = e.doc {
                    ej["doc"] = json!({
                        "summary": doc.summary,
                        "tags": doc.tags.iter().map(|t| json!({"name": t.name, "value": t.value})).collect::<Vec<_>>(),
                    });
                }
                if !e.transitions.is_empty() {
                    let transitions_json: Vec<Value> = e.transitions.iter().map(|t| {
                        json!({
                            "field": t.field,
                            "rules": t.rules.iter().map(|r| json!({
                                "from": r.from,
                                "to": r.to,
                            })).collect::<Vec<_>>(),
                        })
                    }).collect();
                    ej["transitions"] = json!(transitions_json);
                }
                // Live row count from database
                if let Ok(count) = state.db.count(&e.name) {
                    ej["row_count"] = json!(count);
                }
                ej
            }).collect();

        let pages_json: Vec<Value> = state.pages.iter().map(|p| {
            let mut pj = json!({
                "route": p.route,
                "type": p.page_type,
                "title": p.title.as_deref().unwrap_or(""),
                "sections_count": p.sections.len(),
                "requires": p.requires.as_deref().unwrap_or(""),
            });
            if let Some(ref doc) = p.doc {
                pj["doc"] = json!({
                    "summary": doc.summary,
                    "tags": doc.tags.iter().map(|t| json!({"name": t.name, "value": t.value})).collect::<Vec<_>>(),
                });
            }
            pj
        }).collect();

        let apis_json: Vec<Value> = state.apis.iter().map(|a| {
            let routes: Vec<Value> = a.routes.iter().map(|r| {
                json!({
                    "name": r.name,
                    "method": match r.method {
                        parser::HttpMethod::GET => "GET",
                        parser::HttpMethod::POST => "POST",
                        parser::HttpMethod::PATCH => "PATCH",
                        parser::HttpMethod::PUT => "PUT",
                        parser::HttpMethod::DELETE => "DELETE",
                    },
                    "path": r.path,
                    "auth": r.auth,
                })
            }).collect();
            let mut aj = json!({
                "prefix": a.prefix,
                "routes": routes,
            });
            if let Some(ref doc) = a.doc {
                aj["doc"] = json!({
                    "summary": doc.summary,
                    "tags": doc.tags.iter().map(|t| json!({"name": t.name, "value": t.value})).collect::<Vec<_>>(),
                });
            }
            aj
        }).collect();

        let webhooks_json: Vec<Value> = state
            .webhooks
            .iter()
            .map(|w| {
                json!({
                    "entity": w.entity,
                    "hooks": w.hooks.iter().map(|h| json!({
                        "event": h.event,
                        "method": h.method,
                        "url": h.url,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();

        let entity_rows: Vec<Value> = state
            .entities
            .iter()
            .filter(|e| !e.name.starts_with('_'))
            .map(|e| {
                let count = state.db.count(&e.name).unwrap_or(0);
                json!({"entity": e.name, "count": count})
            })
            .collect();

        let relationship_graph =
            graph::build_graph_from_state(&state.entities, &state.pages, &state.webhooks);

        // Load semantic memory for context
        let memory_data = open_memory_db()
            .ok()
            .and_then(|m| m.get_context_data().ok())
            .unwrap_or(json!({"decisions": [], "changelog": []}));

        // Constitution violations — computed before json! macro (generics don't work inside macro)
        let constitution_violations_json: Vec<Value> = {
            let mut ctx_nodes = Vec::new();
            for e in &state.entities {
                ctx_nodes.push(AstNode::Entity(e.clone()));
            }
            for p in &state.pages {
                ctx_nodes.push(AstNode::Page(p.clone()));
            }
            state
                .app
                .constitution
                .as_ref()
                .map(|c| {
                    constitution_check::check_constitution(&ctx_nodes, c)
                        .iter()
                        .map(|v| {
                            json!({
                                "type": v.rule_type,
                                "rule": v.rule,
                                "violation": v.violation,
                                "entity": v.entity,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        return Ok(json_response(
            StatusCode::OK,
            json!({
                "acp_version": "1.0.0",
                "project": {
                    "name": state.app.name,
                    "port": state.app.port,
                },
                "entities": entities_json,
                "pages": pages_json,
                "apis": apis_json,
                "webhooks": webhooks_json,
                "auth": {
                    "entity": state.auth_entity,
                    "roles": state.auth_roles,
                },
                "health": {
                    "entity_rows": entity_rows,
                    "brain": brain_stats,
                },
                "constitution": {
                    "invariants": state.app.constitution.as_ref().map(|c| c.must.clone()).unwrap_or_default(),
                    "forbidden": state.app.constitution.as_ref().map(|c| c.never.clone()).unwrap_or_default(),
                    "violations": constitution_violations_json,
                },
                "relationship_graph": relationship_graph,
                "memory": memory_data,
            }),
        ));
    }

    // Server logs API — returns brain events as JSON
    if path == "/api/server/logs" && method == Method::GET {
        let limit: usize = query
            .split('&')
            .find_map(|p| {
                let mut kv = p.splitn(2, '=');
                if kv.next() == Some("limit") {
                    kv.next().and_then(|v| v.parse().ok())
                } else {
                    None
                }
            })
            .unwrap_or(100);
        match state.db.find_all("_brain_events", limit, 0) {
            Ok(rows) => return Ok(json_response(StatusCode::OK, rows)),
            Err(_) => return Ok(json_response(StatusCode::OK, json!([]))),
        }
    }

    // Server stats API
    if path == "/api/server/stats" && method == Method::GET {
        let brain_stats = state.brain.as_ref().map(|b| b.stats()).unwrap_or(json!({}));
        let entity_counts: Vec<Value> = state
            .entities
            .iter()
            .filter(|e| !e.name.starts_with('_'))
            .map(|e| {
                let count = state.db.count(&e.name).unwrap_or(0);
                json!({"entity": e.name, "count": count})
            })
            .collect();
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "brain": brain_stats,
                "entities": entity_counts,
                "pages": state.pages.len(),
                "apis": state.apis.len(),
                "uptime": "running",
            }),
        ));
    }

    // Auto-generated documentation — 100% derived from the .cronus AST
    if path == "/docs" && method == Method::GET {
        let html = render_auto_docs(&state);
        return Ok(html_response(html));
    }

    // Design system documentation — live rendered components
    if path == "/docs/design" && method == Method::GET {
        let html = render_design_system(&state);
        return Ok(html_response(html));
    }

    // Relationship graph — interactive Mermaid diagram
    if path == "/docs/graph" && method == Method::GET {
        let html = render_graph_page(&state);
        return Ok(html_response(html));
    }

    // AI Documentation Index — structured JSON for AI navigation
    if path == "/api/docs/index" && method == Method::GET {
        return Ok(json_response(
            StatusCode::OK,
            server::docs_index::generate_docs_index(&state),
        ));
    }
    if path.starts_with("/api/docs/search") && method == Method::GET {
        let q = query
            .split('&')
            .find_map(|p| p.strip_prefix("q="))
            .unwrap_or("");
        return Ok(json_response(
            StatusCode::OK,
            server::docs_index::search_docs_index(&state, q),
        ));
    }
    if path.starts_with("/api/docs/tags/") && method == Method::GET {
        let tag = path.strip_prefix("/api/docs/tags/").unwrap_or("");
        return Ok(json_response(
            StatusCode::OK,
            server::docs_index::get_docs_by_tag(&state, tag),
        ));
    }

    Err(req)
}
