//! Entry point of the Hi interpreter.

mod ast;
mod builtins;
mod error;
mod interpreter;
mod modules;
mod parser;
mod repl;
mod utils;
mod value;

use crate::error::InterpError;
use crate::interpreter::Interpreter;
use crate::parser::lexer::Lexer;
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

    let source = match std::fs::read_to_string(root_path) {
        Ok(s) => s,
        Err(e) => {
            report_error(&filename, &e.into());
            std::process::exit(1);
        }
    };

    let tokens = match Lexer::tokenize(&source) {
        Ok(t) => t,
        Err(e) => {
            report_error(&filename, &e.into());
            std::process::exit(1);
        }
    };

    let mut parser = parser::Parser::new(&tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            report_error(&filename, &e.into());
            std::process::exit(1);
        }
    };

    let mut interpreter = Interpreter::new();
    interpreter.set_argv(args.arguments);
    interpreter.current_file = Some(root_path.to_path_buf());

    if let Err(e) = interpreter.run(&program) {
        report_error(&filename, &e);
        std::process::exit(1);
    }

    Ok(())
}

fn report_error(filename: &str, err: &InterpError) {
    eprintln!("{}", format!("error: {}", err).red().bold());
    if let Some(span) = err.span() {
        eprintln!(
            "{} {} {} {} {}",
            "at".yellow().bold(),
            format!("line {}", span.start_line).cyan().bold(),
            "column".yellow().bold(),
            span.start_col.to_string().cyan().bold(),
            format!("in file {}", filename).yellow().bold()
        );
    }
}
