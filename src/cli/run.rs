//! `cronus run` / `cronus debug`: load the app, build `AppState`, start the
//! HTTP server loop (connection service: `routes::serve`).

use crate::cli::verify::cmd_debug_audit;
use crate::parser::{self, ApiNode, AppNode, AstNode, EntityNode, PageNode, StyleNode};
use crate::routes::serve;
use crate::server::state::{AppState, TraceBuffer};
use crate::{
    audit, auth, auth_entity, brain, cli, constitution_check, database, find_all_cronus_files, hmr,
    http_guard, lint, memory, open_memory_db, rate_limit, resolve, scripting, sse, theme, zeus,
    AUDIT_CANVAS, DEBUG_MODE,
};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::Request;
use hyper_util::rt::TokioIo;
use std::fs;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::net::TcpListener;

pub async fn cmd_debug(args: &[String]) {
    let subcmd = args.get(2).map(|s| s.as_str()).unwrap_or("");

    if subcmd == "audit" {
        cmd_debug_audit(args);
        return;
    }

    DEBUG_MODE.store(true, Ordering::Relaxed);
    println!("  \x1b[36m⚡\x1b[0m Debug mode enabled — request tracing active");
    println!("  \x1b[90m  Traces: GET /api/debug/traces  |  Headers: X-Response-Time, X-Request-Id, X-Query-Count\x1b[0m");
    cmd_run(args).await;
}

// cmd_debug_audit moved to cli::verify

pub async fn cmd_run(args: &[String]) {
    let start_time = Instant::now();

    if let Err(e) = auth::check_secret_config() {
        eprintln!("  \x1b[31m✗\x1b[0m {}", e);
        std::process::exit(1);
    }

    // Find .cronus files — supports multi-agent mode
    let files = find_all_cronus_files();
    if files.is_empty() {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found in current directory");
        std::process::exit(1);
    }

    if files.len() > 1 {
        println!(
            "  \x1b[36m⚡\x1b[0m Multi-file mode: {} files detected",
            files.len()
        );
    }
    let (nodes, total_lines) = match parse_app_files(&files) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m {}", e);
            std::process::exit(1);
        }
    };

    let file = files[0].clone(); // for HMR watcher

    // `env { … }` schema: refuse to start on missing or mistyped variables.
    // The report names variables only, never values.
    match crate::env_schema::enforce(&nodes, |name| std::env::var(name).ok()) {
        Ok(0) => {}
        Ok(n) => println!(
            "  \x1b[90mEnv:\x1b[0m       {} declared variable(s) valid",
            n
        ),
        Err(report) => {
            for line in report.lines() {
                eprintln!("  \x1b[31m✗\x1b[0m {}", line);
            }
            std::process::exit(1);
        }
    }

    // Save AST snapshot for changelog diffing
    cli::build::save_ast_snapshot(&nodes);

    // Create semantic memory session and extract business rules
    if let Ok(mem) = open_memory_db() {
        match mem.create_session(Some("cronus-run")) {
            Ok(sid) => println!(
                "  \x1b[32m✓\x1b[0m Memory session: {}",
                &sid[..sid.len().min(20)]
            ),
            Err(e) => eprintln!("  \x1b[33m⚠\x1b[0m Memory session failed: {}", e),
        }
        // Auto-extract business rules from @business/@rule doc tags and constitution
        memory::extract_and_store_business_rules(&mem, &nodes);
    }

    let spec = match collect_app(&nodes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m {}", e);
            std::process::exit(1);
        }
    };
    if spec.implicit_auth {
        println!(
            "  \x1b[90mAuth:\x1b[0m      entity {} not declared — using implicit (email, password, role, name)",
            spec.auth_entity.as_deref().unwrap_or("")
        );
    }
    let CollectedApp {
        app,
        entities,
        pages,
        style,
        apis,
        webhooks,
        components: cronus_components,
        auth_entity,
        auth_roles,
        auth_required_pages,
        auth_redirect,
        session_policy,
        layout,
        route_count,
        implicit_auth: _,
    } = spec;

    let audit_canvas = args.iter().any(|a| a == "--audit-canvas")
        || std::env::var("CRONUS_AUDIT")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    if audit_canvas {
        AUDIT_CANVAS.store(true, Ordering::Relaxed);
    }

    // CLI port takes precedence. `--audit-canvas [port]` default 5176.
    let audit_flag_port = args
        .iter()
        .position(|a| a == "--audit-canvas")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<u16>().ok());
    let cli_port = audit_flag_port.or_else(|| {
        args.iter().skip(2).find_map(|s| {
            if s.starts_with('-') {
                None
            } else {
                s.parse::<u16>().ok()
            }
        })
    });
    let serve_port = if let Some(p) = cli_port {
        p
    } else if audit_canvas {
        5176
    } else {
        app.port
    };
    apply_theme(&app, &style);

    // Database — use CronusDB for all operations
    let db_path = app
        .database
        .as_ref()
        .and_then(|d| d.path.clone())
        .unwrap_or_else(|| "data.db".into());

    let app_db = database::CronusDB::open(&db_path).expect("Failed to open database");
    app_db
        .migrate(&entities)
        .expect("Failed to migrate database");

    ensure_user_password_column(&app_db, &entities);

    let table_count = entities.len();

    // Initialize Hydra Brain
    let brain_db = Arc::new(database::CronusDB::open(&db_path).expect("Failed to open brain DB"));
    // Create brain events table
    let brain_entity = parser::EntityNode {
        name: "_brain_events".to_string(),
        shared: true,
        transitions: Vec::new(),
        effects: Vec::new(),
        fields: vec![
            parser::FieldNode {
                name: "event".to_string(),
                field_type: parser::FieldType::String,
                required: true,
                unique: false,
                sensitive: false,
                optional: false,
                searchable: false,
                index: false,
                featured: false,
                formatted: false,
                array: false,
                enum_values: None,
                reference: None,
                doc: None,
                default_value: None,
                min: None,
                max: None,
                min_length: None,
                max_length: None,
                pattern: None,
            },
            parser::FieldNode {
                name: "metadata".to_string(),
                field_type: parser::FieldType::Text,
                required: false,
                unique: false,
                sensitive: false,
                optional: false,
                searchable: false,
                index: false,
                featured: false,
                formatted: false,
                array: false,
                enum_values: None,
                reference: None,
                doc: None,
                default_value: None,
                min: None,
                max: None,
                min_length: None,
                max_length: None,
                pattern: None,
            },
            parser::FieldNode {
                name: "timestamp".to_string(),
                field_type: parser::FieldType::String,
                required: false,
                unique: false,
                sensitive: false,
                optional: false,
                searchable: false,
                index: false,
                featured: false,
                formatted: false,
                array: false,
                enum_values: None,
                reference: None,
                doc: None,
                default_value: None,
                min: None,
                max: None,
                min_length: None,
                max_length: None,
                pattern: None,
            },
        ],
        reverses: Vec::new(),
        remote_url: None,
        doc: None,
        span: Default::default(),
    };
    let _ = brain_db.migrate(&[brain_entity]);
    let hydra = brain::CronusBrain::init(brain_db);

    // Build app state (reuse app_db from migration)

    let sse_hub = Arc::new(sse::SseHub::new());
    let audit_trail = audit::AuditTrail::open(&db_path).expect("Failed to open audit trail");

    // Load .scriptcronus files
    let script_registry = scripting::ScriptRegistry::load_from_directory(".");

    let state = Arc::new(AppState {
        app: app.clone(),
        entities,
        pages,
        components: cronus_components,
        style,
        apis,
        db_path: db_path.clone(),
        db: app_db,
        brain: Some(hydra),
        auth_entity,
        auth_roles,
        auth_required_pages,
        auth_redirect,
        session_policy,
        layout,
        webhooks,
        rate_limiter: rate_limit::RateLimiter::new(100, 60),
        auth_rate_limiter: rate_limit::RateLimiter::new(10, 60),
        sse_hub,
        audit_trail,
        trace_buffer: Arc::new(TraceBuffer::new()),
        script_registry,
        zeus: Arc::new(zeus::ZeusBuffer::new(200)),
    });
    let live: Arc<RwLock<Arc<AppState>>> = Arc::new(RwLock::new(state.clone()));

    // Start server. Default 127.0.0.1; `--host`/CRONUS_HOST opts into exposure.
    // Audit-canvas always binds loopback.
    let policy = match http_guard::policy_from_env(args, audit_canvas) {
        Ok(p) => http_guard::install_policy(p),
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m {}", e);
            std::process::exit(1);
        }
    };
    if !policy.bind_ip.is_loopback() {
        eprintln!("  \x1b[33m⚠\x1b[0m Binding to {} — the server is reachable from the network. Internal dev routes require an admin session.", policy.bind_ip);
    }
    if policy.mode == http_guard::RunMode::Production {
        println!("  \x1b[90mMode:\x1b[0m      production (internal routes disabled)");
        let env_debug = std::env::var("CRONUS_DEBUG").ok();
        if crate::server::response::debug_overlay_enabled(
            false,
            DEBUG_MODE.load(Ordering::Relaxed),
            env_debug.as_deref(),
        ) {
            // Request tracing and the overlay are dev tooling.
            DEBUG_MODE.store(false, Ordering::Relaxed);
            eprintln!("  \x1b[33m⚠\x1b[0m Debug mode ignored in production (--debug / CRONUS_DEBUG): no overlay, no request tracing");
        }
    }
    {
        let live = live.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                tick.tick().await;
                let st = live.read().unwrap_or_else(|e| e.into_inner()).clone();
                st.rate_limiter.cleanup();
                st.auth_rate_limiter.cleanup();
                http_guard::account_limiter().cleanup_at(Instant::now());
            }
        });
    }
    let addr = std::net::SocketAddr::new(policy.bind_ip, serve_port);
    // Every kernel-authored <script> now carries a marker that html_response
    // turns into the per-request CSP nonce (no 'unsafe-inline' in script-src).
    crate::security::enable_script_nonces();
    let listener = TcpListener::bind(addr).await.unwrap_or_else(|e| {
        eprintln!(
            "  \x1b[31m✗\x1b[0m Cannot bind to port {}: {}",
            serve_port, e
        );
        std::process::exit(1);
    });

    let accent = {
        let state = live.read().unwrap_or_else(|e| e.into_inner());
        state
            .style
            .as_ref()
            .and_then(|s| s.accent.as_deref())
            .unwrap_or("amber")
            .to_string()
    };

    // Start HMR file watcher (dev only: production pages carry no HMR client
    // and /.cronus/version is a 404 there, so polling files would be wasted work).
    // On change: re-parse, migrate, swap live AppState, then bump the poll version
    // so the browser refresh sees the new AST.
    let watch_files = policy.mode != http_guard::RunMode::Production;
    if !watch_files {
        println!("  \x1b[90mHMR:\x1b[0m       off (production)");
    } else {
        let live_w = live.clone();
        let watch_list = files.clone();
        let on_change = move || match try_hot_reload(&watch_list, &live_w) {
            Ok(v) => eprintln!(
                "  \x1b[33m⚡\x1b[0m Spec reloaded — version {} (browser will reload)",
                v
            ),
            Err(e) => eprintln!(
                "  \x1b[31m✗\x1b[0m HMR reload failed (keeping previous spec): {}",
                e
            ),
        };
        if files.len() > 1 {
            hmr::start_directory_watcher(".", on_change);
        } else {
            hmr::start_watcher(&file, on_change);
        }
    }

    // Count total rows across all entities
    let total_rows: usize = state
        .entities
        .iter()
        .map(|e| state.db.count(&e.name).unwrap_or(0))
        .sum();

    let auth_page_count = state.auth_required_pages.len();
    let total_pages = state.pages.len();
    let elapsed_ms = start_time.elapsed().as_millis();

    // ── Clean startup banner ──
    println!();
    println!(
        "  \x1b[36m\x1b[1mCRONUS\x1b[0m \x1b[90mv{}\x1b[0m",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("  \x1b[90mApp:\x1b[0m       \x1b[1m{}\x1b[0m", app.name);
    let listen_url = match policy.bind_ip {
        std::net::IpAddr::V6(ip) => format!("http://[{ip}]:{serve_port}"),
        std::net::IpAddr::V4(ip) => format!("http://{ip}:{serve_port}"),
    };
    println!("  \x1b[90mPort:\x1b[0m      \x1b]8;;{listen_url}\x1b\\{listen_url}\x1b]8;;\x1b\\");
    println!(
        "  \x1b[90mDatabase:\x1b[0m  ./{} ({} entities, {} rows)",
        db_path, table_count, total_rows
    );
    if total_pages > 0 {
        if auth_page_count > 0 {
            println!(
                "  \x1b[90mPages:\x1b[0m     {} ({} require auth)",
                total_pages, auth_page_count
            );
        } else {
            println!("  \x1b[90mPages:\x1b[0m     {}", total_pages);
        }
    }
    if route_count > 0 {
        println!("  \x1b[90mRoutes:\x1b[0m    {} API endpoints", route_count);
    }
    if state.script_registry.block_count() > 0 {
        let sc = &state.script_registry;
        let events = sc
            .scripts
            .iter()
            .flat_map(|s| s.blocks.iter())
            .filter(|b| matches!(b, scripting::ast::ScriptBlock::OnEvent(_)))
            .count();
        let schedules = sc.get_schedules().len();
        let endpoints = sc.get_endpoints().len();
        let webhooks = sc
            .scripts
            .iter()
            .flat_map(|s| s.blocks.iter())
            .filter(|b| matches!(b, scripting::ast::ScriptBlock::OnWebhook(_)))
            .count();
        println!(
            "  \x1b[90mScripts:\x1b[0m   {} ({} events, {} schedules, {} endpoints, {} webhooks)",
            sc.scripts.len(),
            events,
            schedules,
            endpoints,
            webhooks
        );
    }
    if let Some(ref _auth_e) = state.auth_entity {
        if state.auth_roles.is_empty() {
            println!("  \x1b[90mAuth:\x1b[0m      JWT");
        } else {
            println!(
                "  \x1b[90mAuth:\x1b[0m      JWT (roles: {})",
                state.auth_roles.join(", ")
            );
        }
    }
    // ── Integrity checks: dead links, unbound sections, hardcoded data ──
    {
        let page_routes: Vec<&str> = state.pages.iter().map(|p| p.route.as_str()).collect();
        let mut warnings = 0;

        for page in &state.pages {
            // Check sidebar links pointing to non-existent pages
            for section in &page.sections {
                if let Some(ref tpl) = section.template {
                    // Find href="/..." links and check if page exists
                    let mut pos = 0;
                    let bytes = tpl.as_bytes();
                    while pos < tpl.len() {
                        if let Some(idx) = tpl[pos..].find("href=\"/") {
                            let start = pos + idx + 6;
                            if let Some(end) = tpl[start..].find('"') {
                                let href = &tpl[start - 1..start + end];
                                if href != "/"
                                    && href != "#"
                                    && !href.starts_with("/#")
                                    && !href.starts_with("/api/")
                                    && !page_routes.contains(&href)
                                {
                                    println!("  \x1b[31m✗\x1b[0m Dead link: \"{}\" → page {} does not exist", href, href);
                                    warnings += 1;
                                }
                                pos = start + end;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }

                // Check sections without binding that should have data
                let data_sections = ["kpi", "stat-cards", "table"];
                if data_sections.contains(&section.section_type.as_str())
                    && section.binding.is_none()
                    && section.items.is_empty()
                {
                    println!("  \x1b[31m✗\x1b[0m Page \"{}\": section \"{}\" has no data source (no bind, no items)", page.route, section.section_type);
                    warnings += 1;
                }
            }
        }

        if warnings > 0 {
            println!("\n  \x1b[33m⚠ {} integrity warning(s)\x1b[0m", warnings);
        }
    }

    println!();
    println!("  \x1b[32mReady in {}ms\x1b[0m", elapsed_ms);

    // ── Startup audit: if CRONUS_AUDIT_REF is set, print fidelity score ──
    if let Ok(ref_path) = std::env::var("CRONUS_AUDIT_REF") {
        if !ref_path.is_empty() {
            match cli::audit_fidelity::run_audit(&ref_path) {
                Some(result) => {
                    println!();
                    println!(
                        "  \x1b[1mFidelity Audit\x1b[0m (CRONUS_AUDIT_REF={})",
                        ref_path
                    );
                    cli::audit_fidelity::print_fidelity_line(&result);
                    if result.fidelity < 90 {
                        println!("  \x1b[33m⚠ Fidelity below 90% — review missing items:\x1b[0m");
                        cli::audit_fidelity::print_missing_top(&result, 5);
                    }
                    cli::audit_fidelity::save_audit_results(&result);
                }
                None => {
                    eprintln!("  \x1b[33m⚠\x1b[0m CRONUS_AUDIT_REF set but audit failed (check file path or .cronus)");
                }
            }
        }
    }

    println!();
    println!("  Press Ctrl+C to stop.");
    println!();

    // Start script schedules
    {
        let schedules = state.script_registry.get_schedules();
        for (script, sched) in &schedules {
            let interval = scripting::parse_interval(&sched.interval);
            let body = sched.body.clone();
            let db_path = state.db_path.clone();
            let sched_name = sched.name.clone();
            let script_name = script.name.clone();
            tokio::spawn(async move {
                let mut tick = tokio::time::interval(interval);
                tick.tick().await; // skip immediate first tick
                loop {
                    tick.tick().await;
                    eprintln!(
                        "  \x1b[36m[schedule]\x1b[0m running \"{}\" from \"{}\"",
                        sched_name, script_name
                    );
                    let db = match crate::database::CronusDB::open(&db_path) {
                        Ok(db) => db,
                        Err(e) => {
                            eprintln!("  \x1b[31m[schedule]\x1b[0m db open failed: {}", e);
                            continue;
                        }
                    };
                    let mut ctx = scripting::vm::ScriptContext::new(
                        "system",
                        "admin",
                        std::collections::HashMap::new(),
                    );
                    if let Err(e) = scripting::vm::execute_statements(&body, &mut ctx, &db, None) {
                        eprintln!(
                            "  \x1b[31m[schedule]\x1b[0m \"{}\" error: {}",
                            sched_name, e
                        );
                    }
                }
            });
            eprintln!(
                "  \x1b[36m[schedule]\x1b[0m registered \"{}\" (every {})",
                sched.name, sched.interval
            );
        }
    }

    // Resolve pass — verify all cross-references (fatal errors)
    let (_symbol_table, resolve_errors) = resolve::resolve(&nodes);
    if !resolve_errors.is_empty() {
        println!();
        println!("  \x1b[1mResolve Pass\x1b[0m");
        for e in &resolve_errors {
            println!("{}", e);
        }
        println!();
        println!(
            "  \x1b[31m{} resolve error(s)\x1b[0m — aborting",
            resolve_errors.len()
        );
        std::process::exit(1);
    }

    // Zero Hardcode Lint — run on startup (warnings only, never blocks)
    let lint_results = lint::lint_ast(&nodes, false);
    if !lint_results.is_empty() {
        for r in &lint_results {
            println!("{}", r);
        }
    }

    // Constitution enforcement — warn on startup (never blocks run)
    if let Some(ref c) = app.constitution {
        let cv = constitution_check::check_constitution(&nodes, c);
        let real_violations: Vec<_> = cv.iter().filter(|v| v.rule_type != "info").collect();
        if !real_violations.is_empty() {
            println!();
            println!("  \x1b[1mConstitution Warnings\x1b[0m");
            for v in &cv {
                println!("{}", v);
            }
            println!();
            println!(
                "  \x1b[33m\u{26a0} {} constitution violation(s)\x1b[0m",
                real_violations.len()
            );
        }
    }

    // Graceful shutdown: listen for Ctrl+C
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, remote_addr) = match result {
                    Ok(conn) => conn,
                    Err(e) => {
                        eprintln!("  Accept error: {}", e);
                        continue;
                    }
                };
                let io = TokioIo::new(stream);
                let state = live.read().unwrap_or_else(|e| e.into_inner()).clone();

        tokio::task::spawn(async move {
            let service = service_fn(move |req: Request<Incoming>| {
                serve(req, state.clone(), remote_addr)
            });
            if let Err(e) = http_guard::http1_builder().serve_connection(io, service).await {
                http_guard::log_connection_error(&e);
            }
        });
            }
            _ = &mut shutdown => {
                eprintln!("\n  \x1b[36mShutting down gracefully...\x1b[0m");
                // Give in-flight requests 2 seconds to complete
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                eprintln!("  \x1b[32m✓\x1b[0m Server stopped.");
                break;
            }
        }
    }
}

struct CollectedApp {
    app: AppNode,
    entities: Vec<EntityNode>,
    pages: Vec<PageNode>,
    style: Option<StyleNode>,
    apis: Vec<ApiNode>,
    webhooks: Vec<parser::WebhookNode>,
    components: Vec<parser::ComponentNode>,
    auth_entity: Option<String>,
    auth_roles: Vec<String>,
    auth_required_pages: Vec<(String, String)>,
    auth_redirect: Option<String>,
    session_policy: crate::auth::SessionPolicy,
    layout: Option<parser::LayoutNode>,
    route_count: usize,
    implicit_auth: bool,
}

fn parse_app_files(files: &[String]) -> Result<(Vec<AstNode>, usize), String> {
    if files.len() == 1 {
        let source = fs::read_to_string(&files[0])
            .map_err(|e| format!("Error reading {}: {}", files[0], e))?;
        let lines = source.lines().count();
        let n = parser::parse_source_at(&source, std::path::Path::new(&files[0]))
            .map_err(|e| parser::diagnostic::join(&e))?;
        Ok((n, lines))
    } else {
        let mut total = 0;
        for f in files {
            total += fs::read_to_string(f)
                .map(|s| s.lines().count())
                .unwrap_or(0);
        }
        let n = parser::parse_directory(".").map_err(|e| e.to_string())?;
        Ok((n, total))
    }
}

fn collect_app(nodes: &[AstNode]) -> Result<CollectedApp, String> {
    let mut app = AppNode {
        name: "CRONUS App".into(),
        stack: vec![],
        port: 5175,
        database: None,
        tailwind_config: None,
        constitution: None,
        graphql: true,
        doc: None,
        span: Default::default(),
    };
    let mut entities: Vec<EntityNode> = vec![];
    let mut pages: Vec<PageNode> = vec![];
    let mut style: Option<StyleNode> = None;
    let mut apis: Vec<ApiNode> = vec![];
    let mut webhooks: Vec<parser::WebhookNode> = vec![];
    let mut cronus_components: Vec<parser::ComponentNode> = vec![];
    let mut route_count = 0;
    let mut auth_entity: Option<String> = None;
    let mut auth_roles: Vec<String> = Vec::new();
    let mut auth_required_pages: Vec<(String, String)> = Vec::new();
    let mut auth_redirect: Option<String> = None;
    let mut session_policy = crate::auth::SessionPolicy::default();
    let mut layout: Option<parser::LayoutNode> = None;

    for node in nodes {
        match node {
            AstNode::App(a) => app = a.clone(),
            AstNode::Entity(e) => entities.push(e.clone()),
            AstNode::Page(p) => {
                let req = p
                    .requires
                    .clone()
                    .or_else(|| p.config.get("requires").cloned());
                if let Some(ref req_val) = req {
                    auth_required_pages.push((p.route.clone(), req_val.clone()));
                }
                pages.push(p.clone());
            }
            AstNode::Style(s) => style = Some(s.clone()),
            AstNode::Api(a) => {
                route_count += a.routes.len();
                apis.push(a.clone());
            }
            AstNode::Component(c) => cronus_components.push(c.clone()),
            AstNode::Auth(auth) => {
                auth_entity = Some(auth.entity.clone());
                auth_roles = auth.roles.clone();
                session_policy =
                    crate::auth::SessionPolicy::from_session_config(&auth.session_config)?;
                if let Some(r) = auth.session_config.get("redirect") {
                    auth_redirect = Some(r.clone());
                }
            }
            AstNode::Layout(l) => {
                layout = Some(l.clone());
            }
            AstNode::Webhook(w) => {
                webhooks.push(w.clone());
            }
            _ => {}
        }
    }

    let implicit_auth = auth_entity::ensure_declared(&mut entities, auth_entity.as_deref());
    interpolate_components(&mut pages, &cronus_components);

    Ok(CollectedApp {
        app,
        entities,
        pages,
        style,
        apis,
        webhooks,
        components: cronus_components,
        auth_entity,
        auth_roles,
        auth_required_pages,
        auth_redirect,
        session_policy,
        layout,
        route_count,
        implicit_auth,
    })
}

fn apply_theme(app: &AppNode, style: &Option<StyleNode>) {
    let tokens = if let Some(ref tc) = app.tailwind_config {
        theme::parse_from_tailwind_config(tc)
    } else if let Some(ref s) = style {
        theme::derive_palette(
            s.accent.as_deref().unwrap_or(""),
            s.theme.as_deref().unwrap_or("dark"),
            s.font.as_deref().unwrap_or(""),
        )
    } else {
        theme::ThemeTokens::default()
    };
    theme::set_global(tokens);
    let preset = style
        .as_ref()
        .and_then(|s| s.config.get("preset").cloned())
        .unwrap_or_default();
    theme::set_preset(&preset);
    theme::set_mode(
        style
            .as_ref()
            .and_then(|s| s.theme.as_deref())
            .unwrap_or("dark"),
    );
}

fn ensure_user_password_column(db: &database::CronusDB, entities: &[EntityNode]) {
    let has_user = entities.iter().any(|e| {
        let l = e.name.to_lowercase();
        l == "user" || l == "users"
    });
    if has_user {
        let user_table = entities
            .iter()
            .find(|e| {
                let l = e.name.to_lowercase();
                l == "user" || l == "users"
            })
            .map(|e| e.name.as_str())
            .unwrap();
        let _ = db.execute_raw(&format!(
            "ALTER TABLE \"{}\" ADD COLUMN password TEXT",
            user_table
        ));
    }
}

fn interpolate_components(pages: &mut [PageNode], cronus_components: &[parser::ComponentNode]) {
    if cronus_components.is_empty() {
        return;
    }
    let mut comp_instance_counter: u32 = 0;
    for page in pages {
        for section in &mut page.sections {
            if let Some(comp_name) = section.config.get("_component").cloned() {
                if let Some(comp_def) = cronus_components.iter().find(|c| c.name == comp_name) {
                    if let Some(ref tmpl) = comp_def.template {
                        comp_instance_counter += 1;
                        let cid = format!("c{}", comp_instance_counter);

                        let mut rendered = tmpl.clone();
                        for param in &comp_def.params {
                            let placeholder = format!("{{{{{}}}}}", param.name);
                            let value = section
                                .config
                                .get(&param.name)
                                .map(|s| s.as_str())
                                .or(param.default.as_deref())
                                .unwrap_or("");
                            rendered = rendered.replace(&placeholder, value);
                        }
                        for (key, value) in &section.config {
                            if key == "_component" {
                                continue;
                            }
                            let placeholder = format!("{{{{{}}}}}", key);
                            rendered = rendered.replace(&placeholder, value);
                        }

                        if !comp_def.state.is_empty() {
                            rendered = format!(
                                r#"<div data-cid="{cid}">{html}</div>"#,
                                cid = cid,
                                html = rendered
                            );

                            for sv in &comp_def.state {
                                let placeholder = format!("{{{{{}}}}}", sv.name);
                                let span = format!(
                                    r#"<span data-s="{name}">{default}</span>"#,
                                    name = sv.name,
                                    default = sv.default
                                );
                                rendered = rendered.replace(&placeholder, &span);
                            }

                            let mut script_parts: Vec<String> = Vec::new();
                            let mut event_id: u32 = 0;

                            while let Some(pos) = rendered.find("@click=") {
                                event_id += 1;
                                let eid = format!("{cid}_e{event_id}");
                                let after = &rendered[pos + 7..];
                                let (expr, end_offset) = if after.starts_with("\\\"")
                                    || after.starts_with('"')
                                {
                                    let quote_char = if after.starts_with("\\\"") {
                                        "\\\""
                                    } else {
                                        "\""
                                    };
                                    let qlen = quote_char.len();
                                    let expr_start = qlen;
                                    if let Some(expr_end) = after[expr_start..].find(quote_char) {
                                        (
                                            after[expr_start..expr_start + expr_end].to_string(),
                                            7 + expr_start + expr_end + qlen,
                                        )
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                };

                                rendered = format!(
                                    "{}data-eid=\"{}\"{}",
                                    &rendered[..pos],
                                    eid,
                                    &rendered[pos + end_offset..]
                                );

                                let js_expr = expr.replace("\\\"", "\"");
                                script_parts.push(format!(
                                    r#"document.querySelector('[data-eid="{eid}"]').addEventListener('click',function(){{ {update_expr}; _u(); }});"#,
                                    eid = eid, update_expr = format!("_s.{}", js_expr)
                                ));
                            }

                            if !comp_def.state.is_empty() {
                                let mut state_init = String::new();
                                let mut update_dom = String::new();
                                for sv in &comp_def.state {
                                    let default_js = match sv.state_type.as_str() {
                                        "integer" | "number" => sv.default.clone(),
                                        "boolean" => sv.default.clone(),
                                        _ => format!("\"{}\"", sv.default),
                                    };
                                    state_init.push_str(&format!("{}:{},", sv.name, default_js));
                                    update_dom.push_str(&format!(
                                        r#"_c.querySelectorAll('[data-s="{name}"]').forEach(function(el){{ el.textContent=_s.{name}; }});"#,
                                        name = sv.name
                                    ));
                                }

                                let script = format!(
                                    r#"<script>(function(){{ var _c=document.querySelector('[data-cid="{cid}"]'); if(!_c)return; var _s={{{init}}}; function _u(){{{update}}} {events} }})();</script>"#,
                                    cid = cid,
                                    init = state_init,
                                    update = update_dom,
                                    events = script_parts.join(" "),
                                );
                                rendered.push_str(&script);
                            }
                        }

                        section.template = Some(rendered);
                    }
                }
            }
        }
    }
}

fn try_hot_reload(files: &[String], live: &RwLock<Arc<AppState>>) -> Result<u64, String> {
    let (nodes, _) = parse_app_files(files)?;
    crate::env_schema::enforce(&nodes, |name| std::env::var(name).ok())?;
    let spec = collect_app(&nodes)?;
    apply_theme(&spec.app, &spec.style);
    let prev = live.read().unwrap_or_else(|e| e.into_inner()).clone();
    let next = AppState {
        app: spec.app,
        entities: spec.entities,
        pages: spec.pages,
        components: spec.components,
        style: spec.style,
        apis: spec.apis,
        db_path: prev.db_path.clone(),
        db: prev.db.clone(),
        brain: prev.brain.clone(),
        auth_entity: spec.auth_entity,
        auth_roles: spec.auth_roles,
        auth_required_pages: spec.auth_required_pages,
        auth_redirect: spec.auth_redirect,
        session_policy: spec.session_policy,
        layout: spec.layout,
        webhooks: spec.webhooks,
        rate_limiter: prev.rate_limiter.clone(),
        auth_rate_limiter: prev.auth_rate_limiter.clone(),
        sse_hub: Arc::clone(&prev.sse_hub),
        audit_trail: prev.audit_trail.clone(),
        trace_buffer: Arc::clone(&prev.trace_buffer),
        script_registry: scripting::ScriptRegistry::load_from_directory("."),
        zeus: Arc::clone(&prev.zeus),
    };
    let next = hmr::adopt_spec(&prev, next)?;
    ensure_user_password_column(&next.db, &next.entities);
    *live.write().unwrap_or_else(|e| e.into_inner()) = Arc::new(next);
    Ok(hmr::bump_version())
}
