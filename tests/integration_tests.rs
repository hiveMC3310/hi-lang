use hi_interpreter::error::InterpResult;
use hi_interpreter::interpreter::Interpreter;
use std::io::Read;
use stdio_override::StdoutOverride;
use tempfile::NamedTempFile;

fn run_code(code: &str) -> InterpResult<()> {
    let lines: Vec<String> = code.lines().map(|s| s.to_string()).collect();
    let mut interp = Interpreter::new(lines);
    interp.run()
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

// ---------- Arithmetic ----------
#[test]
fn test_arithmetic() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        // С аргументами
        ADD 3 5
        POP result1
        PRINT "3+5=" result1

        // Со стека
        PUSH 10
        PUSH 4
        SUB
        POP result2
        PRINT "10-4=" result2

        // Смешанные типы
        PUSH 7
        MUL 2 SP
        POP result3
        PRINT "7*2=" result3

        DIV 15 3
        POP result4
        PRINT "15/3=" result4
    "#;
    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("3+5=8"));
    assert!(output.contains("10-4=6"));
    assert!(output.contains("7*2=14"));
    assert!(output.contains("15/3=5"));

    result?; // проверяем успешность выполнения
    Ok(())
}

#[test]
fn test_mod_pow() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        MOD 10 3
        POP m
        PRINT "10%3=" m

        POW 2 3
        POP p1
        PRINT "2^3=" p1

        POW 2 -1
        POP p2
        PRINT "2^-1=" p2

        MOD 5.5 2.0
        POP m2
        PRINT "5.5%2.0=" m2
    "#;
    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("10%3=1"));
    assert!(output.contains("2^3=8"));
    assert!(output.contains("2^-1=0.5"));
    assert!(output.contains("5.5%2.0=1.5"));

    result?;
    Ok(())
}

// ---------- Comparison ----------
#[test]
fn test_comparisons() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LT 3 5
        POP c1
        PRINT "3<5=" c1

        GE 10 10
        POP c2
        PRINT "10>=10=" c2

        EQ "hello" "hello"
        POP c3
        PRINT "'hello'=='hello'=" c3

        NE "abc" "def"
        POP c4
        PRINT "'abc'!='def'=" c4

        GT 5 3
        POP c5
        PRINT "5>3=" c5

        LE 3 3
        POP c6
        PRINT "3<=3=" c6
    "#;

    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("3<5=true"));
    assert!(output.contains("10>=10=true"));
    assert!(output.contains("'hello'=='hello'=true"));
    assert!(output.contains("'abc'!='def'=true"));
    assert!(output.contains("5>3=true"));
    assert!(output.contains("3<=3=true"));

    result?;
    Ok(())
}

// ---------- Logic ----------
#[test]
fn test_logic() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        // AND с аргументами
        AND 1 0
        POP a1
        PRINT "1 AND 0=" a1

        // AND со стека
        PUSH True
        PUSH False
        AND
        POP a2
        PRINT "True AND False=" a2

        // OR
        OR 0 1
        POP o1
        PRINT "0 OR 1=" o1

        // NOT с аргументом
        NOT True
        POP n1
        PRINT "NOT True=" n1

        // NOT со стека
        PUSH 0
        NOT
        POP n2
        PRINT "NOT 0=" n2
    "#;

    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("1 AND 0=false"));
    assert!(output.contains("True AND False=false"));
    assert!(output.contains("0 OR 1=true"));
    assert!(output.contains("NOT True=false"));
    assert!(output.contains("NOT 0=true"));

    result?;
    Ok(())
}

// ---------- IF/ELSE ----------
#[test]
fn test_if_else() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET x 5
        LT x 10
        POP cond
        IF cond
            PRINT "x < 10"
        ELSE
            PRINT "x >= 10"
        ENDIF

        LET y 20
        GE y 15
        POP cond2
        IF cond2
            PRINT "y >= 15"
        ELSE
            PRINT "y < 15"
        ENDIF
    "#;

    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("x < 10"));
    assert!(output.contains("y >= 15"));

    result?;
    Ok(())
}

// ---------- WHILE ----------
#[test]
fn test_while() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET i 0
        LET running 1
        LT i 3
        POP running
        WHILE running
            PRINT i
            ADD i 1
            POP i
            LT i 3
            POP running
        DO
        PRINT "Loop finished"
    "#;

    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("0"));
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(output.contains("Loop finished"));

    result?;
    Ok(())
}

// ---------- Functions ----------
#[test]
fn test_functions() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        FUNC greet
            PRINT "Hello from function!"
        RET
        ENDF

        FUNC sum
            // принимает два числа со стека, возвращает сумму
            POP a
            POP b
            ADD b a
            RET
        ENDF

        CALL greet

        PUSH 10
        PUSH 20
        CALL sum
        POP result
        PRINT "Sum=" result
    "#;

    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("Hello from function!"));
    assert!(output.contains("Sum=30"));

    result?;
    Ok(())
}

// ---------- Inline ----------
#[test]
fn test_inline_if() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET x 5
        IF EQ x 5
            PRINT "x is 5"
        ELSE
            PRINT "x is not 5"
        ENDIF

        LET y 10
        IF GT y 5
            PRINT "y > 5"
        ENDIF

        LET flag1 True
        LET flag2 False
        IF AND flag1 flag2
            PRINT "both true"
        ELSE
            PRINT "not both true"
        ENDIF

        IF OR flag1 flag2
            PRINT "at least one true"
        ENDIF
    "#;

    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("x is 5"));
    assert!(output.contains("y > 5"));
    assert!(output.contains("not both true"));
    assert!(output.contains("at least one true"));

    result?;
    Ok(())
}

#[test]
fn test_inline_while() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET i 0
        WHILE LT i 3
            PRINT i
            ADD i 1
            POP i
        DO
        PRINT "Done"
    "#;

    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("0"));
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(output.contains("Done"));

    result?;
    Ok(())
}

// ---------- String methods ----------
#[test]
fn test_string_methods() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LEN "hello"
        POP len
        PRINT "len=" len

        CONCAT "Hello" " World"
        POP concat
        PRINT "concat=" concat

        SUBSTR "Hello, world!" 7 5
        POP substr
        PRINT "substr=" substr

        UPPER "hello"
        POP up
        PRINT "upper=" up

        LOWER "HELLO"
        POP low
        PRINT "lower=" low

        TRIM "  hello  "
        POP trim
        PRINT "trim=" trim
    "#;
    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("len=5"));
    assert!(output.contains("concat=Hello World"));
    assert!(output.contains("substr=world"));
    assert!(output.contains("upper=HELLO"));
    assert!(output.contains("lower=hello"));
    assert!(output.contains("trim=hello"));

    result?;
    Ok(())
}

// ---------- List ----------
#[test]
fn test_lists() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LIST 1 2 3 4
        POP mylist
        PRINT "List: " mylist

        LEN mylist
        POP len
        PRINT "Length: " len

        INDEX mylist 2
        POP third
        PRINT "Third element: " third

        APPEND mylist 42
        POP newlist
        PRINT "New list: " newlist

        LEN newlist
        POP newlen
        PRINT "New length: " newlen
    "#;
    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("List: [1, 2, 3, 4]"));
    assert!(output.contains("Length: 4"));
    assert!(output.contains("Third element: 3"));
    assert!(output.contains("New list: [1, 2, 3, 4, 42]"));
    assert!(output.contains("New length: 5"));

    result?;
    Ok(())
}
