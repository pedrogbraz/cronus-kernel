//! Trust Engine — computable trust for CRONUS blocks and scripts.
//!
//! Trust = binary gates (any fail = 0) × weighted metrics (real production data).
//! A block doesn't become official because someone says it works.
//! It becomes official because the data proves it.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;

// ── Global metrics tracker (in-memory, flushed to brain DB periodically) ──

static METRICS: std::sync::LazyLock<Mutex<HashMap<String, BlockMetrics>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Record an execution for trust tracking.
/// Called from fire_scripts, execute_endpoint, execute_webhook.
pub fn track_execution(block_id: &str, latency_ms: f64, error: Option<&str>) {
    if let Ok(mut map) = METRICS.lock() {
        let metrics = map
            .entry(block_id.to_string())
            .or_insert_with(|| BlockMetrics::new(block_id));
        metrics.record_execution(latency_ms, error);
    }
}

/// Get current metrics for a block.
pub fn get_metrics(block_id: &str) -> Option<BlockMetrics> {
    METRICS.lock().ok()?.get(block_id).cloned()
}

/// Get all tracked metrics.
pub fn all_metrics() -> Vec<BlockMetrics> {
    METRICS
        .lock()
        .ok()
        .map(|m| m.values().cloned().collect())
        .unwrap_or_default()
}

/// Compute trust profile for a block from its metrics.
pub fn compute_trust(block_id: &str) -> Option<TrustProfile> {
    let metrics = get_metrics(block_id)?;
    let evidence = metrics.to_evidence();
    let gates = TrustGates::new_clean(); // gates checked separately
    Some(TrustProfile::from_evidence(&evidence, gates))
}

// ── Evidence Pack ──

/// Production evidence that a block/script carries.
/// This is what makes CRONUS blocks different from npm packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePack {
    /// How many times this block executed in production
    pub production_runs: u64,
    /// How many of those executions had errors
    pub production_errors: u64,
    /// p95 latency in milliseconds
    pub p95_latency_ms: f64,
    /// Number of different apps/contexts that used this block
    pub contexts_used: u32,
    /// Number of conformance tests that pass
    pub tests_passed: u32,
    /// Total conformance tests
    pub tests_total: u32,
    /// Known security issues (0 = clean)
    pub security_issues: u32,
    /// Known failure modes documented
    pub known_failures: Vec<String>,
    /// Timestamp of last execution
    pub last_run: u64,
}

impl EvidencePack {
    pub fn new() -> Self {
        Self {
            production_runs: 0,
            production_errors: 0,
            p95_latency_ms: 0.0,
            contexts_used: 0,
            tests_passed: 0,
            tests_total: 0,
            security_issues: 0,
            known_failures: Vec::new(),
            last_run: 0,
        }
    }

    pub fn error_rate(&self) -> f64 {
        if self.production_runs == 0 {
            return 1.0;
        }
        self.production_errors as f64 / self.production_runs as f64
    }

    pub fn test_pass_rate(&self) -> f64 {
        if self.tests_total == 0 {
            return 0.0;
        }
        self.tests_passed as f64 / self.tests_total as f64
    }

    pub fn to_json(&self) -> Value {
        json!({
            "production_runs": self.production_runs,
            "production_errors": self.production_errors,
            "error_rate": format!("{:.4}", self.error_rate()),
            "p95_latency_ms": self.p95_latency_ms,
            "contexts_used": self.contexts_used,
            "test_pass_rate": format!("{:.2}", self.test_pass_rate()),
            "security_issues": self.security_issues,
            "known_failures": self.known_failures.len(),
        })
    }
}

// ── Trust Gates ──

/// Binary gates — any single failure zeroes the entire trust score.
/// This prevents a fast-but-insecure block from compensating with performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustGates {
    /// Does the block's contract (input/output schema) validate?
    pub contract_valid: bool,
    /// Has the block ever violated owner isolation?
    pub isolation_clean: bool,
    /// Are all side effects declared in the contract?
    pub effects_declared: bool,
    /// Are all dependencies free of known CVEs?
    pub deps_clean: bool,
}

impl TrustGates {
    pub fn all_pass(&self) -> bool {
        self.contract_valid && self.isolation_clean && self.effects_declared && self.deps_clean
    }

    pub fn failing_gates(&self) -> Vec<&'static str> {
        let mut failures = Vec::new();
        if !self.contract_valid {
            failures.push("contract_invalid");
        }
        if !self.isolation_clean {
            failures.push("isolation_violation");
        }
        if !self.effects_declared {
            failures.push("undeclared_side_effects");
        }
        if !self.deps_clean {
            failures.push("dependency_cve");
        }
        failures
    }

    /// Default: all gates pass (for new blocks)
    pub fn new_clean() -> Self {
        Self {
            contract_valid: true,
            isolation_clean: true,
            effects_declared: true,
            deps_clean: true,
        }
    }
}

// ── Trust Profile ──

/// Weighted trust score computed from evidence + gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustProfile {
    pub gates: TrustGates,
    pub evidence: EvidencePack,

    // Weighted axes (0.0 to 1.0 each)
    pub correctness: f64,
    pub reliability: f64,
    pub security: f64,
    pub performance: f64,
    pub reusability: f64,
    pub observability: f64,
}

impl TrustProfile {
    /// Compute final trust score.
    /// Gates are binary: any failure = 0.0.
    /// Weights only apply if ALL gates pass.
    pub fn score(&self) -> f64 {
        if !self.gates.all_pass() {
            return 0.0;
        }

        self.correctness * 0.25
            + self.reliability * 0.20
            + self.security * 0.20
            + self.performance * 0.15
            + self.reusability * 0.10
            + self.observability * 0.10
    }

    /// Compute trust from evidence data.
    pub fn from_evidence(evidence: &EvidencePack, gates: TrustGates) -> Self {
        let correctness = evidence.test_pass_rate();

        let reliability = if evidence.production_runs > 0 {
            1.0 - evidence.error_rate()
        } else {
            0.0
        };

        let security = if evidence.security_issues == 0 {
            1.0
        } else {
            0.0
        };

        let performance = if evidence.p95_latency_ms <= 0.0 {
            0.0
        } else {
            (1.0 - (evidence.p95_latency_ms / 500.0).min(1.0)).max(0.0)
        };

        let reusability = (evidence.contexts_used as f64 / 10.0).min(1.0);

        // Observability: has evidence at all?
        let observability = if evidence.production_runs > 100 {
            1.0
        } else if evidence.production_runs > 10 {
            0.5
        } else {
            0.1
        };

        Self {
            gates,
            evidence: evidence.clone(),
            correctness,
            reliability,
            security,
            performance,
            reusability,
            observability,
        }
    }

    /// Status based on score thresholds.
    pub fn status(&self) -> TrustStatus {
        let s = self.score();
        if s < 0.3 {
            TrustStatus::Sandbox
        } else if s < 0.6 {
            TrustStatus::Approved
        } else if s < 0.8 {
            TrustStatus::Production
        } else {
            TrustStatus::Official
        }
    }

    /// Can this block be promoted from .scriptcronus to .cronus?
    pub fn promotable(&self) -> bool {
        self.score() >= 0.6 && self.gates.all_pass() && self.evidence.production_runs >= 100
    }

    /// Can this block be shared in the global registry?
    pub fn official(&self) -> bool {
        self.score() >= 0.8 && self.gates.all_pass() && self.evidence.production_runs >= 1000
    }

    pub fn to_json(&self) -> Value {
        let score = self.score();
        let status = self.status();
        json!({
            "score": format!("{:.3}", score),
            "status": format!("{:?}", status),
            "promotable": self.promotable(),
            "official": self.official(),
            "gates": {
                "all_pass": self.gates.all_pass(),
                "failures": self.gates.failing_gates(),
            },
            "axes": {
                "correctness": format!("{:.3}", self.correctness),
                "reliability": format!("{:.3}", self.reliability),
                "security": format!("{:.3}", self.security),
                "performance": format!("{:.3}", self.performance),
                "reusability": format!("{:.3}", self.reusability),
                "observability": format!("{:.3}", self.observability),
            },
            "evidence": self.evidence.to_json(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrustStatus {
    /// score < 0.3 — sandbox only, testing
    Sandbox,
    /// 0.3 - 0.6 — approved for staging
    Approved,
    /// 0.6 - 0.8 — production, can be promoted to .cronus
    Production,
    /// 0.8 - 1.0 — official, can be shared in registry
    Official,
}

// ── Lineage ──

/// Evolutionary tree — every block knows where it came from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    /// How this block was created
    pub origin: BlockOrigin,
    /// Parent block (if derived/promoted)
    pub parent: Option<String>,
    /// Blocks derived from this one
    pub children: Vec<String>,
    /// Apps that use this block
    pub apps_used_in: Vec<String>,
    /// Original .scriptcronus file (if promoted)
    pub promoted_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockOrigin {
    /// Written manually
    Manual,
    /// Promoted from .scriptcronus
    Promoted,
    /// Extracted from a running app (distilled)
    Distilled,
    /// Composed from other blocks
    Composed,
    /// Imported from Hydra block library
    HydraImport,
}

impl Lineage {
    pub fn new_manual() -> Self {
        Self {
            origin: BlockOrigin::Manual,
            parent: None,
            children: Vec::new(),
            apps_used_in: Vec::new(),
            promoted_from: None,
        }
    }

    pub fn new_promoted(script_path: &str) -> Self {
        Self {
            origin: BlockOrigin::Promoted,
            parent: None,
            children: Vec::new(),
            apps_used_in: Vec::new(),
            promoted_from: Some(script_path.to_string()),
        }
    }
}

// ── Block Metrics (what gets stored in Brain DB) ──

/// Per-block metrics tracked during execution.
/// Stored in _block_metrics table in Brain DB.
#[derive(Debug, Clone)]
pub struct BlockMetrics {
    pub block_id: String,
    pub executions: u64,
    pub errors: u64,
    pub total_latency_ms: f64,
    pub max_latency_ms: f64,
    pub contexts: Vec<String>,
    pub last_error: Option<String>,
}

impl BlockMetrics {
    pub fn new(block_id: &str) -> Self {
        Self {
            block_id: block_id.to_string(),
            executions: 0,
            errors: 0,
            total_latency_ms: 0.0,
            max_latency_ms: 0.0,
            contexts: Vec::new(),
            last_error: None,
        }
    }

    pub fn record_execution(&mut self, latency_ms: f64, error: Option<&str>) {
        self.executions += 1;
        self.total_latency_ms += latency_ms;
        if latency_ms > self.max_latency_ms {
            self.max_latency_ms = latency_ms;
        }
        if let Some(err) = error {
            self.errors += 1;
            self.last_error = Some(err.to_string());
        }
    }

    pub fn add_context(&mut self, context: &str) {
        if !self.contexts.contains(&context.to_string()) {
            self.contexts.push(context.to_string());
        }
    }

    pub fn avg_latency_ms(&self) -> f64 {
        if self.executions == 0 {
            return 0.0;
        }
        self.total_latency_ms / self.executions as f64
    }

    /// Convert to EvidencePack for trust computation.
    pub fn to_evidence(&self) -> EvidencePack {
        EvidencePack {
            production_runs: self.executions,
            production_errors: self.errors,
            p95_latency_ms: self.max_latency_ms * 0.95, // approximation
            contexts_used: self.contexts.len() as u32,
            tests_passed: 0,
            tests_total: 0,
            security_issues: 0,
            known_failures: self.last_error.iter().cloned().collect(),
            last_run: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_gates_block() {
        let mut gates = TrustGates::new_clean();
        let evidence = EvidencePack {
            production_runs: 5000,
            production_errors: 10,
            p95_latency_ms: 15.0,
            contexts_used: 8,
            tests_passed: 50,
            tests_total: 50,
            security_issues: 0,
            known_failures: vec![],
            last_run: 0,
        };
        let trust = TrustProfile::from_evidence(&evidence, gates.clone());
        assert!(
            trust.score() > 0.8,
            "Clean block should be Official, got {}",
            trust.score()
        );
        assert!(trust.official());

        // Now break a gate
        gates.contract_valid = false;
        let trust_broken = TrustProfile::from_evidence(&evidence, gates);
        assert_eq!(trust_broken.score(), 0.0, "Broken gate should zero score");
        assert!(!trust_broken.promotable());
    }

    #[test]
    fn test_trust_thresholds() {
        let gates = TrustGates::new_clean();

        // New block with no evidence
        let empty = EvidencePack::new();
        let trust = TrustProfile::from_evidence(&empty, gates.clone());
        assert_eq!(trust.status(), TrustStatus::Sandbox);

        // Block with some runs
        let mid = EvidencePack {
            production_runs: 500,
            production_errors: 50,
            p95_latency_ms: 100.0,
            contexts_used: 3,
            tests_passed: 8,
            tests_total: 10,
            security_issues: 0,
            known_failures: vec![],
            last_run: 0,
        };
        let trust_mid = TrustProfile::from_evidence(&mid, gates.clone());
        assert!(
            trust_mid.score() > 0.3,
            "Mid block should be at least Approved"
        );
    }

    #[test]
    fn test_lineage() {
        let lineage = Lineage::new_promoted("integrations.scriptcronus");
        assert!(matches!(lineage.origin, BlockOrigin::Promoted));
        assert_eq!(
            lineage.promoted_from,
            Some("integrations.scriptcronus".to_string())
        );
    }
}
