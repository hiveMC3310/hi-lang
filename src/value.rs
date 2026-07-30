//! Defines the runtime value type.

use std::cell::RefCell;
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
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::List(a), Value::List(b)) => {
                let a_borrow = a.borrow();
                let b_borrow = b.borrow();
                &*a_borrow == &*b_borrow
            }
            _ => false,
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
            Value::List(l) => !l.borrow().is_empty(),
            Value::File(_) => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::List(l) => {
                let vec = l.borrow();
                write!(f, "[")?;
                for (i, val) in vec.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            Value::File(fh) => {
                let handle = fh.borrow();
                write!(f, "<file: {}>", handle.path)
            }
        }
    }
}
