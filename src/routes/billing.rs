//! Payment endpoints (`/api/checkout`, Stripe webhook, status).

use super::*;

pub(super) fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    // Payment endpoints
    if path == "/api/checkout" && method == Method::POST {
        let engine = payments::PaymentEngine::from_env();
        let result = engine.create_checkout_url(
            "starter",
            2900,
            "/billing/success",
            "/billing/cancel",
            None,
        );
        match result {
            Ok(data) => return Ok(json_response(StatusCode::OK, data)),
            Err(e) => {
                return Ok(json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": e}),
                ))
            }
        }
    }
    if path == "/api/webhooks/stripe" && method == Method::POST {
        let engine = payments::PaymentEngine::from_env();
        return Ok(json_response(
            StatusCode::OK,
            json!({"status": "webhook received", "mode": if engine.is_live() { "live" } else { "mock" }}),
        ));
    }
    if path == "/api/payments/status" {
        let engine = payments::PaymentEngine::from_env();
        return Ok(json_response(StatusCode::OK, engine.status()));
    }

    Err(req)
}
