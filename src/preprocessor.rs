//! Preprocessor for handling `IMPORT` directives.
//!
//! Recursively processes source files, expanding `IMPORT "file.hi"` statements
//! by inlining the contents of the referenced file. Detects and prevents
//! cyclic imports and duplicate imports.

use crate::error::{InterpError, InterpResult};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Entry point: preprocesses a root file and returns the expanded contents
/// as a vector of source lines.
pub fn preprocess_file(root_path: &Path) -> InterpResult<Vec<String>> {
    let root_abs = std::fs::canonicalize(root_path).map_err(|e| InterpError::Io(e))?;
    let mut result = Vec::new();
    let mut imported = HashSet::new();
    let mut stack = Vec::new();
    process_file(&root_abs, &mut result, &mut imported, &mut stack)?;
    Ok(result)
}

/// Recursively processes a single file.
///
/// # Arguments
/// * `path` – absolute path to the file (normalized)
/// * `output` – accumulates the expanded source lines
/// * `imported` – set of already processed absolute paths (prevents duplicates)
/// * `stack` – current call stack of files being processed (used for cycle detection)
fn process_file(
    path: &Path,
    output: &mut Vec<String>,
    imported: &mut HashSet<PathBuf>,
    stack: &mut Vec<PathBuf>,
) -> InterpResult<()> {
    let abs_path = path.canonicalize().map_err(|e| InterpError::Io(e))?;

    // Cycle detection: if this file is already in the stack, we have a circular import.
    if stack.contains(&abs_path) {
        let cycle = stack
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(" -> ");
        return Err(InterpError::CyclicImport {
            path: format!("{} (cycle: {})", abs_path.display(), cycle),
        });
    }

    // Skip if already imported (prevents duplication).
    if imported.contains(&abs_path) {
        return Ok(());
    }

    // Mark as imported and push onto the stack for cycle detection.
    imported.insert(abs_path.clone());
    stack.push(abs_path.clone());

    let content = std::fs::read_to_string(path).map_err(|e| InterpError::Io(e))?;
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    for (line_num, line) in lines.into_iter().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        // Strip comments to avoid parsing `IMPORT` inside comments.
        let code_part = if let Some(pos) = trimmed.find("//") {
            &trimmed[..pos]
        } else {
            trimmed
        };
        let code_part = code_part.trim();

        // Try to parse an `IMPORT` directive.
        if let Some(import_path_str) = parse_import(code_part, line_num).map_err(|e| match e {
            InterpError::Syntax { line, message } => InterpError::Syntax {
                line,
                message: format!("{} (in file {})", message, abs_path.display()),
            },
            _ => e,
        })? {
            // Validate file extension.
            if !import_path_str.to_lowercase().ends_with(".hi") {
                return Err(InterpError::ImportError {
                    path: import_path_str.clone(),
                    message: "Imported file must have .hi extension".to_string(),
                    line: line_num,
                });
            }

            // Resolve the imported path relative to the current file's directory.
            let import_path = abs_path
                .parent()
                .unwrap_or(Path::new("."))
                .join(&import_path_str);

            // Recursively process the imported file.
            // Recursively process the imported file.
            if let Err(e) = process_file(&import_path, output, imported, stack) {
                let context_msg = format!(
                    "{} (while importing from file {}, line {})",
                    e,
                    abs_path.display(),
                    line_num
                );
                return Err(InterpError::ImportError {
                    path: import_path_str,
                    message: context_msg,
                    line: line_num,
                });
            }
        } else {
            // Not an `IMPORT` – pass the line through unchanged.
            output.push(line);
        }
    }

    // Clean up: pop this file from the stack.
    stack.pop();
    Ok(())
}

/// Parses a line to detect and extract an `IMPORT` directive.
///
/// Returns `Ok(Some(path))` if the line starts with `IMPORT` followed by a quoted string,
/// `Ok(None)` if the line is not an import, or an error if the syntax is malformed.
fn parse_import(line: &str, line_num: usize) -> InterpResult<Option<String>> {
    let trimmed = line.trim();
    if !trimmed.to_uppercase().starts_with("IMPORT") {
        return Ok(None);
    }
    let rest = trimmed[6..].trim();
    if rest.starts_with('"') && rest.ends_with('"') {
        let path = rest[1..rest.len() - 1].to_string();
        Ok(Some(path))
    } else {
        Err(InterpError::Syntax {
            line: line_num,
            message: "IMPORT must be followed by a quoted filename, e.g. IMPORT \"lib.hi\""
                .to_string(),
        })
    }
}
