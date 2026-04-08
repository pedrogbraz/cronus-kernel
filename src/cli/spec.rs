use std::fs;

pub fn cmd_spec(args: &[String]) {
    let subcmd = args.get(2).map(|s| s.as_str()).unwrap_or("help");
    match subcmd {
        "validate" => spec_validate(args),
        "list" => spec_list(args),
        "codegen" => {
            if args.iter().any(|a| a == "--structs") {
                spec_codegen_structs(args);
            } else if args.iter().any(|a| a == "--docs") {
                spec_codegen_docs(args);
            } else if args.iter().any(|a| a == "--ai-protocol") {
                spec_codegen_ai_protocol(args);
            } else {
                println!("  Usage: cronus spec codegen <--structs|--docs|--ai-protocol>");
            }
        }
        _ => {
            println!("  Usage: cronus spec <validate|list|codegen>");
        }
    }
}

fn extract_field(content: &str, field_name: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", field_name))
            || trimmed.starts_with(&format!("{}=", field_name))
        {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return Some(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

fn has_section(content: &str, section: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == section || trimmed.starts_with(&format!("{}.", &section[..section.len() - 1])) {
            return true;
        }
    }
    false
}

fn validate_spec_content(content: &str, _subdir: &str) -> Vec<String> {
    let mut errors = Vec::new();

    // 1. Must have [meta] with name, version, layer, stability
    if !has_section(content, "[meta]") {
        errors.push("missing [meta] section".to_string());
        return errors;
    }

    let name = extract_field(content, "name");
    let version = extract_field(content, "version");
    let layer = extract_field(content, "layer");
    let stability = extract_field(content, "stability");

    if name.is_none() {
        errors.push("missing meta field: name".to_string());
    }
    if version.is_none() {
        errors.push("missing meta field: version".to_string());
    }
    if layer.is_none() {
        errors.push("missing meta field: layer".to_string());
    }
    if stability.is_none() {
        errors.push("missing meta field: stability".to_string());
    }

    // 2. layer must be core, stdlib, or pattern
    if let Some(ref l) = layer {
        if !["core", "stdlib", "pattern"].contains(&l.as_str()) {
            errors.push(format!("invalid layer '{}' (expected core|stdlib|pattern)", l));
        }
    }

    // 3. stability must be draft, experimental, stable, or deprecated
    if let Some(ref s) = stability {
        if !["draft", "experimental", "stable", "deprecated"].contains(&s.as_str()) {
            errors.push(format!("invalid stability '{}' (expected draft|experimental|stable|deprecated)", s));
        }
    }

    // 4. If layer is "pattern": must have [alias] with canonical
    if let Some(ref l) = layer {
        if l == "pattern" {
            if !has_section(content, "[alias]") {
                errors.push("pattern spec must have [alias] section".to_string());
            } else {
                let canonical = extract_field(content, "canonical");
                if canonical.is_none() {
                    errors.push("pattern spec [alias] must have 'canonical' field".to_string());
                }

                // 5. If alias canonical != "none": must NOT have [keys.structural]
                if let Some(ref c) = canonical {
                    if c != "none" && has_section(content, "[keys.structural") {
                        errors.push("alias with canonical != 'none' must not have [keys.structural]".to_string());
                    }
                }
            }
        }
    }

    // 6. If NOT an alias: should have [shape] section
    let is_alias = layer.as_deref() == Some("pattern")
        && extract_field(content, "canonical").map(|c| c != "none").unwrap_or(false);
    if !is_alias && layer.as_deref() != Some("core") {
        if !has_section(content, "[shape]") {
            errors.push("non-alias spec should have [shape] section".to_string());
        }
    }

    // 7. version should match semver X.Y.Z
    if let Some(ref v) = version {
        let parts: Vec<&str> = v.split('.').collect();
        let valid = parts.len() == 3 && parts.iter().all(|p| p.parse::<u32>().is_ok());
        if !valid {
            errors.push(format!("version '{}' is not valid semver (expected X.Y.Z)", v));
        }
    }

    errors
}

// -- Codegen helpers --

fn extract_structural_keys(content: &str) -> Vec<(String, bool)> {
    let mut keys = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        // Match [keys.structural.KEYNAME]
        if trimmed.starts_with("[keys.structural.") && trimmed.ends_with(']') {
            let key_name = &trimmed[17..trimmed.len() - 1];
            // Look ahead for required = true/false
            let required = content
                .lines()
                .skip_while(|l| l.trim() != trimmed)
                .skip(1)
                .take_while(|l| !l.trim().starts_with('['))
                .any(|l| {
                    let t = l.trim();
                    t.starts_with("required") && t.contains("true")
                });
            keys.push((key_name.to_string(), required));
        }
    }
    keys
}

fn extract_config_keys(content: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut in_config = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[config]" {
            in_config = true;
            continue;
        }
        if trimmed.starts_with('[') && trimmed != "[config]" {
            if in_config {
                break;
            }
            continue;
        }
        if in_config && !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                if !key.is_empty() {
                    keys.push(key.to_string());
                }
            }
        }
    }
    keys
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn capitalize_layer(s: &str) -> String {
    match s {
        "core" => "Core".to_string(),
        "stdlib" => "Stdlib".to_string(),
        "pattern" => "Pattern".to_string(),
        _ => capitalize_first(s),
    }
}

fn capitalize_stability(s: &str) -> String {
    match s {
        "stable" => "Stable".to_string(),
        "experimental" => "Experimental".to_string(),
        "internal" => "Internal".to_string(),
        "deprecated" => "Deprecated".to_string(),
        _ => capitalize_first(s),
    }
}

fn capitalize_fallback(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "warn" => "Warn".to_string(),
        "error" => "Error".to_string(),
        "ignore" => "Ignore".to_string(),
        _ => capitalize_first(s),
    }
}

fn extract_field_bool(content: &str, field_name: &str) -> Option<bool> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", field_name))
            || trimmed.starts_with(&format!("{}=", field_name))
        {
            if trimmed.contains("true") {
                return Some(true);
            }
            if trimmed.contains("false") {
                return Some(false);
            }
        }
    }
    None
}

fn extract_field_usize(content: &str, field_name: &str) -> Option<usize> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", field_name))
            || trimmed.starts_with(&format!("{}=", field_name))
        {
            if let Some(eq_pos) = trimmed.find('=') {
                let val = trimmed[eq_pos + 1..].trim().trim_matches('"');
                if let Ok(n) = val.parse::<usize>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

// -- Codegen: --structs --

fn spec_codegen_structs(args: &[String]) {
    let dir = args.iter().position(|a| a == "--dir").and_then(|i| args.get(i + 1))
        .map(|s| s.as_str()).unwrap_or("specs");

    let write_to_file = args.iter().any(|a| a == "--write");

    let mut output = String::new();

    macro_rules! out {
        ($($arg:tt)*) => {
            use std::fmt::Write as _;
            let _ = writeln!(output, $($arg)*);
        };
    }

    out!("// Auto-generated by `cronus spec codegen --structs`");
    out!("// Do not edit manually. Edit .spec.toml files instead.");
    out!("// Source: {}/", dir);
    out!("");
    out!("use crate::contracts::{{SectionContract, KeyDef, Fallback, Layer, Stability}};");
    out!("");

    let mut all_names: Vec<String> = Vec::new();
    let mut all_consts: Vec<String> = Vec::new();
    let mut alias_map: Vec<(String, String)> = Vec::new();

    for subdir in &["stdlib", "patterns"] {
        let path = format!("{}/{}", dir, subdir);
        let mut entries: Vec<_> = match std::fs::read_dir(&path) {
            Ok(e) => e.flatten().collect(),
            Err(_) => continue,
        };
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let file_path = entry.path();
            if !file_path.extension().map(|e| e == "toml").unwrap_or(false) {
                continue;
            }
            let content = std::fs::read_to_string(&file_path).unwrap_or_default();
            let name = match extract_field(&content, "name") {
                Some(n) => n,
                None => continue,
            };

            // Check if this is a pure alias (has [alias] with canonical != "none")
            if has_section(&content, "[alias]") {
                let canonical = extract_field(&content, "canonical").unwrap_or_default();
                if canonical != "none" && !canonical.is_empty() {
                    alias_map.push((name, canonical));
                    continue;
                }
            }

            let layer = extract_field(&content, "layer").unwrap_or_else(|| "stdlib".to_string());
            let stability = extract_field(&content, "stability").unwrap_or_else(|| "stable".to_string());
            let requires_title = extract_field_bool(&content, "requires_title").unwrap_or(false);
            let requires_items = extract_field_bool(&content, "requires_items").unwrap_or(false);
            let min_items = extract_field_usize(&content, "min_items").unwrap_or(0);

            let entity_binding = has_section(&content, "[keys.entity_bound]")
                && extract_field(&content, "strategy")
                    .map(|s| s == "entity_fields" || s == "column_names")
                    .unwrap_or(false);

            let structural_keys = extract_structural_keys(&content);
            let config_keys = extract_config_keys(&content);

            let on_unknown = extract_field(&content, "on_unknown_structural_key").unwrap_or_else(|| "warn".to_string());
            let on_missing = extract_field(&content, "on_missing_required_key").unwrap_or_else(|| "error".to_string());

            let const_name = name.to_uppercase().replace('-', "_");

            let struct_keys_str = structural_keys
                .iter()
                .map(|(k, req)| {
                    if *req {
                        format!("req(\"{}\")", k)
                    } else {
                        format!("opt(\"{}\")", k)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            let config_keys_str = config_keys
                .iter()
                .map(|k| format!("\"{}\"", k))
                .collect::<Vec<_>>()
                .join(", ");

            out!("pub static {}_CONTRACT: SectionContract = SectionContract {{", const_name);
            out!("    name: \"{}\",", name);
            out!("    layer: Layer::{},", capitalize_layer(&layer));
            out!("    stability: Stability::{},", capitalize_stability(&stability));
            out!("    requires_title: {},", requires_title);
            out!("    requires_items: {},", requires_items);
            out!("    min_items: {},", min_items);
            out!("    structural_keys: &[{}],", struct_keys_str);
            out!("    entity_binding: {},", entity_binding);
            out!("    on_unknown_key: Fallback::{},", capitalize_fallback(&on_unknown));
            out!("    on_missing_required: Fallback::{},", capitalize_fallback(&on_missing));
            out!("    config_keys: &[{}],", config_keys_str);
            out!("}};");
            out!("");

            all_names.push(name.clone());
            all_consts.push(const_name);
        }
    }

    // Generate helpers
    out!("// -- Helpers --");
    out!("");
    out!("const fn req(name: &'static str) -> KeyDef {{ KeyDef {{ name, required: true }} }}");
    out!("const fn opt(name: &'static str) -> KeyDef {{ KeyDef {{ name, required: false }} }}");
    out!("");

    // Generate registry
    out!("// -- Registry --");
    out!("");
    out!("pub static GENERATED_CONTRACTS: &[&SectionContract] = &[");
    for c in &all_consts {
        out!("    &{}_CONTRACT,", c);
    }
    out!("];");
    out!("");
    out!("pub static GENERATED_NAMES: &[&str] = &[");
    for names_chunk in all_names.chunks(5) {
        let line = names_chunk.iter().map(|n| format!("\"{}\"", n)).collect::<Vec<_>>().join(", ");
        out!("    {},", line);
    }
    out!("];");
    out!("");

    // Generate alias resolver
    if !alias_map.is_empty() {
        out!("// -- Aliases --");
        out!("");
        out!("pub fn generated_resolve_alias(name: &str) -> Option<&'static str> {{");
        out!("    match name {{");
        for (alias, canonical) in &alias_map {
            out!("        \"{}\" => Some(\"{}\"),", alias, canonical);
        }
        out!("        _ => None,");
        out!("    }}");
        out!("}}");
    } else {
        out!("pub fn generated_resolve_alias(_name: &str) -> Option<&'static str> {{ None }}");
    }

    if write_to_file {
        // Determine output path: use --out <path> if given, else default to src/contracts_generated.rs
        let out_path = args.iter().position(|a| a == "--out")
            .and_then(|i| args.get(i + 1))
            .map(|s| std::path::PathBuf::from(s))
            .unwrap_or_else(|| {
                std::path::PathBuf::from("src/contracts_generated.rs")
            });
        match fs::write(&out_path, &output) {
            Ok(_) => println!("Written {} contracts to {}", all_consts.len(), out_path.display()),
            Err(e) => eprintln!("Error writing {}: {}", out_path.display(), e),
        }
    } else {
        print!("{}", output);
    }
}

// -- Codegen: --docs --

fn extract_field_after_section(content: &str, section: &str, field_name: &str) -> Option<String> {
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == section;
            continue;
        }
        if in_section {
            if trimmed.starts_with(&format!("{} =", field_name))
                || trimmed.starts_with(&format!("{}=", field_name))
            {
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed[start + 1..].find('"') {
                        return Some(trimmed[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_key_description(content: &str, key_name: &str) -> Option<String> {
    let header = format!("[keys.structural.{}]", key_name);
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            in_section = true;
            continue;
        }
        if trimmed.starts_with('[') && in_section {
            break;
        }
        if in_section
            && (trimmed.starts_with("description =") || trimmed.starts_with("description="))
        {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return Some(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

fn extract_key_type(content: &str, key_name: &str) -> String {
    let header = format!("[keys.structural.{}]", key_name);
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            in_section = true;
            continue;
        }
        if trimmed.starts_with('[') && in_section {
            break;
        }
        if in_section && (trimmed.starts_with("type =") || trimmed.starts_with("type=")) {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return trimmed[start + 1..start + 1 + end].to_string();
                }
            }
        }
    }
    "string".to_string()
}

fn extract_config_details(content: &str) -> Vec<(String, String, String, String)> {
    // Returns (key, type, default, description)
    let mut results = Vec::new();
    let mut in_config = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[config]" {
            in_config = true;
            continue;
        }
        if trimmed.starts_with('[') && trimmed != "[config]" {
            if in_config {
                break;
            }
            continue;
        }
        if in_config && !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_string();
                let rest = trimmed[eq_pos + 1..].trim();
                // Parse inline table: { type = "...", description = "...", default = "..." }
                let mut typ = "string".to_string();
                let mut default = String::new();
                let mut desc = String::new();
                if rest.starts_with('{') {
                    // Extract type
                    if let Some(t_start) = rest.find("type") {
                        let after = &rest[t_start..];
                        if let Some(q1) = after.find('"') {
                            if let Some(q2) = after[q1 + 1..].find('"') {
                                typ = after[q1 + 1..q1 + 1 + q2].to_string();
                            }
                        }
                    }
                    // Extract default
                    if let Some(d_start) = rest.find("default") {
                        let after = &rest[d_start..];
                        if let Some(q1) = after.find('"') {
                            if let Some(q2) = after[q1 + 1..].find('"') {
                                default = after[q1 + 1..q1 + 1 + q2].to_string();
                            }
                        }
                    }
                    // Extract description
                    if let Some(d_start) = rest.find("description") {
                        let after = &rest[d_start..];
                        if let Some(q1) = after.find('"') {
                            if let Some(q2) = after[q1 + 1..].find('"') {
                                desc = after[q1 + 1..q1 + 1 + q2].to_string();
                            }
                        }
                    }
                }
                results.push((key, typ, default, desc));
            }
        }
    }
    results
}

fn spec_codegen_docs(args: &[String]) {
    let dir = args.iter().position(|a| a == "--dir").and_then(|i| args.get(i + 1))
        .map(|s| s.as_str()).unwrap_or("specs");

    println!("# Section Types Reference (Auto-Generated)");
    println!();
    println!("> Generated from .spec.toml files. Do not edit manually.");
    println!();

    for subdir in &["stdlib", "patterns"] {
        println!("## {}", if *subdir == "stdlib" { "Standard Library" } else { "Patterns" });
        println!();

        let path = format!("{}/{}", dir, subdir);
        let mut entries: Vec<_> = match std::fs::read_dir(&path) {
            Ok(e) => e.flatten().collect(),
            Err(_) => continue,
        };
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let file_path = entry.path();
            if !file_path.extension().map(|e| e == "toml").unwrap_or(false) {
                continue;
            }
            let content = std::fs::read_to_string(&file_path).unwrap_or_default();
            let name = match extract_field(&content, "name") {
                Some(n) => n,
                None => continue,
            };
            let stability = extract_field(&content, "stability").unwrap_or_else(|| "stable".to_string());
            let layer = extract_field(&content, "layer").unwrap_or_else(|| "stdlib".to_string());
            let renderer = extract_field(&content, "renderer").unwrap_or_else(|| "generic".to_string());
            let description = extract_field(&content, "description").unwrap_or_default();
            let tags = extract_tags(&content);

            // Check if pure alias
            let is_alias = has_section(&content, "[alias]")
                && extract_field(&content, "canonical")
                    .map(|c| c != "none" && !c.is_empty())
                    .unwrap_or(false);

            println!("### {}", name);
            println!();
            println!("> **Status:** {} | **Layer:** {} | **Renderer:** {}", stability, layer, renderer);
            if !tags.is_empty() {
                println!("> **Tags:** {}", tags.join(", "));
            }
            if is_alias {
                let canonical = extract_field(&content, "canonical").unwrap_or_default();
                println!("> **Alias of:** {}", canonical);
            }
            println!();
            if !description.is_empty() {
                println!("{}", description);
                println!();
            }

            // Skip detailed sections for pure aliases
            if is_alias {
                println!("---");
                println!();
                continue;
            }

            // Structural keys
            let structural_keys = extract_structural_keys(&content);
            if !structural_keys.is_empty() {
                println!("**Structural keys:**");
                println!();
                println!("| Key | Required | Type | Description |");
                println!("|-----|----------|------|-------------|");
                for (key, required) in &structural_keys {
                    let typ = extract_key_type(&content, key);
                    let desc = extract_key_description(&content, key).unwrap_or_default();
                    println!("| `{}` | {} | {} | {} |", key, if *required { "yes" } else { "no" }, typ, desc);
                }
                println!();
            }

            // Config keys
            let config_details = extract_config_details(&content);
            if !config_details.is_empty() {
                println!("**Config keys:**");
                println!();
                println!("| Key | Type | Default | Description |");
                println!("|-----|------|---------|-------------|");
                for (key, typ, default, desc) in &config_details {
                    let def_display = if default.is_empty() { "-".to_string() } else { format!("`{}`", default) };
                    println!("| `{}` | {} | {} | {} |", key, typ, def_display, desc);
                }
                println!();
            }

            // Shape info
            let requires_title = extract_field_bool(&content, "requires_title").unwrap_or(false);
            let requires_items = extract_field_bool(&content, "requires_items").unwrap_or(false);
            let min_items = extract_field_usize(&content, "min_items").unwrap_or(0);
            println!("**Shape:** title={}, items={}, min_items={}",
                if requires_title { "required" } else { "optional" },
                if requires_items { "required" } else { "optional" },
                min_items);
            println!();

            // Example
            println!("**Example:**");
            println!();
            println!("```cronus");
            if structural_keys.is_empty() {
                println!("section {} {{", name);
                println!("  title \"Example\"");
                println!("}}");
            } else {
                println!("section {} {{", name);
                let required_keys: Vec<_> = structural_keys.iter().filter(|(_, r)| *r).collect();
                if required_keys.is_empty() {
                    println!("  item {{");
                    if let Some((first_key, _)) = structural_keys.first() {
                        println!("    {} \"value\"", first_key);
                    }
                    println!("  }}");
                } else {
                    println!("  item {{");
                    for (key, _) in &required_keys {
                        println!("    {} \"value\"", key);
                    }
                    println!("  }}");
                }
                println!("}}");
            }
            println!("```");
            println!();
            println!("---");
            println!();
        }
    }
}

// -- Codegen: --ai-protocol --

fn extract_array_field(content: &str, field_name: &str) -> Vec<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(field_name) && trimmed.contains('=') {
            if let Some(bracket_start) = trimmed.find('[') {
                if let Some(bracket_end) = trimmed.find(']') {
                    let inner = &trimmed[bracket_start + 1..bracket_end];
                    return inner
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
    }
    Vec::new()
}

fn extract_aliases_from_meta(content: &str) -> Vec<String> {
    extract_array_field(content, "aliases")
}

fn extract_tags(content: &str) -> Vec<String> {
    extract_array_field(content, "tags")
}

fn extract_config_validates(content: &str, key_name: &str) -> Option<String> {
    let mut in_config = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[config]" {
            in_config = true;
            continue;
        }
        if trimmed.starts_with('[') && trimmed != "[config]" {
            if in_config { break; }
            continue;
        }
        if in_config && trimmed.starts_with(key_name) {
            if let Some(v_start) = trimmed.find("validates") {
                let after = &trimmed[v_start..];
                if let Some(q1) = after.find('"') {
                    if let Some(q2) = after[q1 + 1..].find('"') {
                        return Some(after[q1 + 1..q1 + 1 + q2].to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_key_default(content: &str, key_name: &str) -> Option<String> {
    let header = format!("[keys.structural.{}]", key_name);
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            in_section = true;
            continue;
        }
        if trimmed.starts_with('[') && in_section { break; }
        if in_section && (trimmed.starts_with("default =") || trimmed.starts_with("default=")) {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return Some(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

fn spec_codegen_ai_protocol(args: &[String]) {
    let dir = args.iter().position(|a| a == "--dir").and_then(|i| args.get(i + 1))
        .map(|s| s.as_str()).unwrap_or("specs");

    let output_path = args.iter().position(|a| a == "-o").and_then(|i| args.get(i + 1))
        .map(|s| s.as_str()).unwrap_or("cronus-schema.json");

    let mut sections = Vec::new();
    let mut section_count = 0usize;

    for subdir in &["core", "stdlib", "patterns"] {
        let path = format!("{}/{}", dir, subdir);
        let mut entries: Vec<_> = match std::fs::read_dir(&path) {
            Ok(e) => e.flatten().collect(),
            Err(_) => continue,
        };
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let file_path = entry.path();
            if !file_path.extension().map(|e| e == "toml").unwrap_or(false) {
                continue;
            }
            let content = std::fs::read_to_string(&file_path).unwrap_or_default();
            let name = match extract_field(&content, "name") {
                Some(n) => n,
                None => continue,
            };

            let layer = extract_field(&content, "layer").unwrap_or_else(|| subdir.to_string());
            let stability = extract_field(&content, "stability").unwrap_or_else(|| "stable".to_string());
            let description = extract_field(&content, "description").unwrap_or_default();
            let intent_desc = extract_field_after_section(&content, "[intent]", "description")
                .unwrap_or_else(|| description.clone());

            // Check if pure alias
            let is_alias = has_section(&content, "[alias]")
                && extract_field(&content, "canonical")
                    .map(|c| c != "none" && !c.is_empty())
                    .unwrap_or(false);

            if is_alias {
                let canonical = extract_field(&content, "canonical").unwrap_or_default();
                let justification = extract_field(&content, "justification").unwrap_or_default();
                sections.push(format!(
                    "    \"{}\": {{\n      \"alias_of\": \"{}\",\n      \"layer\": \"{}\",\n      \"stability\": \"{}\",\n      \"justification\": \"{}\"}}",
                    json_escape(&name), json_escape(&canonical), json_escape(&layer), json_escape(&stability), json_escape(&justification)
                ));
                section_count += 1;
                continue;
            }

            let requires_title = extract_field_bool(&content, "requires_title").unwrap_or(false);
            let requires_items = extract_field_bool(&content, "requires_items").unwrap_or(false);
            let min_items = extract_field_usize(&content, "min_items").unwrap_or(0);

            let entity_binding = has_section(&content, "[keys.entity_bound]")
                && extract_field(&content, "strategy")
                    .map(|s| s == "entity_fields" || s == "column_names")
                    .unwrap_or(false);

            let aliases = extract_aliases_from_meta(&content);

            // Structural keys with full detail
            let structural_keys = extract_structural_keys(&content);
            let mut sk_entries = Vec::new();
            for (key, required) in &structural_keys {
                let typ = extract_key_type(&content, key);
                let desc = extract_key_description(&content, key).unwrap_or_default();
                let mut fields = vec![
                    format!("\"required\": {}", required),
                    format!("\"type\": \"{}\"", json_escape(&typ)),
                    format!("\"description\": \"{}\"", json_escape(&desc)),
                ];
                if let Some(def) = extract_key_default(&content, key) {
                    fields.push(format!("\"default\": \"{}\"", json_escape(&def)));
                }
                sk_entries.push(format!("        \"{}\": {{ {} }}", json_escape(key), fields.join(", ")));
            }

            // Config keys with full detail
            let config_details = extract_config_details(&content);
            let mut ck_entries = Vec::new();
            for (key, typ, default, desc) in &config_details {
                let mut fields = vec![
                    format!("\"type\": \"{}\"", json_escape(typ)),
                ];
                if !desc.is_empty() {
                    fields.push(format!("\"description\": \"{}\"", json_escape(desc)));
                }
                if !default.is_empty() {
                    fields.push(format!("\"default\": \"{}\"", json_escape(default)));
                }
                if let Some(validates) = extract_config_validates(&content, key) {
                    fields.push(format!("\"validates\": \"{}\"", json_escape(&validates)));
                }
                ck_entries.push(format!("        \"{}\": {{ {} }}", json_escape(key), fields.join(", ")));
            }

            let aliases_json = if aliases.is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", aliases.iter().map(|a| format!("\"{}\"", json_escape(a))).collect::<Vec<_>>().join(", "))
            };

            let section_json = format!(
                "    \"{name}\": {{\n      \"layer\": \"{layer}\",\n      \"stability\": \"{stability}\",\n      \"intent\": \"{intent}\",\n      \"shape\": {{\n        \"requires_title\": {rt},\n        \"requires_items\": {ri},\n        \"min_items\": {mi}\n      }},\n      \"structural_keys\": {{\n{sk}\n      }},\n      \"config_keys\": {{\n{ck}\n      }},\n      \"entity_binding\": {eb},\n      \"aliases\": {al}}}",
                name = json_escape(&name),
                layer = json_escape(&layer),
                stability = json_escape(&stability),
                intent = json_escape(&intent_desc),
                rt = requires_title,
                ri = requires_items,
                mi = min_items,
                sk = sk_entries.join(",\n"),
                ck = ck_entries.join(",\n"),
                eb = entity_binding,
                al = aliases_json
            );
            sections.push(section_json);
            section_count += 1;
        }
    }

    let field_types = vec![
        "string", "text", "email", "url", "slug", "phone", "number", "money",
        "percentage", "boolean", "date", "ulid", "json", "enum", "ip", "relation",
    ];
    let field_modifiers = vec![
        "required", "unique", "sensitive", "optional", "searchable",
        "index", "featured", "formatted", "array",
    ];
    let action_verbs = vec![
        "set", "toast", "navigate", "refresh", "create", "confirm", "delete", "validate", "open", "close",
    ];
    let action_events = vec!["click", "submit", "error", "change"];
    let auth_modes = vec!["jwt", "cookie", "token", "public", "api_key", "internal"];
    let page_types = vec!["custom", "form", "list", "dashboard", "landing", "detail"];

    let ft_json = field_types.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ");
    let fm_json = field_modifiers.iter().map(|m| format!("\"{}\"", m)).collect::<Vec<_>>().join(", ");
    let av_json = action_verbs.iter().map(|v| format!("\"{}\"", v)).collect::<Vec<_>>().join(", ");
    let ae_json = action_events.iter().map(|e| format!("\"{}\"", e)).collect::<Vec<_>>().join(", ");
    let am_json = auth_modes.iter().map(|m| format!("\"{}\"", m)).collect::<Vec<_>>().join(", ");
    let pt_json = page_types.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ");

    // Compute today's date without external crate
    let today = {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let total_days = now / 86400;
        let mut y = 1970i64;
        let mut remaining = total_days as i64;
        loop {
            let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
            let days_in_year: i64 = if leap { 366 } else { 365 };
            if remaining < days_in_year { break; }
            remaining -= days_in_year;
            y += 1;
        }
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let month_days: [i64; 12] = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut m = 0usize;
        for (i, &md) in month_days.iter().enumerate() {
            if remaining < md { m = i + 1; break; }
            remaining -= md;
        }
        if m == 0 { m = 12; }
        let d = remaining + 1;
        format!("{:04}-{:02}-{:02}", y, m, d)
    };

    let schema = format!(
        "{{\n  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",\n  \"title\": \"CRONUS Language Schema\",\n  \"description\": \"Schema for validating .cronus file structure, generated from {} .spec.toml contracts\",\n  \"version\": \"1.0.0\",\n  \"generated_at\": \"{}\",\n  \"sections\": {{\n{}\n  }},\n  \"field_types\": [{}],\n  \"field_modifiers\": [{}],\n  \"action_verbs\": [{}],\n  \"action_events\": [{}],\n  \"auth_modes\": [{}],\n  \"page_types\": [{}]\n}}\n",
        section_count,
        today,
        sections.join(",\n"),
        ft_json, fm_json, av_json, ae_json, am_json, pt_json
    );

    match fs::write(output_path, &schema) {
        Ok(_) => {
            println!("  \x1b[32m✓\x1b[0m Generated AI protocol schema: {} sections, {} field types, {} action verbs",
                section_count, field_types.len(), action_verbs.len());
            println!("  \x1b[36m→\x1b[0m {}", output_path);
        }
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Failed to write {}: {}", output_path, e);
        }
    }
}

fn spec_validate(args: &[String]) {
    let dir = args.get(3).map(|s| s.as_str()).unwrap_or("specs");
    println!("  \x1b[36m⚡\x1b[0m Validating specs in {}/\n", dir);

    let mut passed = 0;
    let mut failed = 0;

    for subdir in &["core", "stdlib", "patterns"] {
        let path = format!("{}/{}", dir, subdir);
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.extension().map(|e| e == "toml").unwrap_or(false) {
                    let content = std::fs::read_to_string(&file_path).unwrap_or_default();
                    let errors = validate_spec_content(&content, subdir);
                    if errors.is_empty() {
                        passed += 1;
                        println!("  \x1b[32m✓\x1b[0m {}", file_path.display());
                    } else {
                        failed += 1;
                        println!("  \x1b[31m✗\x1b[0m {}", file_path.display());
                        for err in &errors {
                            println!("    → {}", err);
                        }
                    }
                }
            }
        }
    }

    println!("\n  {} passed, {} failed", passed, failed);
    if failed > 0 {
        std::process::exit(1);
    }
}

fn spec_list(args: &[String]) {
    let dir = args.get(3).map(|s| s.as_str()).unwrap_or("specs");
    println!("  \x1b[36m⚡\x1b[0m Specs in {}/\n", dir);

    for subdir in &["core", "stdlib", "patterns"] {
        let path = format!("{}/{}", dir, subdir);
        if let Ok(entries) = std::fs::read_dir(&path) {
            println!("  \x1b[1m{}:\x1b[0m", subdir);
            let mut files: Vec<_> = entries.flatten().collect();
            files.sort_by_key(|e| e.file_name());
            for entry in files {
                let file_path = entry.path();
                if file_path.extension().map(|e| e == "toml").unwrap_or(false) {
                    let content = std::fs::read_to_string(&file_path).unwrap_or_default();
                    let name = extract_field(&content, "name").unwrap_or_default();
                    let stability = extract_field(&content, "stability").unwrap_or_default();
                    let badge = match stability.as_str() {
                        "stable" => "\x1b[32m●\x1b[0m",
                        "experimental" => "\x1b[33m●\x1b[0m",
                        "deprecated" => "\x1b[31m●\x1b[0m",
                        _ => "\x1b[90m●\x1b[0m",
                    };
                    println!("    {} {} ({})", badge, name, stability);
                }
            }
            println!();
        }
    }
}
