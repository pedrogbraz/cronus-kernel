//! Hydra — Block evolution system for CRONUS.
//!
//! Extracts patterns from running scripts, scores them,
//! promotes validated ones to canonical blocks, and maintains
//! the block registry with lineage tracking.

pub mod registry;
pub mod extract;
pub mod compose;
pub mod microservices;

use registry::{BlockRegistry, CanonicalBlock, BlockTier, BlockStatus};
use crate::scripting::ast::ScriptFile;
use crate::trust::{self, TrustProfile, EvidencePack, TrustGates, Lineage, BlockOrigin};

/// Run the full evolution cycle:
/// 1. Collect metrics from running scripts
/// 2. Score each script block
/// 3. Promote blocks that pass threshold
/// 4. Update registry
pub fn evolve(
    registry: &mut BlockRegistry,
    scripts: &[ScriptFile],
) -> EvolutionReport {
    let mut report = EvolutionReport::new();

    let all_metrics = trust::all_metrics();
    if all_metrics.is_empty() {
        report.message = "No execution metrics yet — run scripts first".into();
        return report;
    }

    for metrics in &all_metrics {
        let evidence = metrics.to_evidence();
        let gates = TrustGates::new_clean();
        let trust = TrustProfile::from_evidence(&evidence, gates);
        let score = trust.score();
        let status = trust.status();

        report.blocks_analyzed += 1;
        report.scores.push(BlockScoreEntry {
            block_id: metrics.block_id.clone(),
            score,
            status: format!("{:?}", status),
            executions: metrics.executions,
            errors: metrics.errors,
            promotable: trust.promotable(),
        });

        // Auto-promote if trust is high enough and not already in registry
        if trust.promotable() && !registry.has_block(&metrics.block_id) {
            // Find the script that owns this block
            let script_name = metrics.block_id.split('.').next().unwrap_or("unknown");
            if let Some(script) = scripts.iter().find(|s| s.name == script_name) {
                // Generate promoted .cronus text
                let promoted_text = crate::promote::promote_to_text(script);

                let block = CanonicalBlock {
                    id: generate_block_id(),
                    name: metrics.block_id.clone(),
                    version: "1.0.0".into(),
                    hash: hash_content(&promoted_text),
                    tier: BlockTier::Composed,
                    status: if trust.official() { BlockStatus::Official } else { BlockStatus::Production },
                    intent: format!("Auto-promoted from script: {}", metrics.block_id),
                    cronus_text: promoted_text,
                    trust,
                    lineage: Lineage::new_promoted(&format!("{}.scriptcronus", script_name)),
                    tags: extract_tags(&metrics.block_id),
                };

                registry.add_block(block);
                report.blocks_promoted += 1;
            }
        }
    }

    report.message = format!(
        "Analyzed {} blocks, promoted {} to registry",
        report.blocks_analyzed, report.blocks_promoted
    );
    report
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EvolutionReport {
    pub blocks_analyzed: usize,
    pub blocks_promoted: usize,
    pub message: String,
    pub scores: Vec<BlockScoreEntry>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BlockScoreEntry {
    pub block_id: String,
    pub score: f64,
    pub status: String,
    pub executions: u64,
    pub errors: u64,
    pub promotable: bool,
}

impl EvolutionReport {
    fn new() -> Self {
        Self {
            blocks_analyzed: 0,
            blocks_promoted: 0,
            message: String::new(),
            scores: Vec::new(),
        }
    }
}

fn generate_block_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("blk_{:016x}", nanos & 0xFFFF_FFFF_FFFF_FFFF)
}

fn hash_content(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn extract_tags(block_id: &str) -> Vec<String> {
    block_id.split('.')
        .map(|s| s.to_lowercase())
        .filter(|s| !s.is_empty() && s.len() > 2)
        .collect()
}
