# SDD — CRONUS Code Quality & Rust-Grade Architecture

> Inspirado na qualidade do Rust: se compila, funciona. Se não compila, o erro explica.
> Author: Zedd + Claude | Date: 2026-04-02 | Status: PROPOSED

---

## 0. Diagnóstico Brutal

### Números que importam

| Arquivo | Linhas | unwrap() | clone() | Testes | Veredicto |
|---------|--------|----------|---------|--------|-----------|
| main.rs | **12.436** | 68 | 136 | 0 | God file, zero tested |
| ui.rs | **12.900** | 6 | 19 | 0 | God file, zero tested |
| parser.rs | 3.981 | 27 | 50 | 44 | Aceitável mas ad-hoc |
| database.rs | 1.314 | 92 | 25 | 20 | Mutex.unwrap() perigoso |
| lint.rs | ~750 | 0 | ~10 | 25 | Bom |
| resolve.rs | ~900 | 0 | ~5 | 16 | Bom |
| **Total** | 64.253 | **187** | **245** | **143** | Precisa refatorar |

### vs Padrões Rust

| Critério | CRONUS | ripgrep/serde/axum | Gap |
|----------|--------|-------------------|-----|
| Arquivo máximo | 12.9K linhas | <1K | **13x** |
| unwrap() por arquivo | 92 (database.rs) | <5 | **18x** |
| Test coverage | ~2% | 30-50% | **15x** |
| Error types | String | Custom enum | Nenhum type safety |
| Module hierarchy | Flat (47 files) | Hierarchical | Nenhuma organização |
| Clippy | Não roda | Pedantic mode | Sem verificação |

### O que está BOM

- Parser tem 44 testes e boa cobertura
- Lint/resolve são módulos limpos com zero unwrap
- 143 testes total é uma base sólida
- Features funcionam (build passa, app roda)
- Rust garante memory safety mesmo com código feio

---

## 1. Plano de Refatoração (9 semanas, incremental)

### Princípio: NUNCA quebrar funcionalidade existente.

Cada PR:
1. Move código sem alterar lógica
2. `cargo build` + `cargo test` passa
3. App funciona igual
4. Re-exports mantêm compatibilidade

### Fase 1: Fundação (Semana 1-2)

**1.1 Criar lib.rs**

```rust
// src/lib.rs (~30 linhas)
pub mod compiler;
pub mod server;
pub mod cli;
pub mod ui;
pub mod database;
// ... re-exports

// Cargo.toml
[lib]
path = "src/lib.rs"
```

**Benefício:** Testes unitários em qualquer módulo. Reuso como biblioteca.

**1.2 Criar error.rs**

```rust
// src/error.rs
#[derive(Debug)]
pub enum CronusError {
    Parse(String),
    Resolve(Vec<ResolveError>),
    Lint(Vec<LintResult>),
    Database(String),
    Auth(String),
    Io(std::io::Error),
    Http(u16, String),
}

impl std::fmt::Display for CronusError { ... }
impl std::error::Error for CronusError { ... }
impl From<rusqlite::Error> for CronusError { ... }

pub type CronusResult<T> = Result<T, CronusError>;
```

**Benefício:** Substituir 187 unwrap() por `?` operator. Type-safe errors.

**1.3 Clippy + config**

```toml
# .cargo/config.toml
[build]
rustflags = ["-W", "clippy::unwrap_used"]

# Cargo.toml
[lints.clippy]
unwrap_used = "warn"
```

**Benefício:** Compilador avisa em CADA novo unwrap().

### Fase 2: Decompor main.rs (Semana 3-4)

```
main.rs (12.4K) → 
  main.rs (50 linhas — entry point only)
  cli/mod.rs (dispatch)
  cli/build.rs (cmd_build)
  cli/run.rs (cmd_run + server setup)
  cli/context.rs (cmd_context)
  cli/doctor.rs (cmd_doctor)
  cli/generate.rs (cmd_generate + templates)
  ... (20+ arquivos de ~100-300 linhas cada)
  server/handler.rs (handle_request)
  server/api.rs (CRUD handler)
  server/response.rs (json_response, html_response)
  server/docs.rs (render_auto_docs, render_design_system)
```

**Sequência:** Extrair 1 comando por PR, do mais simples ao mais complexo.

**Benefício:** De 12.4K → ~50 linhas no main.rs. Cada módulo testável.

### Fase 3: Decompor ui.rs (Semana 5-7)

```
ui.rs (12.9K) →
  ui/mod.rs (render_page dispatcher)
  ui/page/custom.rs (render_custom)
  ui/page/dashboard.rs
  ui/section/hero.rs
  ui/section/kpi.rs
  ui/section/chart.rs
  ui/section/table.rs (move de data_table.rs)
  ui/section/modal.rs
  ui/section/form.rs
  ui/component.rs
  ui/util.rs (accent_to_hex, etc.)
```

**Benefício:** De 12.9K → 20+ arquivos de 200-500 linhas. Navegável.

### Fase 4: Compiler namespace (Semana 8)

```
parser.rs (3.9K) →
  compiler/ast.rs (types: AstNode, EntityNode, FieldNode, etc.)
  compiler/tokenizer.rs (tokenize, Token, is_word_char)
  compiler/parser.rs (parse, parse_entity, parse_field, etc.)
  compiler/mod.rs (re-exports)

  # Re-export para backward compat:
  // src/parser.rs
  pub use crate::compiler::*;
```

**Benefício:** AST types separados do parser. Usados em 15+ módulos.

### Fase 5: Eliminar unwrap() (Semana 9)

**Substituir os 187 unwrap() mais críticos:**

```rust
// ANTES (database.rs):
let conn = self.conn.lock().unwrap();

// DEPOIS:
let conn = self.conn.lock()
    .map_err(|e| CronusError::Database(format!("Lock poisoned: {}", e)))?;
```

**Prioridade:** database.rs (92 unwrap) → main.rs (68) → parser.rs (27)

**Benefício:** Zero panics em runtime. Erros propagados com contexto.

---

## 2. Regras de Código (enforcement)

### Para código Rust do kernel:

```
INVIOLÁVEIS:
1. Zero #![allow(dead_code)] no top-level — limpar ou cfg(test)
2. Zero unwrap() em paths de request — sempre propagate errors
3. Cada módulo novo TEM testes — mínimo 5 por arquivo
4. Cada arquivo < 1000 linhas — split se crescer
5. cargo clippy --all-targets — must pass clean
6. cargo test — must pass ANTES de merge

RECOMENDADOS:
7. &str > String quando possível — evitar allocations
8. pub(crate) > pub — minimizar superfície pública
9. Error types > String — type safety
10. Named fields > positional — clareza
```

### Para a linguagem .cronus:

```
INVIOLÁVEIS:
1. Backward compatible — apps existentes NUNCA quebram
2. Zero ambiguidade — cada construct tem 1 interpretação
3. Vocabulário fechado — enums, não strings livres
4. Defaults seguros — opt-out de segurança
5. Compile-time validation — erros antes de runtime

RECOMENDADOS:
6. Linha = unidade atômica — 1 linha = 1 conceito
7. Localidade de referência — tudo no mesmo bloco
8. Doc-comments que geram docs — /// é semântico
9. Error messages com sugestões — nunca "error"
10. AI-parseable output — structured JSON para --ai
```

---

## 3. Métricas de Qualidade Target

### Antes (hoje):

```
main.rs:     12.436 linhas, 68 unwrap, 0 testes
ui.rs:       12.900 linhas, 6 unwrap, 0 testes
database.rs: 1.314 linhas, 92 unwrap, 20 testes
parser.rs:   3.981 linhas, 27 unwrap, 44 testes
Total:       64.253 linhas, 187 unwrap, 143 testes
Clippy:      nunca rodou
Coverage:    ~2%
```

### Depois (target):

```
Arquivo máximo: <1.000 linhas
unwrap():       <10 em todo o codebase
Testes:         300+ (cobertura 15%+)
Clippy:         clean em pedantic mode
Módulos:        hierárquicos (compiler/, server/, cli/, ui/)
Error types:    CronusError enum
Coverage:       20%+ nos módulos core
```

---

## 4. O Que NÃO Fazer

| Tentação | Por que é má ideia |
|----------|-------------------|
| Reescrever tudo de uma vez | Quebra tudo, leva meses, zero valor entregue |
| Mudar APIs públicas (.cronus syntax) | Apps existentes quebram |
| Adicionar framework (axum) ao invés de hyper | Rewrite do server, não refactor |
| Migrar para async everywhere | Complexidade sem benefício para SQLite |
| Adicionar macro magic | Dificulta debugging e compreensão |
| Over-engineer error types | CronusError com 5 variantes é suficiente |

---

## 5. Ordem de Execução

```
SEMANA 1:  lib.rs + error.rs + .cargo/config.toml
SEMANA 2:  Extrair 5 CLI commands simples (help, stats, parse, doctor, brief)
SEMANA 3:  Extrair 10 CLI commands médios (build, context, graph, export, etc.)
SEMANA 4:  Extrair server/ (handler, api, response, docs)
SEMANA 5:  ui/ — seções (hero, kpi, chart, table, modal)
SEMANA 6:  ui/ — pages (custom, dashboard, list, form)
SEMANA 7:  ui/ — dashboards (billing, payouts, security, etc.)
SEMANA 8:  compiler/ (ast, tokenizer, parser separados)
SEMANA 9:  Eliminar unwrap(), adicionar testes, clippy clean
```

**A cada semana:** cargo build + cargo test + app funciona = merge.
**Se quebrar:** revert imediato, investigar, corrigir, retry.
