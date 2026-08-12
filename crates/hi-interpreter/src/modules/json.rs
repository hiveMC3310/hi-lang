#![allow(clippy::mutable_key_type)]
use crate::ast::Span;
use crate::builtins::ModuleFunction;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::value::Value;
use serde_json::Value as JsonValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Converts a serde_json::Value into a Hi Value.
fn json_to_hi(json_val: JsonValue) -> Value {
    match json_val {
        JsonValue::Null => Value::Nil,
        JsonValue::Bool(b) => Value::Bool(b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                // Fallback: should not happen for valid JSON numbers
                Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        JsonValue::String(s) => Value::String(s),
        JsonValue::Array(arr) => {
            let vec: Vec<Value> = arr.into_iter().map(json_to_hi).collect();
            Value::List(Rc::new(RefCell::new(vec)))
        }
        JsonValue::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                let key = Value::String(k);
                let val = json_to_hi(v);
                map.insert(key, val);
            }
            Value::Dict(Rc::new(RefCell::new(map)))
        }
    }
}

/// Converts a Hi Value into a serde_json::Value.
/// Returns an error if the value contains unsupported types (function, file, module).
fn hi_to_json(hi_val: &Value, span: &Span) -> InterpResult<JsonValue> {
    match hi_val {
        Value::Nil => Ok(JsonValue::Null),
        Value::Bool(b) => Ok(JsonValue::Bool(*b)),
        Value::Int(i) => Ok(JsonValue::Number((*i).into())),
        Value::Float(f) => {
            if f.is_finite() {
                Ok(JsonValue::Number(
                    serde_json::Number::from_f64(*f).ok_or_else(|| InterpError::Runtime {
                        span: *span,
                        message: format!("Invalid float value for JSON: {}", f),
                    })?,
                ))
            } else {
                // Non-finite floats (NaN, Infinity) are not valid JSON.
                Err(InterpError::Runtime {
                    span: *span,
                    message: format!("JSON does not support non-finite float: {}", f),
                })
            }
        }
        Value::String(s) => Ok(JsonValue::String(s.clone())),
        Value::List(list_rc) => {
            let list = list_rc.borrow();
            let mut arr = Vec::with_capacity(list.len());
            for elem in list.iter() {
                arr.push(hi_to_json(elem, span)?);
            }
            Ok(JsonValue::Array(arr))
        }
        Value::Dict(dict_rc) => {
            let dict = dict_rc.borrow();
            let mut obj = serde_json::Map::new();
            for (key, val) in dict.iter() {
                // JSON keys must be strings.
                let key_str = match key {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(InterpError::Runtime {
                            span: *span,
                            message: format!(
                                "Cannot serialize dictionary with non-string key: {}",
                                key
                            ),
                        });
                    }
                };
                obj.insert(key_str, hi_to_json(val, span)?);
            }
            Ok(JsonValue::Object(obj))
        }
        Value::Function(_) | Value::File(_) | Value::Module(_) => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "Cannot serialize value of type {} to JSON",
                crate::utils::type_name(hi_val)
            ),
        }),
    }
}

// ---------- parse ----------
fn parse_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("parse() expects 1 argument (string), got {}", args.len()),
        });
    }

    let json_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "parse() expects a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    let json_val: JsonValue = serde_json::from_str(json_str).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Invalid JSON: {}", e),
    })?;

    Ok(json_to_hi(json_val))
}
inventory::submit! {
    ModuleFunction {
        module: "json",
        name: "parse",
        params: &["string"],
        doc: "Parses a JSON string and returns the corresponding Hi value.",
        func: parse_fn,
    }
}

// ---------- stringify ----------
fn stringify_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("stringify() expects 1 argument, got {}", args.len()),
        });
    }

    let json_val = hi_to_json(&args[0], span)?;

    // Serialize to compact JSON string (no pretty-printing).
    let json_str = serde_json::to_string(&json_val).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Failed to serialize JSON: {}", e),
    })?;

    Ok(Value::String(json_str))
}
inventory::submit! {
    ModuleFunction {
        module: "json",
        name: "stringify",
        params: &["value"],
        doc: "Converts a Hi value to a JSON string. Supports nil, bool, int, float, string, list, dict. Non-finite floats, functions, files, and modules are not supported.",
        func: stringify_fn,
    }
}
