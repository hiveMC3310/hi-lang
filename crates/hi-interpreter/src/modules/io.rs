use crate::ast::Span;
use crate::builtins::ModuleFunction;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::value::{FileHandle, Value};
use std::cell::RefCell;
use std::io::{BufRead, Read, Write};
use std::rc::Rc;

// ---------- open ----------
fn open_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "open() expects 2 arguments (path, mode), got {}",
                args.len()
            ),
        });
    }
    let path_val = &args[0];
    let mode_val = &args[1];
    let path = match path_val {
        Value::String(s) => s.clone(),
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: "open() path must be a string".to_string(),
            });
        }
    };
    let mode = match mode_val {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: "open() mode must be a string".to_string(),
            });
        }
    };

    let handle = match mode.as_str() {
        "r" => {
            let file = std::fs::File::open(&path).map_err(|e| InterpError::Runtime {
                span: *span,
                message: format!("Cannot open file '{}' for reading: {}", path, e),
            })?;
            FileHandle::new_reader(path, file)
        }
        "w" => {
            let file = std::fs::File::create(&path).map_err(|e| InterpError::Runtime {
                span: *span,
                message: format!("Cannot create file '{}' for writing: {}", path, e),
            })?;
            FileHandle::new_writer(path, file)
        }
        "a" => {
            let file = std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .map_err(|e| InterpError::Runtime {
                    span: *span,
                    message: format!("Cannot open file '{}' for appending: {}", path, e),
                })?;
            FileHandle::new_writer(path, file)
        }
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("Invalid file mode '{}', use 'r', 'w', or 'a'", mode),
            });
        }
    };
    Ok(Value::File(Rc::new(RefCell::new(handle))))
}
inventory::submit! {
    ModuleFunction {
        module: "io",
        name: "open",
        params: &["path", "mode"],
        doc: "Opens a file with the given mode ('r', 'w', or 'a') and returns a file handle.",
        func: open_fn,
    }
}

// ---------- close ----------
fn close_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("close() expects 1 argument (file), got {}", args.len()),
        });
    }
    let file_val = &args[0];
    let fh = match file_val {
        Value::File(fh) => fh,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "close() expects a file, got {}",
                    crate::utils::type_name(file_val)
                ),
            });
        }
    };
    let mut handle = fh.borrow_mut();
    if let Some(ref mut writer) = handle.writer {
        writer.flush().map_err(|e| InterpError::Io {
            source: e,
            span: Some(*span),
        })?;
    }
    handle.reader = None;
    handle.writer = None;
    Ok(Value::Nil)
}
inventory::submit! {
    ModuleFunction {
        module: "io",
        name: "close",
        params: &["file"],
        doc: "Closes the file handle, flushing any pending writes.",
        func: close_fn,
    }
}

// ---------- read ----------
fn read_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("read() expects 1 argument (file), got {}", args.len()),
        });
    }
    let file_val = &args[0];
    let fh = match file_val {
        Value::File(fh) => fh,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "read() expects a file, got {}",
                    crate::utils::type_name(file_val)
                ),
            });
        }
    };
    let mut handle = fh.borrow_mut();
    let reader = handle.reader.as_mut().ok_or_else(|| InterpError::Runtime {
        span: *span,
        message: "File is not open for reading".to_string(),
    })?;
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .map_err(|e| InterpError::Io {
            source: e,
            span: Some(*span),
        })?;
    handle.eof = true;
    Ok(Value::String(content))
}
inventory::submit! {
    ModuleFunction {
        module: "io",
        name: "read",
        params: &["file"],
        doc: "Reads the entire contents of the file as a string.",
        func: read_fn,
    }
}

// ---------- readln ----------
fn readln_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("readln() expects 1 argument (file), got {}", args.len()),
        });
    }
    let file_val = &args[0];
    let fh = match file_val {
        Value::File(fh) => fh,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "readln() expects a file, got {}",
                    crate::utils::type_name(file_val)
                ),
            });
        }
    };
    let mut handle = fh.borrow_mut();
    let reader = handle.reader.as_mut().ok_or_else(|| InterpError::Runtime {
        span: *span,
        message: "File is not open for reading".to_string(),
    })?;
    let mut line = String::new();
    let bytes = reader.read_line(&mut line).map_err(|e| InterpError::Io {
        source: e,
        span: Some(*span),
    })?;
    if bytes == 0 {
        handle.eof = true;
    }
    Ok(Value::String(line))
}
inventory::submit! {
    ModuleFunction {
        module: "io",
        name: "readln",
        params: &["file"],
        doc: "Reads a single line from the file (including the newline).",
        func: readln_fn,
    }
}

// ---------- write ----------
fn write_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "write() expects 2 arguments (file, value), got {}",
                args.len()
            ),
        });
    }
    let file_val = &args[0];
    let value = &args[1];
    let fh = match file_val {
        Value::File(fh) => fh,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "write() expects a file, got {}",
                    crate::utils::type_name(file_val)
                ),
            });
        }
    };
    let mut handle = fh.borrow_mut();
    let writer = handle.writer.as_mut().ok_or_else(|| InterpError::Runtime {
        span: *span,
        message: "File is not open for writing".to_string(),
    })?;
    write!(writer, "{}", value).map_err(|e| InterpError::Io {
        source: e,
        span: Some(*span),
    })?;
    Ok(Value::Nil)
}
inventory::submit! {
    ModuleFunction {
        module: "io",
        name: "write",
        params: &["file", "value"],
        doc: "Writes a value's string representation to the file (no newline added).",
        func: write_fn,
    }
}

// ---------- writeln ----------
fn writeln_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "writeln() expects 2 arguments (file, value), got {}",
                args.len()
            ),
        });
    }
    let file_val = &args[0];
    let value = &args[1];
    let fh = match file_val {
        Value::File(fh) => fh,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "writeln() expects a file, got {}",
                    crate::utils::type_name(file_val)
                ),
            });
        }
    };
    let mut handle = fh.borrow_mut();
    let writer = handle.writer.as_mut().ok_or_else(|| InterpError::Runtime {
        span: *span,
        message: "File is not open for writing".to_string(),
    })?;
    writeln!(writer, "{}", value).map_err(|e| InterpError::Io {
        source: e,
        span: Some(*span),
    })?;
    Ok(Value::Nil)
}
inventory::submit! {
    ModuleFunction {
        module: "io",
        name: "writeln",
        params: &["file", "value"],
        doc: "Writes a value's string representation to the file, appending a newline.",
        func: writeln_fn,
    }
}

// ---------- eof ----------
fn eof_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("eof() expects 1 argument (file), got {}", args.len()),
        });
    }
    let file_val = &args[0];
    let fh = match file_val {
        Value::File(fh) => fh,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "eof() expects a file, got {}",
                    crate::utils::type_name(file_val)
                ),
            });
        }
    };
    let handle = fh.borrow();
    let is_eof = handle.eof || handle.reader.is_none() && handle.writer.is_none();
    Ok(Value::Bool(is_eof))
}
inventory::submit! {
    ModuleFunction {
        module: "io",
        name: "eof",
        params: &["file"],
        doc: "Returns true if the file has reached end-of-file or is closed.",
        func: eof_fn,
    }
}
