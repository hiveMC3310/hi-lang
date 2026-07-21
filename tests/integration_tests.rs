use hi_interpreter::error::InterpResult;
use hi_interpreter::interpreter::Interpreter;

fn run_code(code: &str) -> InterpResult<()> {
    let lines: Vec<String> = code.lines().map(|s| s.to_string()).collect();
    let mut interp = Interpreter::new(lines);
    interp.run()
}

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

    assert!(
        result.is_ok(),
        "Программа упала с ошибкой: {:?}",
        result.unwrap_err()
    );

    assert!(
        duration.as_secs() < 1,
        "Тест завис! Цикл не прервался. Длительность: {:?}",
        duration
    );

    println!("Тест пройден за {:?}", duration);
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

    assert!(
        result.is_ok(),
        "Программа упала с ошибкой: {:?}",
        result.unwrap_err()
    );

    assert!(
        duration.as_secs() < 1,
        "Тест завис! Длительность: {:?}",
        duration
    );

    println!("Вложенный тест пройден за {:?}", duration);
}
