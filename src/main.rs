//! Entry point of the Hi interpreter.

mod commands;
mod error;
mod interpreter;
mod repl;
mod tokenizer;
mod utils;
mod value;

use crate::interpreter::Interpreter;
use anyhow::{Context, Result};
use clap::Parser;
use colored::*;

/// Command-line arguments.
#[derive(Parser)]
struct Args {
    /// The .hi file to interpret.
    filename: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.filename.is_none() {
        if let Err(e) = repl::repl_run() {
            eprintln!("{} {}", "REPL error:".red().bold(), e);
            std::process::exit(1);
        }
        return Ok(());
    }

    let filename = args.filename.unwrap();

    if !filename.ends_with(".hi") {
        eprintln!("{}", "Incorrect file extension".red().bold());
        std::process::exit(1);
    }
    let content = std::fs::read_to_string(&filename)
        .with_context(|| format!("Failed to read file '{}'", filename))?;

    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut interpreter = Interpreter::new(lines);

    if let Err(e) = interpreter.run() {
        eprintln!("{} {}", "error:".red().bold(), e);
        if let Some(line) = e.line() {
            eprintln!(
                "{} {} {} {}",
                "at line".yellow().bold(),
                line.to_string().cyan().bold(),
                "in file".yellow().bold(),
                filename.cyan().bold()
            );
        }
        std::process::exit(1);
    }

    Ok(())
}
