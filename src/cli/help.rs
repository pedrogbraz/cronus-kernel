/// Commands shown by `cronus help`, grouped by who needs them.
/// Every verb dispatched in `main.rs` belongs to exactly one group.
pub(crate) const HELP_GROUPS: &[(&str, &[(&str, &str)])] = &[
    (
        "Essentials",
        &[
            (
                "new <name> [--template t]",
                "Create a project (templates: see `cronus new --list`)",
            ),
            (
                "run [port] [--host ip] [--prod]",
                "Serve the app with hot reload (127.0.0.1 by default)",
            ),
            (
                "build [--ai]",
                "Validate the .cronus file (--ai: JSON errors with fix hints)",
            ),
            ("doctor", "Health check: required checks + suggestions"),
            (
                "test [port]",
                "Run auto-generated CRUD tests against a running server",
            ),
            ("seed [count]", "Insert fake rows (default: 10 per entity)"),
            (
                "deploy",
                "Generate deploy artifacts (--fly, --railway, --static)",
            ),
            ("version", "Show version"),
        ],
    ),
    (
        "AI",
        &[
            (
                "generate <desc> [-o file] [--force]",
                "Generate .cronus from a description",
            ),
            (
                "context [--for-claude]",
                "Export project context for an AI assistant",
            ),
            (
                "dump <path>",
                "Convert HTML / Next.js / Prisma / OpenAPI into .cronus",
            ),
            (
                "mcp",
                "MCP server on stdio: validate/parse/context tools for AI clients",
            ),
            ("brief", "Short AI context capsule"),
            (
                "validate [file] [--json]",
                "Validate with contract checks (--strict-ai: warnings = errors)",
            ),
        ],
    ),
    (
        "Advanced",
        &[
            ("parse <file>", "Show the AST"),
            ("debug [port]", "Run with request tracing"),
            ("compose", "Merge all .cronus files and show the result"),
            ("export", "Export to cronus-project.ir.json"),
            ("stats", "Project stats"),
            ("graph", "Entity/page graph"),
            ("audit language|logic|visual|all", "Cronus Audit"),
            ("verify-audit", "Verify audit trail hash chain"),
            ("reconcile <a> <b>", "AST-level merge of two .cronus files"),
            ("spec validate|list|codegen", ".spec.toml tooling"),
            ("status | timeline | changelog", "Semantic project history"),
            (
                "sync | handoff | lease | drift",
                "Multi-agent session state",
            ),
            ("review | segment | memory", "Multi-agent review and memory"),
        ],
    ),
];

pub fn print_help() {
    println!();
    println!(
        "  \x1b[36m\x1b[1mCRONUS\x1b[0m \x1b[90mdeclarative full-stack language (v{})\x1b[0m",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("  \x1b[1mUsage:\x1b[0m cronus <command> [options]");
    println!();
    println!("  \x1b[1mQuick start:\x1b[0m cronus new my-app && cd my-app && cronus run");
    for (group, commands) in HELP_GROUPS {
        println!();
        println!("  \x1b[1m{}\x1b[0m", group);
        for (usage, about) in *commands {
            println!("    \x1b[32m{:<38}\x1b[0m {}", usage, about);
        }
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verbs(group: &str) -> Vec<String> {
        HELP_GROUPS
            .iter()
            .find(|(g, _)| *g == group)
            .map(|(_, cmds)| {
                cmds.iter()
                    .flat_map(|(usage, _)| {
                        usage
                            .split(|c: char| c == '|' || c.is_whitespace())
                            .filter(|w| {
                                !w.is_empty()
                                    && w.chars().all(|c| c.is_ascii_lowercase() || c == '-')
                            })
                            .map(str::to_string)
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn help_groups_essentials_ai_advanced() {
        let names: Vec<&str> = HELP_GROUPS.iter().map(|(g, _)| *g).collect();
        assert_eq!(names, ["Essentials", "AI", "Advanced"]);
        let essentials = verbs("Essentials");
        for v in ["new", "run", "build", "doctor"] {
            assert!(
                essentials.iter().any(|e| e == v),
                "{} missing from Essentials",
                v
            );
        }
    }

    #[test]
    fn help_lists_dump_and_context() {
        let ai = verbs("AI");
        assert!(ai.iter().any(|v| v == "dump"));
        assert!(ai.iter().any(|v| v == "context"));
        assert!(ai.iter().any(|v| v == "mcp"));
    }

    #[test]
    fn internal_verbs_live_under_advanced_only() {
        let advanced = verbs("Advanced");
        for v in ["lease", "drift", "segment", "memory", "handoff", "sync"] {
            assert!(advanced.iter().any(|a| a == v), "{} not in Advanced", v);
            assert!(!verbs("Essentials").iter().any(|a| a == v));
        }
    }
}
