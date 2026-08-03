use crate::ast::Span;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::modules::{BuiltinFn, BuiltinModule};
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Creates a new built-in module for OS-related functions.
pub fn new_os_module() -> BuiltinModule {
    let vars = HashMap::new();
    let mut funcs: HashMap<String, BuiltinFn> = HashMap::new();

    funcs.insert("getenv".to_string(), Rc::new(getenv_fn));
    funcs.insert("setenv".to_string(), Rc::new(setenv_fn));
    funcs.insert("unsetenv".to_string(), Rc::new(unsetenv_fn));
    funcs.insert("exec".to_string(), Rc::new(exec_fn));
    funcs.insert("exit".to_string(), Rc::new(exit_fn));
    funcs.insert("chdir".to_string(), Rc::new(chdir_fn));
    funcs.insert("cwd".to_string(), Rc::new(cwd_fn));
    funcs.insert("listdir".to_string(), Rc::new(listdir_fn));
    funcs.insert("mkdir".to_string(), Rc::new(mkdir_fn));
    funcs.insert("rmdir".to_string(), Rc::new(rmdir_fn));
    funcs.insert("remove".to_string(), Rc::new(remove_fn));
    funcs.insert("rename".to_string(), Rc::new(rename_fn));
    funcs.insert("move".to_string(), Rc::new(move_fn));
    funcs.insert("copy".to_string(), Rc::new(copy_fn));
    funcs.insert("stat".to_string(), Rc::new(stat_fn));
    funcs.insert("exists".to_string(), Rc::new(exists_fn));

    BuiltinModule { funcs, vars }
}

/// exists(path) – returns TRUE if the file or directory exists, FALSE otherwise.
fn exists_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("exists() expects 1 argument (path), got {}", args.len()),
        });
    }
    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "exists() path must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    let exists = Path::new(path).exists();
    Ok(Value::Bool(exists))
}

/// getenv(key) – returns the value of environment variable `key`, or nil if not set.
fn getenv_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("getenv() expects 1 argument (key), got {}", args.len()),
        });
    }
    let key = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "getenv() key must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    match std::env::var(key) {
        Ok(val) => Ok(Value::String(val)),
        Err(std::env::VarError::NotPresent) => Ok(Value::Nil),
        Err(e) => Err(InterpError::Runtime {
            span: *span,
            message: format!("Failed to read environment variable: {}", e),
        }),
    }
}

/// setenv(key, value) – sets an environment variable to the given value. Returns nil.
fn setenv_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "setenv() expects 2 arguments (key, value), got {}",
                args.len()
            ),
        });
    }
    let key = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "setenv() key must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };
    let value = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "setenv() value must be a string, got {}",
                    crate::utils::type_name(&args[1])
                ),
            });
        }
    };

    unsafe {
        std::env::set_var(key, value);
    }
    Ok(Value::Nil)
}

/// unsetenv(key) – removes an environment variable. Returns nil.
fn unsetenv_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("unsetenv() expects 1 argument (key), got {}", args.len()),
        });
    }
    let key = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "unsetenv() key must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    unsafe {
        std::env::remove_var(key);
    }
    Ok(Value::Nil)
}

/// exec(command) – executes a shell command and returns its exit code as an integer,
/// or nil if the command could not be started.
fn exec_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "exec() expects 1 argument (command string), got {}",
                args.len()
            ),
        });
    }
    let cmd = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "exec() command must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    // Determine shell based on OS
    #[cfg(target_os = "windows")]
    let shell = "cmd";
    #[cfg(target_os = "windows")]
    let shell_arg = "/C";

    #[cfg(not(target_os = "windows"))]
    let shell = "sh";
    #[cfg(not(target_os = "windows"))]
    let shell_arg = "-c";

    let status = std::process::Command::new(shell)
        .arg(shell_arg)
        .arg(cmd)
        .status();

    match status {
        Ok(status) => {
            if status.success() {
                Ok(Value::Int(0))
            } else {
                Ok(Value::Int(status.code().unwrap_or(-1) as i64))
            }
        }
        Err(e) => Err(InterpError::Runtime {
            span: *span,
            message: format!("Failed to execute command: {}", e),
        }),
    }
}

/// exit(code) – terminates the interpreter process with the given exit code.
fn exit_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("exit() expects 1 argument (code), got {}", args.len()),
        });
    }
    let code = match &args[0] {
        Value::Int(i) => *i,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "exit() code must be an integer, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    std::process::exit(code as i32);
}

/// chdir(path) – changes the current working directory to `path`. Returns nil.
fn chdir_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("chdir() expects 1 argument (path), got {}", args.len()),
        });
    }
    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "chdir() path must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    std::env::set_current_dir(path).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Failed to change directory: {}", e),
    })?;
    Ok(Value::Nil)
}

/// cwd() – returns the current working directory as a string.
fn cwd_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 0 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("cwd() expects no arguments, got {}", args.len()),
        });
    }

    let path = std::env::current_dir().map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Failed to get current directory: {}", e),
    })?;
    Ok(Value::String(path.to_string_lossy().to_string()))
}

/// listdir(path) – returns a list of filenames in the given directory.
fn listdir_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("listdir() expects 1 argument (path), got {}", args.len()),
        });
    }
    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "listdir() path must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    let entries = fs::read_dir(path).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Failed to read directory: {}", e),
    })?;

    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| InterpError::Runtime {
            span: *span,
            message: format!("Error reading directory entry: {}", e),
        })?;
        let name = entry.file_name().to_string_lossy().to_string();
        files.push(Value::String(name));
    }

    Ok(Value::List(Rc::new(RefCell::new(files))))
}

/// mkdir(path) – creates a new directory. Returns nil.
fn mkdir_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("mkdir() expects 1 argument (path), got {}", args.len()),
        });
    }
    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "mkdir() path must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    fs::create_dir(path).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Failed to create directory: {}", e),
    })?;
    Ok(Value::Nil)
}

/// rmdir(path) – removes an empty directory. Returns nil.
fn rmdir_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("rmdir() expects 1 argument (path), got {}", args.len()),
        });
    }
    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "rmdir() path must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    fs::remove_dir(path).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Failed to remove directory: {}", e),
    })?;
    Ok(Value::Nil)
}

/// remove(path) – deletes a file. Returns nil.
fn remove_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("remove() expects 1 argument (path), got {}", args.len()),
        });
    }
    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "remove() path must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    fs::remove_file(path).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Failed to remove file: {}", e),
    })?;
    Ok(Value::Nil)
}

/// rename(old_path, new_path) – renames a file or empty directory within the same filesystem.
/// Returns nil on success.
fn rename_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "rename() expects 2 arguments (old, new), got {}",
                args.len()
            ),
        });
    }
    let old = get_string_arg(&args[0], "rename", span)?;
    let new = get_string_arg(&args[1], "rename", span)?;

    fs::rename(&old, &new).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Failed to rename '{}' to '{}': {}", old, new, e),
    })?;
    Ok(Value::Nil)
}

/// move(source, dest) – moves a file or empty directory.
/// Tries rename first; for files, falls back to copy+remove if rename fails (e.g., cross-device).
/// For directories, only rename is supported; cross-device move will fail.
fn move_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "move() expects 2 arguments (source, dest), got {}",
                args.len()
            ),
        });
    }
    let source = get_string_arg(&args[0], "move", span)?;
    let dest = get_string_arg(&args[1], "move", span)?;

    // Try rename first
    if let Err(e) = fs::rename(&source, &dest) {
        // If it's a cross-device error, we can fallback to copy + remove for files
        let metadata = fs::metadata(&source).map_err(|e| InterpError::Runtime {
            span: *span,
            message: format!("Cannot access source '{}': {}", source, e),
        })?;
        if metadata.is_file() {
            // Copy file
            fs::copy(&source, &dest).map_err(|e| InterpError::Runtime {
                span: *span,
                message: format!("Failed to copy file '{}' to '{}': {}", source, dest, e),
            })?;
            // Remove source
            fs::remove_file(&source).map_err(|e| InterpError::Runtime {
                span: *span,
                message: format!("Failed to remove source file after copy: {}", e),
            })?;
        } else {
            // For directories, we cannot easily fallback to recursive copy, so we just error.
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("Failed to move directory '{}' to '{}': {}", source, dest, e),
            });
        }
    }
    Ok(Value::Nil)
}

/// copy(source, dest) – copies a file or an empty directory.
/// If source is a directory, it must be empty; otherwise an error is thrown.
/// Returns nil on success.
fn copy_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "copy() expects 2 arguments (source, dest), got {}",
                args.len()
            ),
        });
    }
    let source = get_string_arg(&args[0], "copy", span)?;
    let dest = get_string_arg(&args[1], "copy", span)?;

    let metadata = fs::metadata(&source).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Cannot access source '{}': {}", source, e),
    })?;

    if metadata.is_file() {
        fs::copy(&source, &dest).map_err(|e| InterpError::Runtime {
            span: *span,
            message: format!("Failed to copy file '{}' to '{}': {}", source, dest, e),
        })?;
    } else if metadata.is_dir() {
        // Check if directory is empty
        let mut entries = fs::read_dir(&source).map_err(|e| InterpError::Runtime {
            span: *span,
            message: format!("Failed to read directory '{}': {}", source, e),
        })?;
        if entries.next().is_some() {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("Cannot copy non-empty directory '{}'", source),
            });
        }
        // Create destination directory
        fs::create_dir(&dest).map_err(|e| InterpError::Runtime {
            span: *span,
            message: format!("Failed to create directory '{}': {}", dest, e),
        })?;
    } else {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("Source '{}' is neither a file nor a directory", source),
        });
    }
    Ok(Value::Nil)
}

fn get_string_arg(val: &Value, func_name: &str, span: &Span) -> InterpResult<String> {
    match val {
        Value::String(s) => Ok(s.clone()),
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "{}() path must be a string, got {}",
                func_name,
                crate::utils::type_name(val)
            ),
        }),
    }
}

/// stat(path) – returns a dictionary with file metadata.
/// Fields: size (int), modified (int, unix timestamp), created (int), accessed (int),
/// is_dir (bool), is_file (bool).
fn stat_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("stat() expects 1 argument (path), got {}", args.len()),
        });
    }
    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "stat() path must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    let metadata = fs::metadata(path).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Failed to stat file: {}", e),
    })?;

    let file_type = metadata.file_type();
    let is_dir = file_type.is_dir();
    let is_file = file_type.is_file();

    let size = metadata.len() as i64;

    // Helper to convert SystemTime to seconds since epoch
    let to_timestamp = |time: SystemTime| -> i64 {
        time.duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    };

    let modified = to_timestamp(metadata.modified().unwrap_or(SystemTime::now()));
    let accessed = to_timestamp(metadata.accessed().unwrap_or(SystemTime::now()));
    let created = to_timestamp(metadata.created().unwrap_or(SystemTime::now()));

    let mut map = HashMap::new();
    map.insert(Value::String("size".to_string()), Value::Int(size));
    map.insert(Value::String("modified".to_string()), Value::Int(modified));
    map.insert(Value::String("accessed".to_string()), Value::Int(accessed));
    map.insert(Value::String("created".to_string()), Value::Int(created));
    map.insert(Value::String("is_dir".to_string()), Value::Bool(is_dir));
    map.insert(Value::String("is_file".to_string()), Value::Bool(is_file));

    Ok(Value::Dict(Rc::new(RefCell::new(map))))
}
