//! Defines the abstract syntax tree (AST) commands for the Hi language.

use crate::value::Value;

/// All possible Hi instructions after parsing.
#[derive(Debug)]
pub enum Command {
    Hello,
    Push(Value),
    Pop(Option<String>),
    Let(String, Value),
    Print(Vec<Value>),
    Input(Option<String>, String),
    Add(Option<Value>, Option<Value>),
    Sub(Option<Value>, Option<Value>),
    Mul(Option<Value>, Option<Value>),
    Div(Option<Value>, Option<Value>),
    Gt(Option<Value>, Option<Value>),
    Ge(Option<Value>, Option<Value>),
    Eq(Option<Value>, Option<Value>),
    Ne(Option<Value>, Option<Value>),
    Lt(Option<Value>, Option<Value>),
    Le(Option<Value>, Option<Value>),
    And(Option<Value>, Option<Value>),
    Or(Option<Value>, Option<Value>),
    Not(Option<Value>),
    If(Value),
    IfExpr {
        op: String,
        left: Value,
        right: Value,
    },
    Else,
    Endif,
    While(Value),
    WhileExpr {
        op: String,
        left: Value,
        right: Value,
    },
    Do,
    Break,
    Func(String),
    Ret,
    Endf,
    Call(String),
}
