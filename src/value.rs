//! Defines the runtime value type.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct FileHandle {
    pub path: String,
    pub reader: Option<std::io::BufReader<std::fs::File>>,
    pub writer: Option<std::fs::File>,
    pub eof: bool,
}

impl FileHandle {
    pub fn new_reader(path: String, file: std::fs::File) -> Self {
        Self {
            path,
            reader: Some(std::io::BufReader::new(file)),
            writer: None,
            eof: false,
        }
    }
    pub fn new_writer(path: String, file: std::fs::File) -> Self {
        Self {
            path,
            reader: None,
            writer: Some(file),
            eof: false,
        }
    }
}

/// Represents a value in the Hi language: integer, float, string, or boolean.
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Rc<RefCell<Vec<Value>>>),
    File(Rc<RefCell<FileHandle>>),
    Dict(Rc<RefCell<HashMap<Value, Value>>>),
    Function(String),
    Nil,
}

impl Eq for Value {}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a.to_bits() == b.to_bits(),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::List(a), Value::List(b)) => {
                let a_borrow = a.borrow();
                let b_borrow = b.borrow();
                &*a_borrow == &*b_borrow
            }
            (Value::Dict(a), Value::Dict(b)) => {
                let a_borrow = a.borrow();
                let b_borrow = b.borrow();
                if a_borrow.len() != b_borrow.len() {
                    return false;
                }
                for (k, v) in a_borrow.iter() {
                    if let Some(bv) = b_borrow.get(k) {
                        if v != bv {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Value::Int(i) => i.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::String(s) => s.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Nil => ().hash(state),
            Value::Function(name) => name.hash(state),
            Value::List(_) | Value::Dict(_) | Value::File(_) => {
                panic!("attempted to hash non‑hashable value")
            }
        }
    }
}

impl Value {
    /// Converts the value to a boolean according to Hi semantics.
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Nil => false,
            Value::List(l) => !l.borrow().is_empty(),
            Value::Dict(d) => !d.borrow().is_empty(),
            Value::File(_) => false,
            Value::Function(_) => true,
        }
    }

    pub fn is_hashable(&self) -> bool {
        matches!(
            self,
            Value::Int(_) | Value::Float(_) | Value::String(_) | Value::Bool(_)
        )
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => {
                if fl.fract() == 0.0 {
                    write!(f, "{:.1}", fl)
                } else {
                    write!(f, "{}", fl)
                }
            }
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            Value::Nil => write!(f, "nil"),
            Value::Function(name) => write!(f, "<func {}>", name),
            Value::List(l) => {
                let vec = l.borrow();
                write!(f, "[")?;
                for (i, val) in vec.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    match val {
                        Value::String(s) => write!(f, "\"{}\"", s)?,
                        _ => write!(f, "{}", val)?,
                    }
                }
                write!(f, "]")
            }
            Value::File(fh) => {
                let handle = fh.borrow();
                write!(f, "<file: {}>", handle.path)
            }
            Value::Dict(d) => {
                let map = d.borrow();
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    match k {
                        Value::String(s) => write!(f, "\"{}\"", s)?,
                        _ => write!(f, "{}", k)?,
                    }
                    write!(f, "=")?;
                    match v {
                        Value::String(s) => write!(f, "\"{}\"", s)?,
                        _ => write!(f, "{}", v)?,
                    }
                }
                write!(f, "}}")
            }
        }
    }
}
