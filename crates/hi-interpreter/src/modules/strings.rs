use crate::ast::Span;
use crate::builtins::ModuleFunction;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

// ---------- split ----------
fn split_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("split() expects 2 arguments, got {}", args.len()),
        });
    }
    let base = &args[0];
    let delim = &args[1];
    match (&base, &delim) {
        (Value::String(s), Value::String(d)) => {
            let parts: Vec<Value> = s
                .split(d)
                .map(|part| Value::String(part.to_string()))
                .collect();
            Ok(Value::List(Rc::new(RefCell::new(parts))))
        }
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "split() expects two strings, got {} and {}",
                crate::utils::type_name(&base),
                crate::utils::type_name(&delim)
            ),
        }),
    }
}
inventory::submit! {
    ModuleFunction {
        module: "strings",
        name: "split",
        params: &["string", "delimiter"],
        doc: "Splits a string by a delimiter and returns a list of substrings.",
        func: split_fn,
    }
}

// ---------- replace ----------
fn replace_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 3 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("replace() expects 3 arguments, got {}", args.len()),
        });
    }
    let base = &args[0];
    let old = &args[1];
    let new = &args[2];
    match (&base, &old, &new) {
        (Value::String(s), Value::String(old_str), Value::String(new_str)) => {
            Ok(Value::String(s.replace(old_str, &new_str)))
        }
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "replace() expects three strings, got {}, {}, {}",
                crate::utils::type_name(&base),
                crate::utils::type_name(&old),
                crate::utils::type_name(&new)
            ),
        }),
    }
}
inventory::submit! {
    ModuleFunction {
        module: "strings",
        name: "replace",
        params: &["string", "old", "new"],
        doc: "Replaces all occurrences of a substring with another string.",
        func: replace_fn,
    }
}

// ---------- substr ----------
fn substr_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 3 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("substr() expects 3 arguments, got {}", args.len()),
        });
    }
    let s_val = &args[0];
    let start_val = &args[1];
    let len_val = &args[2];
    let s = match s_val {
        Value::String(s) => s.clone(),
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "substr() expects a string, got {}",
                    crate::utils::type_name(&s_val)
                ),
            });
        }
    };
    let start = match start_val {
        Value::Int(i) => *i,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: "substr() start must be integer".to_string(),
            });
        }
    };
    let len = match len_val {
        Value::Int(i) => *i,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: "substr() length must be integer".to_string(),
            });
        }
    };
    if start < 0 || len < 0 {
        return Err(InterpError::Runtime {
            span: *span,
            message: "substr() start and length must be non-negative".to_string(),
        });
    }
    let start = start as usize;
    let len = len as usize;
    let end = start + len;
    let result = if start >= s.len() {
        String::new()
    } else if end > s.len() {
        s[start..].to_string()
    } else {
        s[start..end].to_string()
    };
    Ok(Value::String(result))
}
inventory::submit! {
    ModuleFunction {
        module: "strings",
        name: "substr",
        params: &["string", "start", "length"],
        doc: "Extracts a substring starting at the given index with the specified length.",
        func: substr_fn,
    }
}

// ---------- starts ----------
fn starts_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("starts() expects 2 arguments, got {}", args.len()),
        });
    }
    let base = &args[0];
    let prefix = &args[1];
    match (&base, &prefix) {
        (Value::String(s), Value::String(p)) => Ok(Value::Bool(s.starts_with(p))),
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "starts() expects two strings, got {} and {}",
                crate::utils::type_name(&base),
                crate::utils::type_name(&prefix)
            ),
        }),
    }
}
inventory::submit! {
    ModuleFunction {
        module: "strings",
        name: "starts",
        params: &["string", "prefix"],
        doc: "Returns true if the string starts with the given prefix.",
        func: starts_fn,
    }
}

// ---------- ends ----------
fn ends_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("ends() expects 2 arguments, got {}", args.len()),
        });
    }
    let base = &args[0];
    let suffix = &args[1];
    match (&base, &suffix) {
        (Value::String(s), Value::String(suf)) => Ok(Value::Bool(s.ends_with(suf))),
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "ends() expects two strings, got {} and {}",
                crate::utils::type_name(&base),
                crate::utils::type_name(&suffix)
            ),
        }),
    }
}
inventory::submit! {
    ModuleFunction {
        module: "strings",
        name: "ends",
        params: &["string", "suffix"],
        doc: "Returns true if the string ends with the given suffix.",
        func: ends_fn,
    }
}

// ---------- upper ----------
fn upper_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("upper() expects 1 argument, got {}", args.len()),
        });
    }
    let val = &args[0];
    match val {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "upper() expects a string, got {}",
                crate::utils::type_name(&val)
            ),
        }),
    }
}
inventory::submit! {
    ModuleFunction {
        module: "strings",
        name: "upper",
        params: &["string"],
        doc: "Converts the string to uppercase.",
        func: upper_fn,
    }
}

// ---------- lower ----------
fn lower_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("lower() expects 1 argument, got {}", args.len()),
        });
    }
    let val = &args[0];
    match val {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "lower() expects a string, got {}",
                crate::utils::type_name(&val)
            ),
        }),
    }
}
inventory::submit! {
    ModuleFunction {
        module: "strings",
        name: "lower",
        params: &["string"],
        doc: "Converts the string to lowercase.",
        func: lower_fn,
    }
}

// ---------- trim ----------
fn trim_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("trim() expects 1 argument, got {}", args.len()),
        });
    }
    let val = &args[0];
    match val {
        Value::String(s) => Ok(Value::String(s.trim().to_string())),
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "trim() expects a string, got {}",
                crate::utils::type_name(&val)
            ),
        }),
    }
}
inventory::submit! {
    ModuleFunction {
        module: "strings",
        name: "trim",
        params: &["string"],
        doc: "Removes leading and trailing whitespace from the string.",
        func: trim_fn,
    }
}

// ---------- join ----------
fn join_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "join() expects 2 arguments (delimiter, list), got {}",
                args.len()
            ),
        });
    }
    let delim = &args[0];
    let list_val = &args[1];

    let delim_str = match delim {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "join() delimiter must be a string, got {}",
                    crate::utils::type_name(delim)
                ),
            });
        }
    };

    let list = match list_val {
        Value::List(l) => l,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "join() expects a list, got {}",
                    crate::utils::type_name(list_val)
                ),
            });
        }
    };

    let list_ref = list.borrow();
    let parts: Vec<String> = list_ref
        .iter()
        .map(|v| match v {
            Value::String(s) => Ok(s.clone()),
            _ => Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "join() list must contain only strings, found {}",
                    crate::utils::type_name(v)
                ),
            }),
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Value::String(parts.join(delim_str)))
}
inventory::submit! {
    ModuleFunction {
        module: "strings",
        name: "join",
        params: &["delimiter", "list"],
        doc: "Joins a list of strings with a delimiter into a single string.",
        func: join_fn,
    }
}
