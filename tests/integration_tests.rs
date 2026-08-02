use hi_interpreter::error::{InterpError, InterpResult};
use hi_interpreter::interpreter::Interpreter;
use hi_interpreter::parser::Parser;
use hi_interpreter::parser::lexer::Lexer;
use hi_interpreter::preprocessor::preprocess_file;
use std::io::{Read, Write};
use std::path::Path;
use stdio_override::StdoutOverride;
use tempfile::NamedTempFile;

fn run_code(code: &str) -> InterpResult<()> {
    let tokens = Lexer::tokenize(code)?;
    let mut parser = Parser::new(&tokens);
    let program = parser.parse()?;
    let mut interpreter = Interpreter::new();
    interpreter.run(&program)?;
    Ok(())
}

fn run_and_capture(code: &str) -> Result<(InterpResult<()>, String), Box<dyn std::error::Error>> {
    let temp_file = NamedTempFile::new()?;
    let _guard = StdoutOverride::from_file(temp_file.path())?;

    let result = run_code(code);

    drop(_guard);

    let mut content = String::new();
    temp_file.reopen()?.read_to_string(&mut content)?;

    Ok((result, content))
}

// ---------- Stack operations ----------

// ---------- Arithmetic ----------
#[test]
fn test_arithmetic() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET result1 = 3 + 5
        PRINT "3+5=", result1

        LET result2 = 10 - 4
        PRINT "10-4=", result2

        LET result3 = 7 * 2
        PRINT "7*2=", result3

        LET result4 = 15 / 3
        PRINT "15/3=", result4
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("3+5=8"));
    assert!(output.contains("10-4=6"));
    assert!(output.contains("7*2=14"));
    assert!(output.contains("15/3=5"));
    result?;
    Ok(())
}

// ---------- Comparison ----------

// ---------- Logic ----------

// ---------- IF/ELSE ----------

// ---------- WHILE ----------

// ---------- Functions ----------

// ---------- Inline ----------

// ---------- String methods ----------

// ---------- List ----------

// ---------- New string/list methods ----------

// ---------- I/O ----------

// ---------- IMPORT tests ----------

// ---------- Dicts ---------
