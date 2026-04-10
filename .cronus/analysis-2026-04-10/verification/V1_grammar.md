# V1 — CRONUS Grammar Verification (source-of-truth)

Scope: verified strictly from `src/parser/tokenizer.rs`, `src/parser/ast.rs`, `src/parser/mod.rs`. No docs, no inference beyond what the code declares.

All line numbers below refer to the files as they exist on 2026-04-10.

---

## 1. KEYWORDS list (`tokenizer.rs`, lines 38–43)

```rust
pub(crate) const KEYWORDS: &[&str] = &[
    "app", "entity", "api", "page", "style", "service", "section",
    "import", "compose", "use", "merge", "on", "worker", "component",
    "middleware", "env", "test", "webhook", "constitution", "must", "never",
    "transition", "deploy",
];
```

Count: 23 keywords.

Note: `auth`, `layout`, `define`, `tailwind_config` are **not** in KEYWORDS — the parser matches them as `TokenKind::Identifier` (see section 7).

## 2. HTTP METHODS list (`tokenizer.rs`, line 45)

```rust
pub(crate) const METHODS: &[&str] = &["GET", "POST", "PATCH", "PUT", "DELETE"];
```

Count: 5.

## 3. TokenKind enum (`tokenizer.rs`, lines 5–29)

```rust
pub(crate) enum TokenKind {
    Keyword,
    Identifier,
    StringLit,
    Number,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Arrow,
    ColonPair,
    Plus,
    Comma,
    Price,
    Method,
    Path,
    EnvRef,
    Operator,
    Pipe,
    DocComment,
    Eof,
}
```

Count: 22 variants.

Notes from tokenizer body:
- `Operator` covers `==`, `=`, `!=`, `>=`, `<=`, `>`, `<` (lines 124–165).
- A standalone `!` is emitted as `Identifier` with value `"!"` (line 142) — used as the required shorthand.
- `#RRGGBB` hex colors are emitted as `Identifier` (lines 79–87).
- `#` otherwise starts a line comment (line 91).
- `$...` is `Price` (lines 174–184).
- `env(...)`, `role(...)`, `sum(...)`, `count(...)` and any `*:role(...)` / `*:env(...)` are consumed as a single word including the parens (lines 203–212).
- `env(...)` specifically becomes `EnvRef` (line 239).
- Words containing `:` (not starting with `/`) become `ColonPair` (line 241).
- `/...` paths become `Path` (lines 186–198, 243).
- DocComment = `///`-prefixed line, trimmed (lines 67–76).

## 4. AST top-level nodes — `AstNode` enum (`ast.rs`, lines 20–40)

```rust
pub enum AstNode {
    App(AppNode),
    Entity(EntityNode),
    Api(ApiNode),
    Page(PageNode),
    Style(StyleNode),
    Service(ServiceNode),
    Component(ComponentNode),
    Import(ImportNode),
    Event(EventNode),
    Worker(WorkerNode),
    Middleware(MiddlewareNode),
    Env(EnvNode),
    Test(TestNode),
    Compose(ComposeNode),
    Auth(AuthNode),
    Layout(LayoutNode),
    Define(DefineNode),
    Webhook(WebhookNode),
    Deploy(DeployNode),
}
```

Count: 19 top-level AST variants.

One-line descriptions (from the struct definitions that follow):
- `App` — `app "Name" { stack, port, database, constitution, tailwind_config }`.
- `Entity` — `entity Name [shared] { fields, transitions, effects }`.
- `Api` — `api /prefix { routes }`; each route carries method/path/auth/roles.
- `Page` — `page "/route" type:X entity:Y requires:Z { sections, components, config }`.
- `Style` — `style { theme, accent, radius, font, ... }`.
- `Service` — `service Name :port { config }`.
- `Component` — `component Name(params) layout:.. style:.. { items, state, tests, sections }`.
- `Import` — `import Alias from "path"`.
- `Event` — `on Name { actions… }` (freeform action strings, one line each).
- `Worker` — `worker Name queue:q { concurrency, retry, timeout, process }`.
- `Middleware` — `middleware Name { applies_to [...], config }`.
- `Env` — `env Name { key value … }`.
- `Test` — `test "Name" { action entity { body } -> expect N }`.
- `Compose` — `compose Name { uses, merges }`.
- `Auth` — `auth { entity, login, session, roles }`.
- `Layout` — `layout Name { sidebar {…} topbar {…} }`.
- `Define` — `define "Name" { section … }` reusable section bundle.
- `Webhook` — `webhook Entity { on event -> METHOD "url" header … }`.
- `Deploy` — `deploy mode { gateway {…}, service name {…} … }`.

No `Constitution` top-level node — `constitution` is parsed inline inside `app` and stored on `AppNode.constitution` (see section 7 / `parse_app`).

## 5. `FieldType` enum (`ast.rs`, lines 137–178)

```rust
pub enum FieldType {
    String,
    Text,
    Email,
    Url,
    Slug,
    Phone,
    Number,
    Money,
    Percentage,
    Boolean,
    Date,
    Ulid,
    Json,
    Enum,
    Ip,
    Relation,
}
```

Count: **16 variants**.

`from_str` mapping (source keywords → variant):

```rust
"string"     => String
"text"       => Text
"email"      => Email
"url"        => Url
"slug"       => Slug
"phone"      => Phone
"number"     => Number
"money"      => Money
"percentage" => Percentage
"boolean"    => Boolean
"date"       => Date
"ulid"       => Ulid
"json"       => Json
"enum"       => Enum
"ip"         => Ip
_            => String   // fallback default
```

Special cases:
- `Relation` has no keyword — it is produced only when the field line starts with `->` (`parse_field`, mod.rs 363–378).
- Any unrecognized type keyword silently becomes `String` (fallback at line 175). There is no error on unknown type names.
- `type[]` array suffix is detected in `parse_field` (mod.rs 385–393) and sets `FieldNode.array = true`.
- `type!` suffix strips the `!` and sets `required = true` (mod.rs 395–400).
- For string-family types (`String`, `Text`, `Email`, `Url`, `Slug`, `Phone`), the parser routes `min:`/`max:` to `min_length`/`max_length` instead of numeric `min`/`max` (mod.rs 456–463).

## 6. `HttpMethod` enum (`ast.rs`, lines 180–200)

```rust
pub enum HttpMethod {
    GET,
    POST,
    PATCH,
    PUT,
    DELETE,
}
```

Count: **5 variants**. `from_str` maps the same five uppercase strings; anything else returns `None` (and `parse_api` then defaults to `GET`, mod.rs 701).

## 7. Top-level parse dispatch (`parser/mod.rs`, `fn parse`, lines 111–185)

The main dispatch loop collects leading doc-comments then branches on `peek()`:

| Token kind | Value | Handler |
|---|---|---|
| Keyword | `import` | `parse_import()` → `AstNode::Import` |
| Keyword | `compose` | `parse_compose()` → `AstNode::Compose` |
| Keyword | `app` | `parse_app()` → `AstNode::App` (attaches pending doc) |
| Keyword | `entity` | `parse_entity()` → `AstNode::Entity` (attaches pending doc) |
| Keyword | `api` | `parse_api()` → `AstNode::Api` (attaches pending doc) |
| Keyword | `webhook` | `parse_webhook()` → `AstNode::Webhook` |
| Keyword | `page` | `parse_page()` → `AstNode::Page` (attaches pending doc) |
| Keyword | `style` | `parse_style()` → `AstNode::Style` |
| Keyword | `service` | `parse_service()` → `AstNode::Service` |
| Identifier | `define` | `parse_define()` → `AstNode::Define` |
| Keyword | `component` | `parse_component()` → `AstNode::Component` |
| Keyword | `on` | `parse_event()` → `AstNode::Event` |
| Keyword | `worker` | `parse_worker()` → `AstNode::Worker` |
| Keyword | `middleware` | `parse_middleware()` → `AstNode::Middleware` |
| Keyword | `env` | `parse_env()` → `AstNode::Env` |
| Keyword | `test` | `parse_test()` → `AstNode::Test` |
| Keyword | `deploy` | `parse_deploy()` → `AstNode::Deploy` |
| Identifier | `auth` | `parse_auth()` → `AstNode::Auth` |
| Identifier | `layout` | consumes `layout`, then `parse_layout()` → `AstNode::Layout` |
| Identifier | `tailwind_config` | inline handler: reads one StringLit, attaches to most-recent `App` node |
| anything else | — | prints warning `⚠ Line N: unknown top-level token '…' (skipped)` and advances |

Not top-level-dispatched (exist in KEYWORDS but handled only inside other blocks):
- `section`, `use`, `merge`, `transition`, `must`, `never`, `constitution` — used inside page/compose/entity/app bodies only.

## 8. Parse helpers inventory (`parser/mod.rs`)

All `fn parse_*` impls on `Parser` (plus the top-level `pub fn parse`):

| Function | Block | Approx. lines |
|---|---|---|
| `parse` (top) | Main dispatch over `AstNode` variants | 111–185 |
| `parse_import` | `import Alias from "path"` | 189–195 |
| `parse_compose` | `compose Name { use …, merge … }` | 199–241 |
| `parse_app` | `app "Name" { stack, port, database, constitution }` | 243–297 |
| `parse_entity` | `entity Name [shared] { fields, transitions, effects }` | 301–353 |
| `parse_field` | One field row inside an entity | 355–475 |
| `parse_transition` | `transition field { from -> to \| other }` | 479–556 |
| `parse_effect_block` | `on create/update[/field]/delete { log, notify, when… }` | 560–685 |
| `parse_api` | `api /prefix { routes }` | 689–724 |
| `parse_webhook` | `webhook Entity { on event -> METHOD "url" header … }` | 728–774 |
| `parse_auth` | `auth { entity, login, session, roles }` | 778–817 |
| `parse_layout` | `layout Name { sidebar {…} topbar {…} }` | 821–901 |
| `parse_page` | `page "/route" type:X { … }` | 905–1010 |
| `parse_section` | `section type [config] { … }` | 1014–1439 |
| `parse_section_item` | `item "text" [-> "href"] [attrs] [ { … } ]` | 1440–1530 |
| `parse_action_block` | `on click/submit/error/change { set, toast, navigate, … }` | 1532–1612 |
| `parse_plan` | `plan "Name" $price [featured] [features]` | 1614–1632 |
| `parse_binding` | `bind Entity { query, where, order, limit, offset, group, aggregate }` | 1636–1744 |
| `parse_style` | `style { theme, accent, radius, font, … }` | 1748–1774 |
| `parse_service` | `service Name :port { key … }` | 1778–1826 |
| `parse_define` | `define "Name" { section … }` | 1832–1856 |
| `parse_component` | `component Name(params) [attrs] { … }` | 1858–2107 |
| `parse_event` | `on Name { action lines… }` | 2108–2128 |
| `parse_worker` | `worker Name :queue { concurrency, retry, timeout, process }` | 2132–2159 |
| `parse_middleware` | `middleware Name { applies_to, config }` | 2163–2190 |
| `parse_env` | `env Name { key value … }` | 2194–2213 |
| `parse_test` | `test "Name" { action entity { body } -> expect N }` | 2217–2271 |
| `parse_array` | `[ item, item, … ]` (validates identifiers via P040/P041) | 2330–2346 |
| `parse_string_array` | `[ …raw tokens… ]` no validation | 2348–2359 |
| `parse_deploy` | `deploy mode { gateway {…}, service … }` | 2363–2445 |
| `parse_with_imports` (pub, module-level) | Driver that resolves `import` statements and merges | 2464–… |
| `parse_directory` (pub) | Reads all `.cronus` files under a dir and parses them | 2554–… |

Count of `fn parse_*` helpers on `Parser` (excluding top-level `parse` / `parse_with_imports` / `parse_directory`): **29**.

## 9. Field modifiers (`parse_field`, lines 419–453)

Relevant match arms after type parsing:

```rust
// colon-pair modifiers (default:"..", min:N, max:N, match:"regex")
match k.as_str() {
    "default" => { default_value = Some(v); }
    "min"     => { min = v.parse::<f64>().ok(); }
    "max"     => { max = v.parse::<f64>().ok(); }
    "match"   => { pattern = Some(v); }
    _ => {}
}
```

```rust
// bare identifier modifiers
match mod_val.as_str() {
    "!"          => required = true,
    "required"   => required = true,
    "unique"     => unique = true,
    "sensitive"  => sensitive = true,
    "optional"   => optional = true,
    "searchable" => searchable = true,
    "index"      => index = true,
    "featured"   => featured = true,
    "formatted"  => formatted = true,
    _ => {}
}
```

Additional forms handled outside the match:
- `type!` suffix on the type word (e.g. `email!`) → `required = true` (395–400).
- `type[]` suffix → `array = true` (385–393).
- `[A, B, C]` immediately after the type → `enum_values = Some([...])` via `parse_array` (422–423).

Modifiers that the docs may mention but which are **not** handled by name in `parse_field`:
- `onupdate:` — no match arm. (Only `default`, `min`, `max`, `match` are recognized as colon-pairs.)
- `searchable` is recognized; `indexed` is **not** (`index` is the recognized identifier).
- Any unknown colon-pair key is silently dropped (falls through the `_ => {}` arm).
- Any unknown bare identifier is silently dropped.

The field loop terminates when either an `RBrace`/`EOF` appears or when the next token is on a different source line (mod.rs 419–420).

## 10. Layout / sidebar parser behavior (`parse_layout`, lines 821–901)

Structure accepted:

```text
layout Name {
    sidebar {
        brand "..."                        // sidebar_config["brand"]
        [nav] "Label" -> "/route" [icon:x] [requires:role]
        divider
        …
    }
    topbar {
        key value                           // topbar_config[key] = value
        search placeholder:"…"              // topbar_config["search_placeholder"]
        any:colon:pair                      // topbar_config[key] = value
    }
}
```

Key facts from the code:
- Line 838–844: the loop matches **either** `nav` as an `Identifier`, **or** a bare `StringLit`. If `nav` is present it is consumed; otherwise the string label is used directly.

```rust
} else if self.matches(TokenKind::Identifier, Some("nav"))
    || self.peek().kind == TokenKind::StringLit
{
    // `nav` keyword is optional: `"Label" -> "/route"` also works
    if self.matches(TokenKind::Identifier, Some("nav")) {
        self.advance();
    }
    let label = self.expect(TokenKind::StringLit)?.value;
```

→ **Confirmed**: `nav` is optional after the 2026-04-10 fix. Both forms produce a `LayoutNavItem` with the same fields.

- After the label, an optional `-> "/route"` (or Path token) supplies `route` (847–853).
- Trailing `icon:x` and `requires:role` colon-pairs are consumed (856–860).
- `divider` (identifier) inside sidebar emits a `LayoutNavItem { is_divider: true }` with empty label/route (862–867).
- `brand "..."` sets `sidebar_config["brand"]`.
- Everything else inside `sidebar { }` is silently skipped (`self.advance()`, 868–870).
- Inside `topbar { }`, any ColonPair goes directly into `topbar_config`; a bare identifier followed by a ColonPair is folded into `topbar_config["<ident>_<k>"] = v` (875–892), which is how `search placeholder:"…"` becomes `topbar_config["search_placeholder"]`.
- There is no handling of `title`, `actions`, `user`, or any other sub-key — unknown inputs are advanced past silently.

## 11. Entity block supported content (`parse_entity`, lines 301–353)

Inside `entity Name [shared] { … }` the parser accepts:

1. **Fields** — any `Identifier`/`Keyword` token that is not `transition` or `on` is handed to `parse_field` (340–345).
2. **Transitions** — `transition field { from -> to [| to …] }` (via `parse_transition`, 479–556). The referenced field must exist and be of `FieldType::Enum`; all `from`/`to` values must be in the field's `enum_values` or parsing errors out (lines 495–546).
3. **Effects** — `on create { … }` / `on update [field] { … }` / `on delete { … }` (via `parse_effect_block`, 560–685). Supported actions inside: `log "msg"`, `notify "provider" "channel" "message"`, and a generic fallback that collects trailing string literals. `when "value" { … }` blocks attach a `condition` to each action.
4. **`shared` modifier** — recognized before the opening brace (309–314) and sets `EntityNode.shared = true`.
5. **Doc-comments** (`///`) before any field are attached via `collect_doc_comments` (323–324, 343).

Not accepted inside an entity block:
- No `enum` keyword standalone — enums are only a field *type* with `[A,B,C]` values.
- No `owner_id` keyword — nothing special in the parser; it would just be a field named `owner_id`.
- No `indexes`, `constraints`, `unique(a,b)`, etc. — the only index-like modifier is the per-field `index` identifier.
- No `shared { remote_url }` block — `shared` is only a boolean flag; `EntityNode.remote_url` exists in the struct (ast.rs 83) but is not populated anywhere in the parser (set to `None` at line 352 and never touched).

## 12. API block supported content (`parse_api`, lines 689–724)

Inside `api /prefix { … }` each route line is:

```text
routeName METHOD /path [auth:value] [ [roles, …] ]
```

- `prefix` is read with `expect(TokenKind::Path)` (line 691), i.e. it must begin with `/`.
- A route starts with an `Identifier` (the route name, 698–699).
- Then `expect(TokenKind::Method)` — only `GET`, `POST`, `PATCH`, `PUT`, `DELETE` (700–701). Unknown methods fall back to `GET` via `HttpMethod::from_str`.
- Then `expect(TokenKind::Path)` for the route path (702).
- Any number of trailing `ColonPair` tokens — only `auth:<value>` is stored; all other keys are discarded (707–710).
- A trailing `[ … ]` array becomes `roles` via `parse_array` (712–714). `parse_array` validates each identifier against P040/P041 rules.
- **Doc-comments** (`///`) immediately before a route line become `RouteNode.doc` (697, 716).

Not handled inside an API block: no `middleware:`, no request/response schemas, no inline validators — keys other than `auth` in colon-pairs are silently ignored.

## 13. Page block supported content (`parse_page`, lines 905–1010)

Signature line: `page "/route" [type:X] [entity:Y] [requires:Z] [any:other] { … }`.

- Route is a `StringLit` (907).
- Any inline `ColonPair` tokens before the brace are read into `inline_config` and also extracted into `page_type` (from `type:`) and `entity` (from `entity:`) (913–918). Default `page_type` is `"custom"`.
- `requires` is pulled out of the final `config` map into `PageNode.requires` (1008).

Inside the body:

| Input | Effect |
|---|---|
| `use ComponentName` | Appends name to `PageNode.components` (932–936). |
| `title "…"` | Sets `PageNode.title` (937–939). |
| `columns [..]`, `stats [..]`, `actions [..]`, `fields [..]` | Stored in `config` as comma-joined strings (940–946). |
| `search …` / `filters …` | Stored in `config`; array or single value (947–954). |
| `recent Entity [limit:N]` | `config["recent"] = Entity`, `config["recent_limit"] = N` (955–962). |
| `section type [attrs] { … }` (Keyword `section`) | `parse_section` result pushed to `sections` (963–966). |
| PascalCase identifier + props | Treated as a component invocation and stored as a synthetic `SectionNode` with `section_type = <Name>` and `config["_component"] = <Name>` (967–998). |
| Bare `ColonPair` | Added to `config` (999–1001). |
| Anything else | Skipped (`self.advance()`, 1002–1004). |
| Leading `///` lines | Become `doc` on the next section via `collect_doc_comments` (929, 965, 997). |

`page_type` is stored verbatim as a string — the parser does not restrict or validate it (so values like `list`, `form`, `dashboard`, `custom`, or any free-form string all pass through).

## 14. Blocks the parser accepts — presence of `parse_*` functions

For each block frequently claimed by docs, whether a dedicated `parse_*` helper exists:

| Block | Helper? | Notes |
|---|---|---|
| `hydra` | **No** — no `parse_hydra` anywhere; no AST node. |
| `compose` | Yes — `parse_compose` (199–241), `AstNode::Compose`. |
| `use` | No dedicated helper — handled inline inside `parse_page` (932–936) and inside `parse_compose`. Not a standalone top-level node. |
| `merge` | No dedicated helper — handled inside `parse_compose` only. |
| `webhook` | Yes — `parse_webhook` (728–774), `AstNode::Webhook`. |
| `constitution` | No standalone helper — parsed inline inside `parse_app` (272–289), stored on `AppNode.constitution`. Not a top-level `AstNode` variant. |
| `worker` | Yes — `parse_worker` (2132–2159). |
| `middleware` | Yes — `parse_middleware` (2163–2190). |
| `env` | Yes — `parse_env` (2194–2213). |
| `test` | Yes — `parse_test` (2217–2271). |
| `service` | Yes — `parse_service` (1778–1826). |
| `deploy` | Yes — `parse_deploy` (2363–2445). |
| `import` | Yes — `parse_import` (189–195). |

Additional blocks with their own `parse_*` helpers not in the request list:
- `parse_auth` (778–817) — top-level `auth { … }`, dispatched on `Identifier("auth")`.
- `parse_layout` (821–901) — top-level `layout Name { … }`, dispatched on `Identifier("layout")`.
- `parse_define` (1832–1856) — top-level `define "Name" { section … }`, dispatched on `Identifier("define")`.
- `parse_component` (1858–2107) — top-level `component Name(params) { … }`, dispatched on `Keyword("component")`.
- `parse_transition` (479–556) — only inside `entity` bodies.
- `parse_effect_block` (560–685) — only inside `entity` bodies (`on create/update/delete`).
- `parse_event` (2108–2128) — top-level `on Name { … }` freeform event, dispatched on `Keyword("on")`.
- `parse_section` (1014–1439), `parse_section_item` (1440–1530), `parse_action_block` (1532–1612), `parse_plan` (1614–1632), `parse_binding` (1636–1744) — only inside page/component/section bodies.
- `parse_style` (1748–1774) — top-level `style { … }`.
- Also: `tailwind_config "…"` is handled inline in the top-level dispatch (mod.rs 165–174) with no helper — it attaches the JS string to the last `App` node.

## 15. SQL reserved words (`validate_identifier`, lines 2311–2315)

```rust
const SQL_RESERVED: &[&str] = &[
    "SELECT", "DROP", "INSERT", "DELETE", "UPDATE", "TABLE", "FROM",
    "WHERE", "OR", "AND", "UNION", "ALTER", "CREATE", "INDEX", "EXEC",
    "EXECUTE", "INTO", "VALUES", "SET", "NULL", "TRUE", "FALSE",
];
```

Count: 22 reserved words. Comparison is case-insensitive (`name.to_uppercase()`, line 2317).

Additional P040 rules enforced by `validate_identifier` (lines 2276–2326):
- Must start with an ASCII letter (`[a-zA-Z]`).
- Subsequent chars must be `[a-zA-Z0-9_]`.
- Maximum length: 64 characters.
- Empty identifiers are rejected.

`validate_identifier` is called from `parse_entity` (for entity names, 307), `parse_field` (for field names, 360), and `parse_array` (for unquoted enum values, 2338–2340). It is **not** called for route names, component names, page routes, section types, worker/middleware/env names, or colon-pair keys — those can technically contain any non-whitespace characters the tokenizer allows.

---

## Summary counts

- **KEYWORDS**: 23
- **METHODS**: 5
- **TokenKind variants**: 22
- **AstNode variants**: 19
- **FieldType variants**: 16
- **HttpMethod variants**: 5
- **`fn parse_*` helpers on `Parser`**: 29 (excluding top-level `parse`, `parse_with_imports`, `parse_directory`)
- **SQL reserved words**: 22
