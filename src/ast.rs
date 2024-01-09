//! Implementation of glouton's abstract syntax tree.
//!
//! The glouton AST uses the apporach of a flat data structure where instead
//! of using traditional tree like structure where each node holds a pointer
//! in this case `Box<Node>`, nodes hold handles or reference indices to nodes
//! stored in an arena represented by a `Vec<Node>`.
//!
//! This data oriented approach has several pros, the first being that arenas
//! are borrow checker friendly. This is especially important in Rust where
//! an initial design might be invalidated because it doesn't have a borrow
//! checker friendly representation.
//!
//! The second argument for this is that complex self references and circular
//! references are not an issue anymore because there is no lifetime associated
//! with the reference you use, the lifetime is now associated with the entire
//! arena and the individual entries hold indices into the arena which are just
//! `u32`.
//!
//! Another argument although less impressive at this scale is speed, because
//! fetches aren't done via pointers and because AST nodes have nice locality
//! if your arena fits in cache then walking it becomes much faster than going
//! through the pointer fetch road.
//!
//! One point that must need to be though of before using the approach is how
//! the ownership of references is "oriented", i.e is your lifetime represented
//! as a tree of resources or a graph. This is important because the direction
//! or path of ownership could hinder the design.
//!
//! In our case the AST represents a program, the root node is an entry point
//! and the program itself is a sequence of *statements*. Where each statement
//! either represents control flow or expressions. Since expressions *will not*
//! reference *statements*, the AST can be represented as a tuple of `StmtPool`
//! and `ExprPool`.
//!
//! Where `StmtPool` holds the statement nodes, each node holds an `ExprRef`
//! that can reference an expression or a `StmtRef` that references statements.
//!
//! This approach is not new and has been used in LuaJIT, Zig, Sorbet, ECS
//! game engines and more, see[1] for more details.
//!
//! [1]: https://www.cs.cornell.edu/~asampson/blog/flattening.html

use core::fmt;

/// Node references are represented as `usize` handles to the AST arena
/// this avoides type casting everytime we want to access a node and down
/// casting when building references from indices.
///
/// `StmtRef` is used to reference statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StmtRef(usize);
/// `ExprRef` is used to reference expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExprRef(usize);

/// `ExprPool` represents an arena of AST expression nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprPool {
    nodes: Vec<Expr>,
}

impl ExprPool {
    /// Create a new node pool with a pre-allocated capacity.
    pub fn new() -> Self {
        Self {
            nodes: Vec::with_capacity(4096),
        }
    }

    /// Return a reference to a node given its `NodeRef`.
    pub fn get(&self, node_ref: ExprRef) -> Option<&Expr> {
        self.nodes.get(node_ref.0)
    }

    /// Push a new expression into the pool.
    fn add(&mut self, expr: Expr) -> ExprRef {
        let node_ref = self.nodes.len();
        self.nodes.push(expr);
        ExprRef(node_ref)
    }
}

/// `StmtPool` represents an arena of AST statement nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StmtPool {
    nodes: Vec<Stmt>,
}

impl StmtPool {
    /// Create a new node pool with a pre-allocated capacity.
    pub fn new() -> Self {
        Self {
            nodes: Vec::with_capacity(4096),
        }
    }

    /// Return a reference to a node given its `NodeRef`.
    pub fn get(&self, node_ref: StmtRef) -> Option<&Stmt> {
        self.nodes.get(node_ref.0)
    }

    /// Push a new expression into the pool.
    fn add(&mut self, stmt: Stmt) -> StmtRef {
        let node_ref = self.nodes.len();
        self.nodes.push(stmt);
        StmtRef(node_ref)
    }
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Neg,
    Not,
}

/// Expression nodes are used to represent expressions.
/// TODO make Expr homogenous by storing `LiteralRef`, `StringRef` and so on
/// in a separate storage array stored in the AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    // Named values (variables),
    Named(String),
    // Integer literal values.
    IntLiteral(i32),
    // Grouping expressions (parenthesised expressions).
    Grouping(ExprRef),
    // Binary operations (arithmetic, boolean, bitwise).
    BinOp {
        left: ExprRef,
        operator: BinaryOperator,
        right: ExprRef,
    },
    // Unary operations (boolean not and arithmetic negation).
    UnaryOp {
        operator: UnaryOperator,
        operand: ExprRef,
    },
}

/// Statement nodes are used to represent statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    // Return statements.
    Return(ExprRef),
    // Expression statements.
    Expr(ExprRef),
}

/// `AST` represents the AST generated by the parser when processing a list
/// of tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AST {
    statements: StmtPool,
    expressions: ExprPool,
}

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.statements.nodes {
            let _ = match stmt {
                Stmt::Return(expr_ref) => {
                    if let Some(expr) = self.get_expr(*expr_ref) {
                        write!(f, "Return({})", display_expr_node(self, &expr))
                    } else {
                        unreachable!("Return statement is missing expression ref")
                    }
                }
                Stmt::Expr(expr_ref) => {
                    if let Some(expr) = self.get_expr(*expr_ref) {
                        write!(f, "Expr({})", display_expr_node(self, &expr))
                    } else {
                        unreachable!("Expr statement is missing expression ref")
                    }
                }
            };
        }
        Ok(())
    }
}

fn display_expr_node(ast: &AST, node: &Expr) -> String {
    match node {
        &Expr::IntLiteral(value) => value.to_string(),
        &Expr::UnaryOp { operator, operand } => {
            if let Some(operand) = ast.get_expr(operand) {
                match operator {
                    UnaryOperator::Neg => format!("Neg({})", display_expr_node(ast, operand)),
                    UnaryOperator::Not => format!("Not({})", display_expr_node(ast, operand)),
                }
            } else {
                unreachable!("unary node is missing operand")
            }
        }
        &Expr::BinOp {
            left,
            operator,
            right,
        } => {
            if let (Some(left), Some(right)) = (ast.get_expr(left), ast.get_expr(right)) {
                match operator {
                    BinaryOperator::Add => format!(
                        "Add({}, {})",
                        display_expr_node(ast, left),
                        display_expr_node(ast, right)
                    ),
                    BinaryOperator::Sub => format!(
                        "Sub({}, {})",
                        display_expr_node(ast, left),
                        display_expr_node(ast, right)
                    ),
                    BinaryOperator::Mul => format!(
                        "Mul({}, {})",
                        display_expr_node(ast, left),
                        display_expr_node(ast, right)
                    ),
                    BinaryOperator::Div => format!(
                        "Div({}, {})",
                        display_expr_node(ast, left),
                        display_expr_node(ast, right)
                    ),
                }
            } else {
                unreachable!("binary node is missing operand")
            }
        }
        &Expr::Grouping(expr_ref) => {
            if let Some(expr) = ast.get_expr(expr_ref) {
                format!("Grouping({})", display_expr_node(ast, expr))
            } else {
                unreachable!("unary node is missing operand")
            }
        }
        _ => todo!("Unimplemented display for Node {:?}", node),
    }
}

impl AST {
    /// Create a new empty AST.
    pub fn new() -> Self {
        Self {
            statements: StmtPool::new(),
            expressions: ExprPool::new(),
        }
    }

    /// Push a new statement node to the AST returning a reference to it.
    pub fn push_stmt(&mut self, stmt: Stmt) -> StmtRef {
        self.statements.add(stmt)
    }

    /// Push a new expression node to the AST returning a reference to it.
    pub fn push_expr(&mut self, expr: Expr) -> ExprRef {
        self.expressions.add(expr)
    }

    /// Fetches an expression node by its reference, returning `None`
    /// if the expression doesn't exist.
    pub fn get_expr(&self, expr_ref: ExprRef) -> Option<&Expr> {
        self.expressions.get(expr_ref)
    }

    /// Fetches a statement node by its reference, returning `None`
    /// if the statement node deosn't exist.
    pub fn get_stmt(&self, stmt_ref: StmtRef) -> Option<&Stmt> {
        self.statements.get(stmt_ref)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{ExprPool, StmtPool};

    use super::{Expr, Stmt};

    #[test]
    fn can_create_and_use_node_pool() {
        let mut expr_pool = ExprPool::new();
        let mut stmt_pool = StmtPool::new();

        for _ in 0..100 {
            let expr_ref = expr_pool.add(Expr::IntLiteral(42));
            let node_ref = stmt_pool.add(Stmt::Return(expr_ref));

            assert_eq!(expr_pool.get(expr_ref), Some(&Expr::IntLiteral(42)));
            assert_eq!(stmt_pool.get(node_ref), Some(&Stmt::Return(expr_ref)));
        }
    }
}
