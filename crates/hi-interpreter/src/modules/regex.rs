use crate::ast::Span;
use crate::builtins::ModuleFunction;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::value::Value;
use regex::Regex;
use std::cell::RefCell;
use std::rc::Rc;

/// Helper: compiles a regex pattern, returning an error on failure.
fn compile_regex(pattern: &str, span: &Span) -> InterpResult<Regex> {
    Regex::new(pattern).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Invalid regex pattern: {}", e),
    })
}

// ---------- match ----------
fn match_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "match() expects 2 arguments (pattern, string), got {}",
                args.len()
            ),
        });
    }
    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "match() pattern must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };
    let text = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "match() text must be a string, got {}",
                    crate::utils::type_name(&args[1])
                ),
            });
        }
    };

    let re = compile_regex(pattern, span)?;
    Ok(Value::Bool(re.is_match(text)))
}
inventory::submit! {
    ModuleFunction {
        module: "regex",
        name: "match",
        params: &["pattern", "string"],
        doc: "Returns true if the pattern matches anywhere in the string.",
        func: match_fn,
    }
}

// ---------- find ----------
fn find_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "find() expects 2 arguments (pattern, string), got {}",
                args.len()
            ),
        });
    }
    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "find() pattern must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };
    let text = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "find() text must be a string, got {}",
                    crate::utils::type_name(&args[1])
                ),
            });
        }
    };

    let re = compile_regex(pattern, span)?;
    if let Some(m) = re.find(text) {
        Ok(Value::String(m.as_str().to_string()))
    } else {
        Ok(Value::Nil)
    }
}
inventory::submit! {
    ModuleFunction {
        module: "regex",
        name: "find",
        params: &["pattern", "string"],
        doc: "Finds the first match of the pattern and returns it as a string, or nil if none.",
        func: find_fn,
    }
}

// ---------- find_all ----------
fn findall_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "find_all() expects 2 arguments (pattern, string), got {}",
                args.len()
            ),
        });
    }
    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "find_all() pattern must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };
    let text = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "find_all() text must be a string, got {}",
                    crate::utils::type_name(&args[1])
                ),
            });
        }
    };

    let re = compile_regex(pattern, span)?;
    let matches: Vec<Value> = re
        .find_iter(text)
        .map(|m| Value::String(m.as_str().to_string()))
        .collect();
    Ok(Value::List(Rc::new(RefCell::new(matches))))
}
inventory::submit! {
    ModuleFunction {
        module: "regex",
        name: "find_all",
        params: &["pattern", "string"],
        doc: "Finds all matches of the pattern and returns them as a list of strings.",
        func: findall_fn,
    }
}

// ---------- replace ----------
fn replace_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 3 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "replace() expects 3 arguments (pattern, string, replacement), got {}",
                args.len()
            ),
        });
    }
    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "replace() pattern must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };
    let text = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "replace() text must be a string, got {}",
                    crate::utils::type_name(&args[1])
                ),
            });
        }
    };
    let replacement = match &args[2] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "replace() replacement must be a string, got {}",
                    crate::utils::type_name(&args[2])
                ),
            });
        }
    };

    let re = compile_regex(pattern, span)?;
    let result = re.replace_all(text, replacement.as_str()).to_string();
    Ok(Value::String(result))
}
inventory::submit! {
    ModuleFunction {
        module: "regex",
        name: "replace",
        params: &["pattern", "string", "replacement"],
        doc: "Replaces all matches of the pattern with the replacement string.",
        func: replace_fn,
    }
}

// ---------- split ----------
fn split_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "split() expects 2 arguments (pattern, string), got {}",
                args.len()
            ),
        });
    }
    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "split() pattern must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };
    let text = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "split() text must be a string, got {}",
                    crate::utils::type_name(&args[1])
                ),
            });
        }
    };

    let re = compile_regex(pattern, span)?;
    let parts: Vec<Value> = re
        .split(text)
        .map(|s| Value::String(s.to_string()))
        .collect();
    Ok(Value::List(Rc::new(RefCell::new(parts))))
}
inventory::submit! {
    ModuleFunction {
        module: "regex",
        name: "split",
        params: &["pattern", "string"],
        doc: "Splits the string by regex pattern matches and returns a list of substrings.",
        func: split_fn,
    }
}
