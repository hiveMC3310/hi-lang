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
