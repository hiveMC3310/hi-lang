use crate::ast::Span;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::modules::{BuiltinFn, BuiltinModule};
use crate::value::{FileHandle, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, Read, Write};
use std::rc::Rc;

pub fn new_io_module() -> BuiltinModule {
    let vars: HashMap<String, Value> = HashMap::new();
    let mut funcs: HashMap<String, BuiltinFn> = HashMap::new();

    funcs.insert("open".to_string(), Rc::new(open_fn));
    funcs.insert("read".to_string(), Rc::new(read_fn));
    funcs.insert("readln".to_string(), Rc::new(readln_fn));
    funcs.insert("write".to_string(), Rc::new(write_fn));
    funcs.insert("writeln".to_string(), Rc::new(writeln_fn));
    funcs.insert("close".to_string(), Rc::new(close_fn));
    funcs.insert("eof".to_string(), Rc::new(eof_fn));

    BuiltinModule { vars, funcs }
}

// ---------- Files methods ----------
pub fn open_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
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
                    crate::utils::type_name(&file_val)
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
                    crate::utils::type_name(&file_val)
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
                    crate::utils::type_name(&file_val)
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
                    crate::utils::type_name(&file_val)
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
                    crate::utils::type_name(&file_val)
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
                    crate::utils::type_name(&file_val)
                ),
            });
        }
    };
    let handle = fh.borrow();
    let is_eof = handle.eof || handle.reader.is_none() && handle.writer.is_none();
    Ok(Value::Bool(is_eof))
}
