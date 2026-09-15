//! `/graphql` playground, execution (session required) and schema.

use super::*;

pub(super) async fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    // GraphQL endpoint
    if path == "/graphql" && method == Method::GET {
        return Ok(html_response(graphql::playground_html()));
    }
    if path == "/graphql" && method == Method::POST {
        // SECURITY: GraphQL requires a session; owner scope/redaction in graphql.rs.
        let gql_access = access::Access::from_headers(req.headers(), state.auth_entity.clone());
        if gql_access.viewer.is_none() {
            return Ok(json_response(
                StatusCode::UNAUTHORIZED,
                graphql::gql_error("UNAUTHENTICATED", "authentication required"),
            ));
        }
        let body_bytes = match http_guard::read_body(req).await {
            Ok(b) => b,
            Err(r) => return Ok(r),
        };
        let body_str = String::from_utf8_lossy(&body_bytes);
        let body_json: Value = serde_json::from_str(&body_str).unwrap_or(json!({}));
        let query = body_json
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let variables = body_json.get("variables").cloned().unwrap_or(json!({}));
        let schema = graphql::GraphQLSchema::from_entities(&state.entities);
        let result = graphql::execute_graphql(
            query,
            &variables,
            &schema,
            &state.db,
            &gql_access,
            &state.db_path,
        );
        return Ok(json_response(StatusCode::OK, result));
    }
    if path == "/graphql/schema" && method == Method::GET {
        if access::viewer_from_headers(req.headers(), &auth::default_secret()).is_none() {
            return Ok(json_response(
                StatusCode::UNAUTHORIZED,
                graphql::gql_error("UNAUTHENTICATED", "authentication required"),
            ));
        }
        let schema = graphql::GraphQLSchema::from_entities(&state.entities);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Full::new(Bytes::from(schema.sdl)))
            .unwrap());
    }

    Err(req)
}
