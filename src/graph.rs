//! Relationship graph extraction from the CRONUS AST.
//! Analyzes entities, pages, and webhooks to build a complete dependency map.

use serde::Serialize;
use crate::parser::{AstNode, EntityNode, PageNode, WebhookNode};

// ══════════════════════════════════════════════════
// GRAPH TYPES
// ══════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct RelationshipGraph {
    pub entity_relations: Vec<EntityRelation>,
    pub page_bindings: Vec<PageBinding>,
    pub webhook_flows: Vec<WebhookFlow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntityRelation {
    pub from_entity: String,
    pub to_entity: String,
    pub field: String,
    pub relation_type: String, // "belongs_to", "has_many"
}

#[derive(Debug, Clone, Serialize)]
pub struct PageBinding {
    pub page_route: String,
    pub entity: String,
    pub section_type: String, // "kpi", "table", "chart"
}

#[derive(Debug, Clone, Serialize)]
pub struct WebhookFlow {
    pub entity: String,
    pub event: String,
    pub target_url: String,
}

// ══════════════════════════════════════════════════
// GRAPH BUILDER
// ══════════════════════════════════════════════════

pub fn build_graph(nodes: &[AstNode]) -> RelationshipGraph {
    let mut entities: Vec<&EntityNode> = Vec::new();
    let mut pages: Vec<&PageNode> = Vec::new();
    let mut webhooks: Vec<&WebhookNode> = Vec::new();

    for node in nodes {
        match node {
            AstNode::Entity(e) => entities.push(e),
            AstNode::Page(p) => pages.push(p),
            AstNode::Webhook(w) => webhooks.push(w),
            _ => {}
        }
    }

    let entity_relations = extract_entity_relations(&entities);
    let page_bindings = extract_page_bindings(&pages);
    let webhook_flows = extract_webhook_flows(&webhooks);

    RelationshipGraph {
        entity_relations,
        page_bindings,
        webhook_flows,
    }
}

/// Build graph directly from state vectors (used by the server endpoint).
pub fn build_graph_from_state(
    entities: &[EntityNode],
    pages: &[PageNode],
    webhooks: &[WebhookNode],
) -> RelationshipGraph {
    let ent_refs: Vec<&EntityNode> = entities.iter().collect();
    let page_refs: Vec<&PageNode> = pages.iter().collect();
    let wh_refs: Vec<&WebhookNode> = webhooks.iter().collect();

    RelationshipGraph {
        entity_relations: extract_entity_relations(&ent_refs),
        page_bindings: extract_page_bindings(&page_refs),
        webhook_flows: extract_webhook_flows(&wh_refs),
    }
}

/// Scan entity fields for `reference: Some(target)` → belongs_to.
/// Auto-infer inverse has_many from the target back to the source.
fn extract_entity_relations(entities: &[&EntityNode]) -> Vec<EntityRelation> {
    let mut relations = Vec::new();

    for entity in entities {
        for field in &entity.fields {
            if let Some(ref target) = field.reference {
                // Forward: this entity belongs_to the target
                relations.push(EntityRelation {
                    from_entity: entity.name.clone(),
                    to_entity: target.clone(),
                    field: field.name.clone(),
                    relation_type: "belongs_to".into(),
                });
                // Inverse: target has_many of this entity
                relations.push(EntityRelation {
                    from_entity: target.clone(),
                    to_entity: entity.name.clone(),
                    field: field.name.clone(),
                    relation_type: "has_many".into(),
                });
            }
        }
    }

    relations
}

/// Scan page sections for `binding: Some(b)` → page binds to entity.
fn extract_page_bindings(pages: &[&PageNode]) -> Vec<PageBinding> {
    let mut bindings = Vec::new();

    for page in pages {
        for section in &page.sections {
            if let Some(ref binding) = section.binding {
                bindings.push(PageBinding {
                    page_route: page.route.clone(),
                    entity: binding.entity.clone(),
                    section_type: section.section_type.clone(),
                });
            }
        }
    }

    bindings
}

/// Extract webhook flows from webhook nodes.
fn extract_webhook_flows(webhooks: &[&WebhookNode]) -> Vec<WebhookFlow> {
    let mut flows = Vec::new();

    for webhook in webhooks {
        for hook in &webhook.hooks {
            flows.push(WebhookFlow {
                entity: webhook.entity.clone(),
                event: hook.event.clone(),
                target_url: hook.url.clone(),
            });
        }
    }

    flows
}

// ══════════════════════════════════════════════════
// MERMAID OUTPUT
// ══════════════════════════════════════════════════

pub fn to_mermaid(graph: &RelationshipGraph) -> String {
    let mut lines = Vec::new();
    lines.push("graph LR".into());

    // Entity relations
    for rel in &graph.entity_relations {
        if rel.relation_type == "belongs_to" {
            lines.push(format!(
                "    {}-- \"{} ({})\" -->{}",
                rel.from_entity, rel.field, rel.relation_type, rel.to_entity
            ));
        }
    }

    // Page bindings
    for binding in &graph.page_bindings {
        let page_id = binding.page_route.replace('/', "_").trim_matches('_').to_string();
        let page_id = if page_id.is_empty() { "index".to_string() } else { page_id };
        lines.push(format!(
            "    page_{}[\"{}\"]-. \"{}\" .->{}",
            page_id, binding.page_route, binding.section_type, binding.entity
        ));
    }

    // Webhook flows
    for flow in &graph.webhook_flows {
        let url_short = if flow.target_url.len() > 40 {
            format!("{}...", &flow.target_url[..37])
        } else {
            flow.target_url.clone()
        };
        lines.push(format!(
            "    {}-- \"on {}\" -->webhook_{}[\"{}\"]",
            flow.entity,
            flow.event,
            flow.entity.to_lowercase(),
            url_short
        ));
    }

    if lines.len() == 1 {
        lines.push("    empty[\"No relationships detected\"]".into());
    }

    lines.join("\n")
}
