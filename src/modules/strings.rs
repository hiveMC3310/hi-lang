use crate::ast::Span;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::modules::{BuiltinFn, BuiltinModule};
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn new_strings_module() -> BuiltinModule {
    let vars: HashMap<String, Value> = HashMap::new();
    let mut funcs: HashMap<String, BuiltinFn> = HashMap::new();

    funcs.insert("split".to_string(), Rc::new(split_fn));
    funcs.insert("replace".to_string(), Rc::new(replace_fn));
    funcs.insert("starts".to_string(), Rc::new(starts_fn));
    funcs.insert("ends".to_string(), Rc::new(ends_fn));
    funcs.insert("upper".to_string(), Rc::new(upper_fn));
    funcs.insert("lower".to_string(), Rc::new(lower_fn));
    funcs.insert("trim".to_string(), Rc::new(trim_fn));
    funcs.insert("substr".to_string(), Rc::new(substr_fn));
    funcs.insert("join".to_string(), Rc::new(join_fn));

    BuiltinModule { funcs, vars }
}

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
