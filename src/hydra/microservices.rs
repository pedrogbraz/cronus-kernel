//! Microservices splitter for CRONUS compose system.
//!
//! Splits a monolithic .cronus into per-service .cronus files,
//! generates API gateway config, Docker Compose, and Dockerfiles.

use std::collections::HashMap;

use super::compose::{AuthConfig, ComposeOptions, EntityRemap, StyleConfig};

/// A single service split from the monolith.
#[derive(Debug, Clone)]
pub struct ServiceSplit {
    pub name: String,
    pub port: u16,
    pub db_path: String,
    pub cronus_source: String,
}

/// API gateway definition that routes to services.
#[derive(Debug, Clone)]
pub struct GatewayDef {
    pub port: u16,
    pub services: Vec<(String, u16, Vec<String>)>, // (name, port, route_patterns)
    pub cronus_source: String,
}

/// Microservice definition used by templates.
#[derive(Debug, Clone)]
pub struct ServiceDef {
    pub name: String,
    pub port: u16,
    pub db: Option<String>,
    pub entities: Vec<String>,
    pub apis: Vec<String>,
    pub pages: Vec<String>,
}

impl ServiceDef {
    pub fn new(name: &str, port: u16, entities: &[&str]) -> Self {
        let apis: Vec<String> = entities.iter().map(|e| e.to_lowercase() + "s").collect();
        let pages: Vec<String> = entities
            .iter()
            .flat_map(|e| {
                let lower = e.to_lowercase();
                vec![format!("/{}s", lower), format!("/{}s/new", lower)]
            })
            .collect();
        Self {
            name: name.to_string(),
            port,
            db: Some(format!("./data-{}.db", name)),
            entities: entities.iter().map(|s| s.to_string()).collect(),
            apis,
            pages,
        }
    }
}

/// Split a monolithic .cronus source into per-service .cronus files.
///
/// `full_source` - the complete composed .cronus text
/// `all_entities` - the entity remaps from the template
/// `service_defs` - how to split entities across services
/// `gateway_port` - the port for the API gateway
/// `options` - original compose options (for style/auth extraction)
pub fn split_services(
    _full_source: &str,
    all_entities: &[EntityRemap],
    service_defs: &[ServiceDef],
    gateway_port: u16,
    options: &ComposeOptions,
) -> (GatewayDef, Vec<ServiceSplit>) {
    let entity_map: HashMap<&str, &EntityRemap> = all_entities
        .iter()
        .map(|e| (e.entity.as_str(), e))
        .collect();

    // Collect all entity names across all services for cross-reference detection
    let mut service_entity_sets: HashMap<&str, Vec<&str>> = HashMap::new();
    for svc in service_defs {
        service_entity_sets.insert(
            svc.name.as_str(),
            svc.entities.iter().map(|s| s.as_str()).collect(),
        );
    }

    let mut splits = Vec::new();
    let mut gateway_routes: Vec<(String, u16, Vec<String>)> = Vec::new();

    for svc in service_defs {
        let mut source = String::new();

        // App block
        source.push_str(&format!("app \"{}-{}\" {{\n", "app", svc.name));
        source.push_str(&format!("  port {}\n", svc.port));
        let db = svc.db.as_deref().unwrap_or("./data.db");
        source.push_str(&format!("  database sqlite \"{}\"\n", db));
        source.push_str("  theme dark\n");
        source.push_str("}\n\n");

        // Style block (copied from original)
        if let Some(ref style) = options.style {
            source.push_str("style {\n");
            source.push_str(&format!("  theme {}\n", style.theme));
            source.push_str(&format!("  accent {}\n", style.accent));
            source.push_str(&format!("  font \"{}\"\n", style.font));
            source.push_str("}\n\n");
        }

        // Auth block if service has User entity
        let has_user = svc.entities.iter().any(|e| e == "User");
        if has_user || svc.name == "auth" {
            if let Some(ref auth_cfg) = options.auth_config {
                source.push_str("auth {\n");
                source.push_str("  entity User\n");
                source.push_str(&format!("  login {}\n", auth_cfg.login_fields));
                source.push_str(&format!(
                    "  session {} expires:{}\n",
                    auth_cfg.session_type, auth_cfg.expires
                ));
                if !auth_cfg.roles.is_empty() {
                    source.push_str(&format!("  roles [{}]\n", auth_cfg.roles.join(", ")));
                }
                source.push_str("}\n\n");

                // User entity
                source.push_str("entity User {\n");
                source.push_str("  name string!\n");
                source.push_str("  email email! unique\n");
                source.push_str("  password_hash string sensitive\n");
                if !auth_cfg.roles.is_empty() {
                    let default_role = auth_cfg.roles.last().map(|s| s.as_str()).unwrap_or("user");
                    source.push_str(&format!(
                        "  role enum [{}]! default:\"{}\"\n",
                        auth_cfg
                            .roles
                            .iter()
                            .map(|r| format!("\"{}\"", r))
                            .collect::<Vec<_>>()
                            .join(", "),
                        default_role
                    ));
                }
                source.push_str("}\n\n");
            }
        }

        // Collect referenced entities from other services for remote stubs
        let mut remote_entities: Vec<String> = Vec::new();
        for ent_name in &svc.entities {
            if let Some(entity) = entity_map.get(ent_name.as_str()) {
                for field in &entity.fields {
                    if let Some(ref target) = field.reference {
                        if !svc.entities.contains(target) && target != "User" {
                            if !remote_entities.contains(target) {
                                remote_entities.push(target.clone());
                            }
                        }
                    }
                }
            }
        }

        // Remote entity stubs for cross-service references
        for remote in &remote_entities {
            source.push_str(&format!("entity {} remote {{\n", remote));
            source.push_str("  // Cross-service reference — resolved via gateway\n");
            source.push_str("}\n\n");
        }

        // Entities belonging to this service
        for ent_name in &svc.entities {
            if let Some(entity) = entity_map.get(ent_name.as_str()) {
                source.push_str(&generate_entity_text(entity));
                source.push('\n');
            }
        }

        // API routes for this service's entities
        let auth_str = if options.auth { " auth:jwt" } else { "" };
        for ent_name in &svc.entities {
            let lower = ent_name.to_lowercase();
            source.push_str(&format!("api /{}s {{\n", lower));
            source.push_str(&format!("  list GET /{}s{}\n", lower, auth_str));
            source.push_str(&format!("  get GET /{}s/:id{}\n", lower, auth_str));
            source.push_str(&format!("  create POST /{}s{}\n", lower, auth_str));
            source.push_str(&format!("  update PATCH /{}s/:id{}\n", lower, auth_str));
            source.push_str(&format!("  remove DELETE /{}s/:id{}\n", lower, auth_str));
            source.push_str("}\n\n");
        }

        // Route patterns for gateway
        let route_patterns: Vec<String> = svc
            .entities
            .iter()
            .map(|e| format!("/{}s/*", e.to_lowercase()))
            .collect();
        gateway_routes.push((svc.name.clone(), svc.port, route_patterns));

        splits.push(ServiceSplit {
            name: svc.name.clone(),
            port: svc.port,
            db_path: db.to_string(),
            cronus_source: source,
        });
    }

    // Generate gateway .cronus
    let gateway_source = generate_gateway_source(gateway_port, &gateway_routes);
    let gateway = GatewayDef {
        port: gateway_port,
        services: gateway_routes,
        cronus_source: gateway_source,
    };

    (gateway, splits)
}

fn generate_entity_text(e: &EntityRemap) -> String {
    let shared_str = if e.shared { " shared" } else { "" };
    let mut out = format!("entity {}{} {{\n", e.entity, shared_str);

    for f in &e.fields {
        if let Some(ref target) = f.reference {
            out.push_str(&format!("  {} -> {}\n", f.name, target));
            continue;
        }

        let req_mark = if f.required { "!" } else { "" };
        out.push_str(&format!("  {} {}{}", f.name, f.field_type, req_mark));
        if f.unique {
            out.push_str(" unique");
        }
        if f.searchable {
            out.push_str(" searchable");
        }
        if f.sensitive {
            out.push_str(" sensitive");
        }
        if let Some(ref vals) = f.enum_values {
            out.push_str(&format!(
                " [{}]",
                vals.iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if let Some(ref d) = f.default {
            out.push_str(&format!(" default:\"{}\"", d));
        }
        out.push('\n');
    }

    for t in &e.transitions {
        out.push_str(&format!("\n  transition {} {{\n", t.field));
        for (from, to) in &t.rules {
            out.push_str(&format!("    {} -> {}\n", from, to));
        }
        out.push_str("  }\n");
    }

    for eff in &e.effects {
        if !eff.actions.is_empty() {
            out.push_str(&format!("\n  on {} {{\n", eff.event));
            for a in &eff.actions {
                out.push_str(&format!("    {}\n", a));
            }
            out.push_str("  }\n");
        }
        if !eff.when_clauses.is_empty() {
            out.push_str(&format!("\n  on {} {{\n", eff.event));
            for (val, actions) in &eff.when_clauses {
                out.push_str(&format!("    when \"{}\" {{\n", val));
                for a in actions {
                    out.push_str(&format!("      {}\n", a));
                }
                out.push_str("    }\n");
            }
            out.push_str("  }\n");
        }
    }

    out.push_str("}\n");
    out
}

fn generate_gateway_source(port: u16, services: &[(String, u16, Vec<String>)]) -> String {
    let mut out = String::new();

    out.push_str(&format!("app \"api-gateway\" {{\n"));
    out.push_str(&format!("  port {}\n", port));
    out.push_str("  mode gateway\n");
    out.push_str("}\n\n");

    out.push_str("gateway {\n");
    for (name, svc_port, routes) in services {
        out.push_str(&format!("  service \"{}\" port:{} {{\n", name, svc_port));
        for route in routes {
            out.push_str(&format!("    route \"{}\"\n", route));
        }
        out.push_str("  }\n");
    }
    out.push_str("}\n");

    out
}

/// Generate a deploy block for the monolithic .cronus file.
pub fn generate_deploy_block(
    entities: &[EntityRemap],
    gateway_port: u16,
    service_splits: &[(String, u16, Vec<String>)], // (service_name, port, entity_names)
) -> String {
    let mut out = String::new();

    out.push_str("deploy microservices {\n");
    out.push_str(&format!("  gateway port:{} {{\n", gateway_port));
    for (name, port, _) in service_splits {
        out.push_str(&format!(
            "    proxy \"/{}-api/*\" -> localhost:{}\n",
            name, port
        ));
    }
    out.push_str("  }\n\n");

    for (name, port, ent_names) in service_splits {
        out.push_str(&format!("  service \"{}\" port:{} {{\n", name, port));
        out.push_str(&format!(
            "    entities [{}]\n",
            ent_names
                .iter()
                .map(|e| format!("\"{}\"", e))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        out.push_str("  }\n");
    }

    out.push_str("}\n");
    out
}

/// Get microservice split definitions for a template name.
pub fn get_micro_split(template: &str) -> Option<(Vec<ServiceDef>, u16)> {
    match template {
        "saas-billing" => Some((
            vec![
                ServiceDef::new("auth", 5221, &["User"]),
                ServiceDef::new(
                    "billing",
                    5222,
                    &[
                        "Customer",
                        "Plan",
                        "Subscription",
                        "Invoice",
                        "PaymentMethod",
                        "UsageRecord",
                    ],
                ),
            ],
            5220,
        )),

        "blog" => Some((
            vec![
                ServiceDef::new("auth", 5211, &["User"]),
                ServiceDef::new("content", 5212, &["Post", "Category", "Tag", "Comment"]),
            ],
            5210,
        )),

        "crm" => Some((
            vec![
                ServiceDef::new("auth", 5221, &["User"]),
                ServiceDef::new("crm", 5222, &["Company", "Contact", "Deal", "Activity"]),
            ],
            5220,
        )),

        "helpdesk" => Some((
            vec![
                ServiceDef::new("auth", 5221, &["User"]),
                ServiceDef::new("support", 5222, &["Agent", "Customer", "Ticket", "Message"]),
            ],
            5220,
        )),

        "ecommerce" => Some((
            vec![
                ServiceDef::new("auth", 5201, &["User"]),
                ServiceDef::new("catalog", 5202, &["Category", "Product"]),
                ServiceDef::new("orders", 5203, &["Customer", "Order", "OrderItem"]),
            ],
            5200,
        )),

        _ => None,
    }
}

/// Generate a docker-compose.yml for the microservices deployment.
pub fn generate_docker_compose(gateway: &GatewayDef, services: &[ServiceSplit]) -> String {
    let mut out = String::new();

    out.push_str("version: \"3.8\"\n\n");
    out.push_str("services:\n");

    // Gateway service
    out.push_str("  gateway:\n");
    out.push_str("    build:\n");
    out.push_str("      context: .\n");
    out.push_str("      dockerfile: Dockerfile\n");
    out.push_str(&format!(
        "    ports:\n      - \"{}:{}\"\n",
        gateway.port, gateway.port
    ));
    out.push_str("    volumes:\n");
    out.push_str("      - ./gateway.cronus:/app/app.cronus:ro\n");
    out.push_str(&format!(
        "    command: [\"cronus\", \"run\", \".\", \"{}\"]\n",
        gateway.port
    ));
    out.push_str("    depends_on:\n");
    for svc in services {
        out.push_str(&format!("      - {}\n", svc.name));
    }
    out.push_str("    networks:\n      - cronus-net\n\n");

    // Service containers
    for svc in services {
        out.push_str(&format!("  {}:\n", svc.name));
        out.push_str("    build:\n");
        out.push_str("      context: .\n");
        out.push_str("      dockerfile: Dockerfile\n");
        out.push_str(&format!("    expose:\n      - \"{}\"\n", svc.port));
        out.push_str("    volumes:\n");
        out.push_str(&format!(
            "      - ./services/{}.cronus:/app/app.cronus:ro\n",
            svc.name
        ));
        out.push_str(&format!("      - {}-data:/app/data\n", svc.name));
        out.push_str(&format!(
            "    command: [\"cronus\", \"run\", \".\", \"{}\"]\n",
            svc.port
        ));
        out.push_str("    networks:\n      - cronus-net\n\n");
    }

    // Volumes
    out.push_str("volumes:\n");
    for svc in services {
        out.push_str(&format!("  {}-data:\n", svc.name));
    }

    // Network
    out.push_str("\nnetworks:\n");
    out.push_str("  cronus-net:\n");
    out.push_str("    driver: bridge\n");

    out
}

/// Generate a minimal Dockerfile for running a .cronus service.
pub fn generate_dockerfile() -> String {
    let mut out = String::new();

    out.push_str("FROM debian:bookworm-slim\n\n");
    out.push_str("RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*\n\n");
    out.push_str("WORKDIR /app\n\n");
    out.push_str("COPY cronus /usr/local/bin/cronus\n");
    out.push_str("COPY *.cronus /app/\n\n");
    out.push_str("EXPOSE 5220\n\n");
    out.push_str("CMD [\"cronus\", \"run\", \".\"]\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_micro_split_all_templates() {
        assert!(get_micro_split("saas-billing").is_some());
        assert!(get_micro_split("blog").is_some());
        assert!(get_micro_split("crm").is_some());
        assert!(get_micro_split("helpdesk").is_some());
        assert!(get_micro_split("ecommerce").is_some());
        assert!(get_micro_split("nonexistent").is_none());
    }

    #[test]
    fn test_ecommerce_has_three_services() {
        let (defs, gw_port) = get_micro_split("ecommerce").unwrap();
        assert_eq!(defs.len(), 3);
        assert_eq!(gw_port, 5200);
        assert_eq!(defs[0].name, "auth");
        assert_eq!(defs[1].name, "catalog");
        assert_eq!(defs[2].name, "orders");
    }

    #[test]
    fn test_generate_deploy_block() {
        let entities = vec![];
        let splits = vec![
            ("auth".to_string(), 5221u16, vec!["User".to_string()]),
            (
                "billing".to_string(),
                5222u16,
                vec!["Customer".to_string(), "Invoice".to_string()],
            ),
        ];
        let block = generate_deploy_block(&entities, 5220, &splits);
        assert!(block.contains("deploy microservices"));
        assert!(block.contains("gateway port:5220"));
        assert!(block.contains("service \"auth\" port:5221"));
        assert!(block.contains("service \"billing\" port:5222"));
    }

    #[test]
    fn test_generate_dockerfile() {
        let df = generate_dockerfile();
        assert!(df.contains("FROM debian"));
        assert!(df.contains("COPY cronus"));
        assert!(df.contains("CMD"));
    }

    #[test]
    fn test_generate_docker_compose() {
        let gateway = GatewayDef {
            port: 5220,
            services: vec![("auth".to_string(), 5221, vec!["/users/*".to_string()])],
            cronus_source: String::new(),
        };
        let services = vec![ServiceSplit {
            name: "auth".to_string(),
            port: 5221,
            db_path: "./data-auth.db".to_string(),
            cronus_source: String::new(),
        }];
        let yml = generate_docker_compose(&gateway, &services);
        assert!(yml.contains("gateway:"));
        assert!(yml.contains("auth:"));
        assert!(yml.contains("cronus-net"));
        assert!(yml.contains("auth-data:"));
    }
}
