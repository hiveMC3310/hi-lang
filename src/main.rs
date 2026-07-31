//! Entry point of the Hi interpreter.

mod commands;
mod error;
mod interpreter;
mod preprocessor;
mod repl;
mod tokenizer;
mod utils;
mod value;

use crate::interpreter::Interpreter;
use crate::preprocessor::preprocess_file;
use anyhow::Result;
use clap::Parser;
use colored::*;
use std::path::Path;

/// Command-line arguments.
#[derive(Parser)]
struct Args {
    /// The .hi file to interpret.
    filename: Option<String>,

    /// All remaining arguments are passed to the script.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    arguments: Vec<String>,
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

    let root_path = Path::new(&filename);
    let processed_lines = preprocess_file(root_path)?;
    let mut interpreter = Interpreter::new(processed_lines);
    interpreter.set_argv(args.arguments);

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
