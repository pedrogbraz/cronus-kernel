use crate::dump;
use std::fs;

pub fn cmd_dump(args: &[String]) {
    let audit_mode = args.iter().any(|a| a == "--audit");
    let nextjs_mode = args.iter().any(|a| a == "--nextjs");

    let file = args.iter().skip(2)
        .find(|a| !a.starts_with("--"))
        .unwrap_or_else(|| {
            eprintln!("  \x1b[31m✗\x1b[0m Usage: cronus dump <file.html|.json|.prisma|dir/> [-o output.cronus] [--audit] [--nextjs]");
            std::process::exit(1);
        });

    // Black Hole mode: dump entire project directory
    let path = std::path::Path::new(file);
    if path.is_dir() {
        // Auto-detect Next.js or use --nextjs flag
        let is_nextjs = nextjs_mode
            || path.join("next.config.ts").exists()
            || path.join("next.config.mjs").exists()
            || path.join("next.config.js").exists()
            || {
                let pkg = path.join("package.json");
                pkg.exists()
                    && fs::read_to_string(&pkg)
                        .map(|c| c.contains("\"next\"") || c.contains("\"vinext\""))
                        .unwrap_or(false)
            };

        let output = if is_nextjs {
            eprintln!("  \x1b[36m⚡\x1b[0m Detected Next.js/VINEXT project");
            dump::nextjs::dump_nextjs(path)
        } else {
            dump::project::dump_project(path)
        };

        // Determine output file name
        let out_file = args
            .iter()
            .position(|a| a == "-o")
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                format!("{}.cronus", name)
            });

        fs::write(&out_file, &output).expect("Failed to write output");
        eprintln!("  \x1b[32m✓\x1b[0m Written to {}", out_file);
        return;
    }

    let html = fs::read_to_string(file).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Error reading {}: {}", file, e);
        std::process::exit(1);
    });

    eprintln!(
        "  \x1b[36m⚡\x1b[0m Dumping {} ({} bytes)...",
        file,
        html.len()
    );

    // Detect file format
    let cronus = if file.ends_with(".prisma") {
        eprintln!("  \x1b[36m⚡\x1b[0m Detected Prisma schema");
        dump::prisma::dump_prisma(&html)
    } else if file.ends_with(".json") {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&html) {
            if parsed.get("openapi").is_some() || parsed.get("swagger").is_some() {
                eprintln!("  \x1b[36m⚡\x1b[0m Detected OpenAPI spec");
                dump::openapi::dump_openapi(&html)
            } else {
                dump::dump_html(&html)
            }
        } else {
            dump::dump_html(&html)
        }
    } else {
        dump::dump_html(&html)
    };

    // Check for -o flag
    let output_file = args
        .iter()
        .position(|a| a == "-o")
        .and_then(|i| args.get(i + 1));
    if let Some(out) = output_file {
        fs::write(out, &cronus).unwrap_or_else(|e| {
            eprintln!("  \x1b[31m✗\x1b[0m Error writing {}: {}", out, e);
            std::process::exit(1);
        });
        eprintln!("  \x1b[32m✓\x1b[0m Written to {}", out);
    } else {
        println!("{}", cronus);
    }

    // ── Post-dump audit: compare dumped .cronus output against original HTML ──
    if audit_mode && file.ends_with(".html") {
        eprintln!();
        eprintln!("  \x1b[1mPost-Dump Fidelity Audit\x1b[0m");
        eprintln!("  Reference: {}", file);

        // Parse the just-generated .cronus output and audit against original HTML
        match crate::parser::parse(&cronus) {
            Ok(nodes) => {
                match crate::cli::audit_fidelity::run_audit_from_nodes(&html, &nodes) {
                    Some(result) => {
                        crate::cli::audit_fidelity::print_fidelity_line(&result);
                        if result.fidelity < 95 {
                            crate::cli::audit_fidelity::print_missing_top(&result, 10);
                        }
                        // Save results
                        crate::cli::audit_fidelity::save_audit_results(&result);
                        eprintln!(
                            "  \x1b[32m✓\x1b[0m Audit results saved to .cronus/audit-results.json"
                        );
                    }
                    None => {
                        eprintln!("  \x1b[33m⚠\x1b[0m Could not run audit comparison");
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "  \x1b[33m⚠\x1b[0m Dumped .cronus has parse errors, skipping audit: {}",
                    e
                );
            }
        }
    } else if audit_mode {
        eprintln!("  \x1b[33m⚠\x1b[0m --audit only works with HTML input files");
    }
}
