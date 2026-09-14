//! Pattern extraction — analyzes scripts to find block candidates.
//!
//! Scans running scripts, identifies stable patterns, and suggests
//! which ones should be promoted to canonical blocks.

use crate::scripting::ast::*;
use crate::trust;

/// A candidate block extracted from a script.
#[derive(Debug, Clone)]
pub struct BlockCandidate {
    pub name: String,
    pub source_script: String,
    pub block_type: CandidateType,
    pub entity: Option<String>,
    pub trust_score: f64,
    pub executions: u64,
    pub promotable: bool,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum CandidateType {
    EventHandler, // on Entity.event
    Endpoint,     // endpoint METHOD /path
    Schedule,     // schedule "name"
    Webhook,      // on webhook "/path"
}

/// Extract block candidates from a list of scripts + their execution metrics.
pub fn extract_candidates(scripts: &[ScriptFile]) -> Vec<BlockCandidate> {
    let mut candidates = Vec::new();
    let all_metrics = trust::all_metrics();

    for script in scripts {
        for block in &script.blocks {
            let (block_id, candidate_type, entity) = match block {
                ScriptBlock::OnEvent(on) => (
                    format!("{}.{}.{}", script.name, on.entity, on.event),
                    CandidateType::EventHandler,
                    Some(on.entity.clone()),
                ),
                ScriptBlock::Endpoint(ep) => (
                    format!("{}.endpoint.{}.{}", script.name, ep.method, ep.path),
                    CandidateType::Endpoint,
                    None,
                ),
                ScriptBlock::Schedule(sched) => (
                    format!("{}.schedule.{}", script.name, sched.name),
                    CandidateType::Schedule,
                    None,
                ),
                ScriptBlock::OnWebhook(wh) => (
                    format!("{}.webhook.{}", script.name, wh.path),
                    CandidateType::Webhook,
                    None,
                ),
            };

            // Find metrics for this block
            let metrics = all_metrics.iter().find(|m| m.block_id == block_id);

            let (trust_score, executions, promotable, reason) = if let Some(m) = metrics {
                let evidence = m.to_evidence();
                let gates = trust::TrustGates::new_clean();
                let trust = trust::TrustProfile::from_evidence(&evidence, gates);
                let score = trust.score();
                let promo = trust.promotable();
                let reason = if promo {
                    "Ready for promotion — trust threshold met".into()
                } else if m.executions < 100 {
                    format!("Needs more runs ({}/100 minimum)", m.executions)
                } else if score < 0.6 {
                    format!("Trust too low ({:.3} < 0.6)", score)
                } else {
                    "Not yet promotable".into()
                };
                (score, m.executions, promo, reason)
            } else {
                (0.0, 0, false, "No execution data yet".into())
            };

            candidates.push(BlockCandidate {
                name: block_id,
                source_script: script.name.clone(),
                block_type: candidate_type,
                entity,
                trust_score,
                executions,
                promotable,
                reason,
            });
        }
    }

    candidates
}

/// Find patterns that appear across multiple scripts (reusable patterns).
pub fn find_common_patterns(scripts: &[ScriptFile]) -> Vec<PatternMatch> {
    let mut patterns: Vec<PatternMatch> = Vec::new();

    // Collect all entity event handlers
    let mut entity_events: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for script in scripts {
        for block in &script.blocks {
            if let ScriptBlock::OnEvent(on) = block {
                let key = format!("{}.{}", on.entity, on.event);
                entity_events
                    .entry(key)
                    .or_default()
                    .push(script.name.clone());
            }
        }
    }

    // Patterns that appear in 2+ scripts
    for (pattern, sources) in &entity_events {
        if sources.len() >= 2 {
            patterns.push(PatternMatch {
                pattern: pattern.clone(),
                occurrences: sources.len(),
                sources: sources.clone(),
                suggestion: format!(
                    "Entity event '{}' appears in {} scripts — candidate for shared block",
                    pattern,
                    sources.len()
                ),
            });
        }
    }

    patterns
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PatternMatch {
    pub pattern: String,
    pub occurrences: usize,
    pub sources: Vec<String>,
    pub suggestion: String,
}
