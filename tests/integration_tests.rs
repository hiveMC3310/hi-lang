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

fn run_and_capture(code: &str) -> (InterpResult<()>, String) {
    let temp_file = NamedTempFile::new().unwrap();
    let _guard = StdoutOverride::from_file(temp_file.path()).unwrap();

    let result = run_code(code);

    drop(_guard);

    let mut content = String::new();
    temp_file
        .reopen()
        .unwrap()
        .read_to_string(&mut content)
        .unwrap();

    (result, content)
}

// ---------- Arithmetic ----------
#[test]
fn test_arithmetic() {
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
    let (result, output) = run_and_capture(code);
    assert!(
        result.is_ok(),
        "Арифметика упала: {:?}",
        result.unwrap_err()
    );

    assert!(output.contains("3+5=8"));
    assert!(output.contains("10-4=6"));
    assert!(output.contains("7*2=14"));
    assert!(output.contains("15/3=5"));
}
#[test]
fn test_mod_pow() {
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
    let (result, output) = run_and_capture(code);
    assert!(result.is_ok(), "MOD/POW failed: {:?}", result.unwrap_err());
    assert!(output.contains("10%3=1"));
    assert!(output.contains("2^3=8"));
    assert!(output.contains("2^-1=0.5"));
    assert!(output.contains("5.5%2.0=1.5")); // float modulo
}

// ---------- Comparison ----------
#[test]
fn test_comparisons() {
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

    let (result, output) = run_and_capture(code);
    assert!(result.is_ok(), "Сравнения упали: {:?}", result.unwrap_err());

    assert!(output.contains("3<5=true"));
    assert!(output.contains("10>=10=true"));
    assert!(output.contains("'hello'=='hello'=true"));
    assert!(output.contains("'abc'!='def'=true"));
    assert!(output.contains("5>3=true"));
    assert!(output.contains("3<=3=true"));
}

// ---------- Logic ----------
#[test]
fn test_logic() {
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

    let (result, output) = run_and_capture(code);
    assert!(result.is_ok(), "Логика упала: {:?}", result.unwrap_err());

    assert!(output.contains("1 AND 0=false"));
    assert!(output.contains("True AND False=false"));
    assert!(output.contains("0 OR 1=true"));
    assert!(output.contains("NOT True=false"));
    assert!(output.contains("NOT 0=true"));
}

// ---------- IF/ELSE ----------
#[test]
fn test_if_else() {
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

    let (result, output) = run_and_capture(code);
    assert!(result.is_ok(), "IF/ELSE упал: {:?}", result.unwrap_err());

    assert!(output.contains("x < 10"));
    assert!(output.contains("y >= 15"));
}

// ---------- WHILE ----------
#[test]
fn test_while() {
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

    let (result, output) = run_and_capture(code);
    assert!(result.is_ok(), "WHILE упал: {:?}", result.unwrap_err());

    assert!(output.contains("0"));
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(output.contains("Loop finished"));
}

// ---------- Functions ----------
#[test]
fn test_functions() {
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

    let (result, output) = run_and_capture(code);
    assert!(result.is_ok(), "Функции упали: {:?}", result.unwrap_err());

    assert!(output.contains("Hello from function!"));
    assert!(output.contains("Sum=30"));
}

// ---------- Inline ----------
#[test]
fn test_inline_if() {
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

    let (result, output) = run_and_capture(code);
    assert!(result.is_ok(), "Inline IF упал: {:?}", result.unwrap_err());

    assert!(output.contains("x is 5"));
    assert!(output.contains("y > 5"));
    assert!(output.contains("not both true"));
    assert!(output.contains("at least one true"));
}

#[test]
fn test_inline_while() {
    let code = r#"
        LET i 0
        WHILE LT i 3
            PRINT i
            ADD i 1
            POP i
        DO
        PRINT "Done"
    "#;

    let (result, output) = run_and_capture(code);
    assert!(
        result.is_ok(),
        "Inline WHILE упал: {:?}",
        result.unwrap_err()
    );

    assert!(output.contains("0"));
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(output.contains("Done"));
}

// ---------- String methods ----------
#[test]
fn test_string_methods() {
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
    let (result, output) = run_and_capture(code);
    assert!(
        result.is_ok(),
        "Методы строк упали: {:?}",
        result.unwrap_err()
    );
    assert!(output.contains("len=5"));
    assert!(output.contains("concat=Hello World"));
    assert!(output.contains("substr=world"));
    assert!(output.contains("upper=HELLO"));
    assert!(output.contains("lower=hello"));
    assert!(output.contains("trim=hello"));
}
