# `cronus mcp` — Model Context Protocol server

`cronus mcp` runs a [Model Context Protocol](https://modelcontextprotocol.io) server over
stdio. It gives LLM clients (Claude Code, Claude Desktop, Cursor, any MCP client) the kernel's
own validator, parser summary, grammar lists and project context, so a model can write a
`.cronus` app and check it in the same loop.

Source: `src/cli/mcp.rs` (transport and methods), `src/cli/mcp/tools.rs` (tools),
`src/cli/mcp/ast_summary.rs`, `src/cli/mcp/error_codes.rs`. Contract tests:
`tests/mcp_cli.rs` (runs the real binary).

## Register

The `cronus` binary must be on `PATH` (or use its absolute path). The server works in the
directory the client starts it in; the `context` tool can only read inside that directory.

### Claude Code

```bash
claude mcp add cronus -- cronus mcp
# project-scoped (writes .mcp.json for the team):
claude mcp add --scope project cronus -- cronus mcp
```

### Claude Desktop

Edit `claude_desktop_config.json` (macOS: `~/Library/Application Support/Claude/`,
Windows: `%APPDATA%\Claude\`):

```json
{
  "mcpServers": {
    "cronus": {
      "command": "/usr/local/bin/cronus",
      "args": ["mcp"],
      "cwd": "/path/to/your/cronus-project"
    }
  }
}
```

Claude Desktop does not start servers in your project directory unless told to; without
`cwd`, only the source-based tools (`validate`, `parse_ast`, lists, `new_app`) are useful.

### Cursor

`.cursor/mcp.json` in the project:

```json
{ "mcpServers": { "cronus": { "command": "cronus", "args": ["mcp"] } } }
```

## Protocol

- Transport: stdio. One JSON-RPC 2.0 message per line (UTF-8, no embedded newlines) on
  stdin and stdout. Logs go to stderr only; stdout carries nothing but responses.
- Protocol versions: `2025-11-25`, `2025-06-18`, `2025-03-26`, `2024-11-05`.
  `initialize` echoes the client's version when supported, otherwise answers with
  `2025-11-25`.
- Methods: `initialize`, `ping`, `tools/list`, `tools/call`, `resources/list`,
  `resources/read`, `resources/templates/list` (empty). Notifications
  (`notifications/initialized`, `notifications/cancelled`, …) are accepted and never
  answered. Capabilities: `tools`, `resources` (no subscriptions, no list changes).
- Errors: invalid JSON or UTF-8 `-32700` (id `null`); not an object, batch array, missing
  `method`, wrong `jsonrpc`, bad `id` `-32600`; unknown method `-32601`; unknown tool,
  unknown resource, missing `uri` or non-object `arguments` `-32602`. Lines over 16 MiB are
  rejected with `-32600`.
- Failures a model can fix (missing or mistyped arguments, source that does not parse in
  `parse_ast`, unknown template or error code, path outside the working directory) are tool
  results with `isError: true` and an explanation in `content`.
- EOF on stdin exits with code 0.

Tool results carry a `text` content item; JSON tools also set `structuredContent` to the same
value.

## Tools

All tools are read-only and never write files.

| Tool | Arguments | Returns |
|---|---|---|
| `validate` | `source` (string, required), `file` (label for locations, default `app.cronus`; never read) | Exactly the `cronus build --ai` JSON (strict profile, schema in LANGUAGE.md §15.9). An invalid app is a normal result with `valid: false`. |
| `parse_ast` | `source` (required) | Compact JSON: `app`, `auth`, `entities` (fields with `type`, `required`, `modifiers`, `enum_values`, `reference`, `default`; `transitions`, `effects`), `apis` (routes with `method`, `path`, `auth`, `roles`), `pages` (sections with `binding`, item count, `config`, actions), `layouts`, `style`, `other_blocks`, `stats`. |
| `list_sections` | none | `builtin` canonical section types, `aliases` with their `canonical` name, `families` (cronus-ui). Same tables as `cargo test context_grammar`. |
| `list_field_types` | none | `field_types` plus relation, enum and required syntax. |
| `list_families` | none | cronus-ui `families` and `count`. |
| `explain_error` | `code` (required, case-insensitive) | `code`, `severity`, `category`, `description`, `example_fix`. Covers every code in LANGUAGE.md §15.9 (a test keeps them equal). |
| `context` | `path` (optional directory or `.cronus` file inside the working directory) | Markdown, same as `cronus context --for-claude`: project summary, current build errors, grammar and types. A directory uses `app.cronus`, else its first `.cronus` file. |
| `new_app` | `template` (optional, default `saas`), `name` (optional app name) | `source` of the `cronus new` template with the `app "…"` name replaced. |

**Path confinement.** `context` rejects any `path` with a `..` component, and any path that
resolves (after following symlinks) outside the server's working directory, including
absolute paths. Nothing else reads the filesystem.

## Resources

| URI | Content |
|---|---|
| `cronus://llms-full.txt` | Canonical grammar, valid section and field types, a clean example, never-do list |
| `cronus://language` | `LANGUAGE.md`, the long-form language reference |

Both are embedded in the binary, so they always match the kernel version that serves them.

## Examples

Handshake:

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"demo","version":"1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
```

Validate a source with a type typo:

```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"validate","arguments":{"source":"entity Task {\n  title strin!\n}\n"}}}
```

`structuredContent` of the result (abridged):

```json
{"schema_version":1,"valid":false,"exit_code":1,"file":"app.cronus",
 "errors":[{"code":"TYPE_001","severity":"error","category":"type",
   "location":{"file":"app.cronus","line":2,"col":9},
   "fix":{"action":"replace","target":"strin","replacement":"string"}}],
 "warnings":[]}
```

Explain a code:

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"explain_error","arguments":{"code":"CONTRACT_004"}}}
```

Project context for a subdirectory:

```json
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"context","arguments":{"path":"apps/crm"}}}
```

Try it by hand:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cronus mcp
```

A good authoring loop for a model: read `cronus://llms-full.txt`, start from `new_app`, edit,
call `validate`, apply each `fix.replacement` or `explain_error` hint, repeat until
`valid: true`, then save `app.cronus` and run `cronus run`.
