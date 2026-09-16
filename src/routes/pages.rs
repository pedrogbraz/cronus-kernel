//! SSR pages: component catalog, auto auth pages, page auth middleware,
//! page rendering and the 404 page.

use super::*;

fn query_error(query: &str) -> Option<String> {
    crate::session::json_from_urlencoded(query.as_bytes())
        .get("error")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

pub(super) fn route(req: Request<Incoming>, ctx: &Ctx) -> Routed {
    let Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    } = ctx;

    // Serve pages
    let accent = state
        .style
        .as_ref()
        .and_then(|s| s.accent.as_deref())
        .unwrap_or("amber");
    let app_name = &state.app.name;

    // ── Component preview route (skip if user defined a /components page) ──
    let has_components_page_main = state.pages.iter().any(|p| p.route == "/components");
    if path == "/components" && !has_components_page_main {
        let body = if state.components.is_empty() {
            r#"<div data-slot="catalog"><header data-slot="catalog-header"><h1>Kit</h1><p data-slot="catalog-lead">No components defined.</p></header></div>"#.to_string()
        } else {
            ui::render_components_page(&state.components)
        };
        let html = if let Some(ref layout) = state.layout {
            ui::render_layout_declarative(app_name, layout, "/_components", &body)
        } else {
            ui::render_layout(app_name, &state.pages, accent, &body)
        };
        return Ok(html_response(html));
    }

    // ── Auto-generated auth pages (when auth block exists) ──
    if state.auth_entity.is_some() {
        if path == "/login" {
            let err = query_error(query);
            let html = generate_login_page(&state, err.as_deref());
            return Ok(html_response(html));
        }
        if path == "/register" || path == "/signup" {
            let err = query_error(query);
            let html = generate_register_page(&state, err.as_deref());
            return Ok(html_response(html));
        }
        if path == "/logout" {
            // GET must not clear the cookie (CSRF logout). Sign-out is POST
            // `/api/auth/logout`. A bookmark here only reaches the login page.
            return Ok(Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", "/login")
                .body(Full::new(Bytes::new()))
                .unwrap());
        }
    }

    // ── Auth middleware — protect pages that require authentication ──
    // SECURITY: exact route-pattern match (`/orders/:id`), never prefix.
    let matched_requires = access::best_matching_route(
        state.auth_required_pages.iter().map(|(r, _)| r.as_str()),
        &path,
    )
    .and_then(|route| {
        state
            .auth_required_pages
            .iter()
            .find(|(r, _)| r == route)
            .map(|(_, req)| req.clone())
    });

    if let Some(requires_str) = matched_requires {
        let token = req
            .headers()
            .get("cookie")
            .and_then(|c| c.to_str().ok())
            .and_then(|c| c.split(';').find(|s| s.trim().starts_with("cronus_token=")))
            .map(|s| s.trim().trim_start_matches("cronus_token=").to_string())
            .or_else(|| {
                req.headers()
                    .get("authorization")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|h| h.strip_prefix("Bearer "))
                    .map(|s| s.to_string())
            });

        let secret = auth::default_secret();

        let redirect_to_login = || {
            Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", "/login")
                .body(Full::new(Bytes::new()))
                .unwrap()
        };

        if requires_str.starts_with("role(") {
            // Role-based access control
            let required_role = requires_str
                .trim_start_matches("role(")
                .trim_end_matches(')');
            match &token {
                Some(t) => {
                    match auth::verify_token(t, &secret) {
                        Ok(claims) => {
                            if claims.role != required_role && claims.role != "admin" {
                                // User is logged in but lacks the role — redirect to portal
                                return Ok(Response::builder()
                                    .status(StatusCode::FOUND)
                                    .header("Location", "/")
                                    .body(Full::new(Bytes::new()))
                                    .unwrap());
                            }
                        }
                        Err(_) => return Ok(redirect_to_login()),
                    }
                }
                None => return Ok(redirect_to_login()),
            }
        } else {
            // Simple auth check
            let authenticated = match &token {
                Some(t) => auth::verify_token(t, &secret).is_ok(),
                None => false,
            };
            if !authenticated {
                return Ok(redirect_to_login());
            }
        }
    }

    // Find matching page (static `/projects/new` beats `/projects/:id`).
    let page = access::best_matching_route(state.pages.iter().map(|p| p.route.as_str()), &path)
        .and_then(|route| state.pages.iter().find(|p| p.route == route));

    if let Some(page) = page {
        // Extract route params from parameterized routes (e.g. /orders/:id/edit)
        let route_params: std::collections::HashMap<String, String> = {
            let mut params = std::collections::HashMap::new();
            if page.route.contains(':') {
                let route_parts: Vec<&str> = page.route.split('/').collect();
                let path_parts: Vec<&str> = path.split('/').collect();
                for (rp, pp) in route_parts.iter().zip(path_parts.iter()) {
                    if let Some(param_name) = rp.strip_prefix(':') {
                        params.insert(param_name.to_string(), pp.to_string());
                    }
                }
            }
            params
        };

        if let Some(source_path) = page.config.get("source") {
            if http_guard::is_production() {
                return Ok(http_guard::not_found());
            }
            match std::fs::read_to_string(source_path) {
                Ok(html) => {
                    // Developer-authored file with no interpolated data: trusted scripts.
                    return Ok(html_response(crate::security::mark_kernel_scripts(&html)));
                }
                Err(err) => {
                    // Detail goes to the log once; the page never shows IO errors or paths.
                    eprintln!("  \x1b[31m✗\x1b[0m source page {}: {}", page.route, err);
                    let body = r#"<div style="padding:40px">
  <h1 style="font-size:16px;color:var(--foreground);margin-bottom:8px">Failed to load source HTML</h1>
</div>"#.to_string();
                    let html = if let Some(ref layout) = state.layout {
                        ui::render_layout_declarative(app_name, layout, &path, &body)
                    } else {
                        ui::render_layout(app_name, &state.pages, accent, &body)
                    };
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(Full::new(Bytes::from(html)))
                        .unwrap());
                }
            }
        }

        if page.config.get("layout").map(|s| s.as_str()) == Some("light-app") {
            let referenced: Vec<parser::ComponentNode> = if !page.components.is_empty() {
                page.components
                    .iter()
                    .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                    .cloned()
                    .collect()
            } else {
                state.components.clone()
            };
            let html = ui::render_light_app_page(app_name, &referenced);
            return Ok(html_response(html));
        }

        // Auth pages — standalone login/signup with no layout chrome
        let route_lower = page.route.to_lowercase();
        let title_lower = page.title.as_deref().unwrap_or("").to_lowercase();
        let is_auth_page = route_lower == "/login"
            || route_lower == "/signup"
            || title_lower.contains("sign in")
            || title_lower.contains("sign up")
            || title_lower.contains("login")
            || title_lower.contains("signup");
        // Only use built-in auth renderer if page has NO custom template sections
        let has_custom_template =
            page.page_type == "custom" && page.sections.iter().any(|s| s.template.is_some());
        if is_auth_page && !has_custom_template {
            let is_login = route_lower == "/login"
                || title_lower.contains("login")
                || title_lower.contains("sign in");
            let html = ui::render_auth_page(page, is_login);
            return Ok(html_response(html));
        }

        let theme = state
            .style
            .as_ref()
            .and_then(|s| s.theme.as_deref())
            .unwrap_or("dark");

        // SECURITY: viewer for SSR bindings (owner scope, auth.* refs, redaction).
        let page_access = access::Access::from_state(req.headers(), state.as_ref());

        // Check if page has inline sidebar/topbar sections (not components)
        let has_section_sidebar = page.sections.iter().any(|s| s.section_type == "sidebar");

        // Settings page — full-page renderer with its own sidebar/topbar
        // Order Detail page — full-page renderer
        let is_order_detail = page
            .sections
            .iter()
            .any(|s| s.section_type == "order-header" || s.section_type == "line-items");
        if is_order_detail {
            let referenced_comps: Vec<parser::ComponentNode> = page
                .components
                .iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            let html = ui::render_order_detail_dashboard(
                app_name,
                &page.sections,
                &referenced_comps,
                theme,
                page.route.as_str(),
            );
            return Ok(html_response(html));
        }

        // Settings page — full-page renderer
        let is_settings_page = page
            .sections
            .iter()
            .any(|s| s.section_type == "settings-profile" || s.section_type == "subscription-card");
        if is_settings_page {
            let referenced_comps: Vec<parser::ComponentNode> = page
                .components
                .iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            let html = ui::render_settings_dashboard(
                app_name,
                &page.sections,
                &referenced_comps,
                theme,
                page.route.as_str(),
            );
            return Ok(html_response(html));
        }

        // Dumped pages with HTML templates — use landing layout with Tailwind CDN,
        // EXCEPT for auth-protected pages that have a declarative layout (these
        // must share the same sidebar shell across all pages for consistency).
        let has_templates = page
            .sections
            .iter()
            .any(|s| s.template.is_some() || s.config.get("template").is_some());
        let is_auth_page = page.requires.as_deref() == Some("auth")
            || page
                .requires
                .as_deref()
                .map(|r| r.starts_with("role("))
                .unwrap_or(false);
        let has_declarative_layout = state.layout.is_some();
        if has_templates && !(is_auth_page && has_declarative_layout) {
            let body = ui::render_page(
                page,
                &state.entities,
                accent,
                theme,
                Some(&state.db),
                &route_params,
                &page_access,
            );
            let html = ui::render_layout_landing_ex(
                app_name,
                &body,
                theme,
                state.style.as_ref(),
                state.app.tailwind_config.as_deref(),
            );
            return Ok(html_response(html));
        }

        if has_section_sidebar {
            // Check for specialized dashboard renderers BEFORE falling back to generic
            let billing_types = [
                "current-plan",
                "usage-status",
                "billing-stats",
                "payment-methods",
                "recent-invoices",
            ];
            let is_billing_page = page
                .sections
                .iter()
                .any(|s| s.section_type == "current-plan" || s.section_type == "billing-stats");
            if is_billing_page {
                let referenced_comps: Vec<parser::ComponentNode> = page
                    .components
                    .iter()
                    .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                    .cloned()
                    .collect();
                let html = ui::render_billing_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    &path,
                );
                return Ok(html_response(html));
            }

            // Generic dashboard wrapper — sidebar + any sections
            let body = ui::render_page(
                page,
                &state.entities,
                accent,
                theme,
                Some(&state.db),
                &route_params,
                &page_access,
            );
            let html = ui::render_layout_dashboard(&state.app.name, &body, theme);
            return Ok(html_response(html));
        }

        let mut body = if page.page_type == "components" && !state.components.is_empty() {
            ui::render_components_page(&state.components)
        } else {
            ui::render_page(
                page,
                &state.entities,
                accent,
                theme,
                Some(&state.db),
                &route_params,
                &page_access,
            )
        };

        // `use ComponentName` on a real page — widgets only, no kit chrome.
        // type:components already rendered the full catalog above.
        let has_sidebar_component_early = !page.components.is_empty()
            && page.components.iter().any(|comp_name| {
                state.components.iter().any(|c| {
                    c.name == *comp_name
                        && (c.style.as_deref().unwrap_or("").contains("sidenav")
                            || c.layout.as_deref().unwrap_or("") == "sidebar")
                })
            });
        if page.page_type != "components" {
            if !page.components.is_empty() && !has_sidebar_component_early {
                let referenced: Vec<parser::ComponentNode> = page
                    .components
                    .iter()
                    .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                    .cloned()
                    .collect();
                if !referenced.is_empty() {
                    body.push_str("\n");
                    body.push_str(&ui::render_components_inline(&referenced));
                }
            }

            // custom pages with no sections — fallback to the kit catalog
            if page.page_type == "custom"
                && page.sections.is_empty()
                && !state.components.is_empty()
            {
                body.push_str("\n");
                body.push_str(&ui::render_components_page(&state.components));
            }
        }

        // FIX 1: Detect sidebar component — if page uses a Sidenav component, it's a dashboard page
        let has_sidebar_component = !page.components.is_empty()
            && page.components.iter().any(|comp_name| {
                state.components.iter().any(|c| {
                    c.name == *comp_name
                        && (c.style.as_deref().unwrap_or("").contains("sidenav")
                            || c.layout.as_deref().unwrap_or("") == "sidebar")
                })
            });

        // Landing/checkout pages use full-width layout, no sidebar
        let landing_section_types = [
            "hero",
            "topbar",
            "checkout",
            "features",
            "pricing",
            "cta",
            "testimonial",
            "faq",
            "trusted",
            "footer",
        ];
        // If ANY section has a template, it's a dumped page — always use landing layout
        let has_templates = page
            .sections
            .iter()
            .any(|s| s.template.is_some() || s.config.get("template").is_some());
        let is_landing = has_templates
            || (!has_sidebar_component
                && (page.page_type == "checkout"
                    || (page.page_type == "custom"
                        && page
                            .sections
                            .iter()
                            .any(|s| landing_section_types.contains(&s.section_type.as_str())))));
        let dashboard_types = [
            "sidebar",
            "card",
            "page-header",
            "stat-cards",
            "product-grid",
            "team-list",
            "policies",
            "activity-table",
            "status-card",
            "links",
            "live-keys",
            "test-keys",
            "webhooks",
            "quick-links",
            "current-plan",
            "usage-status",
            "billing-stats",
            "payment-methods",
            "recent-invoices",
            "balance-card",
            "upcoming-card",
            "payout-history",
            "support-banner",
            "checkout-form",
            "product-summary",
            "trust-indicators",
            "team-members",
            "security-status",
            "security-policies",
            "login-activity",
            "settings-profile",
            "api-keys",
            "security-grid",
            "subscription-card",
            "invoices-list",
            "support-card",
            "danger-zone",
            "order-header",
            "line-items",
            "price-breakdown",
            "payment-info",
            "customer-profile",
            "shipping-timeline",
            "staff-notes",
        ];
        let is_dashboard = has_sidebar_component
            || page
                .sections
                .iter()
                .any(|s| dashboard_types.contains(&s.section_type.as_str()));
        let is_billing = page
            .sections
            .iter()
            .any(|s| s.section_type == "current-plan" || s.section_type == "billing-stats");
        let is_payouts = page
            .sections
            .iter()
            .any(|s| s.section_type == "balance-card" || s.section_type == "payout-history");
        let is_unified =
            page.sections
                .iter()
                .any(|s| s.section_type == "balance-card")
                && page.sections.iter().any(|s| {
                    s.section_type == "billing-stats" || s.section_type == "recent-invoices"
                });
        let is_payment_links = page
            .sections
            .iter()
            .any(|s| s.section_type == "product-grid")
            && page.sections.iter().any(|s| s.section_type == "stat-cards");
        let is_checkout = page
            .sections
            .iter()
            .any(|s| s.section_type == "checkout-form" || s.section_type == "product-summary");
        let is_security = page
            .sections
            .iter()
            .any(|s| s.section_type == "team-members" || s.section_type == "login-activity");
        let is_settings = page
            .sections
            .iter()
            .any(|s| s.section_type == "settings-profile" || s.section_type == "subscription-card");
        let current_route = page.route.as_str();
        // Auth pages with a declarative layout share the sidebar shell
        let auth_with_layout = is_auth_page && has_declarative_layout;
        let html = if auth_with_layout {
            if let Some(ref layout) = state.layout {
                ui::render_layout_declarative(app_name, layout, current_route, &body)
            } else {
                ui::render_layout(app_name, &state.pages, accent, &body)
            }
        } else if has_templates {
            // Dumped page with original HTML templates — use landing layout, no sidebar
            ui::render_layout_landing_ex(
                app_name,
                &body,
                theme,
                state.style.as_ref(),
                state.app.tailwind_config.as_deref(),
            )
        } else if is_checkout {
            // Checkout page: no sidebar, centered layout
            let referenced_comps: Vec<parser::ComponentNode> = page
                .components
                .iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            ui::render_checkout_dashboard(app_name, &page.sections, &referenced_comps, theme)
        } else if is_dashboard {
            // Dedicated dashboard renderer: produces the ENTIRE page in one shot
            let referenced_comps: Vec<parser::ComponentNode> = page
                .components
                .iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            if is_unified {
                ui::render_unified_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            } else if is_payouts {
                ui::render_payouts_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            } else if is_billing {
                ui::render_billing_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            } else if is_settings {
                ui::render_settings_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            } else if is_security {
                ui::render_security_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            } else if is_payment_links {
                ui::render_payment_links_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            } else {
                // Generic dashboard wrapper — sidebar + any sections
                ui::render_generic_dashboard(
                    app_name,
                    &body,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            }
        } else if is_landing {
            ui::render_layout_landing(app_name, &body, theme, state.style.as_ref())
        } else if let Some(ref layout) = state.layout {
            ui::render_layout_declarative(app_name, layout, current_route, &body)
        } else {
            ui::render_layout(app_name, &state.pages, accent, &body)
        };
        return Ok(html_response(html));
    }

    Err(req)
}

/// The HTML 404 page. Never echoes the requested path.
pub(super) fn not_found(state: &AppState) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let accent = state
        .style
        .as_ref()
        .and_then(|s| s.accent.as_deref())
        .unwrap_or("amber");
    let app_name = &state.app.name;

    // 404
    let body = format!(
        "<div class=\"flex items-center justify-center min-h-[60vh]\"><div class=\"text-center\"><h1 class=\"text-6xl font-bold text-neutral-600\">404</h1><p class=\"mt-4 text-neutral-400\">Page not found</p><a href=\"/\" class=\"mt-6 inline-block text-{}-400 hover:underline\">← Back home</a></div></div>",
        accent
    );
    let html = if let Some(ref layout) = state.layout {
        ui::render_layout_declarative(app_name, layout, "/404", &body)
    } else {
        ui::render_layout(app_name, &state.pages, accent, &body)
    };
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Full::new(Bytes::from(html)))
        .unwrap())
}
