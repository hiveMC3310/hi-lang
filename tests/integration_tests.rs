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

// ---------- Stack operations ----------
#[test]
fn test_stack_ops() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        PUSH 10
        PUSH 20
        DUP
        POP dup_val
        PRINT "dup=" dup_val

        PUSH 100
        SWAP
        POP a
        POP b
        PRINT "swap=" a " " b

        PUSH 999
        POP
        PUSH 42
        POP result
        PRINT "pop=" result
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("dup=20"));
    assert!(output.contains("swap=20 100"));
    assert!(output.contains("pop=42"));
    result?;
    Ok(())
}

// ---------- Arithmetic ----------
#[test]
fn test_arithmetic() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        ADD 3 5
        POP result1
        PRINT "3+5=" result1

        PUSH 10
        PUSH 4
        SUB
        POP result2
        PRINT "10-4=" result2

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

    result?;
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
        AND 1 0
        POP a1
        PRINT "1 AND 0=" a1

        PUSH True
        PUSH False
        AND
        POP a2
        PRINT "True AND False=" a2

        OR 0 1
        POP o1
        PRINT "0 OR 1=" o1

        NOT True
        POP n1
        PRINT "NOT True=" n1

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
fn test_list_methods() -> Result<(), Box<dyn std::error::Error>> {
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

// ---------- New string/list methods ----------
#[test]
fn test_new_string_methods() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        STARTS "hello world" "hello"
        POP s1
        PRINT "starts_hello=" s1

        STARTS "hello" "he"
        POP s2
        PRINT "starts_he=" s2

        ENDS "hello world" "world"
        POP e1
        PRINT "ends_world=" e1

        ENDS "hello" "lo"
        POP e2
        PRINT "ends_lo=" e2

        REPLACE "hello world" "world" "Rust"
        POP rpl
        PRINT "replace=" rpl

        REPLACE "abc123abc" "abc" "XYZ"
        POP rpl2
        PRINT "replace2=" rpl2

        SPLIT "one,two,three" ","
        POP spl
        PRINT "split=" spl

        SPLIT "a.b.c" "."
        POP spl2
        PRINT "split2=" spl2

        CONTAINS "hello world" "world"
        POP c1
        PRINT "contains_world=" c1

        CONTAINS "hello" "xyz"
        POP c2
        PRINT "contains_xyz=" c2
    "#;
    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("starts_hello=true"));
    assert!(output.contains("starts_he=true"));
    assert!(output.contains("ends_world=true"));
    assert!(output.contains("ends_lo=true"));
    assert!(output.contains("replace=hello Rust"));
    assert!(output.contains("replace2=XYZ123XYZ"));
    assert!(output.contains("split=[one, two, three]"));
    assert!(output.contains("split2=[a, b, c]"));
    assert!(output.contains("contains_world=true"));
    assert!(output.contains("contains_xyz=false"));

    result?;
    Ok(())
}

#[test]
fn test_new_list_methods() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LIST 1 2 3 4 5
        POP mylist

        INSERT mylist 2 99
        POP newlist
        PRINT "inserted=" newlist

        REMOVE newlist 1
        POP remlist
        PRINT "removed=" remlist

        APPEND remlist 42
        POP applist
        PRINT "appended=" applist

        PRINT "original=" mylist

        SLICE applist 1 3
        POP sliced
        PRINT "slice=" sliced

        REVERSE applist
        POP rev
        PRINT "reverse=" rev

        INDEXOF applist 99
        POP idx1
        PRINT "indexof_99=" idx1

        INDEXOF applist 42
        POP idx2
        PRINT "indexof_42=" idx2

        INDEXOF applist 999
        POP idx3
        PRINT "indexof_999=" idx3

        CONTAINS applist 42
        POP c1
        PRINT "contains_42=" c1

        CONTAINS applist 100
        POP c2
        PRINT "contains_100=" c2
    "#;
    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("inserted=[1, 2, 99, 3, 4, 5]"));
    assert!(output.contains("removed=[1, 99, 3, 4, 5]"));
    assert!(output.contains("appended=[1, 99, 3, 4, 5, 42]"));
    assert!(output.contains("original=[1, 2, 3, 4, 5]"));

    assert!(output.contains("slice=[99, 3, 4]"));

    assert!(output.contains("reverse=[42, 5, 4, 3, 99, 1]"));

    assert!(output.contains("indexof_99=1"));
    assert!(output.contains("indexof_42=5"));
    assert!(output.contains("indexof_999=-1"));

    assert!(output.contains("contains_42=true"));
    assert!(output.contains("contains_100=false"));

    result?;
    Ok(())
}

// ---------- I/O ----------
#[test]
fn test_file_write_read() -> Result<(), Box<dyn std::error::Error>> {
    let temp_file = NamedTempFile::new()?;
    let path = temp_file.path().to_str().unwrap().to_string();
    let code = format!(
        r#"
        OPEN "{}" "w"
        POP f
        WRITE f "Hello, world!"
        CLOSE f

        OPEN "{}" "r"
        POP f
        READ f
        POP content
        CLOSE f
        "#,
        path, path
    );
    let result = run_code(&code);
    result?;

    let content = std::fs::read_to_string(&path)?;
    assert_eq!(content, "Hello, world!");
    Ok(())
}

#[test]
fn test_file_writeln_readln() -> Result<(), Box<dyn std::error::Error>> {
    let temp_file = NamedTempFile::new()?;
    let path = temp_file.path().to_str().unwrap().to_string();
    let code = format!(
        r#"
        OPEN "{}" "w"
        POP f
        WRITELN f "Line 1"
        WRITELN f "Line 2"
        CLOSE f

        OPEN "{}" "r"
        POP f
        READLN f
        POP line1
        READLN f
        POP line2
        CLOSE f
        "#,
        path, path
    );
    let result = run_code(&code);
    result?;

    let content = std::fs::read_to_string(&path)?;
    assert_eq!(content, "Line 1\nLine 2\n");
    Ok(())
}

#[test]
fn test_file_append() -> Result<(), Box<dyn std::error::Error>> {
    let temp_file = NamedTempFile::new()?;
    let path = temp_file.path().to_str().unwrap().to_string();

    std::fs::write(&path, "Initial\n")?;
    let code = format!(
        r#"
        OPEN "{}" "a"
        POP f
        WRITE f "Appended"
        CLOSE f
        "#,
        path
    );
    let result = run_code(&code);
    result?;

    let content = std::fs::read_to_string(&path)?;
    assert_eq!(content, "Initial\nAppended");
    Ok(())
}

#[test]
fn test_file_eof() -> Result<(), Box<dyn std::error::Error>> {
    let temp_file = NamedTempFile::new()?;
    let path = temp_file.path().to_str().unwrap().to_string();

    std::fs::write(&path, "Hello")?;

    let code = format!(
        r#"
        OPEN "{}" "r"
        POP f
        READ f
        POP content
        EOF f
        POP is_eof
        CLOSE f
        "#,
        path
    );
    let (result, output) = run_and_capture(&code)?;
    result?;

    let code_with_print = format!(
        r#"
        OPEN "{}" "r"
        POP f
        READ f
        POP content
        EOF f
        POP is_eof
        PRINT is_eof
        CLOSE f
        "#,
        path
    );
    let (result, output) = run_and_capture(&code_with_print)?;
    result?;
    assert!(output.contains("true"));

    let code_partial = format!(
        r#"
        OPEN "{}" "r"
        POP f
        READLN f
        POP line
        EOF f
        POP is_eof
        PRINT is_eof
        CLOSE f
        "#,
        path
    );
    let (result, output) = run_and_capture(&code_partial)?;
    result?;
    assert!(output.contains("false"));

    Ok(())
}

#[test]
fn test_file_open_error() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        OPEN "nonexistent_file_12345.txt" "r"
    "#;
    let result = run_code(code);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_file_write_read_with_variable() -> Result<(), Box<dyn std::error::Error>> {
    let temp_file = NamedTempFile::new()?;
    let path = temp_file.path().to_str().unwrap().to_string();
    std::fs::write(&path, "Data")?;
    let code = format!(
        r#"
        OPEN "{}" "r"
        POP f
        READ f
        POP content
        PRINT "Content: " content
        CLOSE f
        "#,
        path
    );
    let (result, output) = run_and_capture(&code)?;
    result?;
    assert!(output.contains("Content: Data"));
    Ok(())
}

#[test]
fn test_file_multiple_operations() -> Result<(), Box<dyn std::error::Error>> {
    let temp_file = NamedTempFile::new()?;
    let path = temp_file.path().to_str().unwrap().to_string();
    let code = format!(
        r#"
        OPEN "{}" "w"
        POP f
        WRITELN f "First"
        CLOSE f

        OPEN "{}" "a"
        POP f
        WRITELN f "Second"
        CLOSE f

        OPEN "{}" "r"
        POP f
        READLN f
        POP line1
        READLN f
        POP line2
        CLOSE f
        "#,
        path, path, path
    );
    let result = run_code(&code);
    result?;

    let content = std::fs::read_to_string(&path)?;
    assert_eq!(content, "First\nSecond\n");
    Ok(())
}
