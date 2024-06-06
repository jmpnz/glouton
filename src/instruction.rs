//! Glouton IR instructions.
use std::fmt;

/// Types used in the IR.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Type {
    // Empty type.
    #[default]
    Unit,
    // Integers, defaults to i32.
    Int,
    // Booleans.
    Bool,
    // Characters.
    Char,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => write!(f, ""),
            Self::Int => write!(f, "int"),
            Self::Bool => write!(f, "bool"),
            Self::Char => write!(f, "char"),
        }
    }
}

/// Literal values.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Literal {
    /// Empty value.
    #[default]
    Empty,
    /// Integers
    Int(i32),
    /// Booleans.
    Bool(bool),
    /// Characters.
    Char(char),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "NONE"),
            Self::Int(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::Char(value) => write!(f, "{value}"),
        }
    }
}
/// Symbol references are used as an alternative to variable names.
pub struct SymbolRef(usize /* Reference */, Type);

/// Symbols can represent variable or function names.
type Symbol = (String, Type);

/// Labels are used to designate branch targets in control flow operations.
///
/// Each label is a relative offset to the target branch first instruction.
pub struct Label(usize);

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "__LABEL_{}", self.0)
    }
}

/// Every value in the intermediate representation is either a symbol reference
/// to a storage location or a literal value.
enum Value {
    StorageLocation(Symbol),
    ConstantLiteral(Literal),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StorageLocation(symbol) => write!(f, "{}", symbol.0),
            Self::ConstantLiteral(lit) => write!(f, "{lit}"),
        }
    }
}

/// OPCode is a type wrapper around all opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OPCode {
    // Indirect jumps.
    Jump,
    // Condtional branches.
    Branch,
    // Function calls that don't produce values.
    Call,
    // Return statements.
    Return,
    /// `const` operation.
    Const,
    // Arithmetic operators.
    Add,
    Sub,
    Mul,
    Div,
    // Comparison operators.
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    // Unary operators.
    Not,
    Neg,
    // Boolean operators.
    And,
    Or,
    // Identity operator.
    Id,
    // Label pseudo instruction.
    Label,
    // Nop instruction.
    Nop,
}

enum Instruction {
    // `const` operation.
    Const(
        Symbol,  /* Destination */
        Literal, /* Literal value assigned to the destination */
    ),
    // Arithmetic operators.
    Add(
        Symbol, /* Destination */
        Value,  /* LHS */
        Value,  /* RHS */
    ),
    Sub(
        Symbol, /* Destination */
        Value,  /* LHS */
        Value,  /* RHS */
    ),
    Mul(
        Symbol, /* Destination */
        Value,  /* LHS */
        Value,  /* RHS */
    ),
    Div(
        Symbol, /* Destination */
        Value,  /* LHS */
        Value,  /* RHS */
    ),
    // Binary boolean operators.
    And(
        Symbol, /* Destination */
        Value,  /* LHS */
        Value,  /* RHS */
    ),
    Or(
        Symbol, /* Destination */
        Value,  /* LHS */
        Value,  /* RHS */
    ),
    // Unary operators.
    Not(Symbol /* Destination */, Value /* LHS */),
    Neg(Symbol /* Destination */, Value /* LHS */),
    // Comparison operators.
    Eq(
        Symbol, /* Destination */
        Value,  /* LHS */
        Value,  /* RHS */
    ),
    Neq(
        Symbol, /* Destination */
        Value,  /* LHS */
        Value,  /* RHS */
    ),
    Lt(
        Symbol, /* Destination */
        Value,  /* LHS */
        Value,  /* RHS */
    ),
    Lte(
        Symbol, /* Destination */
        Value,  /* LHS */
        Value,  /* RHS */
    ),
    Gt(
        Symbol, /* Destination */
        Value,  /* LHS */
        Value,  /* RHS */
    ),
    Gte(
        Symbol, /* Destination */
        Value,  /* LHS */
        Value,  /* RHS */
    ),
    // Return statements.
    Return(Value /* Return value */),
    // Function calls.
    Call(
        Symbol,     /* Call Target */
        Vec<Value>, /* Arguments */
    ),
    // Direct jump to label.
    Jump(Label /* Offset */),
    // Condtional branches.
    Branch(
        Symbol, /* Condition */
        Label,  /* Then Target Offset */
        Label,  /* Else Target Offset */
    ),
    // Identity operator.
    Id(Symbol /* Destination symbol*/, Value),
    // Label pseudo instruction, acts as a data marker when generating code.
    Label(usize /* Label handle or offset */),
    // Nop instruction.
    Nop,
}

impl Instruction {
    pub fn opcode(&self) -> OPCode {
        match self {
            Instruction::Const(..) => OPCode::Const,
            Instruction::Add(..) => OPCode::Add,
            Instruction::Sub(..) => OPCode::Sub,
            Instruction::Mul(..) => OPCode::Mul,
            Instruction::Div(..) => OPCode::Div,
            Instruction::And(..) => OPCode::And,
            Instruction::Or(..) => OPCode::Or,
            Instruction::Neg(..) => OPCode::Neg,
            Instruction::Not(..) => OPCode::Not,
            Instruction::Eq(..) => OPCode::Eq,
            Instruction::Neq(..) => OPCode::Neq,
            Instruction::Lt(..) => OPCode::Lt,
            Instruction::Lte(..) => OPCode::Lte,
            Instruction::Gt(..) => OPCode::Gt,
            Instruction::Gte(..) => OPCode::Gte,
            Instruction::Return(..) => OPCode::Return,
            Instruction::Call(..) => OPCode::Call,
            Instruction::Jump(..) => OPCode::Jump,
            Instruction::Branch(..) => OPCode::Branch,
            Instruction::Id(..) => OPCode::Id,
            Instruction::Nop => OPCode::Nop,
            Instruction::Label(..) => OPCode::Label,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Const(dst, lit) => {
                write!(f, "{}: {} const = {lit}", dst.0, dst.1)
            }
            Instruction::Add(dst, lhs, rhs) => {
                write!(f, "{} : {} = add {lhs} {rhs}", dst.0, dst.1)
            }
            Instruction::Sub(dst, lhs, rhs) => {
                write!(f, "{} : {} = sub {lhs} {rhs}", dst.0, dst.1)
            }
            Instruction::Mul(dst, lhs, rhs) => {
                write!(f, "{} : {} = mul {lhs} {rhs}", dst.0, dst.1)
            }
            Instruction::Div(dst, lhs, rhs) => {
                write!(f, "{} : {} = div {lhs} {rhs}", dst.0, dst.1)
            }
            Instruction::And(dst, lhs, rhs) => {
                write!(f, "{} : {} = and {lhs} {rhs}", dst.0, dst.1)
            }
            Instruction::Or(dst, lhs, rhs) => {
                write!(f, "{} : {} = or {lhs} {rhs}", dst.0, dst.1)
            }
            Instruction::Neg(dst, operand) => {
                write!(f, "{} : {} = neg {operand}", dst.0, dst.1)
            }
            Instruction::Not(dst, operand) => {
                write!(f, "{} : {} = not {operand}", dst.0, dst.1)
            }
            Instruction::Eq(dst, lhs, rhs) => {
                write!(f, "{} : {} = or {lhs} {rhs}", dst.0, dst.1)
            }
            Instruction::Neq(dst, lhs, rhs) => {
                write!(f, "{} : {} = or {lhs} {rhs}", dst.0, dst.1)
            }
            Instruction::Lt(dst, lhs, rhs) => {
                write!(f, "{} : {} = or {lhs} {rhs}", dst.0, dst.1)
            }
            Instruction::Lte(dst, lhs, rhs) => {
                write!(f, "{} : {} = or {lhs} {rhs}", dst.0, dst.1)
            }
            Instruction::Gt(dst, lhs, rhs) => {
                write!(f, "{} : {} = or {lhs} {rhs}", dst.0, dst.1)
            }
            Instruction::Gte(dst, lhs, rhs) => {
                write!(f, "{} : {} = or {lhs} {rhs}", dst.0, dst.1)
            }
            Instruction::Return(value) => write!(f, "return {value}"),
            Instruction::Call(def, args) => {
                write!(f, "@{}", def.0)?;
                for arg in args {
                    write!(f, "{arg} ")?;
                }
                write!(f, "")
            }
            Instruction::Jump(target) => write!(f, "jump {}", target),
            Instruction::Branch(cond, then_target, else_target) => {
                write!(f, "br {} {then_target} {else_target}", cond.0)
            }
            Instruction::Id(dst, value) => {
                write!(f, "{} : {} = id {value}", dst.0, dst.1)
            }
            Instruction::Nop => write!(f, "nop"),
            Instruction::Label(addr) => write!(f, "__LABEL_{addr}"),
        }
    }
}