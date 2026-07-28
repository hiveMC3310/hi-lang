//! Read-Eval-Print Loop (REPL) for interactive Hi sessions.

use crate::interpreter::{CallFrame, Interpreter};
use crate::tokenizer::Tokenizer;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

/// Starts the REPL loop.
pub fn repl_run() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = DefaultEditor::new()?;
    let mut interpreter = Interpreter::new(vec![]);

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
            interpreter.stack.clear();
            interpreter.globals.clear();
            interpreter.call_stack.clear();
            interpreter.call_stack.push(CallFrame::new(0));
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
