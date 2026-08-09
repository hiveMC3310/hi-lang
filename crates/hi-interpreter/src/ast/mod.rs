//! Abstract Syntax Tree (AST) definitions for the Hi language.
//!
//! This module defines the data structures that represent the syntax of Hi programs.
//! The AST is produced by the parser and consumed by the interpreter and analysis passes.
//! Each node carries source location information (`Span`) to enable accurate error reporting.

use hi_common::Symbol;

/// Represents a source code location range (1‑based lines and columns).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Starting line number (1‑based)
    pub start_line: usize,
    /// Starting column number (1‑based)
    pub start_col: usize,
    /// Ending line number (1‑based)
    pub end_line: usize,
    /// Ending column number (1‑based)
    pub end_col: usize,
}

impl Span {
    /// Merges this span with another span, creating a span that covers from the start of `self`
    /// to the end of `other`. Useful for combining spans of sub‑expressions.
    pub fn merge(self, other: &Self) -> Self {
        Self {
            start_line: self.start_line,
            start_col: self.start_col,
            end_line: other.end_line,
            end_col: other.end_col,
        }
    }

    /// Returns a dummy span (all zeros) for testing or placeholder use.
    pub const fn dummy() -> Self {
        Self {
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 0,
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.start_line, self.start_col)
    }
}

/// A block is simply a list of statements.
pub type Block = Vec<Stmt>;

/// The root of the AST: a program consists of a block of statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Block,
}

/// All possible statements in the Hi language.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Variable declaration and initialization.
    /// Parameters: `(name, initializer_expr, name_span, full_span)`
    Let(Symbol, Expr, Span, Span),

    /// Input statement: reads a value from stdin, optionally with a prompt.
    /// Parameters: `(prompt_string, variable_name, span)`
    Input(Option<String>, Symbol, Span),

    /// Conditional statement: `IF condition THEN ... [ELSE ...] END`.
    /// Parameters: `(condition_expr, then_block, else_block_opt, span)`
    If(Expr, Block, Option<Block>, Span),

    /// While loop: `WHILE condition DO ... END`.
    /// Parameters: `(condition_expr, body_block, span)`
    While(Expr, Block, Span),

    /// For loop: `FOR var = start TO end DO ... NEXT [step]`.
    /// Parameters: `(var_symbol, start_expr, end_expr, step_expr_opt, body_block, var_span, full_span)`
    For(
        Symbol,
        Box<Expr>,
        Box<Expr>,
        Option<Box<Expr>>,
        Block,
        Span,
        Span,
    ),

    /// Break statement: exits the nearest enclosing loop.
    /// Parameter: `(span)`
    Break(Span),

    /// Function definition: `FUNC name(params) ... END`.
    /// Parameters: `(name, param_list, body_block, doc_string_opt, name_span, full_span)`
    Func(Symbol, Vec<Symbol>, Block, Option<String>, Span, Span),

    /// Return statement: `RET [expr]`.
    /// Parameters: `(return_value_expr_opt, span)`
    Return(Option<Expr>, Span),

    /// Print statement: `PRINT expr, expr, ...`.
    /// Parameters: `(list_of_exprs, span)`
    Print(Vec<Expr>, Span),

    /// Simple assignment: `lhs = rhs`.
    /// Parameters: `(left_expr, right_expr, span)`
    Assign(Box<Expr>, Box<Expr>, Span),

    /// Compound assignment: `lhs op= rhs`, where `op` is one of `+`, `-`, `*`, `/`, `%`, `^`.
    /// Parameters: `(left_expr, operator, right_expr, span)`
    CompoundAssign(Box<Expr>, BinOp, Box<Expr>, Span),

    /// Expression used as a statement (e.g. function call or literal).
    /// Parameter: `(expr, span)`
    Expr(Expr, Span),

    /// Module import: `IMPORT "path" [AS alias]`.
    /// Parameters: `(path_string, alias_symbol_opt, span)`
    Import(String, Option<Symbol>, Span),
}

/// All possible expressions in the Hi language.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal: `42`.
    Int(i64, Span),

    /// Float literal: `3.14`.
    Float(f64, Span),

    /// String literal: `"hello"`.
    String(String, Span),

    /// Boolean literal: `TRUE` or `FALSE`.
    Bool(bool, Span),

    /// Variable or function name reference.
    Variable(Symbol, Span),

    /// Binary operation: `left op right`.
    Binary(BinOp, Box<Expr>, Box<Expr>, Span),

    /// Unary operation: `op expr` (currently `NOT` and `-`).
    Unary(UnOp, Box<Expr>, Span),

    /// Indexing: `base[index]` (works on lists and dicts).
    Index(Box<Expr>, Box<Expr>, Span),

    /// List literal: `[expr, expr, ...]`.
    List(Vec<Expr>, Span),

    /// Dictionary literal: `{key = value, key = value, ...}`.
    Dict(Vec<(Expr, Expr)>, Span),

    /// Function call: `name(arg1, arg2, ...)`.
    Call(Symbol, Vec<Expr>, Span),

    /// Module variable access: `module:var`.
    ModuleAccess(Symbol, Symbol, Span),

    /// Module function call: `module:func(args)`.
    CallModule(Symbol, Symbol, Vec<Expr>, Span),
}

impl Expr {
    /// Returns the source span covering the entire expression.
    pub fn span(&self) -> Span {
        match self {
            Expr::Int(_, span) => *span,
            Expr::Float(_, span) => *span,
            Expr::String(_, span) => *span,
            Expr::Bool(_, span) => *span,
            Expr::Variable(_, span) => *span,
            Expr::Binary(_, _, _, span) => *span,
            Expr::Unary(_, _, span) => *span,
            Expr::Index(_, _, span) => *span,
            Expr::Call(_, _, span) => *span,
            Expr::List(_, span) => *span,
            Expr::Dict(_, span) => *span,
            Expr::ModuleAccess(_, _, span) => *span,
            Expr::CallModule(_, _, _, span) => *span,
        }
    }
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Mod, // %
    Pow, // ^
    Eq,  // ==
    Ne,  // !=
    Gt,  // >
    Ge,  // >=
    Lt,  // <
    Le,  // <=
    And, // AND
    Or,  // OR
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Not, // NOT or !
    Neg, // -
}
