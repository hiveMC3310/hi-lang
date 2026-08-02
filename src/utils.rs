//! Utility functions for parsing and type checks.

use crate::value::Value;

/// Parses a string into a Value (Int, Float, or String).
pub fn parse(s: &str) -> Value {
    if let Ok(i) = s.parse::<i64>() {
        return Value::Int(i);
    }
    if let Ok(f) = s.parse::<f64>() {
        return Value::Float(f);
    }

    let trimmed = s.trim();
    if trimmed == "True" {
        return Value::Bool(true);
    }
    if trimmed == "False" {
        return Value::Bool(false);
    }

    Value::String(s.to_string())
}

/// Checks if a Value is numerically zero.
pub fn is_zero(v: &Value) -> bool {
    match v {
        Value::Int(0) => true,
        Value::Float(f) if *f == 0.0 => true,
        _ => false,
    }
}

/// Returns a human-readable type name for a Value
pub fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Int(_) => "integer",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::Bool(_) => "boolean",
        Value::List(_) => "list",
        Value::File(_) => "file",
        Value::Dict(_) => "dict",
        Value::Nil => "nil",
    }
}
