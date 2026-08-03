//! Read-Eval-Print Loop (REPL) for interactive Hi sessions.

use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::parser::lexer::{Lexer, TokenKind};
use colored::Colorize;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Starts the REPL loop.
pub fn repl_run() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = DefaultEditor::new()?;
    let mut interpreter = Interpreter::new();
    let mut loaded_files: HashSet<PathBuf> = HashSet::new();
    let mut buffer: Vec<String> = Vec::new();

    println!(
        "Hi REPL v{} — type :exit or :quit to quit",
        env!("CARGO_PKG_VERSION")
    );
    println!();

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
            interpreter = Interpreter::new();
            loaded_files.clear();
            buffer.clear();
            continue;
        }
        if trimmed == ":vars" {
            for (k, v) in &interpreter.env.vars {
                println!("{} = {}", k, v);
            }
            if !interpreter.env.functions.is_empty() {
                println!("Functions:");
                for (name, (params, _)) in &interpreter.env.functions {
                    println!("  {}({})", name, params.join(", "));
                }
            }
            continue;
        }
        if trimmed == ":stack" {
            println!("Stack is not used in AST mode.");
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

            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Cannot read file: {}", e);
                    continue;
                }
            };

            match Lexer::tokenize(&source) {
                Ok(tokens) => {
                    let mut parser = Parser::new(&tokens);
                    match parser.parse() {
                        Ok(program) => {
                            loaded_files.insert(abs_path.clone());
                            interpreter.current_file = Some(abs_path);
                            if let Err(e) = interpreter.run(&program) {
                                eprintln!("{} {}", "error:".red().bold(), e);
                                if let Some(span) = e.span() {
                                    eprintln!(
                                        "{} {} {} {} {}",
                                        "at".yellow().bold(),
                                        format!("line {}", span.start_line).cyan().bold(),
                                        "column".yellow().bold(),
                                        span.start_col.to_string().cyan().bold(),
                                        format!("in file {}", path_str).yellow().bold()
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {}", "parse error:".red().bold(), e);
                            if let Some(span) = e.span() {
                                eprintln!(
                                    "{} {} {} {} {}",
                                    "at".yellow().bold(),
                                    format!("line {}", span.start_line).cyan().bold(),
                                    "column".yellow().bold(),
                                    span.start_col.to_string().cyan().bold(),
                                    format!("in file {}", path_str).yellow().bold()
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "lex error:".red().bold(), e);
                    if let Some(span) = e.span() {
                        eprintln!(
                            "{} {} {} {} {}",
                            "at".yellow().bold(),
                            format!("line {}", span.start_line).cyan().bold(),
                            "column".yellow().bold(),
                            span.start_col.to_string().cyan().bold(),
                            format!("in file {}", path_str).yellow().bold()
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
            let source = buffer.join("\n");
            match Lexer::tokenize(&source) {
                Ok(tokens) => {
                    let mut parser = Parser::new(&tokens);
                    match parser.parse() {
                        Ok(program) => {
                            if let Err(e) = interpreter.run(&program) {
                                eprintln!("{}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Parse error: {}", e);
                            if let Some(span) = e.span() {
                                eprintln!(
                                    "  at line {}, column {}",
                                    span.start_line, span.start_col
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Lex error: {}", e);
                    if let Some(span) = e.span() {
                        eprintln!("  at line {}, column {}", span.start_line, span.start_col);
                    }
                }
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
        if let Ok(tokens) = Lexer::tokenize(line) {
            for token in tokens {
                match token.kind {
                    TokenKind::If | TokenKind::While | TokenKind::For | TokenKind::Func => {
                        balance += 1
                    }
                    TokenKind::End | TokenKind::Next => balance -= 1,
                    _ => (),
                }
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
