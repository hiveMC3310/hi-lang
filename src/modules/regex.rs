use crate::ast::Span;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::modules::{BuiltinFn, BuiltinModule};
use crate::value::Value;
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Creates a new built-in module for regular expression operations.
pub fn new_regex_module() -> BuiltinModule {
    let vars = HashMap::new();
    let mut funcs: HashMap<String, BuiltinFn> = HashMap::new();

    funcs.insert("match".to_string(), Rc::new(match_fn));
    funcs.insert("find".to_string(), Rc::new(find_fn));
    funcs.insert("find_all".to_string(), Rc::new(findall_fn));
    funcs.insert("replace".to_string(), Rc::new(replace_fn));
    funcs.insert("split".to_string(), Rc::new(split_fn));

    BuiltinModule { funcs, vars }
}

/// Helper: compiles a regex pattern, returning an error on failure.
fn compile_regex(pattern: &str, span: &Span) -> InterpResult<Regex> {
    Regex::new(pattern).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Invalid regex pattern: {}", e),
    })
}

/// regex:match(pattern, string) – returns TRUE if the pattern matches anywhere in the string.
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

/// regex:find(pattern, string) – returns the first match as a string, or nil if no match.
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

/// regex:find_all(pattern, string) – returns a list of all non-overlapping matches.
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

/// regex:replace(pattern, string, replacement) – replaces all occurrences of the pattern with the replacement string.
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

/// regex:split(pattern, string) – splits the string by the regex pattern and returns a list of substrings.
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
