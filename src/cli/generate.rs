use std::fs;
use crate::parser;

const GENERATE_SYSTEM_PROMPT: &str = r#"You generate .cronus files — a declarative full-stack language. One file = database + REST API + GraphQL + Auth + UI + SSR. Output ONLY valid .cronus code, no markdown.

RULES:
1. Start with app block, then auth (if needed), style, entities, api, layout, pages
2. Field syntax: `name type! modifiers` — the `!` means required
3. 16 types: string text email url slug phone number money percentage boolean date ulid json enum ip
4. Relations: `field -> OtherEntity`
5. Money = centavos (2990 = R$29.90)
6. Data sections MUST have `bind Entity { query ... }`
7. NEVER add id, created_at, updated_at — auto-generated
8. NEVER put sensitive fields in columns
9. Forms MUST have `on submit` with action

EXAMPLE 1 — Task Manager:
app "TaskFlow" {
  stack react + tailwind
  port 5175
  database sqlite "./data.db"
}

auth {
  entity User
  login email + password
  session jwt expires:24h
  roles [admin, user]
}

style {
  theme dark
  accent blue
  font "Inter"
}

entity User {
  name     string!
  email    email!   unique
  password string!  sensitive
  role     enum [admin, user]
}

entity Project {
  name   string!  searchable
  slug   slug!    unique
  owner  -> User
}

entity Task {
  title    string!  searchable
  body     text
  status   enum [todo, doing, done]
  priority enum [low, medium, high, urgent]
  project  -> Project
  assignee -> User

  transition status {
    todo -> doing
    doing -> done | todo
    done -> todo
  }
}

api /projects {
  list   GET    /     auth:jwt
  create POST   /     auth:jwt
  detail GET    /:id  auth:jwt
}

api /tasks {
  list   GET    /     auth:jwt
  create POST   /     auth:jwt
  detail GET    /:id  auth:jwt
  update PATCH  /:id  auth:jwt
  delete DELETE /:id  auth:role(admin)
}

layout Main {
  brand "TaskFlow"
  sidebar {
    "Dashboard" -> "/" icon:dashboard
    "Projects"  -> "/projects" icon:folder
    "Tasks"     -> "/tasks" icon:task_alt
  }
}

page "/" type:dashboard requires:auth {
  section kpi {
    bind Task { aggregate count }
    item "Total Tasks" value:bind icon:task_alt
  }
  section kpi {
    bind Task { aggregate count where status eq:"doing" }
    item "In Progress" value:bind icon:pending
  }
  section table {
    bind Task { query all order created_at desc limit 20 }
    columns "title, status, priority, assignee, created_at"
    search true
  }
}

page "/tasks" requires:auth {
  section table {
    bind Task { query all order priority desc }
    columns "title, status, priority, project, assignee"
    search true
  }
}

page "/tasks/new" requires:auth {
  section form {
    bind Task
    on submit {
      create Task
      toast "Task created" success
      navigate "/tasks"
    }
  }
}

page "/tasks/:id" type:detail requires:auth {
  section card {
    bind Task { query one where id eq:route.id }
  }
}

page "/projects" requires:auth {
  section table {
    bind Project { query all order name asc }
    columns "name, slug, owner"
    search true
  }
}

EXAMPLE 2 — E-Commerce:
app "Shop" {
  stack react + tailwind
  port 5175
  database sqlite "./data.db"
  constitution {
    must "prices in centavos"
    never "expose customer addresses in API lists"
  }
}

auth {
  entity User
  login email + password
  session jwt expires:24h
  roles [admin, user]
}

style {
  theme light
  accent emerald
  font "Inter"
}

entity User {
  name     string!
  email    email!   unique
  password string!  sensitive
  role     enum [admin, user]
}

entity Category {
  name string! searchable
  slug slug!   unique
}

entity Product {
  name     string!   searchable
  slug     slug!     unique
  price    money!
  stock    number    default:0
  category -> Category
  active   boolean   default:true
}

entity Order {
  customer -> User
  total    money!
  status   enum [cart, pending, paid, shipped, delivered, cancelled]

  transition status {
    cart -> pending
    pending -> paid | cancelled
    paid -> shipped
    shipped -> delivered
  }

  on create {
    log "Order #{{id}} placed"
  }
}

entity OrderItem {
  order    -> Order
  product  -> Product
  quantity number!  min:1
  price    money!
}

api /products {
  list   GET    /     auth:public
  create POST   /     auth:role(admin)
  detail GET    /:id  auth:public
  update PATCH  /:id  auth:role(admin)
}

api /orders {
  list   GET    /     auth:jwt
  create POST   /     auth:jwt
  detail GET    /:id  auth:jwt
  update PATCH  /:id  auth:jwt
}

layout Main {
  brand "Shop"
  sidebar {
    "Dashboard" -> "/" icon:dashboard
    "Products"  -> "/products" icon:inventory
    "Orders"    -> "/orders" icon:receipt
    "Categories" -> "/categories" icon:category
  }
}

page "/" type:dashboard requires:auth {
  section kpi {
    bind Product { aggregate count }
    item "Products" value:bind icon:inventory
  }
  section kpi {
    bind Order { aggregate count where status eq:"paid" }
    item "Paid Orders" value:bind icon:receipt
  }
  section kpi {
    bind Order { aggregate sum field:total }
    item "Revenue" value:bind icon:attach_money
  }
  section chart {
    bind Order { aggregate sum field:total group_by:created_at interval:month }
    chart_type bar
  }
  section table {
    bind Order { query all order created_at desc limit 10 }
    columns "customer, total, status, created_at"
  }
}

page "/products" requires:auth {
  section table {
    bind Product { query all order name asc }
    columns "name, price, stock, category, active"
    search true
  }
}

page "/products/new" requires:auth {
  section form {
    bind Product
    on submit {
      create Product
      toast "Product added" success
      navigate "/products"
    }
  }
}

page "/orders" requires:auth {
  section table {
    bind Order { query all order created_at desc }
    columns "customer, total, status, created_at"
  }
}

page "/orders/:id" type:detail requires:auth {
  section card {
    bind Order { query one where id eq:route.id }
  }
}
"#;

pub fn cmd_generate(args: &[String]) {
    let mut description: Option<String> = None;
    let mut dry_run = false;
    let mut from_file: Option<String> = None;
    let mut output_path = "app.cronus".to_string();
    let mut auto_go = false;

    // Parse flags from args[2..] (args[0] = "cronus", args[1] = "generate"/"gen")
    let flag_args = if args.len() > 2 { &args[2..] } else { &[] as &[String] };
    let mut i = 0;
    while i < flag_args.len() {
        match flag_args[i].as_str() {
            "--dry-run" => { dry_run = true; }
            "--go" => { auto_go = true; }
            "--from-file" => {
                i += 1;
                if i >= flag_args.len() {
                    eprintln!("  \x1b[31m✗\x1b[0m --from-file requires a path");
                    std::process::exit(1);
                }
                from_file = Some(flag_args[i].clone());
            }
            "--output" | "-o" | "--save" => {
                i += 1;
                if i >= flag_args.len() {
                    eprintln!("  \x1b[31m✗\x1b[0m --output requires a filename");
                    std::process::exit(1);
                }
                output_path = flag_args[i].clone();
            }
            other => {
                // Positional arg = description (join remaining non-flag words)
                if description.is_none() && !other.starts_with("--") {
                    let mut desc_parts = vec![other.to_string()];
                    while i + 1 < flag_args.len() && !flag_args[i + 1].starts_with("--") {
                        i += 1;
                        desc_parts.push(flag_args[i].clone());
                    }
                    description = Some(desc_parts.join(" "));
                }
            }
        }
        i += 1;
    }

    // MODE: --from-file
    if let Some(ref file_path) = from_file {
        cmd_generate_from_file(file_path, &output_path, auto_go);
        return;
    }

    // Need a description for generate
    let desc = match description {
        Some(d) => d,
        None => {
            eprintln!("  Usage:");
            eprintln!("    cronus generate \"description\" [--dry-run] [--output file.cronus] [--save file.cronus] [--go]");
            eprintln!("    cronus generate --from-file output.txt [--output file.cronus] [--go]");
            eprintln!();
            eprintln!("  Without ANTHROPIC_API_KEY, generates from built-in templates.");
            eprintln!();
            eprintln!("  Examples:");
            eprintln!("    cronus generate \"veterinary clinic with pets owners and appointments\" --dry-run");
            eprintln!("    cronus generate --from-file chatgpt-output.txt");
            eprintln!("    cronus generate \"blog with posts and comments\"");
            std::process::exit(1);
        }
    };

    // Build the user message
    let user_message = format!("Generate a .cronus file for: {}", desc);

    // Check for API key
    let api_key = std::env::var("ANTHROPIC_API_KEY").ok().filter(|k| !k.is_empty());

    if dry_run {
        // Explicit dry-run: save prompt to file
        cmd_generate_dry_run(&desc, &user_message);
    } else if api_key.is_none() {
        // No API key: use template-based generator
        cmd_generate_template(&desc, &output_path, auto_go);
    } else {
        // API mode
        cmd_generate_api(&desc, &user_message, &api_key.unwrap(), &output_path, auto_go);
    }
}

// ── Template-based .cronus generator ──────────────────────────────────────────

struct EntityTemplate {
    keywords: &'static [&'static str],
    canonical_name: &'static str,
    fields: &'static [(&'static str, &'static str, &'static [&'static str], bool, bool)],
}

const ENTITY_TEMPLATES: &[EntityTemplate] = &[
    EntityTemplate {
        keywords: &["user", "member", "team", "person", "people", "staff", "employee"],
        canonical_name: "Member",
        fields: &[
            ("name", "string", &[], true, false),
            ("email", "email", &[], true, true),
            ("role", "enum", &["admin", "member", "viewer"], true, false),
            ("avatar", "url", &[], false, false),
        ],
    },
    EntityTemplate {
        keywords: &["task", "todo", "ticket", "issue"],
        canonical_name: "Task",
        fields: &[
            ("title", "string", &[], true, false),
            ("description", "text", &[], false, false),
            ("priority", "enum", &["low", "medium", "high"], true, false),
            ("status", "enum", &["todo", "in_progress", "done"], true, false),
            ("due_date", "date", &[], false, false),
        ],
    },
    EntityTemplate {
        keywords: &["project", "workspace", "board"],
        canonical_name: "Project",
        fields: &[
            ("name", "string", &[], true, false),
            ("description", "text", &[], false, false),
            ("status", "enum", &["active", "archived", "draft"], true, false),
        ],
    },
    EntityTemplate {
        keywords: &["product", "item", "good", "merchandise"],
        canonical_name: "Product",
        fields: &[
            ("name", "string", &[], true, false),
            ("description", "text", &[], false, false),
            ("price", "money", &[], true, false),
            ("category", "string", &[], false, false),
            ("image", "url", &[], false, false),
        ],
    },
    EntityTemplate {
        keywords: &["order", "purchase", "sale", "transaction"],
        canonical_name: "Order",
        fields: &[
            ("total", "money", &[], true, false),
            ("status", "enum", &["pending", "paid", "shipped", "delivered", "cancelled"], true, false),
        ],
    },
    EntityTemplate {
        keywords: &["post", "article", "blog", "entry"],
        canonical_name: "Post",
        fields: &[
            ("title", "string", &[], true, false),
            ("content", "text", &[], true, false),
            ("slug", "slug", &[], true, true),
            ("published", "boolean", &[], false, false),
            ("published_at", "date", &[], false, false),
        ],
    },
    EntityTemplate {
        keywords: &["comment", "reply", "feedback", "review"],
        canonical_name: "Comment",
        fields: &[
            ("content", "text", &[], true, false),
            ("author_name", "string", &[], true, false),
            ("author_email", "email", &[], false, false),
            ("approved", "boolean", &[], false, false),
        ],
    },
    EntityTemplate {
        keywords: &["event", "meeting", "appointment", "session", "booking"],
        canonical_name: "Event",
        fields: &[
            ("title", "string", &[], true, false),
            ("date", "date", &[], true, false),
            ("location", "string", &[], false, false),
            ("description", "text", &[], false, false),
        ],
    },
    EntityTemplate {
        keywords: &["message", "chat", "notification"],
        canonical_name: "Message",
        fields: &[
            ("content", "text", &[], true, false),
            ("read", "boolean", &[], false, false),
        ],
    },
    EntityTemplate {
        keywords: &["category", "tag", "label", "group"],
        canonical_name: "Category",
        fields: &[
            ("name", "string", &[], true, true),
            ("description", "text", &[], false, false),
        ],
    },
    EntityTemplate {
        keywords: &["author", "writer", "creator", "contributor"],
        canonical_name: "Author",
        fields: &[
            ("name", "string", &[], true, false),
            ("email", "email", &[], true, true),
            ("bio", "text", &[], false, false),
            ("avatar", "url", &[], false, false),
        ],
    },
    EntityTemplate {
        keywords: &["customer", "client", "buyer"],
        canonical_name: "Customer",
        fields: &[
            ("name", "string", &[], true, false),
            ("email", "email", &[], true, true),
            ("phone", "phone", &[], false, false),
        ],
    },
    EntityTemplate {
        keywords: &["invoice", "bill", "receipt"],
        canonical_name: "Invoice",
        fields: &[
            ("number", "string", &[], true, true),
            ("total", "money", &[], true, false),
            ("status", "enum", &["draft", "sent", "paid", "overdue"], true, false),
            ("due_date", "date", &[], false, false),
        ],
    },
    EntityTemplate {
        keywords: &["page", "document", "doc", "note", "wiki"],
        canonical_name: "Document",
        fields: &[
            ("title", "string", &[], true, false),
            ("content", "text", &[], true, false),
            ("slug", "slug", &[], true, true),
        ],
    },
    EntityTemplate {
        keywords: &["file", "attachment", "upload", "media", "image", "photo"],
        canonical_name: "File",
        fields: &[
            ("name", "string", &[], true, false),
            ("url", "url", &[], true, false),
            ("size", "number", &[], false, false),
        ],
    },
];

struct DetectedEntity {
    name: String,
    template_idx: Option<usize>,
    relations: Vec<String>,
}

fn parse_generate_description(desc: &str) -> (String, Vec<DetectedEntity>) {
    let lower = desc.to_lowercase();

    let tokens: Vec<&str> = lower
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '.')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let stop_words: &[&str] = &[
        "a", "an", "the", "and", "or", "with", "for", "to", "of", "in", "on", "at",
        "by", "from", "that", "this", "is", "are", "has", "have", "app", "application",
        "system", "platform", "website", "web", "site", "service", "tool", "manage",
        "management", "manager", "managing", "tracker", "tracking",
    ];

    let mut matched: Vec<(usize, String)> = Vec::new();
    let mut used_templates: Vec<usize> = Vec::new();

    for (tidx, tmpl) in ENTITY_TEMPLATES.iter().enumerate() {
        for &kw in tmpl.keywords {
            if used_templates.contains(&tidx) { break; }
            let kw_plural = format!("{}s", kw);
            let kw_plural2 = if kw.ends_with('y') {
                format!("{}ies", &kw[..kw.len()-1])
            } else if kw.ends_with('s') || kw.ends_with('x') {
                format!("{}es", kw)
            } else {
                kw_plural.clone()
            };

            for &tok in &tokens {
                if tok == kw || tok == kw_plural || tok == kw_plural2 {
                    matched.push((tidx, kw.to_string()));
                    used_templates.push(tidx);
                    break;
                }
            }
        }
    }

    let entity_names: Vec<String> = matched
        .iter()
        .map(|(idx, _)| ENTITY_TEMPLATES[*idx].canonical_name.to_string())
        .collect();

    let mut entities: Vec<DetectedEntity> = Vec::new();
    for (tidx, _kw) in &matched {
        let tmpl = &ENTITY_TEMPLATES[*tidx];
        let relations: Vec<String> = entity_names
            .iter()
            .filter(|n| n.as_str() != tmpl.canonical_name)
            .cloned()
            .collect();

        entities.push(DetectedEntity {
            name: tmpl.canonical_name.to_string(),
            template_idx: Some(*tidx),
            relations,
        });
    }

    if entities.is_empty() {
        entities.push(DetectedEntity {
            name: "Item".to_string(),
            template_idx: None,
            relations: vec![],
        });
    }

    let app_name = gen_derive_app_name(&tokens, stop_words);
    (app_name, entities)
}

fn gen_derive_app_name(tokens: &[&str], stop_words: &[&str]) -> String {
    let meaningful: Vec<&str> = tokens
        .iter()
        .filter(|t| !stop_words.contains(t) && t.len() > 2)
        .take(3)
        .copied()
        .collect();

    if meaningful.is_empty() {
        return "My App".to_string();
    }

    let titled = gen_title_case(meaningful[0]);
    if meaningful.len() == 1 {
        format!("{} App", titled)
    } else {
        let titled2 = gen_title_case(meaningful[1]);
        format!("{} {}", titled, titled2)
    }
}

fn gen_title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn gen_pluralize(name: &str) -> String {
    if name.ends_with('s') || name.ends_with('x') {
        format!("{}es", name)
    } else if name.ends_with('y') && !name.ends_with("ey") && !name.ends_with("ay") && !name.ends_with("oy") {
        format!("{}ies", &name[..name.len()-1])
    } else {
        format!("{}s", name)
    }
}

fn gen_field_title(s: &str) -> String {
    s.split('_')
        .map(|w| gen_title_case(w))
        .collect::<Vec<_>>()
        .join(" ")
}

fn gen_entity_columns(ent: &DetectedEntity) -> String {
    if let Some(tidx) = ent.template_idx {
        let tmpl = &ENTITY_TEMPLATES[tidx];
        tmpl.fields.iter()
            .take(4)
            .map(|&(name, _, _, _, _)| gen_field_title(name))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "Name, Description, Status".to_string()
    }
}

fn generate_cronus_from_entities(app_name: &str, entities: &[DetectedEntity]) -> String {
    let mut out = String::new();

    // App block
    out.push_str(&format!("app \"{}\" {{\n", app_name));
    out.push_str("  stack fullstack\n");
    out.push_str("  port 3000\n");
    out.push_str("  database sqlite \"./data.db\"\n");
    out.push_str("  theme dark\n");
    out.push_str("}\n\n");

    // Entity blocks
    for ent in entities {
        out.push_str(&format!("entity {} {{\n", ent.name));

        if let Some(tidx) = ent.template_idx {
            let tmpl = &ENTITY_TEMPLATES[tidx];
            for &(fname, ftype, enum_vals, required, unique) in tmpl.fields {
                let mut line = format!("  {} {}", fname, ftype);
                if ftype == "enum" && !enum_vals.is_empty() {
                    let vals: Vec<String> = enum_vals.iter().map(|v| format!("\"{}\"", v)).collect();
                    line.push_str(&format!(" [{}]", vals.join(", ")));
                }
                if required { line.push_str(" required"); }
                if unique { line.push_str(" unique"); }
                out.push_str(&line);
                out.push('\n');
            }
        } else {
            out.push_str("  name string required\n");
            out.push_str("  description text\n");
            out.push_str("  status enum [\"active\", \"inactive\"] required\n");
        }

        for rel in &ent.relations {
            let field_name = rel.to_lowercase();
            out.push_str(&format!("  {} -> {}\n", field_name, rel));
        }

        out.push_str("}\n\n");
    }

    // API blocks
    for ent in entities {
        let slug = ent.name.to_lowercase() + "s";
        let prefix = format!("/{}", slug);
        out.push_str(&format!("api {} {{\n", prefix));
        out.push_str(&format!("  list GET {}\n", prefix));
        out.push_str(&format!("  find GET {}/:id\n", prefix));
        out.push_str(&format!("  create POST {}\n", prefix));
        out.push_str(&format!("  update PATCH {}/:id\n", prefix));
        out.push_str(&format!("  remove DELETE {}/:id\n", prefix));
        out.push_str("}\n\n");
    }

    // Dashboard page
    let first = &entities[0];
    out.push_str("page \"/\" type:custom {\n");
    out.push_str("  section kpi cols:3 {\n");
    out.push_str(&format!("    bind {} {{ query count }}\n", first.name));
    out.push_str("  }\n");
    out.push_str("  section table style:dark {\n");
    out.push_str(&format!("    title \"Recent {}\"\n", gen_pluralize(&first.name)));
    out.push_str(&format!("    columns \"{}\"\n", gen_entity_columns(first)));
    out.push_str(&format!("    bind {} {{ query all order created_at desc limit 10 }}\n", first.name));
    out.push_str("  }\n");
    out.push_str("}\n\n");

    // List pages per entity
    for (i, ent) in entities.iter().enumerate() {
        let slug = ent.name.to_lowercase() + "s";
        out.push_str(&format!("page \"/{}\" type:custom {{\n", slug));
        out.push_str("  section table style:dark {\n");
        out.push_str(&format!("    title \"{}\"\n", gen_pluralize(&ent.name)));
        out.push_str(&format!("    columns \"{}\"\n", gen_entity_columns(ent)));
        out.push_str(&format!("    bind {} {{ query all }}\n", ent.name));
        out.push_str("  }\n");
        out.push_str("}\n");
        if i < entities.len() - 1 {
            out.push('\n');
        }
    }

    out
}

pub fn cmd_generate_template(desc: &str, output_path: &str, auto_go: bool) {
    println!("  Generating .cronus from template for: \"{}\"", desc);
    println!();

    let (app_name, entities) = parse_generate_description(desc);
    let source = generate_cronus_from_entities(&app_name, &entities);

    // Validate by parsing
    match parser::parse(&source) {
        Ok(nodes) => {
            let (ent_count, page_count, route_count) = parser::stats(&nodes);
            let lines = source.lines().count();

            println!("{}", source);

            fs::write(output_path, &source).unwrap_or_else(|e| {
                eprintln!("  \x1b[31m✗\x1b[0m Failed to write {}: {}", output_path, e);
                std::process::exit(1);
            });

            println!("  \x1b[32m✓\x1b[0m Generated \x1b[1m{}\x1b[0m", output_path);
            println!("    {} lines, {} entities, {} pages, {} routes",
                     lines, ent_count, page_count, route_count);
            println!();

            if auto_go {
                println!("  Running seed + run...");
                let _ = std::process::Command::new(std::env::current_exe().unwrap())
                    .args(&["seed", output_path])
                    .status();
                let _ = std::process::Command::new(std::env::current_exe().unwrap())
                    .args(&["run", output_path])
                    .status();
            } else {
                println!("  Next: \x1b[1mcronus run {}\x1b[0m", output_path);
            }
        }
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Generated template has parse errors:");
            eprintln!("    {}", e);
            eprintln!();
            eprintln!("--- Generated output ---");
            eprintln!("{}", source);
            eprintln!("--- End ---");
            eprintln!();
            eprintln!("  This is a bug in the template generator. Please report it.");
            std::process::exit(1);
        }
    }
}

pub fn cmd_generate_dry_run(desc: &str, user_message: &str) {
    let full_prompt = format!(
        "=== SYSTEM PROMPT ===\n{}\n\n=== USER MESSAGE ===\n{}\n",
        GENERATE_SYSTEM_PROMPT, user_message
    );

    fs::write(".cronus-prompt.txt", &full_prompt).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Failed to write .cronus-prompt.txt: {}", e);
        std::process::exit(1);
    });

    println!("  \x1b[32m✓\x1b[0m Prompt saved to \x1b[1m.cronus-prompt.txt\x1b[0m");
    println!();
    println!("  Next steps:");
    println!("    1. Copy the contents of .cronus-prompt.txt");
    println!("    2. Paste into Claude or ChatGPT");
    println!("    3. Save the output to a file (e.g. output.txt)");
    println!("    4. Run: \x1b[1mcronus generate --from-file output.txt\x1b[0m");
    println!();
    println!("  Or set ANTHROPIC_API_KEY to generate directly:");
    println!("    export ANTHROPIC_API_KEY=sk-ant-...");
    println!("    cronus generate \"{}\"", desc);
}

pub fn cmd_generate_from_file(file_path: &str, output_path: &str, auto_go: bool) {
    let raw = fs::read_to_string(file_path).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Failed to read {}: {}", file_path, e);
        std::process::exit(1);
    });

    let source = strip_markdown_fences(&raw);

    match parser::parse(&source) {
        Ok(nodes) => {
            let (entities, pages, routes) = parser::stats(&nodes);
            let lines = source.lines().count();

            fs::write(output_path, &source).unwrap_or_else(|e| {
                eprintln!("  \x1b[31m✗\x1b[0m Failed to write {}: {}", output_path, e);
                std::process::exit(1);
            });

            println!("  \x1b[32m✓\x1b[0m Valid .cronus file written to \x1b[1m{}\x1b[0m", output_path);
            println!("    {} lines, {} entities, {} pages, {} routes", lines, entities, pages, routes);
            println!();

            if auto_go {
                println!("  Running seed + run...");
                let _ = std::process::Command::new(std::env::current_exe().unwrap())
                    .args(&["seed", output_path])
                    .status();
                let _ = std::process::Command::new(std::env::current_exe().unwrap())
                    .args(&["run", output_path])
                    .status();
            } else {
                println!("  Next: \x1b[1mcronus run {}\x1b[0m", output_path);
            }
        }
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Parse error in {}:", file_path);
            eprintln!("    {}", e);
            eprintln!();
            eprintln!("  The LLM output is not valid .cronus syntax.");
            eprintln!("  Try regenerating or fix the errors manually.");
            std::process::exit(1);
        }
    }
}

pub fn cmd_generate_api(desc: &str, user_message: &str, api_key: &str, output_path: &str, auto_go: bool) {
    println!("  Generating .cronus for: \"{}\"", desc);
    println!();

    let max_retries = 3;
    let mut attempt = 0;
    let mut extra_context = String::new();

    loop {
        attempt += 1;
        if attempt > 1 {
            println!("  Retry {}/{}...", attempt, max_retries);
        }

        let messages_with_context = if extra_context.is_empty() {
            format!(r#"[{{"role":"user","content":"{}"}}]"#,
                user_message.replace('\\', "\\\\").replace('"', "\\\""))
        } else {
            format!(r#"[{{"role":"user","content":"{}"}},{{"role":"assistant","content":"{}"}},{{"role":"user","content":"{}"}}]"#,
                user_message.replace('\\', "\\\\").replace('"', "\\\""),
                "I'll generate the .cronus file now.".replace('"', "\\\""),
                extra_context.replace('\\', "\\\\").replace('"', "\\\""))
        };

        let body = format!(
            r#"{{"model":"claude-sonnet-4-20250514","max_tokens":4096,"system":"{}","messages":{}}}"#,
            GENERATE_SYSTEM_PROMPT.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"),
            messages_with_context
        );

        let output = std::process::Command::new("curl")
            .args(&[
                "-s", "-X", "POST",
                "https://api.anthropic.com/v1/messages",
                "-H", &format!("x-api-key: {}", api_key),
                "-H", "anthropic-version: 2023-06-01",
                "-H", "content-type: application/json",
                "-d", &body,
            ])
            .output();

        let output = match output {
            Ok(o) => o,
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m Failed to call API (curl): {}", e);
                std::process::exit(1);
            }
        };

        let response_str = String::from_utf8_lossy(&output.stdout).to_string();

        // Extract content[0].text from JSON response (minimal parsing, no deps)
        let generated = extract_api_text(&response_str);
        if generated.is_empty() {
            if attempt >= max_retries {
                eprintln!("  \x1b[31m✗\x1b[0m API returned no usable content after {} attempts", max_retries);
                eprintln!("  Response: {}", &response_str[..response_str.len().min(500)]);
                std::process::exit(1);
            }
            extra_context = "The previous response was empty. Please output ONLY valid .cronus code.".to_string();
            continue;
        }

        let source = strip_markdown_fences(&generated);

        match parser::parse(&source) {
            Ok(nodes) => {
                let (entities, pages, routes) = parser::stats(&nodes);
                let lines = source.lines().count();

                fs::write(output_path, &source).unwrap_or_else(|e| {
                    eprintln!("  \x1b[31m✗\x1b[0m Failed to write {}: {}", output_path, e);
                    std::process::exit(1);
                });

                println!("  \x1b[32m✓\x1b[0m Generated \x1b[1m{}\x1b[0m ({} lines)", output_path, lines);
                println!("    {} entities, {} pages, {} routes", entities, pages, routes);
                println!();

                if auto_go {
                    println!("  Running seed + run...");
                    let _ = std::process::Command::new(std::env::current_exe().unwrap())
                        .args(&["seed", output_path])
                        .status();
                    let _ = std::process::Command::new(std::env::current_exe().unwrap())
                        .args(&["run", output_path])
                        .status();
                } else {
                    println!("  Next: \x1b[1mcronus run {}\x1b[0m", output_path);
                }
                return;
            }
            Err(e) => {
                if attempt >= max_retries {
                    eprintln!("  \x1b[31m✗\x1b[0m Failed to generate valid .cronus after {} attempts", max_retries);
                    eprintln!("  Last parse error: {}", e);
                    // Save the last attempt for debugging
                    let debug_path = format!("{}.failed.txt", output_path);
                    let _ = fs::write(&debug_path, &source);
                    eprintln!("  Raw output saved to {}", debug_path);
                    std::process::exit(1);
                }
                println!("  \x1b[33m!\x1b[0m Attempt {} had parse errors, retrying...", attempt);
                extra_context = format!(
                    "Your previous output had parse errors:\n{}\n\nPlease fix these errors and output ONLY valid .cronus code.",
                    e
                );
            }
        }
    }
}

fn strip_markdown_fences(input: &str) -> String {
    let trimmed = input.trim();

    // Check for ```cronus or ``` at the start
    if trimmed.starts_with("```") {
        let after_opening = if let Some(pos) = trimmed.find('\n') {
            &trimmed[pos + 1..]
        } else {
            return trimmed.to_string();
        };
        // Remove trailing ```
        let result = if let Some(pos) = after_opening.rfind("```") {
            &after_opening[..pos]
        } else {
            after_opening
        };
        return result.trim().to_string();
    }

    trimmed.to_string()
}

fn extract_api_text(json: &str) -> String {
    // Minimal JSON parsing: find "text":" and extract the value
    // Looking for: "content":[{"type":"text","text":"..."}]
    if let Some(text_pos) = json.find("\"text\":\"") {
        let start = text_pos + 8; // skip "text":"
        let rest = &json[start..];
        let mut result = String::new();
        let mut chars = rest.chars();
        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    if let Some(escaped) = chars.next() {
                        match escaped {
                            'n' => result.push('\n'),
                            't' => result.push('\t'),
                            '"' => result.push('"'),
                            '\\' => result.push('\\'),
                            '/' => result.push('/'),
                            _ => { result.push('\\'); result.push(escaped); }
                        }
                    }
                }
                '"' => break,
                _ => result.push(c),
            }
        }
        result
    } else {
        String::new()
    }
}


fn extract_app_name(desc: &str) -> String {
    let words: Vec<&str> = desc.split_whitespace().collect();
    if words.len() <= 3 {
        return desc.split_whitespace().map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + &c.as_str().to_lowercase(),
            }
        }).collect::<Vec<_>>().join(" ");
    }
    words.iter()
        .filter(|w| !["a", "an", "the", "with", "and", "for", "my"].contains(&w.to_lowercase().as_str()))
        .take(3)
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + &c.as_str().to_lowercase(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn generate_saas_template() -> String {
    r#"# Generated by CRONUS — SaaS Template

app "MyApp" {
  stack react + tailwind
  port 5175
  database sqlite "./data.db"
  theme dark
}

style {
  theme dark
  accent blue
  background neutral-950
  radius xl
  font "SF Pro Display"
}

entity User {
  email     email     required unique
  name      string    required
  role      enum      [admin, member, viewer]
  createdAt date
}

entity Project {
  name        string    required
  description text
  status      enum      [active, paused, archived]
  owner       string    required
  createdAt   date
}

entity Task {
  name      string    required
  priority  enum      [low, medium, high, critical]
  status    enum      [todo, in_progress, done]
  createdAt date
}

component HeroMain {
  layout hero
  style premium+dark
  items [
    badge "SAAS TEMPLATE"
    title "Ship Faster With CRONUS"
    subtitle "Full-stack SaaS from a single .cronus file"
    cta "Get Started" -> "/signup" tone:primary
    cta "Dashboard" -> "/dashboard" tone:secondary
  ]
}

component MetricUsers {
  layout stack
  style card+metric
  items [
    label "Users"
    value "0"
  ]
}

component MetricProjects {
  layout stack
  style card+metric
  items [
    label "Projects"
    value "0"
  ]
}

api /auth {
  login     POST   /login     auth:public
  register  POST   /register  auth:public
  me        GET    /me        auth:jwt
}

api /projects {
  list    GET    /          auth:jwt
  create  POST   /          auth:jwt
  detail  GET    /:id       auth:jwt
  edit    PATCH  /:id       auth:jwt
  delete  DELETE /:id       auth:jwt
}

api /tasks {
  list    GET    /          auth:jwt
  create  POST   /          auth:jwt
}

page "/" type:custom {
  section hero {
    badge "SAAS TEMPLATE"
    title "Ship Faster With CRONUS"
    subtitle "Full-stack SaaS from a single .cronus file"
    cta "Get Started" -> "/signup" primary
  }
}

page "/login" type:form entity:User {
  title "Login"
  fields [email, password]
}

page "/signup" type:form entity:User {
  title "Sign Up"
  fields [name, email, password]
}

page "/dashboard" type:dashboard {
  title "Dashboard"
  use MetricUsers
  use MetricProjects
}

page "/projects" type:list entity:Project {
  title "Projects"
  columns [name, status, owner, createdAt]
}

page "/tasks" type:list entity:Task {
  title "Tasks"
  columns [name, priority, status, createdAt]
}

page "/settings" type:form entity:User {
  title "Settings"
  fields [name, email]
}
"#.to_string()
}

fn generate_landing_template() -> String {
    r###"# Generated by CRONUS — Premium Landing Page
# Style: Vercel/Linear dark theme

app "MyProduct" {
  stack react + tailwind
  port 5175
  database sqlite "./data.db"
  theme dark
}

style {
  theme dark
  accent blue
  background neutral-950
  radius xl
  font "SF Pro Display"
  mono "JetBrains Mono"
}

# ── Entity ──

entity WaitlistSubscriber {
  email     email     required unique
  name      string
  source    string
  createdAt date
}

# ── Components ──

component HeroLanding {
  layout hero
  style premium+dark
  items [
    badge "NOW IN BETA"
    title "Ship Faster. Scale Smarter."
    subtitle "The developer platform that eliminates boilerplate. Define your app in one file, deploy everywhere."
    cta "Start Building" -> "/signup" tone:primary
    cta "View Docs" -> "/docs" tone:secondary
  ]
}

component FeatureGrid {
  layout grid
  style cards+3col
  items [
    item "Zero Config" icon:zap tone:default
    item "Type Safe" icon:shield tone:default
    item "Edge Ready" icon:globe tone:default
    item "Auto API" icon:code tone:default
    item "Built-in Auth" icon:lock tone:default
    item "Real-time" icon:activity tone:default
  ]
}

component StatsRow {
  layout grid
  style stats+4col
  items [
    value "99.99%" label:"Uptime"
    value "12ms" label:"Latency"
    value "50k+" label:"Developers"
    value "2M+" label:"Requests/day"
  ]
}

component PricingPlans {
  layout grid
  style pricing+3col
  items [
    plan "Starter" price:$0/mo tone:default
    plan "Pro" price:$29/mo featured:true tone:primary
    plan "Enterprise" price:$99/mo tone:default
  ]
}

component CTABottom {
  layout hero
  style minimal+dark
  items [
    title "Ready to ship?"
    subtitle "Join the waitlist. No credit card required."
    cta "Get Early Access" -> "/signup" tone:primary
  ]
}

# ── API ──

api /waitlist {
  create  POST   /        auth:public
}

# ── Pages ──

page "/" type:custom {
  use HeroLanding
  use FeatureGrid
  use StatsRow
  use PricingPlans
  use CTABottom

  section hero {
    badge "NOW IN BETA"
    title "Ship Faster. Scale Smarter."
    subtitle "The developer platform that eliminates boilerplate. Define your app in one file, deploy everywhere."
    bullets [
      "Zero-config deployments",
      "Automatic API generation",
      "Built-in auth and database"
    ]
    cta "Start Building" -> "/signup" primary
    cta "View Docs" -> "/docs" secondary
  }

  section features cols:3 style:cards {
    item "Zero Config" icon:zap {
      "No webpack, no babel, no config files. Just write .cronus and run."
    }
    item "Type Safe" icon:shield {
      "Every entity, route, and component is validated at parse time."
    }
    item "Edge Ready" icon:globe {
      "Deploy to any edge runtime. Sub-10ms response times globally."
    }
    item "Auto API" icon:code {
      "CRUD endpoints generated automatically from your entity definitions."
    }
    item "Built-in Auth" icon:lock {
      "JWT authentication with Argon2 password hashing out of the box."
    }
    item "Real-time" icon:activity {
      "Live data updates via Server-Sent Events. No WebSocket setup."
    }
  }

  section pricing cols:3 {
    plan "Starter" $0/mo [
      "1 project",
      "SQLite database",
      "Community support",
      "Basic analytics"
    ]
    plan "Pro" $29/mo featured [
      "Unlimited projects",
      "PostgreSQL support",
      "Priority support",
      "Advanced analytics",
      "Custom domains",
      "Team collaboration"
    ]
    plan "Enterprise" $99/mo [
      "Everything in Pro",
      "SLA 99.99%",
      "Dedicated support",
      "On-premise option",
      "SSO & SAML",
      "Audit logs"
    ]
  }

  section cta {
    title "Ready to ship?"
    subtitle "Join thousands of developers building faster with CRONUS"
    cta "Get Early Access" -> "/signup" primary
    cta "Talk to Sales" -> "/contact" secondary
  }
}

page "/signup" type:form entity:WaitlistSubscriber {
  title "Join the Waitlist"
  fields [name, email]
}
"###.to_string()
}
