use crate::{auth, database, find_cronus_file, parser};
use std::fs;
use std::time::Instant;

pub fn cmd_seed(args: &[String]) {
    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found");
        std::process::exit(1);
    });

    let source = fs::read_to_string(&file).unwrap();
    let nodes = parser::parse(&source).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e);
        std::process::exit(1);
    });

    // Find app for database path
    let app = nodes.iter().find_map(|n| {
        if let parser::AstNode::App(a) = n {
            Some(a)
        } else {
            None
        }
    });
    let db_path = app
        .and_then(|a| a.database.as_ref())
        .and_then(|d| d.path.clone())
        .unwrap_or_else(|| "./data.db".into());

    let db = database::CronusDB::open(&db_path).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Cannot open database: {}", e);
        std::process::exit(1);
    });

    let entities: Vec<&parser::EntityNode> = nodes
        .iter()
        .filter_map(|n| {
            if let parser::AstNode::Entity(e) = n {
                Some(e)
            } else {
                None
            }
        })
        .collect();

    if entities.is_empty() {
        eprintln!("  \x1b[33m⊘\x1b[0m No entities found in {}", file);
        return;
    }

    let count = args
        .get(2)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    // Migrate tables first
    let entity_refs: Vec<parser::EntityNode> = entities.iter().map(|e| (*e).clone()).collect();
    db.migrate(&entity_refs).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Migration failed: {}", e);
        std::process::exit(1);
    });

    let seed_start = Instant::now();
    println!(
        "\n  Seeding {} entities \u{00d7} {} rows...\n",
        entities.len(),
        count
    );

    for entity in &entities {
        let mut seeded = 0;
        let first_names = [
            "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry", "Iris", "Jack",
            "Kate", "Leo", "Mia", "Noah", "Olivia", "Pete", "Quinn", "Rosa", "Sam", "Tina",
        ];
        let last_names = [
            "Smith",
            "Johnson",
            "Williams",
            "Brown",
            "Jones",
            "Garcia",
            "Miller",
            "Davis",
            "Rodriguez",
            "Martinez",
            "Anderson",
            "Thomas",
            "Jackson",
            "White",
            "Harris",
            "Clark",
            "Lewis",
            "Young",
            "King",
            "Wright",
        ];
        let companies = [
            "Acme Corp",
            "TechFlow",
            "DataSync",
            "CloudBase",
            "NetPrime",
            "CodeVault",
            "PixelForge",
            "ByteWave",
            "SkyStack",
            "NanoGrid",
            "Quantum Labs",
            "Apex Digital",
            "Iris Systems",
            "Bolt.io",
            "Vertex AI",
            "Nebula Inc",
            "Spark Ops",
            "Iron Cloud",
            "Pulse Dev",
            "Orbit HQ",
        ];
        let titles = [
            "Important task",
            "Follow up needed",
            "Review required",
            "New request",
            "Bug fix",
            "Feature request",
            "Documentation update",
            "Testing round",
            "Deployment prep",
            "Migration plan",
            "Security patch",
            "Performance tuning",
            "UI redesign",
            "API integration",
            "Data cleanup",
            "Onboarding flow",
            "Billing issue",
            "Support ticket",
            "Release notes",
            "Sprint planning",
        ];

        for i in 0..count {
            let mut obj = serde_json::Map::new();

            for field in &entity.fields {
                // Skip timestamp fields
                let lower = field.name.to_lowercase();
                if lower == "createdat"
                    || lower == "created_at"
                    || lower == "updatedat"
                    || lower == "updated_at"
                    || lower == "id"
                {
                    continue;
                }

                let val: serde_json::Value = match field.field_type {
                    parser::FieldType::String | parser::FieldType::Text => {
                        let nm = field.name.to_lowercase();
                        if nm.contains("name") || nm.contains("customer") || nm.contains("author") {
                            serde_json::Value::String(format!(
                                "{} {}",
                                first_names[i % 20],
                                last_names[i % 20]
                            ))
                        } else if nm.contains("company") || nm.contains("org") {
                            serde_json::Value::String(companies[i % 20].to_string())
                        } else if nm.contains("title") || nm.contains("subject") {
                            serde_json::Value::String(format!(
                                "Item #{} — {}",
                                i + 1,
                                titles[i % 20]
                            ))
                        } else if nm.contains("description")
                            || nm.contains("content")
                            || nm.contains("body")
                            || nm.contains("bio")
                            || nm.contains("excerpt")
                        {
                            serde_json::Value::String(format!("Sample content for item {}. This is realistic test data generated by cronus seed.", i + 1))
                        } else if nm.contains("password") {
                            serde_json::Value::String(auth::hash_password("password123"))
                        } else if nm.contains("address") {
                            serde_json::Value::String(format!(
                                "{} {} St, Suite {}",
                                100 + i * 37 % 900,
                                last_names[i % 20],
                                i + 1
                            ))
                        } else if nm.contains("color") {
                            let colors = [
                                "#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6", "#ec4899",
                                "#06b6d4", "#f97316", "#14b8a6", "#6366f1",
                            ];
                            serde_json::Value::String(colors[i % 10].to_string())
                        } else {
                            serde_json::Value::String(format!("{}_{}", field.name, i + 1))
                        }
                    }
                    parser::FieldType::Email => serde_json::Value::String(format!(
                        "{}.{}@example.com",
                        first_names[i % 20].to_lowercase(),
                        last_names[i % 20].to_lowercase()
                    )),
                    parser::FieldType::Phone => serde_json::Value::String(format!(
                        "+1 555-{:03}-{:04}",
                        100 + i * 3,
                        1000 + i * 7
                    )),
                    parser::FieldType::Url => serde_json::Value::String(format!(
                        "https://example.com/{}/{}",
                        field.name,
                        i + 1
                    )),
                    parser::FieldType::Number => {
                        serde_json::Value::String(format!("{}", (i + 1) * 10 + (i * 7) % 100))
                    }
                    parser::FieldType::Money => {
                        serde_json::Value::String(format!("{}", (i + 1) * 1990 + (i * 500) % 10000))
                    }
                    parser::FieldType::Percentage => {
                        serde_json::Value::String(format!("{}", 15 + (i * 8) % 85))
                    }
                    parser::FieldType::Boolean => {
                        serde_json::Value::String(if i % 3 == 0 { "0" } else { "1" }.to_string())
                    }
                    parser::FieldType::Date => {
                        let day = 1 + (i % 28);
                        let month = 1 + (i % 12);
                        serde_json::Value::String(format!("2026-{:02}-{:02}", month, day))
                    }
                    parser::FieldType::DateTime => {
                        let day = 1 + (i % 28);
                        let month = 1 + (i % 12);
                        serde_json::Value::String(format!("2026-{:02}-{:02}T10:00:00", month, day))
                    }
                    parser::FieldType::File => serde_json::Value::String(format!(
                        "https://example.com/files/{}/{}",
                        field.name,
                        i + 1
                    )),
                    parser::FieldType::Enum => {
                        if let Some(ref vals) = field.enum_values {
                            serde_json::Value::String(vals[i % vals.len()].clone())
                        } else {
                            let statuses =
                                ["active", "pending", "completed", "cancelled", "processing"];
                            serde_json::Value::String(statuses[i % 5].to_string())
                        }
                    }
                    parser::FieldType::Slug => {
                        serde_json::Value::String(format!("{}-{}", field.name, i + 1))
                    }
                    parser::FieldType::Ip => serde_json::Value::String(format!(
                        "192.168.{}.{}",
                        1 + i / 255,
                        1 + i % 255
                    )),
                    parser::FieldType::Relation => {
                        if let Some(ref target) = field.reference {
                            match db.find_all(target, 1, i) {
                                Ok(rows) => {
                                    if let Some(arr) = rows.as_array() {
                                        if let Some(first) = arr.first() {
                                            if let Some(id) =
                                                first.get("id").and_then(|v| v.as_str())
                                            {
                                                serde_json::Value::String(id.to_string())
                                            } else {
                                                serde_json::Value::Null
                                            }
                                        } else {
                                            serde_json::Value::Null
                                        }
                                    } else {
                                        serde_json::Value::Null
                                    }
                                }
                                Err(_) => serde_json::Value::Null,
                            }
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    _ => serde_json::Value::String(format!("value_{}", i + 1)),
                };

                if !val.is_null() {
                    obj.insert(field.name.clone(), val);
                }
            }

            if db
                .insert(&entity.name, &serde_json::Value::Object(obj))
                .is_ok()
            {
                seeded += 1;
            }
        }
        let sample: String = if seeded > 0 {
            format!(
                " ({})",
                entity.fields.first().map(|f| f.name.as_str()).unwrap_or("")
            )
        } else {
            String::new()
        };
        println!(
            "  \x1b[32m\u{2713}\x1b[0m {:<12} {} rows",
            entity.name, seeded
        );
    }

    let seed_ms = seed_start.elapsed().as_millis();
    println!("\n  Done in {}ms. Run: \x1b[1mcronus run\x1b[0m\n", seed_ms);
}
