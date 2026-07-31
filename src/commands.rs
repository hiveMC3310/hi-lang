//! Defines the abstract syntax tree (AST) commands for the Hi language.

use crate::value::Value;

/// All possible Hi instructions after parsing.
#[derive(Debug, Clone)]
pub enum Command {
    Hello,

    Push(Value),
    Pop(Option<String>),
    Let(String, Value),

    Dup,
    Swap,

    Print(Vec<Value>),
    Input(Option<String>, String),

    Binary(BinOp, Option<Value>, Option<Value>),
    Not(Option<Value>),

    If(Value),
    Else,
    Endif,

    While(Value),
    Do,
    Break,

    Func(String),
    Ret,
    Endf,
    Call(String),

    Len(Option<Value>),
    Concat(Option<Value>, Option<Value>),
    Substr(Option<Value>, Option<Value>, Option<Value>),
    Upper(Option<Value>),
    Lower(Option<Value>),
    Trim(Option<Value>),

    List(Vec<Value>),
    Index(Value, Value),
    Append(Value, Value),

    Contains(Value, Value),
    Starts(Value, Value),
    Ends(Value, Value),
    Replace(Value, Value, Value),
    Split(Value, Value),

    Slice(Value, Value, Value),
    Reverse(Value),
    Insert(Value, Value, Value),
    IndexOf(Value, Value),

    Open(Value, Value),
    Close(Option<Value>),
    Read(Value, Option<String>),
    Write(Value, Value),
    Readln(Value, Option<String>),
    Writeln(Value, Value),
    Eof(Option<Value>),

    Dict(Option<String>),
    Put(String, Value, Value),
    Get(String, Value),
    Has(String, Value),
    Keys(String),
    Values(String),

    Remove(String, Value),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    And,
    Or,
    Mod,
    Pow,
}
