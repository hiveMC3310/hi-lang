use crate::ast::Span;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::modules::{BuiltinFn, BuiltinModule};
use crate::value::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub fn new_path_module() -> BuiltinModule {
    let vars = HashMap::new();
    let mut funcs: HashMap<String, BuiltinFn> = HashMap::new();

    funcs.insert("join".to_string(), Rc::new(join_fn));
    funcs.insert("basename".to_string(), Rc::new(basename_fn));
    funcs.insert("dirname".to_string(), Rc::new(dirname_fn));
    funcs.insert("extname".to_string(), Rc::new(extname_fn));
    funcs.insert("is_absolute".to_string(), Rc::new(is_absolute_fn));
    funcs.insert("normalize".to_string(), Rc::new(normalize_fn));

    BuiltinModule { funcs, vars }
}

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

/// join(parts...) – concatenates path components.
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

/// basename(path) – returns the last component, or nil if path ends with a separator.
fn basename_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("basename() expects 1 argument, got {}", args.len()),
        });
    }
    let s = as_str(&args[0], "basename", span)?;
    // If path ends with a separator, there is no basename
    if s.ends_with('/') || s.ends_with('\\') {
        return Ok(Value::Nil);
    }
    let path = Path::new(s);
    match path.file_name() {
        Some(name) => Ok(Value::String(name.to_string_lossy().to_string())),
        None => Ok(Value::Nil),
    }
}

/// dirname(path) – returns the parent directory of the path.
/// If the path ends with a separator, returns the path without trailing separators.
/// Otherwise, returns the path without its last component.
/// Returns nil if there is no parent (e.g., root or relative path without directory).
fn dirname_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("dirname() expects 1 argument, got {}", args.len()),
        });
    }
    let s = as_str(&args[0], "dirname", span)?;

    // Check if the path ends with a separator
    let ends_with_sep = s.ends_with('/') || s.ends_with('\\');
    if ends_with_sep {
        // Remove all trailing separators
        let trimmed = s.trim_end_matches(&['/', '\\'] as &[_]);
        if trimmed.is_empty() {
            // Path consisted only of separators (e.g., "/" or "\\")
            // Return nil, or could return the root itself, but we choose nil for simplicity.
            return Ok(Value::Nil);
        }
        return Ok(Value::String(trimmed.to_string()));
    } else {
        // No trailing separator, use standard parent
        let path = Path::new(s);
        match path.parent() {
            Some(parent) => Ok(Value::String(parent.to_string_lossy().to_string())),
            None => Ok(Value::Nil),
        }
    }
}

/// extname(path) – returns the extension with leading dot, or empty string.
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

/// is_absolute(path) – returns TRUE if absolute, FALSE otherwise.
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

/// normalize(path) – resolves . and .. without touching the filesystem.
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
                    // If we cannot pop, push ".." (for relative paths that go above root)
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
