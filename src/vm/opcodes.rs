//! Bytecode opcodes for the CRONUS VM
//!
//! Each opcode maps to a single VM instruction. DB operations cost 10 fuel,
//! everything else costs 1 fuel.

/// Bytecode instruction set for .scriptcronus execution
#[derive(Debug, Clone)]
pub enum OpCode {
    // === Stack operations ===
    /// Push a string value onto the stack
    PushStr(String),
    /// Push a numeric value onto the stack
    PushNum(f64),
    /// Push a boolean value onto the stack
    PushBool(bool),
    /// Push null onto the stack
    PushNull,
    /// Discard top of stack
    Pop,

    // === Variables ===
    /// Load a local variable by index onto the stack
    LoadLocal(u16),
    /// Pop top of stack and store into local variable by index
    StoreLocal(u16),

    // === Control flow ===
    /// Unconditional jump to instruction index
    Jump(u32),
    /// Pop top of stack; jump if falsy
    JumpIfFalse(u32),
    /// Stop execution
    Halt,

    // === Comparison / logic ===
    /// Pop two values, push (a == b)
    Equal,
    /// Pop two values, push (a != b)
    NotEqual,
    /// Pop two values, push (a < b)
    LessThan,
    /// Pop two values, push (a > b)
    GreaterThan,
    /// Pop two values, push (a <= b)
    LessOrEqual,
    /// Pop two values, push (a >= b)
    GreaterOrEqual,
    /// Pop two values, push (a && b)
    And,
    /// Pop two values, push (a || b)
    Or,
    /// Pop two values, push a.contains(b)
    Contains,

    // === Object access ===
    /// Pop object from stack, push object[field]
    GetField(String),

    // === Builtins ===
    /// Pop message, print to log
    Log,
    /// Query entity, push result array (cost: 10 fuel)
    DbQuery(String),
    /// Pop field map, insert into entity (cost: 10 fuel)
    DbInsert(String),
    /// Pop id + field map, update entity (cost: 10 fuel)
    DbUpdate(String),
    /// Pop id, delete from entity (cost: 10 fuel)
    DbDelete(String),

    // === Iteration ===
    /// Pop array from stack, begin iteration (push to iter_stack)
    IterBegin,
    /// Push next item from iterator, or jump to target if exhausted
    IterNext(u32),
    /// End iteration, pop iter_stack
    IterEnd,

    // === Response ===
    /// Pop body from stack, set response with given status code
    Respond(u16),

    // === Misc ===
    /// Push current unix timestamp
    Now,
    /// Push environment variable value
    EnvVar(String),
    /// Push auth user object
    AuthGetUser,
    /// Push auth role check result
    AuthCheckRole(String),

    // === Map building ===
    /// Push an empty JSON map onto the stack
    PushMap,
    /// Pop value, pop map, insert field, push map back
    MapInsert(String),
}

impl OpCode {
    /// Fuel cost for this instruction. DB ops cost 10, everything else 1.
    pub fn fuel_cost(&self) -> u32 {
        match self {
            OpCode::DbQuery(_)
            | OpCode::DbInsert(_)
            | OpCode::DbUpdate(_)
            | OpCode::DbDelete(_) => 10,
            _ => 1,
        }
    }
}
