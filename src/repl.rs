//! Read-Eval-Print Loop (REPL) for interactive Hi sessions.

use crate::interpreter::Interpreter;
use crate::preprocessor::preprocess_file;
use crate::tokenizer::Tokenizer;
use colored::Colorize;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Starts the REPL loop.
pub fn repl_run() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = DefaultEditor::new()?;
    let mut interpreter = Interpreter::new(vec![]);
    let mut loaded_files: HashSet<PathBuf> = HashSet::new();

    println!(
        "Hi REPL v{} — type :exit or :quit to quit",
        env!("CARGO_PKG_VERSION")
    );
    println!("Enter commands (multi-line blocks like IF/WHILE/FUNC are supported)");
    println!();

    let mut buffer: Vec<String> = Vec::new();

    loop {
        let balance = block_balance(&buffer);
        let prompt = if balance > 0 { "...> " } else { "hi> " };

        let line = match rl.readline(prompt) {
            Ok(l) => l,
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Special commands
        if trimmed == ":exit" || trimmed == ":quit" {
            break;
        }
        if trimmed == ":clear" {
            interpreter.clear_state();
            loaded_files.clear();
            continue;
        }
        if trimmed == ":vars" {
            for (k, v) in &interpreter.globals {
                println!("{} = {}", k, v);
            }
            continue;
        }
        if trimmed == ":stack" {
            println!("Stack: {:?}", interpreter.stack);
            continue;
        }

        if trimmed.starts_with(":load") {
            // Extract the argument after `:load` – may be quoted.

            let arg = trimmed[5..].trim();
            let path_str = parse_load_arg(arg);
            let path = Path::new(&path_str);
            let abs_path = match std::fs::canonicalize(path) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Cannot resolve path: {}", e);
                    continue;
                }
            };

            // Prevent reloading the same file (use :clear to reset).
            if loaded_files.contains(&abs_path) {
                eprintln!(
                    "File '{}' already loaded. Use :clear and :load to reload, or :reload to force.",
                    path_str
                );
                continue;
            }

            match preprocess_file(path) {
                Ok(lines) => {
                    loaded_files.insert(abs_path.clone());
                    let start_line = interpreter.lines.len();
                    interpreter.lines.extend(lines);
                    // Rebuild jump maps after adding lines.
                    if let Err(e) = interpreter.build_maps() {
                        eprintln!("{}", e);
                        continue;
                    }
                    // Execute the newly added lines.
                    if let Err(e) = interpreter.run_from(start_line) {
                        eprintln!("{} {}", "error:".red().bold(), e);
                        if let Some(line) = e.line() {
                            eprintln!(
                                "{} {} {} {}",
                                "at line".yellow().bold(),
                                line.to_string().cyan().bold(),
                                "in file".yellow().bold(),
                                path_str.cyan().bold()
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "error:".red().bold(), e);
                    if let Some(line) = e.line() {
                        eprintln!(
                            "{} {} {} {}",
                            "at line".yellow().bold(),
                            line.to_string().cyan().bold(),
                            "in file".yellow().bold(),
                            path_str.cyan().bold()
                        );
                    }
                }
            }
            continue;
        }

        rl.add_history_entry(&line)?;
        buffer.push(line);

        // If the block is complete (balanced), execute it
        if block_balance(&buffer) == 0 {
            let start_line = interpreter.lines.len();
            interpreter.lines.extend(buffer.clone());

            // Execute only the new lines
            if let Err(e) = interpreter.run_from(start_line) {
                eprintln!("{}", e);
            }
            buffer.clear();
        }
    }

    Ok(())
}

/// Counts the nesting level of IF/WHILE/FUNC blocks.
/// Returns 0 when blocks are balanced.
fn block_balance(lines: &[String]) -> i32 {
    let mut balance = 0;
    for line in lines {
        if let Ok(tokens) = Tokenizer::tokenize(line, 0) {
            if tokens.is_empty() {
                continue;
            }
            let cmd = tokens[0].to_uppercase();
            match cmd.as_str() {
                "IF" | "WHILE" | "FUNC" => balance += 1,
                "ENDIF" | "DO" | "ENDF" => balance -= 1,
                _ => (),
            }
        }
        if balance < 0 {
            balance = 0;
        }
    }
    balance
}

/// Parses the argument of the `:load` command.
///
/// If the argument is enclosed in double quotes, strips them;
/// otherwise returns the argument unchanged.
fn parse_load_arg(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}
