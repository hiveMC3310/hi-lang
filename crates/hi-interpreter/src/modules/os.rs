use crate::ast::Span;
use crate::builtins::ModuleFunction;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------- Вспомогательная функция ----------
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

// ---------- exists ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "exists",
        params: &["path"],
        doc: "Returns true if the file or directory at the given path exists.",
        func: exists_fn,
    }
}

// ---------- getenv ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "getenv",
        params: &["key"],
        doc: "Returns the value of the environment variable, or nil if not set.",
        func: getenv_fn,
    }
}

// ---------- setenv ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "setenv",
        params: &["key", "value"],
        doc: "Sets an environment variable. Uses unsafe internally and is not thread-safe.",
        func: setenv_fn,
    }
}

// ---------- unsetenv ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "unsetenv",
        params: &["key"],
        doc: "Removes an environment variable. Uses unsafe internally and is not thread-safe.",
        func: unsetenv_fn,
    }
}

// ---------- exec ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "exec",
        params: &["command"],
        doc: "Executes a shell command and returns the exit code (0 for success). Note: uses system shell, avoid unsanitized input.",
        func: exec_fn,
    }
}

// ---------- exit ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "exit",
        params: &["code"],
        doc: "Terminates the current process with the given exit code.",
        func: exit_fn,
    }
}

// ---------- chdir ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "chdir",
        params: &["path"],
        doc: "Changes the current working directory to the given path.",
        func: chdir_fn,
    }
}

// ---------- cwd ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "cwd",
        params: &[],
        doc: "Returns the current working directory as a string.",
        func: cwd_fn,
    }
}

// ---------- listdir ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "listdir",
        params: &["path"],
        doc: "Returns a list of filenames in the given directory.",
        func: listdir_fn,
    }
}

// ---------- mkdir ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "mkdir",
        params: &["path"],
        doc: "Creates a new directory at the given path.",
        func: mkdir_fn,
    }
}

// ---------- rmdir ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "rmdir",
        params: &["path"],
        doc: "Removes an empty directory at the given path.",
        func: rmdir_fn,
    }
}

// ---------- remove ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "remove",
        params: &["path"],
        doc: "Deletes a file at the given path.",
        func: remove_fn,
    }
}

// ---------- rename ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "rename",
        params: &["old", "new"],
        doc: "Renames (moves) a file or directory from old to new path. Works across filesystems? Not guaranteed.",
        func: rename_fn,
    }
}

// ---------- move ----------
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

    if let Err(e) = fs::rename(&source, &dest) {
        let metadata = fs::metadata(&source).map_err(|e| InterpError::Runtime {
            span: *span,
            message: format!("Cannot access source '{}': {}", source, e),
        })?;
        if metadata.is_file() {
            fs::copy(&source, &dest).map_err(|e| InterpError::Runtime {
                span: *span,
                message: format!("Failed to copy file '{}' to '{}': {}", source, dest, e),
            })?;
            fs::remove_file(&source).map_err(|e| InterpError::Runtime {
                span: *span,
                message: format!("Failed to remove source file after copy: {}", e),
            })?;
        } else {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("Failed to move directory '{}' to '{}': {}", source, dest, e),
            });
        }
    }
    Ok(Value::Nil)
}
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "move",
        params: &["source", "dest"],
        doc: "Moves a file or directory. Tries rename first; if that fails (e.g. cross-device), falls back to copy+delete for files only (directories fail).",
        func: move_fn,
    }
}

// ---------- copy ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "copy",
        params: &["source", "dest"],
        doc: "Copies a file or an empty directory (non-empty directories are not supported).",
        func: copy_fn,
    }
}

// ---------- stat ----------
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
inventory::submit! {
    ModuleFunction {
        module: "os",
        name: "stat",
        params: &["path"],
        doc: "Returns a dictionary with file metadata: size, modified, accessed, created timestamps (seconds since epoch), and boolean flags is_dir, is_file.",
        func: stat_fn,
    }
}
