use crate::ast::Span;
use crate::builtins::ModuleFunction;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::value::Value;
use std::path::{Path, PathBuf};

/// Helper: converts a Hi Value to a &str.
fn as_str<'a>(val: &'a Value, func_name: &str, span: &Span) -> InterpResult<&'a str> {
    match val {
        Value::String(s) => Ok(s.as_str()),
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "{}() expects a string argument, got {}",
                func_name,
                crate::utils::type_name(val)
            ),
        }),
    }
}

// ---------- join ----------
fn join_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }
    let mut path = PathBuf::new();
    for (i, arg) in args.iter().enumerate() {
        let s = as_str(arg, "join", span)?;
        if i == 0 {
            path = PathBuf::from(s);
        } else {
            path.push(s);
        }
    }
    Ok(Value::String(path.to_string_lossy().to_string()))
}
inventory::submit! {
    ModuleFunction {
        module: "path",
        name: "join",
        params: &["...parts"],
        doc: "Joins any number of path parts using the system's path separator.",
        func: join_fn,
    }
}

// ---------- basename ----------
fn basename_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("basename() expects 1 argument, got {}", args.len()),
        });
    }
    let s = as_str(&args[0], "basename", span)?;
    if s.ends_with('/') || s.ends_with('\\') {
        return Ok(Value::Nil);
    }
    let path = Path::new(s);
    match path.file_name() {
        Some(name) => Ok(Value::String(name.to_string_lossy().to_string())),
        None => Ok(Value::Nil),
    }
}
inventory::submit! {
    ModuleFunction {
        module: "path",
        name: "basename",
        params: &["path"],
        doc: "Returns the last component of the path, or nil if it ends with a separator.",
        func: basename_fn,
    }
}

// ---------- dirname ----------
fn dirname_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("dirname() expects 1 argument, got {}", args.len()),
        });
    }
    let s = as_str(&args[0], "dirname", span)?;

    let ends_with_sep = s.ends_with('/') || s.ends_with('\\');
    if ends_with_sep {
        let trimmed = s.trim_end_matches(&['/', '\\'] as &[_]);
        if trimmed.is_empty() {
            return Ok(Value::Nil);
        }
        return Ok(Value::String(trimmed.to_string()));
    } else {
        let path = Path::new(s);
        match path.parent() {
            Some(parent) => Ok(Value::String(parent.to_string_lossy().to_string())),
            None => Ok(Value::Nil),
        }
    }
}
inventory::submit! {
    ModuleFunction {
        module: "path",
        name: "dirname",
        params: &["path"],
        doc: "Returns the parent directory of the path, or nil if none. Removes trailing separators.",
        func: dirname_fn,
    }
}

// ---------- extname ----------
fn extname_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("extname() expects 1 argument, got {}", args.len()),
        });
    }
    let s = as_str(&args[0], "extname", span)?;
    let path = Path::new(s);
    match path.extension() {
        Some(ext) => {
            let ext_str = ext.to_string_lossy();
            if ext_str.starts_with('.') {
                Ok(Value::String(ext_str.to_string()))
            } else {
                Ok(Value::String(format!(".{}", ext_str)))
            }
        }
        None => Ok(Value::String(String::new())),
    }
}
inventory::submit! {
    ModuleFunction {
        module: "path",
        name: "extname",
        params: &["path"],
        doc: "Returns the file extension including the leading dot, or empty string if none.",
        func: extname_fn,
    }
}

// ---------- is_absolute ----------
fn is_absolute_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("is_absolute() expects 1 argument, got {}", args.len()),
        });
    }
    let s = as_str(&args[0], "is_absolute", span)?;
    let path = Path::new(s);
    Ok(Value::Bool(path.is_absolute()))
}
inventory::submit! {
    ModuleFunction {
        module: "path",
        name: "is_absolute",
        params: &["path"],
        doc: "Returns true if the path is absolute, false otherwise.",
        func: is_absolute_fn,
    }
}

// ---------- normalize ----------
fn normalize_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("normalize() expects 1 argument, got {}", args.len()),
        });
    }
    let s = as_str(&args[0], "normalize", span)?;
    let path = Path::new(s);
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::RootDir => {
                result.push(component.as_os_str());
            }
            std::path::Component::CurDir => {
                // skip
            }
            std::path::Component::ParentDir => {
                if !result.pop() {
                    if result.as_os_str().is_empty() {
                        result.push("..");
                    }
                }
            }
            _ => {
                result.push(component.as_os_str());
            }
        }
    }
    Ok(Value::String(result.to_string_lossy().to_string()))
}
inventory::submit! {
    ModuleFunction {
        module: "path",
        name: "normalize",
        params: &["path"],
        doc: "Normalizes a path by resolving '.' and '..' components without accessing the filesystem.",
        func: normalize_fn,
    }
}
