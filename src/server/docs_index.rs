#![allow(dead_code)]
//! AI-optimized documentation index for CRONUS kernel.
//!
//! Serves a structured JSON index of all language topics,
//! optimized for AI navigation and search.

use std::collections::HashMap;
use serde_json::{json, Value};
use super::state::AppState;

// ──────────────────────────────────────────────
// Entry builder helpers
// ──────────────────────────────────────────────

fn entry(
    id: &str,
    title: &str,
    path: &str,
    tags: &[&str],
    keywords: &[&str],
    category: &str,
    depth: &str,
    summary: &str,
    related: &[&str],
) -> Value {
    json!({
        "id": id,
        "title": title,
        "path": path,
        "tags": tags,
        "keywords": keywords,
        "category": category,
        "depth": depth,
        "summary": summary,
        "related": related,
    })
}

fn section_entry(id: &str, title: &str, summary: &str, keywords: &[&str], related: &[&str]) -> Value {
    entry(
        id, title,
        &format!("/docs/design#{}", id),
        &["section", "ui", "rendering"],
        keywords, "section", "reference", summary, related,
    )
}

fn cli_entry(id: &str, title: &str, summary: &str, keywords: &[&str]) -> Value {
    entry(
        id, title,
        &format!("/docs#{}", id),
        &["cli", "command"],
        keywords, "cli", "reference", summary, &[],
    )
}

// ──────────────────────────────────────────────
// Static entries
// ──────────────────────────────────────────────

fn core_entries() -> Vec<Value> {
    vec![
        entry("app-block", "App Declaration", "/docs#app-block",
            &["core", "config", "app"], &["port", "database", "stack", "theme", "constitution", "app"],
            "core", "reference", "Top-level app block: name, port, database, stack, theme, constitution",
            &["style-block", "entity-block", "deploy-block"]),
        entry("entity-block", "Entity Definitions", "/docs#entity-block",
            &["core", "entity", "model"], &["entity", "fields", "transitions", "effects", "shared", "remote"],
            "core", "reference", "Define data models with typed fields, state machines, and lifecycle hooks",
            &["field-types", "field-modifiers", "transitions", "effects"]),
        entry("field-types", "Field Types", "/docs#field-types",
            &["core", "entity", "types"], &["string", "text", "email", "url", "slug", "phone", "number", "money", "percentage", "boolean", "date", "ulid", "json", "enum", "ip", "relation"],
            "core", "reference", "16 built-in field types for entity definitions",
            &["entity-block", "field-modifiers"]),
        entry("field-modifiers", "Field Modifiers", "/docs#field-modifiers",
            &["core", "entity", "validation"], &["required", "unique", "sensitive", "searchable", "optional", "array", "min", "max", "pattern"],
            "core", "reference", "Field constraint modifiers: required, unique, sensitive, searchable, optional, array, min, max, pattern",
            &["entity-block", "field-types"]),
        entry("page-block", "Page Declarations", "/docs#page-block",
            &["core", "page", "ui"], &["page", "custom", "dashboard", "list", "form", "detail", "checkout", "route"],
            "core", "reference", "Declare pages with types: custom, dashboard, list, form, detail, checkout",
            &["layout-block", "component-block"]),
        entry("style-block", "Style Configuration", "/docs#style-block",
            &["core", "design", "theme"], &["theme", "accent", "font", "mono", "radius", "cards", "style"],
            "core", "reference", "Global style config: theme (light/dark), accent color, fonts, border radius",
            &["app-block"]),
        entry("auth-block", "Authentication", "/docs#auth-block",
            &["core", "auth", "security"], &["auth", "jwt", "session", "roles", "guards", "login", "signup"],
            "core", "reference", "Authentication configuration: provider (jwt/session), roles, page guards",
            &["api-block", "auth-endpoints"]),
        entry("api-block", "API Route Definitions", "/docs#api-block",
            &["core", "api", "routes"], &["api", "get", "post", "patch", "delete", "auth", "public", "protected"],
            "core", "reference", "Define custom API routes with HTTP methods and auth levels",
            &["auth-block", "crud-endpoints"]),
        entry("binding-syntax", "Data Binding", "/docs#binding-syntax",
            &["core", "data", "query"], &["query", "all", "one", "count", "where", "order", "limit", "aggregate", "group", "live", "binding"],
            "core", "reference", "Bind sections to data: query all/one/count, where clauses, ordering, aggregation, live updates",
            &["entity-block", "page-block"]),
        entry("action-verbs", "Action System", "/docs#action-verbs",
            &["core", "actions", "interactivity"], &["set", "toast", "navigate", "refresh", "create", "update", "delete", "validate", "open", "close", "confirm", "action"],
            "core", "reference", "Built-in action verbs: set, toast, navigate, refresh, create, update, delete, validate, open, close, confirm",
            &["page-block", "component-block"]),
        entry("import-block", "Import Statements", "/docs#import-block",
            &["core", "module"], &["import", "from", "module", "include"],
            "core", "reference", "Import definitions from other .cronus files",
            &["define-block", "component-block"]),
        entry("layout-block", "Layout Definitions", "/docs#layout-block",
            &["core", "layout", "ui"], &["layout", "topbar", "sidebar", "footer", "slot"],
            "core", "reference", "Define reusable page layouts with topbar, sidebar, footer, and content slots",
            &["page-block", "define-block"]),
        entry("define-block", "Reusable Section Templates", "/docs#define-block",
            &["core", "template", "reuse"], &["define", "template", "reusable", "section"],
            "core", "reference", "Create reusable section templates that can be referenced across pages",
            &["import-block", "page-block"]),
        entry("component-block", "Component v2", "/docs#component-block",
            &["core", "component", "reactive"], &["component", "params", "state", "template", "signals", "props"],
            "core", "reference", "Component v2 with params, local state, HTML template, and reactive signals",
            &["page-block", "define-block"]),
        entry("worker-block", "Background Workers", "/docs#worker-block",
            &["core", "background", "async"], &["worker", "background", "schedule", "interval", "cron"],
            "core", "reference", "Define background workers for scheduled or recurring tasks",
            &["api-block", "scriptcronus-overview"]),
        entry("middleware-block", "Request Middleware", "/docs#middleware-block",
            &["core", "middleware", "http"], &["middleware", "before", "after", "request", "response"],
            "core", "reference", "Define middleware for request/response transformation",
            &["api-block", "auth-block"]),
        entry("env-block", "Environment Variables", "/docs#env-block",
            &["core", "config", "env"], &["env", "environment", "variable", "secret", "config"],
            "core", "reference", "Declare environment variables with defaults and validation",
            &["app-block", "deploy-block"]),
        entry("test-block", "Test Definitions", "/docs#test-block",
            &["core", "testing"], &["test", "assert", "expect", "describe", "it"],
            "core", "reference", "Define inline tests for entities, pages, and API routes",
            &["entity-block", "api-block"]),
        entry("webhook-block", "Entity Webhooks", "/docs#webhook-block",
            &["core", "webhook", "integration"], &["webhook", "on", "create", "update", "delete", "http", "notify"],
            "core", "reference", "Define webhooks triggered by entity lifecycle events",
            &["entity-block", "effects"]),
        entry("deploy-block", "Microservices Deployment", "/docs#deploy-block",
            &["core", "deploy", "infrastructure"], &["deploy", "service", "gateway", "microservice", "split"],
            "core", "reference", "Define microservices deployment topology with gateway and service splitting",
            &["app-block", "docker-deploy"]),
        entry("constitution", "Unbreakable Rules", "/docs#constitution",
            &["core", "rules", "constraints"], &["constitution", "must", "never", "rule", "constraint", "invariant"],
            "core", "reference", "Define must/never rules that are enforced at compile time and runtime",
            &["app-block", "entity-block"]),
        entry("transitions", "Entity State Machines", "/docs#transitions",
            &["core", "entity", "state-machine"], &["transition", "from", "to", "state", "machine", "flow"],
            "core", "reference", "Define valid state transitions for entity fields (e.g., draft -> published -> archived)",
            &["entity-block", "effects"]),
        entry("effects", "Entity Lifecycle Hooks", "/docs#effects",
            &["core", "entity", "hooks"], &["effect", "on", "create", "update", "delete", "hook", "lifecycle", "trigger"],
            "core", "reference", "Run actions on entity create/update/delete events (logging, notifications, cascades)",
            &["entity-block", "transitions", "webhook-block"]),
        entry("visibility", "Conditional Section Rendering", "/docs#visibility",
            &["core", "conditional", "ui"], &["show", "when", "visibility", "condition", "if", "hide"],
            "core", "reference", "Conditionally show/hide sections based on data or auth state using show:when",
            &["page-block", "binding-syntax"]),
    ]
}

fn section_entries() -> Vec<Value> {
    vec![
        section_entry("section-hero", "Hero Section", "Full-width hero with title, subtitle, CTA buttons, and optional image/video", &["hero", "title", "subtitle", "cta", "image", "video", "gradient"], &["section-cta", "section-features"]),
        section_entry("section-features", "Features Section", "Grid of feature cards with icon, title, and description", &["features", "icon", "grid", "card", "columns"], &["section-bento", "section-features-split"]),
        section_entry("section-pricing", "Pricing Section", "Pricing table with plans, tiers, and comparison", &["pricing", "plan", "tier", "price", "billing", "monthly", "yearly"], &["section-checkout", "section-cta"]),
        section_entry("section-cta", "Call to Action", "Prominent call-to-action block with button and description", &["cta", "button", "action", "convert"], &["section-hero", "section-pricing"]),
        section_entry("section-faq", "FAQ Section", "Accordion-style frequently asked questions", &["faq", "question", "answer", "accordion", "expand"], &["section-accordion"]),
        section_entry("section-testimonial", "Testimonial Section", "Customer testimonials with avatar, name, role, and quote", &["testimonial", "review", "quote", "avatar", "rating"], &["section-trusted"]),
        section_entry("section-trusted", "Trusted By Section", "Logo bar of trusted companies/partners", &["trusted", "logo", "partner", "brand", "client"], &["section-testimonial"]),
        section_entry("section-topbar", "Topbar Section", "Top navigation bar with logo, links, and actions", &["topbar", "nav", "navigation", "logo", "menu", "header"], &["section-sidebar", "section-footer"]),
        section_entry("section-sidebar", "Sidebar Section", "Side navigation panel with links and sections", &["sidebar", "nav", "menu", "panel", "drawer"], &["section-topbar", "layout-block"]),
        section_entry("section-breadcrumb", "Breadcrumb Section", "Breadcrumb navigation trail", &["breadcrumb", "trail", "navigation", "path"], &["section-topbar", "section-page-header"]),
        section_entry("section-tabs", "Tabs Section", "Tabbed content switcher", &["tabs", "tab", "switch", "panel"], &["section-accordion"]),
        section_entry("section-footer", "Footer Section", "Page footer with links, copyright, and social", &["footer", "links", "copyright", "social", "bottom"], &["section-topbar"]),
        section_entry("section-page-header", "Page Header Section", "Page title area with breadcrumb and actions", &["page-header", "title", "breadcrumb", "actions"], &["section-topbar", "section-breadcrumb"]),
        section_entry("section-links", "Links Section", "Group of navigational links", &["links", "link", "nav", "url"], &["section-footer", "section-sidebar"]),
        section_entry("section-table", "Table Section", "Data table with columns, sorting, and pagination", &["table", "columns", "sort", "pagination", "rows", "data"], &["section-activity-table", "section-filters"]),
        section_entry("section-chart", "Chart Section", "Data visualization chart (bar, line, pie, area)", &["chart", "bar", "line", "pie", "area", "graph", "visualization"], &["section-kpi", "section-stat-cards"]),
        section_entry("section-kpi", "KPI Section", "Key performance indicator cards with values and trends", &["kpi", "metric", "value", "trend", "indicator", "stats"], &["section-stat-cards", "section-chart"]),
        section_entry("section-stat-cards", "Stat Cards Section", "Grid of statistic cards with labels and values", &["stat-cards", "stats", "metric", "card", "number"], &["section-kpi", "section-chart"]),
        section_entry("section-timeline", "Timeline Section", "Chronological event timeline", &["timeline", "event", "chronological", "history", "date"], &["section-activity-table", "section-progress"]),
        section_entry("section-progress", "Progress Section", "Progress bars or step indicators", &["progress", "bar", "step", "percentage", "completion"], &["section-timeline", "section-kpi"]),
        section_entry("section-kanban", "Kanban Section", "Kanban board with draggable columns and cards", &["kanban", "board", "column", "card", "drag", "status"], &["section-table", "section-status-card"]),
        section_entry("section-form", "Form Section", "Input form with fields, validation, and submit", &["form", "input", "field", "submit", "validation", "textarea", "select"], &["section-checkout", "page-block"]),
        section_entry("section-checkout", "Checkout Section", "Payment checkout flow with order summary", &["checkout", "payment", "order", "cart", "buy", "stripe"], &["section-form", "section-pricing"]),
        section_entry("section-card", "Card Section", "Content card with image, title, and description", &["card", "image", "content", "tile"], &["section-product-grid", "section-bento"]),
        section_entry("section-product-grid", "Product Grid Section", "Grid of product cards with price and image", &["product-grid", "product", "grid", "shop", "price", "image"], &["section-card", "section-pricing"]),
        section_entry("section-bento", "Bento Grid Section", "Asymmetric bento-style grid layout", &["bento", "grid", "asymmetric", "layout", "masonry"], &["section-features", "section-card"]),
        section_entry("section-team-list", "Team List Section", "Team member cards with photo, name, and role", &["team-list", "team", "member", "avatar", "role"], &["section-card", "section-testimonial"]),
        section_entry("section-status-card", "Status Card Section", "Status indicator cards with state and color", &["status-card", "status", "indicator", "state", "badge"], &["section-kpi", "section-kanban"]),
        section_entry("section-activity-table", "Activity Table Section", "Recent activity log table", &["activity-table", "activity", "log", "recent", "event"], &["section-table", "section-timeline"]),
        section_entry("section-alert", "Alert Section", "Alert/banner messages (info, warning, error, success)", &["alert", "banner", "message", "warning", "error", "info", "success"], &["section-toast", "section-info-bar"]),
        section_entry("section-modal", "Modal Section", "Dialog overlay with content and actions", &["modal", "dialog", "overlay", "popup"], &["section-sheet", "section-form"]),
        section_entry("section-sheet", "Sheet Section", "Slide-in bottom/side panel", &["sheet", "panel", "slide", "drawer", "bottom-sheet"], &["section-modal", "section-sidebar"]),
        section_entry("section-toast", "Toast Section", "Notification toast messages", &["toast", "notification", "snackbar", "message"], &["section-alert"]),
        section_entry("section-skeleton", "Skeleton Section", "Loading skeleton placeholder", &["skeleton", "loading", "placeholder", "shimmer"], &["section-loading", "section-empty"]),
        section_entry("section-empty", "Empty State Section", "Empty state with illustration and message", &["empty", "no-data", "placeholder", "illustration"], &["section-skeleton", "section-error"]),
        section_entry("section-error", "Error Section", "Error state display with retry action", &["error", "failure", "retry", "500", "crash"], &["section-not-found", "section-empty"]),
        section_entry("section-not-found", "Not Found Section", "404 not found page content", &["not-found", "404", "missing", "page"], &["section-error", "section-empty"]),
        section_entry("section-command", "Command Palette Section", "Command palette / search overlay (Cmd+K style)", &["command", "palette", "search", "cmd-k", "spotlight"], &["section-modal", "section-filters"]),
        section_entry("section-dropdown", "Dropdown Section", "Dropdown menu with options", &["dropdown", "menu", "select", "option", "popover"], &["section-command", "section-tabs"]),
        section_entry("section-notifications", "Notifications Section", "Notification feed/list", &["notifications", "feed", "inbox", "bell", "unread"], &["section-activity-table", "section-toast"]),
        section_entry("section-dark-mode", "Dark Mode Toggle Section", "Theme toggle between light and dark modes", &["dark-mode", "toggle", "theme", "light", "switch"], &["style-block"]),
        section_entry("section-layout", "Layout Section", "Generic layout container (flex, grid)", &["layout", "container", "flex", "grid", "wrapper"], &["layout-block"]),
        section_entry("section-edge", "Edge Section", "Edge-to-edge full-bleed section", &["edge", "full-bleed", "full-width", "bleed"], &["section-hero"]),
        section_entry("section-accordion", "Accordion Section", "Collapsible accordion panels", &["accordion", "collapse", "expand", "panel", "toggle"], &["section-faq", "section-tabs"]),
        section_entry("section-pagination", "Pagination Section", "Page navigation controls", &["pagination", "page", "next", "previous", "pager"], &["section-table", "section-filters"]),
        section_entry("section-filters", "Filters Section", "Data filter controls (search, select, date range)", &["filters", "filter", "search", "select", "date-range", "facet"], &["section-table", "section-pagination"]),
        section_entry("section-promo", "Promo Section", "Promotional banner or highlight", &["promo", "banner", "promotion", "highlight", "sale"], &["section-hero", "section-cta"]),
        section_entry("section-info-bar", "Info Bar Section", "Informational top/bottom bar", &["info-bar", "announcement", "bar", "notice"], &["section-alert", "section-topbar"]),
        section_entry("section-features-split", "Features Split Section", "Alternating image+text feature blocks", &["features-split", "split", "alternating", "image-text"], &["section-features", "section-hero"]),
        section_entry("section-policies", "Policies Section", "Legal/policy content blocks (terms, privacy)", &["policies", "terms", "privacy", "legal", "tos"], &["section-footer", "section-accordion"]),
        section_entry("section-loading", "Loading Section", "Full loading state with spinner/progress", &["loading", "spinner", "progress", "wait"], &["section-skeleton"]),
        section_entry("section-stats", "Stats Section", "Statistics display (alias for kpi/stat-cards)", &["stats", "statistics", "numbers", "metrics"], &["section-kpi", "section-stat-cards"]),
    ]
}

fn cli_entries() -> Vec<Value> {
    vec![
        cli_entry("cli-run", "cronus run", "Start the dev server with HMR and auto-reload", &["run", "start", "dev", "server", "--port", "--debug"]),
        cli_entry("cli-debug", "cronus debug", "Start in debug mode with Zeus tracer and detailed logging", &["debug", "trace", "verbose", "inspect"]),
        cli_entry("cli-build", "cronus build", "Compile .cronus files to optimized production output", &["build", "compile", "production", "--release", "--output"]),
        cli_entry("cli-parse", "cronus parse", "Parse .cronus file and output AST as JSON", &["parse", "ast", "syntax", "tree", "--json"]),
        cli_entry("cli-new", "cronus new", "Scaffold a new CRONUS project from template", &["new", "init", "scaffold", "create", "project"]),
        cli_entry("cli-seed", "cronus seed", "Populate database with sample data from entity definitions", &["seed", "data", "populate", "sample", "fake"]),
        cli_entry("cli-deploy", "cronus deploy", "Deploy app to production (Docker/systemd/cloud)", &["deploy", "production", "docker", "ship"]),
        cli_entry("cli-doctor", "cronus doctor", "Diagnose environment and configuration issues", &["doctor", "diagnose", "check", "health", "fix"]),
        cli_entry("cli-stats", "cronus stats", "Show project statistics (entities, pages, sections, lines)", &["stats", "info", "count", "summary"]),
        cli_entry("cli-export", "cronus export", "Export app as standalone HTML/JSON package", &["export", "html", "json", "package", "bundle"]),
        cli_entry("cli-test", "cronus test", "Run inline test blocks and report results", &["test", "assert", "spec", "check"]),
        cli_entry("cli-compose", "cronus compose", "Generate .cronus files from templates (SaaS, landing, dashboard)", &["compose", "generate", "template", "scaffold", "--template"]),
        cli_entry("cli-generate", "cronus generate", "Generate entity/page/component scaffolds", &["generate", "scaffold", "entity", "page", "component"]),
        cli_entry("cli-dump", "cronus dump", "Reverse-engineer HTML into .cronus section primitives", &["dump", "reverse", "html", "convert", "import"]),
        cli_entry("cli-clone", "cronus clone", "Clone an existing CRONUS app as starting point", &["clone", "copy", "fork", "duplicate"]),
        cli_entry("cli-validate", "cronus validate", "Validate .cronus syntax and semantic rules", &["validate", "lint", "check", "syntax"]),
        cli_entry("cli-graph", "cronus graph", "Generate entity relationship graph visualization", &["graph", "erd", "diagram", "relationship", "visualize"]),
        cli_entry("cli-brief", "cronus brief", "Generate AI-readable project summary for context", &["brief", "summary", "context", "ai", "overview"]),
        cli_entry("cli-context", "cronus context", "Export full project context for AI assistants", &["context", "export", "ai", "prompt"]),
        cli_entry("cli-sync", "cronus sync", "Sync shared entities from remote sources", &["sync", "remote", "shared", "pull"]),
        cli_entry("cli-handoff", "cronus handoff", "Generate handoff document for team collaboration", &["handoff", "document", "team", "transfer"]),
        cli_entry("cli-lease", "cronus lease", "Manage entity field leases for concurrent editing", &["lease", "lock", "concurrent", "edit"]),
        cli_entry("cli-drift", "cronus drift", "Detect schema drift between .cronus and database", &["drift", "schema", "migration", "diff"]),
        cli_entry("cli-spec", "cronus spec", "Generate OpenAPI/Swagger spec from API definitions", &["spec", "openapi", "swagger", "api-doc"]),
        cli_entry("cli-segment", "cronus segment", "Analyze and segment app into microservices", &["segment", "microservice", "split", "analyze"]),
        cli_entry("cli-reconcile", "cronus reconcile", "Reconcile field types between entities and database", &["reconcile", "field", "type", "database", "fix"]),
        cli_entry("cli-review", "cronus review", "AI-assisted code review of .cronus files", &["review", "ai", "feedback", "quality"]),
        cli_entry("cli-timeline", "cronus timeline", "Show project change timeline", &["timeline", "history", "changelog", "changes"]),
        cli_entry("cli-status", "cronus status", "Show current project status and health", &["status", "info", "state", "overview"]),
        cli_entry("cli-changelog", "cronus changelog", "Generate changelog from project history", &["changelog", "release", "notes", "version"]),
        cli_entry("cli-memory", "cronus memory", "Manage persistent project memory for AI context", &["memory", "remember", "context", "persist"]),
        cli_entry("cli-verify-audit", "cronus verify-audit", "Verify audit results against fidelity threshold", &["verify-audit", "fidelity", "check", "threshold"]),
        cli_entry("cli-audit", "cronus audit", "Run full fidelity audit comparing rendered output to source", &["audit", "fidelity", "compare", "measure"]),
        cli_entry("cli-help", "cronus help", "Show help for all commands and options", &["help", "usage", "commands", "manual"]),
    ]
}

fn backend_entries() -> Vec<Value> {
    vec![
        entry("auth-endpoints", "Auth Endpoints", "/docs#auth-endpoints",
            &["backend", "auth", "api"], &["/api/auth/login", "/api/auth/signup", "/api/auth/me", "/api/auth/logout", "jwt", "token"],
            "backend", "reference", "Authentication endpoints: login, signup, me, logout with JWT tokens",
            &["auth-block", "crud-endpoints"]),
        entry("crud-endpoints", "CRUD Endpoints", "/docs#crud-endpoints",
            &["backend", "crud", "api"], &["/api/{entity}s", "list", "get", "create", "update", "delete", "pagination", "filter"],
            "backend", "reference", "Auto-generated CRUD endpoints for every entity: GET/POST/PATCH/DELETE",
            &["entity-block", "auth-endpoints"]),
        entry("health-endpoints", "Health Endpoints", "/docs#health-endpoints",
            &["backend", "health", "monitoring"], &["/api/health", "/api/_health", "status", "uptime", "version"],
            "backend", "reference", "Health check endpoints for monitoring and load balancers",
            &[]),
        entry("schema-endpoint", "Schema Endpoint", "/docs#schema-endpoint",
            &["backend", "schema", "introspection"], &["/api/schema", "entities", "fields", "types", "introspect"],
            "backend", "reference", "Returns full app schema: entities, fields, types, and relations as JSON",
            &["entity-block", "graphql-endpoint"]),
        entry("seed-endpoint", "Seed Endpoint", "/docs#seed-endpoint",
            &["backend", "seed", "data"], &["/api/_seed", "populate", "sample", "fake", "dev"],
            "backend", "reference", "POST to populate database with generated sample data for development",
            &["cli-seed"]),
        entry("sse-endpoint", "SSE Endpoint", "/docs#sse-endpoint",
            &["backend", "sse", "realtime"], &["/api/sse", "server-sent-events", "realtime", "live", "stream", "push"],
            "backend", "reference", "Server-Sent Events endpoint for real-time data push to clients",
            &["binding-syntax"]),
        entry("graphql-endpoint", "GraphQL Endpoint", "/docs#graphql-endpoint",
            &["backend", "graphql", "query"], &["/graphql", "query", "mutation", "subscription", "playground", "schema"],
            "backend", "reference", "GraphQL API with auto-generated queries, mutations, and playground UI",
            &["schema-endpoint", "crud-endpoints"]),
        entry("audit-endpoints", "Audit Endpoints", "/docs#audit-endpoints",
            &["backend", "audit", "fidelity"], &["/api/audit/trigger", "/api/audit/results", "/api/audit/verify", "fidelity", "compare"],
            "backend", "reference", "Audit system endpoints: trigger audit, retrieve results, verify fidelity scores",
            &["audit-system", "cli-audit"]),
        entry("hydra-endpoints", "Hydra Endpoints", "/docs#hydra-endpoints",
            &["backend", "hydra", "evolution"], &["/api/hydra/registry", "/api/hydra/evolve", "/api/hydra/candidates", "trust", "promote"],
            "backend", "reference", "Hydra block evolution endpoints: registry, evolve cycle, candidate blocks",
            &["hydra-system", "hydra-dashboard"]),
        entry("brain-endpoints", "Brain Endpoints", "/docs#brain-endpoints",
            &["backend", "brain", "ai"], &["/api/brain/stats", "/api/brain/suggest", "learning", "suggestion", "intelligence"],
            "backend", "reference", "Brain learning system endpoints: stats and AI-powered suggestions",
            &[]),
        entry("payment-endpoints", "Payment Endpoints", "/docs#payment-endpoints",
            &["backend", "payment", "stripe"], &["/api/checkout", "/api/payments", "stripe", "charge", "subscription", "invoice"],
            "backend", "reference", "Payment processing endpoints: checkout creation, payment status, webhooks",
            &["section-checkout", "section-pricing"]),
        entry("context-endpoint", "AI Context Protocol", "/docs#context-endpoint",
            &["backend", "ai", "context"], &["/api/_context", "ai", "context", "protocol", "llm", "prompt"],
            "backend", "reference", "AI Context Protocol endpoint: returns structured project context for LLM consumption",
            &["docs-index"]),
        entry("docs-index", "Docs Index Endpoint", "/docs#docs-index",
            &["backend", "docs", "index", "ai"], &["/api/docs/index", "/api/docs/search", "/api/docs/tags", "index", "search", "navigate"],
            "backend", "reference", "AI-optimized documentation index with search and tag filtering (this endpoint)",
            &["context-endpoint", "auto-docs"]),
    ]
}

fn advanced_dashboard_entries() -> Vec<Value> {
    vec![
        entry("auto-docs", "Auto-Generated API Docs", "/docs",
            &["advanced", "docs", "api"], &["docs", "documentation", "api", "auto-generated", "reference"],
            "advanced", "reference", "Auto-generated API documentation page from entity and route definitions",
            &["docs-index", "schema-endpoint"]),
        entry("design-system", "Design System Page", "/docs/design",
            &["advanced", "design", "ui"], &["design", "system", "components", "tokens", "theme", "preview"],
            "advanced", "reference", "Auto-generated design system showing all section types with live preview",
            &["style-block", "auto-docs"]),
        entry("entity-graph", "Entity Graph Visualization", "/docs/graph",
            &["advanced", "graph", "erd"], &["graph", "erd", "relationship", "entity", "visualization", "diagram"],
            "advanced", "reference", "Interactive entity relationship diagram with D3.js visualization",
            &["entity-block", "cli-graph"]),
        entry("block-explorer", "Block Explorer", "/blocks",
            &["advanced", "blocks", "3d"], &["blocks", "explorer", "3d", "browser", "interactive"],
            "advanced", "reference", "3D block browser for exploring all defined blocks and their relationships",
            &["hydra-dashboard"]),
        entry("zeus-tracer", "Zeus Request Tracer", "/zeus",
            &["advanced", "tracing", "debug"], &["zeus", "tracer", "request", "debug", "performance", "timing"],
            "advanced", "reference", "Request tracing dashboard showing method, path, status, duration, and query count",
            &["health-endpoints"]),
        entry("trust-dashboard", "Trust Dashboard", "/trust",
            &["advanced", "trust", "security"], &["trust", "profile", "score", "reputation", "verification"],
            "advanced", "reference", "Trust profile dashboard showing entity trust scores and verification status",
            &["hydra-system"]),
        entry("hydra-dashboard", "Hydra Dashboard", "/hydra",
            &["advanced", "hydra", "evolution"], &["hydra", "evolution", "cycle", "candidate", "promote", "block"],
            "advanced", "reference", "Hydra evolution cycle dashboard: view candidates, promote blocks, track trust",
            &["hydra-endpoints", "hydra-system"]),
    ]
}

fn scripting_entries() -> Vec<Value> {
    vec![
        entry("scriptcronus-overview", "ScriptCronus Overview", "/docs#scriptcronus",
            &["scripting", "scriptcronus"], &["scriptcronus", ".scriptcronus", "script", "automation", "vm"],
            "scripting", "reference", ".scriptcronus file format: event hooks, schedules, custom endpoints, webhooks",
            &["event-hooks", "schedules", "custom-endpoints", "webhooks-script", "builtins"]),
        entry("event-hooks", "Event Hooks", "/docs#event-hooks",
            &["scripting", "events", "hooks"], &["on", "Entity.create", "Entity.update", "Entity.delete", "event", "hook", "trigger"],
            "scripting", "reference", "React to entity lifecycle events: on Entity.create/update/delete",
            &["effects", "scriptcronus-overview"]),
        entry("schedules", "Scheduled Tasks", "/docs#schedules",
            &["scripting", "schedule", "cron"], &["schedule", "every", "interval", "cron", "periodic", "timer"],
            "scripting", "reference", "Define scheduled tasks: schedule \"name\" every:interval (1m, 1h, 1d)",
            &["worker-block", "scriptcronus-overview"]),
        entry("custom-endpoints", "Custom Script Endpoints", "/docs#custom-endpoints",
            &["scripting", "endpoint", "api"], &["endpoint", "GET", "POST", "PATCH", "DELETE", "auth", "public", "protected"],
            "scripting", "reference", "Define custom API endpoints in .scriptcronus with auth levels",
            &["api-block", "scriptcronus-overview"]),
        entry("webhooks-script", "Script Webhooks", "/docs#webhooks-script",
            &["scripting", "webhook"], &["webhook", "on", "path", "http", "incoming", "payload"],
            "scripting", "reference", "Handle incoming webhooks: on webhook \"/path\" with payload access",
            &["webhook-block", "scriptcronus-overview"]),
        entry("builtins", "Built-in Namespaces", "/docs#builtins",
            &["scripting", "stdlib", "api"], &["db", "http", "sse", "log", "format", "env", "auth", "builtin", "namespace"],
            "scripting", "reference", "7 built-in namespaces: db (CRUD), http (fetch), sse (push), log, format, env, auth",
            &["scriptcronus-overview"]),
        entry("vm-bytecode", "VM & Bytecode", "/docs#vm-bytecode",
            &["scripting", "vm", "runtime"], &["vm", "bytecode", "opcode", "fuel", "sandbox", "execution", "stack"],
            "scripting", "reference", "Stack-based VM with 30+ opcodes, fuel system for resource limiting, sandboxed execution",
            &["scriptcronus-overview", "script-promotion"]),
        entry("script-promotion", "Script Promotion", "/docs#script-promotion",
            &["scripting", "trust", "promotion"], &["promotion", "trust", "engine", "pipeline", "candidate", "stable"],
            "scripting", "reference", "Trust Engine promotion pipeline: candidate -> testing -> stable based on trust score",
            &["vm-bytecode", "hydra-system"]),
    ]
}

fn deploy_entries() -> Vec<Value> {
    vec![
        entry("docker-deploy", "Docker Deployment", "/docs#docker-deploy",
            &["deploy", "docker", "container"], &["docker", "dockerfile", "docker-compose", "container", "image", "build"],
            "deploy", "reference", "Deploy with Dockerfile and docker-compose: multi-stage build, volumes, networking",
            &["production-deploy", "deploy-block"]),
        entry("production-deploy", "Production Deployment", "/docs#production-deploy",
            &["deploy", "production", "server"], &["systemd", "nginx", "ssl", "tls", "letsencrypt", "reverse-proxy", "production"],
            "deploy", "reference", "Production deployment with systemd service, nginx reverse proxy, and SSL/TLS",
            &["docker-deploy", "deploy-block"]),
        entry("microservices", "Microservices Deployment", "/docs#microservices",
            &["deploy", "microservices", "gateway"], &["microservice", "gateway", "service", "split", "federation", "mesh"],
            "deploy", "reference", "Split app into microservices using deploy block: gateway, service discovery, federation",
            &["deploy-block", "docker-deploy"]),
    ]
}

fn advanced_system_entries() -> Vec<Value> {
    vec![
        entry("compiler-pipeline", "Compiler Pipeline", "/docs#compiler-pipeline",
            &["advanced", "compiler", "internals"], &["tokenizer", "parser", "ast", "contracts", "render", "html", "compile", "pipeline"],
            "advanced", "reference", "Full compilation pipeline: tokenizer -> parser -> AST -> contracts -> render -> HTML",
            &["audit-system", "compose-system"]),
        entry("hydra-system", "Hydra Evolution System", "/docs#hydra-system",
            &["advanced", "hydra", "evolution"], &["hydra", "evolution", "trust", "scoring", "promotion", "candidate", "block"],
            "advanced", "reference", "Block evolution system: trust scoring, A/B testing, automatic promotion of improvements",
            &["hydra-endpoints", "hydra-dashboard"]),
        entry("compose-system", "Compose System", "/docs#compose-system",
            &["advanced", "compose", "templates"], &["compose", "template", "saas", "landing", "dashboard", "builder", "entity-remap"],
            "advanced", "reference", "5 composition templates with entity remapping and builder API for project generation",
            &["cli-compose", "compiler-pipeline"]),
        entry("dump-system", "Dump System", "/docs#dump-system",
            &["advanced", "dump", "reverse"], &["dump", "html", "reverse-engineer", "convert", "import", "decompose"],
            "advanced", "reference", "Reverse-engineer HTML into .cronus section primitives for visual fidelity",
            &["cli-dump", "audit-system"]),
        entry("audit-system", "Audit System", "/docs#audit-system",
            &["advanced", "audit", "fidelity"], &["audit", "fidelity", "text-matching", "number-matching", "compare", "score"],
            "advanced", "reference", "Fidelity measurement system: text/number matching between source HTML and rendered output",
            &["audit-endpoints", "cli-audit"]),
        entry("cache-system", "Cache System", "/docs#cache-system",
            &["advanced", "cache", "performance"], &["cache", "lru", "ttl", "invalidation", "memory", "hit-rate"],
            "advanced", "reference", "LRU cache with TTL, auto-invalidation on entity mutations, configurable size",
            &["rate-limiting"]),
        entry("rate-limiting", "Rate Limiting", "/docs#rate-limiting",
            &["advanced", "security", "api"], &["rate-limit", "token-bucket", "per-ip", "throttle", "429", "retry-after"],
            "advanced", "reference", "Token bucket rate limiting with per-IP tracking and configurable limits",
            &["cache-system", "middleware-block"]),
        entry("i18n", "Internationalization", "/docs#i18n",
            &["advanced", "i18n", "locale"], &["i18n", "translation", "locale", "fallback", "language", "intl"],
            "advanced", "reference", "Translation system with locale fallback chains and dynamic string interpolation",
            &["app-block"]),
        entry("hmr", "Hot Module Replacement", "/docs#hmr",
            &["advanced", "hmr", "dev"], &["hmr", "hot", "reload", "watch", "file-watcher", "live-reload"],
            "advanced", "reference", "Hot Module Replacement with file watcher, version tracking, and instant page refresh",
            &["cli-run"]),
        entry("reactive", "Reactive State", "/docs#reactive",
            &["advanced", "reactive", "client"], &["reactive", "state", "signal", "effect", "computed", "store", "client-side"],
            "advanced", "reference", "Client-side reactive state management with signals, computed values, and effects",
            &["component-block", "binding-syntax"]),
    ]
}

// ──────────────────────────────────────────────
// Dynamic entries from AppState
// ──────────────────────────────────────────────

fn dynamic_entity_entries(state: &AppState) -> Vec<Value> {
    state.entities.iter().map(|e| {
        let field_names: Vec<String> = e.fields.iter().map(|f| f.name.clone()).collect();
        let relations: Vec<String> = e.fields.iter()
            .filter(|f| matches!(f.field_type, crate::parser::FieldType::Relation))
            .map(|f| format!("{} -> {}", f.name, f.reference.as_deref().unwrap_or("?")))
            .collect();
        let field_kws: Vec<&str> = field_names.iter().map(|s| s.as_str()).collect();

        json!({
            "id": format!("entity-{}", e.name.to_lowercase()),
            "title": format!("Entity: {}", e.name),
            "path": format!("/docs#entity-{}", e.name.to_lowercase()),
            "tags": ["dynamic", "entity", "model"],
            "keywords": field_kws,
            "category": "core",
            "depth": "instance",
            "summary": format!("{} entity with {} fields{}", e.name, e.fields.len(),
                if relations.is_empty() { String::new() } else { format!(", relations: {}", relations.join(", ")) }),
            "related": ["entity-block", "field-types"],
        })
    }).collect()
}

fn dynamic_page_entries(state: &AppState) -> Vec<Value> {
    state.pages.iter().map(|p| {
        let section_types: Vec<String> = p.sections.iter().map(|s| s.section_type.clone()).collect();
        let section_kws: Vec<&str> = section_types.iter().map(|s| s.as_str()).collect();
        let title = p.title.as_deref().unwrap_or(&p.route);

        json!({
            "id": format!("page-{}", p.route.trim_start_matches('/').replace('/', "-")),
            "title": format!("Page: {}", title),
            "path": p.route,
            "tags": ["dynamic", "page", &p.page_type],
            "keywords": section_kws,
            "category": "core",
            "depth": "instance",
            "summary": format!("{} page at {} with {} sections: {}", p.page_type, p.route, p.sections.len(), section_types.join(", ")),
            "related": ["page-block"],
        })
    }).collect()
}

// ──────────────────────────────────────────────
// Spec tag loading
// ──────────────────────────────────────────────

/// Reads all .spec.toml files from the specs directory and extracts tags.
/// Returns a map from spec name to its tags (e.g. "hero" -> ["section", "ui", "cta"]).
fn load_spec_tags() -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();
    let specs_dir = std::path::Path::new("specs");
    if !specs_dir.exists() {
        return result;
    }
    for subdir in &["core", "stdlib", "patterns"] {
        let path = specs_dir.join(subdir);
        let entries = match std::fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let file_path = entry.path();
            if !file_path.extension().map(|e| e == "toml").unwrap_or(false) {
                continue;
            }
            let content = match std::fs::read_to_string(&file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            // Extract name field
            let name = content.lines().find_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("name =") || trimmed.starts_with("name=") {
                    if let Some(start) = trimmed.find('"') {
                        if let Some(end) = trimmed[start + 1..].find('"') {
                            return Some(trimmed[start + 1..start + 1 + end].to_string());
                        }
                    }
                }
                None
            });
            // Extract tags array
            let tags: Vec<String> = content.lines().find_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("tags") && trimmed.contains('=') {
                    if let Some(bracket_start) = trimmed.find('[') {
                        if let Some(bracket_end) = trimmed.find(']') {
                            let inner = &trimmed[bracket_start + 1..bracket_end];
                            let parsed: Vec<String> = inner
                                .split(',')
                                .map(|s| s.trim().trim_matches('"').to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                            if !parsed.is_empty() {
                                return Some(parsed);
                            }
                        }
                    }
                }
                None
            }).unwrap_or_default();

            if let Some(name) = name {
                if !tags.is_empty() {
                    result.insert(name, tags);
                }
            }
        }
    }
    result
}

/// Merges spec tags into entries. Maps spec name to entry id "section-{name}".
fn merge_spec_tags(entries: &mut [Value], spec_tags: &HashMap<String, Vec<String>>) {
    for entry in entries.iter_mut() {
        let id = entry.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        // Match section entries: "section-hero" -> spec name "hero"
        if let Some(spec_name) = id.strip_prefix("section-") {
            if let Some(tags_from_spec) = spec_tags.get(spec_name) {
                // Merge: keep existing tags and add new ones from spec
                if let Some(existing) = entry.get("tags").and_then(|v| v.as_array()) {
                    let mut merged: Vec<String> = existing
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    for tag in tags_from_spec {
                        if !merged.contains(tag) {
                            merged.push(tag.clone());
                        }
                    }
                    entry["tags"] = json!(merged);
                }
            }
        }
    }
}

// ──────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────

pub(crate) fn generate_docs_index(state: &AppState) -> Value {
    let mut entries = Vec::new();
    entries.extend(core_entries());
    entries.extend(section_entries());
    entries.extend(cli_entries());
    entries.extend(backend_entries());
    entries.extend(advanced_dashboard_entries());
    entries.extend(scripting_entries());
    entries.extend(deploy_entries());
    entries.extend(advanced_system_entries());
    entries.extend(dynamic_entity_entries(state));
    entries.extend(dynamic_page_entries(state));

    // Enrich section entries with tags from .spec.toml files
    let spec_tags = load_spec_tags();
    if !spec_tags.is_empty() {
        merge_spec_tags(&mut entries, &spec_tags);
    }

    let total = entries.len();
    json!({
        "version": "1.0",
        "app": "CRONUS Language",
        "total_entries": total,
        "categories": ["core", "section", "backend", "cli", "deploy", "advanced", "scripting"],
        "entries": entries,
    })
}

pub(crate) fn search_docs_index(state: &AppState, query: &str) -> Value {
    let index = generate_docs_index(state);
    let q = query.to_lowercase();
    if q.is_empty() {
        return index;
    }

    let entries = index.get("entries").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let matched: Vec<Value> = entries.into_iter().filter(|e| {
        let check_str = |key: &str| -> bool {
            e.get(key).and_then(|v| v.as_str()).map(|s| s.to_lowercase().contains(&q)).unwrap_or(false)
        };
        let check_arr = |key: &str| -> bool {
            e.get(key).and_then(|v| v.as_array()).map(|arr| {
                arr.iter().any(|item| item.as_str().map(|s| s.to_lowercase().contains(&q)).unwrap_or(false))
            }).unwrap_or(false)
        };
        check_str("title") || check_str("summary") || check_str("id") || check_arr("tags") || check_arr("keywords")
    }).collect();

    let total = matched.len();
    json!({
        "query": query,
        "total_results": total,
        "entries": matched,
    })
}

pub(crate) fn get_docs_by_tag(state: &AppState, tag: &str) -> Value {
    let index = generate_docs_index(state);
    let t = tag.to_lowercase();
    if t.is_empty() {
        return index;
    }

    let entries = index.get("entries").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let matched: Vec<Value> = entries.into_iter().filter(|e| {
        e.get("tags").and_then(|v| v.as_array()).map(|arr| {
            arr.iter().any(|item| item.as_str().map(|s| s.to_lowercase() == t).unwrap_or(false))
        }).unwrap_or(false)
    }).collect();

    let total = matched.len();
    json!({
        "tag": tag,
        "total_results": total,
        "entries": matched,
    })
}
