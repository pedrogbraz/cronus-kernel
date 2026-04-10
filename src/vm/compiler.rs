//! AST-to-bytecode compiler for .scriptcronus
//!
//! Compiles ScriptFile AST nodes into Vec<OpCode> bytecode sequences.
//! Each ScriptBlock produces one bytecode program (Vec<OpCode>).

use crate::scripting::ast::*;
use super::opcodes::OpCode;

/// Compiler state — tracks local variable name-to-index mapping
pub struct Compiler {
    locals: Vec<String>,
}

/// A compiled script block with metadata
#[derive(Debug, Clone)]
pub struct CompiledBlock {
    pub kind: CompiledBlockKind,
    pub bytecode: Vec<OpCode>,
}

/// Identifies what kind of block was compiled
#[derive(Debug, Clone)]
pub enum CompiledBlockKind {
    OnEvent { entity: String, event: String },
    OnWebhook { path: String },
    Schedule { name: String, interval: String },
    Endpoint { method: String, path: String, auth: Option<String> },
}

impl Compiler {
    pub fn new() -> Self {
        Self { locals: Vec::new() }
    }

    /// Compile an entire ScriptFile into a list of compiled blocks
    pub fn compile(script: &ScriptFile) -> Vec<CompiledBlock> {
        script.blocks.iter().map(|block| {
            let mut compiler = Compiler::new();
            match block {
                ScriptBlock::OnEvent(b) => CompiledBlock {
                    kind: CompiledBlockKind::OnEvent {
                        entity: b.entity.clone(),
                        event: b.event.clone(),
                    },
                    bytecode: compiler.compile_block_body(&b.body),
                },
                ScriptBlock::OnWebhook(b) => CompiledBlock {
                    kind: CompiledBlockKind::OnWebhook { path: b.path.clone() },
                    bytecode: compiler.compile_block_body(&b.body),
                },
                ScriptBlock::Schedule(b) => CompiledBlock {
                    kind: CompiledBlockKind::Schedule {
                        name: b.name.clone(),
                        interval: b.interval.clone(),
                    },
                    bytecode: compiler.compile_block_body(&b.body),
                },
                ScriptBlock::Endpoint(b) => CompiledBlock {
                    kind: CompiledBlockKind::Endpoint {
                        method: b.method.clone(),
                        path: b.path.clone(),
                        auth: b.auth.clone(),
                    },
                    bytecode: compiler.compile_block_body(&b.body),
                },
            }
        }).collect()
    }

    fn compile_block_body(&mut self, stmts: &[Statement]) -> Vec<OpCode> {
        let mut code = self.compile_statements(stmts);
        code.push(OpCode::Halt);
        code
    }

    fn compile_statements(&mut self, stmts: &[Statement]) -> Vec<OpCode> {
        let mut code = Vec::new();
        for stmt in stmts {
            code.extend(self.compile_statement(stmt));
        }
        code
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Vec<OpCode> {
        match stmt {
            Statement::Let { name, value } => {
                let mut code = self.compile_expr(value);
                let idx = self.get_or_create_local(name);
                code.push(OpCode::StoreLocal(idx));
                code
            }

            Statement::Log { message } => {
                let mut code = self.compile_expr(message);
                code.push(OpCode::Log);
                code
            }

            Statement::DbCreate { entity, fields } => {
                let mut code = Vec::new();
                code.push(OpCode::PushMap);
                for (key, val_expr) in fields {
                    code.extend(self.compile_expr(val_expr));
                    code.push(OpCode::MapInsert(key.clone()));
                }
                code.push(OpCode::DbInsert(entity.clone()));
                code
            }

            Statement::DbUpdate { entity, id, fields } => {
                let mut code = Vec::new();
                // Push the id first (will be below the map on stack)
                code.extend(self.compile_expr(id));
                // Build the field map
                code.push(OpCode::PushMap);
                for (key, val_expr) in fields {
                    code.extend(self.compile_expr(val_expr));
                    code.push(OpCode::MapInsert(key.clone()));
                }
                code.push(OpCode::DbUpdate(entity.clone()));
                code
            }

            Statement::DbDelete { entity, id } => {
                let mut code = self.compile_expr(id);
                code.push(OpCode::DbDelete(entity.clone()));
                code
            }

            Statement::SseBroadcast { event: _, data: _ } => {
                // SSE not yet supported in bytecode VM — emit no-op
                Vec::new()
            }

            Statement::For { var, iter, body } => {
                let mut code = Vec::new();
                // Compile the iterable expression and begin iteration
                code.extend(self.compile_expr(iter));
                code.push(OpCode::IterBegin);

                // Placeholder for IterNext — we'll patch the jump target
                let iter_next_pos = code.len();
                code.push(OpCode::IterNext(0)); // placeholder

                // Store current item into the loop variable
                let var_idx = self.get_or_create_local(var);
                code.push(OpCode::StoreLocal(var_idx));

                // Compile body
                code.extend(self.compile_statements(body));

                // Jump back to IterNext
                code.push(OpCode::Jump(iter_next_pos as u32));

                // End of loop — IterNext jumps here when exhausted
                let end_pos = code.len();
                code[iter_next_pos] = OpCode::IterNext(end_pos as u32);

                code.push(OpCode::IterEnd);
                code
            }

            Statement::If { condition, then_body, else_body } => {
                let mut code = Vec::new();
                // Compile condition
                code.extend(self.compile_expr(condition));

                // JumpIfFalse to else branch (placeholder)
                let jump_else_pos = code.len();
                code.push(OpCode::JumpIfFalse(0)); // placeholder

                // Compile then body
                code.extend(self.compile_statements(then_body));

                if else_body.is_empty() {
                    // Patch: JumpIfFalse jumps to right after then body
                    let end_pos = code.len();
                    code[jump_else_pos] = OpCode::JumpIfFalse(end_pos as u32);
                } else {
                    // Jump over else body (placeholder)
                    let jump_end_pos = code.len();
                    code.push(OpCode::Jump(0)); // placeholder

                    // Patch: JumpIfFalse jumps to else body
                    let else_start = code.len();
                    code[jump_else_pos] = OpCode::JumpIfFalse(else_start as u32);

                    // Compile else body
                    code.extend(self.compile_statements(else_body));

                    // Patch: Jump over else jumps to end
                    let end_pos = code.len();
                    code[jump_end_pos] = OpCode::Jump(end_pos as u32);
                }

                code
            }

            Statement::Respond { status, body, headers: _ } => {
                let mut code = self.compile_expr(body);
                code.push(OpCode::Respond(*status));
                code
            }

            Statement::ExprStatement(expr) => {
                let mut code = self.compile_expr(expr);
                // Discard result
                code.push(OpCode::Pop);
                code
            }
        }
    }

    fn compile_expr(&mut self, expr: &Expr) -> Vec<OpCode> {
        match expr {
            Expr::StringLit(s) => vec![OpCode::PushStr(s.clone())],
            Expr::NumberLit(n) => vec![OpCode::PushNum(*n)],
            Expr::BoolLit(b) => vec![OpCode::PushBool(*b)],
            Expr::Now => vec![OpCode::Now],
            Expr::EnvVar(key) => vec![OpCode::EnvVar(key.clone())],

            Expr::Path(parts) => {
                if parts.len() == 1 {
                    // Simple variable reference
                    if let Some(idx) = self.find_local(&parts[0]) {
                        vec![OpCode::LoadLocal(idx)]
                    } else {
                        // Could be an unresolved variable — push null
                        vec![OpCode::PushNull]
                    }
                } else {
                    // Multi-part path: load root, then GetField for each part
                    let mut code = Vec::new();
                    if let Some(idx) = self.find_local(&parts[0]) {
                        code.push(OpCode::LoadLocal(idx));
                    } else {
                        // "event" and other implicit roots — load as string for the executor
                        // to resolve via context
                        code.push(OpCode::PushStr(parts[0].clone()));
                    }
                    for part in &parts[1..] {
                        code.push(OpCode::GetField(part.clone()));
                    }
                    code
                }
            }

            Expr::DbQuery { entity, filters: _, order: _, limit: _ } => {
                // v1: simple query without filter compilation
                vec![OpCode::DbQuery(entity.clone())]
            }

            Expr::DbCount { entity, filters: _ } => {
                // Reuse DbQuery for now, executor handles count vs query
                vec![OpCode::DbQuery(entity.clone())]
            }

            Expr::HttpCall { .. } => {
                // HTTP not yet in bytecode VM — push null
                vec![OpCode::PushNull]
            }

            Expr::FormatCsv { .. } | Expr::FormatJson { .. } => {
                // Format ops not yet in bytecode VM — push null
                vec![OpCode::PushNull]
            }

            Expr::BinOp { left, op, right } => {
                let mut code = self.compile_expr(left);
                code.extend(self.compile_expr(right));
                code.push(match op {
                    BinOperator::Eq => OpCode::Equal,
                    BinOperator::Ne => OpCode::NotEqual,
                    BinOperator::Lt => OpCode::LessThan,
                    BinOperator::Gt => OpCode::GreaterThan,
                    BinOperator::Lte => OpCode::LessOrEqual,
                    BinOperator::Gte => OpCode::GreaterOrEqual,
                    BinOperator::And => OpCode::And,
                    BinOperator::Or => OpCode::Or,
                    BinOperator::Contains => OpCode::Contains,
                });
                code
            }

            Expr::AuthCheckRole(role) => vec![OpCode::AuthCheckRole(role.clone())],
            Expr::AuthGetUser => vec![OpCode::AuthGetUser],
        }
    }

    fn get_or_create_local(&mut self, name: &str) -> u16 {
        if let Some(idx) = self.find_local(name) {
            idx
        } else {
            let idx = self.locals.len() as u16;
            self.locals.push(name.to_string());
            idx
        }
    }

    fn find_local(&self, name: &str) -> Option<u16> {
        self.locals.iter().position(|n| n == name).map(|i| i as u16)
    }
}
