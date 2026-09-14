#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Service Orchestrator
//!
//! Manages multiple services declared in .cronus files.
//! Each service runs in its own tokio task with its own port.
//! Includes health checks, inter-service communication, and basic circuit breaker.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

// ══════════════════════════════════════════════════
// SERVICE CONFIG
// ══════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub port: u16,
    pub service_type: ServiceType,
    pub entities: Vec<String>,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceType {
    Api,
    Auth,
    Worker,
    Frontend,
    Gateway,
}

impl ServiceType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "auth" => ServiceType::Auth,
            "worker" => ServiceType::Worker,
            "frontend" => ServiceType::Frontend,
            "gateway" => ServiceType::Gateway,
            _ => ServiceType::Api,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ServiceType::Api => "api",
            ServiceType::Auth => "auth",
            ServiceType::Worker => "worker",
            ServiceType::Frontend => "frontend",
            ServiceType::Gateway => "gateway",
        }
    }
}

// ══════════════════════════════════════════════════
// CIRCUIT BREAKER (basic)
// ══════════════════════════════════════════════════

struct CircuitBreaker {
    failures: AtomicU32,
    open: AtomicBool,
    threshold: u32,
}

impl CircuitBreaker {
    fn new(threshold: u32) -> Self {
        CircuitBreaker {
            failures: AtomicU32::new(0),
            open: AtomicBool::new(false),
            threshold,
        }
    }

    fn record_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
        self.open.store(false, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        let fails = self.failures.fetch_add(1, Ordering::Relaxed) + 1;
        if fails >= self.threshold {
            self.open.store(true, Ordering::Relaxed);
        }
    }

    fn is_open(&self) -> bool {
        self.open.load(Ordering::Relaxed)
    }

    fn failure_count(&self) -> u32 {
        self.failures.load(Ordering::Relaxed)
    }
}

// ══════════════════════════════════════════════════
// SERVICE REGISTRY (runtime state)
// ══════════════════════════════════════════════════

struct ServiceEntry {
    config: ServiceConfig,
    healthy: AtomicBool,
    circuit: CircuitBreaker,
}

// ══════════════════════════════════════════════════
// ORCHESTRATOR
// ══════════════════════════════════════════════════

pub struct ServiceOrchestrator {
    services: Vec<Arc<ServiceEntry>>,
}

impl ServiceOrchestrator {
    /// Create orchestrator from parsed service configs.
    pub fn new(configs: Vec<ServiceConfig>) -> Self {
        let services = configs
            .into_iter()
            .map(|config| {
                Arc::new(ServiceEntry {
                    config,
                    healthy: AtomicBool::new(false),
                    circuit: CircuitBreaker::new(5), // 5 failures → circuit open
                })
            })
            .collect();
        ServiceOrchestrator { services }
    }

    /// Start all services. Each gets its own tokio task.
    /// Returns handles that can be awaited.
    pub async fn start_all(&self) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::new();

        for entry in &self.services {
            let entry = Arc::clone(entry);
            let handle = tokio::spawn(async move {
                let name = &entry.config.name;
                let port = entry.config.port;
                let stype = entry.config.service_type.as_str();

                eprintln!(
                    "  \x1b[32m✓\x1b[0m Service '{}' ({}) starting on port {}",
                    name, stype, port
                );

                // Mark as healthy once started
                entry.healthy.store(true, Ordering::Relaxed);

                // Keep alive — in a real impl, each service type would have its own server loop
                // For now, we just hold the task open
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                }
            });
            handles.push(handle);
        }

        handles
    }

    /// Health check all services. Returns vec of (name, healthy, failures).
    pub fn health_check(&self) -> Vec<(String, bool, u32)> {
        self.services
            .iter()
            .map(|entry| {
                (
                    entry.config.name.clone(),
                    entry.healthy.load(Ordering::Relaxed),
                    entry.circuit.failure_count(),
                )
            })
            .collect()
    }

    /// Get health as JSON (for /api/health endpoint).
    pub fn health_json(&self) -> Value {
        let services: Vec<Value> = self
            .services
            .iter()
            .map(|entry| {
                let healthy = entry.healthy.load(Ordering::Relaxed);
                let circuit_open = entry.circuit.is_open();
                json!({
                    "name": entry.config.name,
                    "type": entry.config.service_type.as_str(),
                    "port": entry.config.port,
                    "healthy": healthy,
                    "circuit_breaker": if circuit_open { "open" } else { "closed" },
                    "failures": entry.circuit.failure_count(),
                    "entities": entry.config.entities,
                })
            })
            .collect();

        let all_healthy = self
            .services
            .iter()
            .all(|e| e.healthy.load(Ordering::Relaxed));

        json!({
            "status": if all_healthy { "healthy" } else { "degraded" },
            "services": services,
            "total": self.services.len(),
        })
    }

    /// Inter-service HTTP call with circuit breaker.
    pub async fn call_service(&self, name: &str, path: &str) -> Result<String, String> {
        let entry = self
            .services
            .iter()
            .find(|e| e.config.name == name)
            .ok_or_else(|| format!("Service '{}' not found", name))?;

        // Check circuit breaker
        if entry.circuit.is_open() {
            return Err(format!("Circuit breaker open for service '{}'", name));
        }

        let url = format!("http://127.0.0.1:{}{}", entry.config.port, path);

        // Simple HTTP GET via TCP (no external HTTP client needed)
        match tokio::time::timeout(tokio::time::Duration::from_secs(5), tcp_get(&url)).await {
            Ok(Ok(body)) => {
                entry.circuit.record_success();
                Ok(body)
            }
            Ok(Err(e)) => {
                entry.circuit.record_failure();
                Err(format!("Service '{}' error: {}", name, e))
            }
            Err(_) => {
                entry.circuit.record_failure();
                Err(format!("Service '{}' timeout", name))
            }
        }
    }

    /// List all registered services.
    pub fn list_services(&self) -> Vec<Value> {
        self.services
            .iter()
            .map(|e| {
                json!({
                    "name": e.config.name,
                    "type": e.config.service_type.as_str(),
                    "port": e.config.port,
                    "entities": e.config.entities,
                })
            })
            .collect()
    }
}

// ══════════════════════════════════════════════════
// Simple TCP-based HTTP GET (no hyper client needed)
// ══════════════════════════════════════════════════

async fn tcp_get(url: &str) -> Result<String, String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    // Parse url: http://host:port/path
    let url = url.strip_prefix("http://").unwrap_or(url);
    let (host_port, path) = url.split_once('/').unwrap_or((url, ""));
    let path = format!("/{}", path);

    let mut stream = TcpStream::connect(host_port)
        .await
        .map_err(|e| format!("connect: {}", e))?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, host_port
    );
    stream
        .write_all(request.as_bytes())
        .await
        .map_err(|e| format!("write: {}", e))?;

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .await
        .map_err(|e| format!("read: {}", e))?;

    let response = String::from_utf8_lossy(&buf);

    // Extract body (after \r\n\r\n)
    if let Some(pos) = response.find("\r\n\r\n") {
        Ok(response[pos + 4..].to_string())
    } else {
        Ok(response.to_string())
    }
}
