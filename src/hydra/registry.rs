//! Block Registry — stores canonical blocks with lineage and evidence.
//!
//! Blocks graduate: Sandbox → Approved → Production → Official.
//! Each block carries its trust profile and full lineage.

use crate::trust::{TrustProfile, Lineage};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalBlock {
    pub id: String,
    pub name: String,
    pub version: String,
    pub hash: String,
    pub tier: BlockTier,
    pub status: BlockStatus,
    pub intent: String,
    pub cronus_text: String,      // promoted .cronus source
    pub trust: TrustProfile,
    pub lineage: Lineage,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockTier {
    Atomic,       // single operation (db.query, http.post)
    Composed,     // multiple operations (CRUD, sync)
    Feature,      // complete business logic (stripe integration)
    Application,  // full system (NovaPay)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockStatus {
    Sandbox,      // score < 0.3
    Approved,     // 0.3 - 0.6
    Production,   // 0.6 - 0.8 (promotable)
    Official,     // 0.8 - 1.0 (shareable)
}

/// In-memory block registry.
/// Persists to JSON file for durability across restarts.
pub struct BlockRegistry {
    blocks: HashMap<String, CanonicalBlock>,
    file_path: Option<String>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        Self { blocks: HashMap::new(), file_path: None }
    }

    /// Open or create registry from a JSON file
    pub fn open(path: &str) -> Self {
        let mut reg = Self {
            blocks: HashMap::new(),
            file_path: Some(path.to_string()),
        };
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(blocks) = serde_json::from_str::<Vec<CanonicalBlock>>(&data) {
                for b in blocks {
                    reg.blocks.insert(b.name.clone(), b);
                }
            }
        }
        reg
    }

    pub fn add_block(&mut self, block: CanonicalBlock) {
        eprintln!("  \x1b[32m[hydra]\x1b[0m registered block: {} (trust: {:.3}, {:?})",
            block.name, block.trust.score(), block.status);
        self.blocks.insert(block.name.clone(), block);
        self.save();
    }

    pub fn has_block(&self, name: &str) -> bool {
        self.blocks.contains_key(name)
    }

    pub fn get_block(&self, name: &str) -> Option<&CanonicalBlock> {
        self.blocks.get(name)
    }

    pub fn all_blocks(&self) -> Vec<&CanonicalBlock> {
        self.blocks.values().collect()
    }

    pub fn count(&self) -> usize {
        self.blocks.len()
    }

    pub fn blocks_by_status(&self, status: BlockStatus) -> Vec<&CanonicalBlock> {
        self.blocks.values().filter(|b| b.status == status).collect()
    }

    pub fn blocks_by_tier(&self, tier: BlockTier) -> Vec<&CanonicalBlock> {
        self.blocks.values().filter(|b| b.tier == tier).collect()
    }

    pub fn search(&self, query: &str) -> Vec<&CanonicalBlock> {
        let q = query.to_lowercase();
        self.blocks.values().filter(|b| {
            b.name.to_lowercase().contains(&q)
                || b.intent.to_lowercase().contains(&q)
                || b.tags.iter().any(|t| t.contains(&q))
        }).collect()
    }

    pub fn to_json(&self) -> Value {
        let blocks: Vec<Value> = self.blocks.values().map(|b| {
            json!({
                "id": b.id,
                "name": b.name,
                "version": b.version,
                "tier": format!("{:?}", b.tier),
                "status": format!("{:?}", b.status),
                "intent": b.intent,
                "trust_score": format!("{:.3}", b.trust.score()),
                "executions": b.trust.evidence.production_runs,
                "tags": b.tags,
                "lineage": {
                    "origin": format!("{:?}", b.lineage.origin),
                    "promoted_from": b.lineage.promoted_from,
                    "children": b.lineage.children.len(),
                },
            })
        }).collect();
        json!({
            "total": self.blocks.len(),
            "official": self.blocks_by_status(BlockStatus::Official).len(),
            "production": self.blocks_by_status(BlockStatus::Production).len(),
            "blocks": blocks,
        })
    }

    fn save(&self) {
        if let Some(ref path) = self.file_path {
            let blocks: Vec<&CanonicalBlock> = self.blocks.values().collect();
            if let Ok(data) = serde_json::to_string_pretty(&blocks) {
                let _ = std::fs::write(path, data);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trust::*;

    #[test]
    fn test_registry_add_and_search() {
        let mut reg = BlockRegistry::new();

        let evidence = EvidencePack {
            production_runs: 500,
            production_errors: 2,
            p95_latency_ms: 8.0,
            contexts_used: 3,
            tests_passed: 10,
            tests_total: 10,
            security_issues: 0,
            known_failures: vec![],
            last_run: 0,
        };
        let gates = TrustGates::new_clean();
        let trust = TrustProfile::from_evidence(&evidence, gates);

        let block = CanonicalBlock {
            id: "blk_test_001".into(),
            name: "customer-sync".into(),
            version: "1.0.0".into(),
            hash: "abc123".into(),
            tier: BlockTier::Composed,
            status: BlockStatus::Production,
            intent: "Sync customer data to Stripe".into(),
            cronus_text: "entity Customer { }".into(),
            trust,
            lineage: Lineage::new_promoted("stripe.scriptcronus"),
            tags: vec!["customer".into(), "stripe".into(), "sync".into()],
        };

        reg.add_block(block);
        assert_eq!(reg.count(), 1);
        assert!(reg.has_block("customer-sync"));

        let results = reg.search("stripe");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "customer-sync");

        let json = reg.to_json();
        assert_eq!(json["total"], 1);
    }
}
