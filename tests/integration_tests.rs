use hi_interpreter::error::InterpResult;
use hi_interpreter::interpreter::Interpreter;

fn run_code(code: &str) -> InterpResult<()> {
    let lines: Vec<String> = code.lines().map(|s| s.to_string()).collect();
    let mut interp = Interpreter::new(lines);
    interp.run()
}

// ---------- BREAK ----------
#[test]
fn break_test_time_limit() {
    use std::time::Instant;

    let code_break = r#"
        PUSH 1
        WHILE SP
            PUSH 10
            PRINT SP
            BREAK
        DO
        PRINT "End"
    "#;

    let start = Instant::now();
    let result = run_code(code_break);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Программа упала: {:?}", result.unwrap_err());
    assert!(
        duration.as_secs() < 1,
        "Зависла! Длительность: {:?}",
        duration
    );
}

#[test]
fn break_nested_loops_test() {
    use std::time::Instant;

    let code = r#"
        PUSH 1
        WHILE SP
            PUSH 5
            WHILE SP
                PUSH "Inner"
                PRINT SP
                BREAK
            DO
            PUSH "Outer after break"
            PRINT SP
            BREAK
        DO
    "#;

    let start = Instant::now();
    let result = run_code(code);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Программа упала: {:?}", result.unwrap_err());
    assert!(
        duration.as_secs() < 1,
        "Зависла! Длительность: {:?}",
        duration
    );
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
    let result = run_code(code);
    assert!(
        result.is_ok(),
        "Арифметика упала: {:?}",
        result.unwrap_err()
    );
}

// ---------- Comparison ----------
#[test]
fn test_comparisons() {
    let code = r#"
        // Числа
        LT 3 5
        POP c1
        PRINT "3<5=" c1

        GE 10 10
        POP c2
        PRINT "10>=10=" c2

        // Строки
        EQ "hello" "hello"
        POP c3
        PRINT "'hello'=='hello'=" c3

        NE "abc" "def"
        POP c4
        PRINT "'abc'!='def'=" c4
    "#;
    let result = run_code(code);
    assert!(result.is_ok(), "Сравнения упали: {:?}", result.unwrap_err());
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
    let result = run_code(code);
    assert!(result.is_ok(), "Логика упала: {:?}", result.unwrap_err());
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
    let result = run_code(code);
    assert!(result.is_ok(), "IF/ELSE упал: {:?}", result.unwrap_err());
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
    let result = run_code(code);
    assert!(result.is_ok(), "WHILE упал: {:?}", result.unwrap_err());
}

// ---------- Функции ----------
#[test]
fn test_functions() {
    let code = r#"
        FUNC greet
            PRINT "Hello from function!"
        RET
        ENDF

        FUNC sum
            // принимает два числа со стека, возвращает сумму
            POP a   // копируем верхний (второй аргумент)
            POP b   // копируем ещё раз (первый аргумент)
            ADD a b
            RET
        ENDF

        CALL greet

        PUSH 10
        PUSH 20
        CALL sum
        POP result
        PRINT "Sum=" result
    "#;
    let result = run_code(code);
    assert!(result.is_ok(), "Функции упали: {:?}", result.unwrap_err());
}
