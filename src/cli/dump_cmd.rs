use std::fs;
use crate::dump;

pub fn cmd_dump(args: &[String]) {
    let file = args.get(2).unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m Usage: cronus dump <file.html|.json|.prisma|dir/> [-o output.cronus]");
        std::process::exit(1);
    });

    // Black Hole mode: dump entire project directory
    let path = std::path::Path::new(file);
    if path.is_dir() {
        let output = dump::project::dump_project(path);

        // Determine output file name
        let out_file = args.iter().position(|a| a == "-o")
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

    eprintln!("  \x1b[36m⚡\x1b[0m Dumping {} ({} bytes)...", file, html.len());

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
    let output_file = args.iter().position(|a| a == "-o").and_then(|i| args.get(i + 1));
    if let Some(out) = output_file {
        fs::write(out, &cronus).unwrap_or_else(|e| {
            eprintln!("  \x1b[31m✗\x1b[0m Error writing {}: {}", out, e);
            std::process::exit(1);
        });
        eprintln!("  \x1b[32m✓\x1b[0m Written to {}", out);
    } else {
        println!("{}", cronus);
    }
}

