//! Entry point of the Hi interpreter.

mod ast;
mod error;
mod interpreter;
mod parser;
mod preprocessor;
mod repl;
mod utils;
mod value;

use crate::interpreter::Interpreter;
use crate::parser::lexer::Lexer;
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
    let source = processed_lines.join("\n");
    let tokens = Lexer::tokenize(&source)?;
    let mut parser = parser::Parser::new(&tokens);
    let program = parser.parse()?;
    let mut interpreter = Interpreter::new();
    // interpreter.set_argv(args.arguments);

    if let Err(e) = interpreter.run(&program) {
        eprintln!("{} {}", "error:".red().bold(), e);
        if let Some(span) = e.span() {
            eprintln!(
                "{} {} {} {} {}",
                "at".yellow().bold(),
                format!("line {}", span.start_line).cyan().bold(),
                "column".yellow().bold(),
                span.start_col.to_string().cyan().bold(),
                format!("in file {}", filename).yellow().bold()
            );
        }
        std::process::exit(1);
    }

    Ok(())
}
