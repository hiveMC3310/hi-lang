//! Abstract Syntax Tree (AST) definitions for the Hi language.

use hi_common::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Span {
    pub fn merge(self, other: &Self) -> Self {
        Self {
            start_line: self.start_line,
            start_col: self.start_col,
            end_line: other.end_line,
            end_col: other.end_col,
        }
    }

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

pub type Block = Vec<Stmt>;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let(Symbol, Expr, Span, Span),
    Input(Option<String>, Symbol, Span),
    If(Expr, Block, Option<Block>, Span),
    While(Expr, Block, Span),
    For(Symbol, Box<Expr>, Box<Expr>, Option<Box<Expr>>, Block, Span),
    Break(Span),
    Func(Symbol, Vec<Symbol>, Block, Option<String>, Span),
    Return(Option<Expr>, Span),
    Print(Vec<Expr>, Span),
    Assign(Box<Expr>, Box<Expr>, Span),
    Expr(Expr, Span),
    Import(String, Option<Symbol>, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64, Span),
    Float(f64, Span),
    String(String, Span),
    Bool(bool, Span),
    Variable(Symbol, Span),
    Binary(BinOp, Box<Expr>, Box<Expr>, Span),
    Unary(UnOp, Box<Expr>, Span),
    Index(Box<Expr>, Box<Expr>, Span),
    List(Vec<Expr>, Span),
    Dict(Vec<(Expr, Expr)>, Span),
    Call(Symbol, Vec<Expr>, Span),
    ModuleAccess(Symbol, Symbol, Span),
    CallModule(Symbol, Symbol, Vec<Expr>, Span),
}

impl Expr {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Not,
    Neg,
}
