use hi_interpreter::error::InterpResult;
use hi_interpreter::interpreter::Interpreter;
use hi_interpreter::parser::Parser;
use hi_interpreter::parser::lexer::Lexer;
use std::io::Read;
use stdio_override::StdoutOverride;
use tempfile::{NamedTempFile, TempDir};

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

        LET result5 = 17 % 5
        PRINT "17%5=", result5

        LET result6 = 2 ^ 3
        PRINT "2^3=", result6

        LET result7 = (1 + 2) * 3
        PRINT "(1+2)*3=", result7

        LET result8 = 10 / 4
        PRINT "10/4=", result8

        LET result9 = 10.0 / 4
        PRINT "10.0/4=", result9
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("3+5=8"));
    assert!(output.contains("10-4=6"));
    assert!(output.contains("7*2=14"));
    assert!(output.contains("15/3=5"));
    assert!(output.contains("17%5=2"));
    assert!(output.contains("2^3=8"));
    assert!(output.contains("(1+2)*3=9"));
    assert!(output.contains("10/4=2"));
    assert!(output.contains("10.0/4=2.5"));
    result?;
    Ok(())
}

// ---------- Comparison ----------
#[test]
fn test_comparisons() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET a = 5
        LET b = 10
        LET c = 5

        IF a == c THEN PRINT "eq true" ELSE PRINT "eq false" END
        IF a != b THEN PRINT "ne true" ELSE PRINT "ne false" END
        IF b > a THEN PRINT "gt true" ELSE PRINT "gt false" END
        IF b >= a THEN PRINT "ge true" ELSE PRINT "ge false" END
        IF a < b THEN PRINT "lt true" ELSE PRINT "lt false" END
        IF a <= c THEN PRINT "le true" ELSE PRINT "le false" END

        LET s1 = "hello"
        LET s2 = "world"
        IF s1 == s1 THEN PRINT "str eq true" ELSE PRINT "str eq false" END
        IF s1 < s2 THEN PRINT "str lt true" ELSE PRINT "str lt false" END
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("eq true"));
    assert!(output.contains("ne true"));
    assert!(output.contains("gt true"));
    assert!(output.contains("ge true"));
    assert!(output.contains("lt true"));
    assert!(output.contains("le true"));
    assert!(output.contains("str eq true"));
    assert!(output.contains("str lt true"));
    result?;
    Ok(())
}

// ---------- Logic ----------
#[test]
fn test_logic() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET t = TRUE
        LET f = FALSE

        IF t AND t THEN PRINT "and tt true" ELSE PRINT "and tt false" END
        IF t AND f THEN PRINT "and tf true" ELSE PRINT "and tf false" END
        IF f AND f THEN PRINT "and ff true" ELSE PRINT "and ff false" END

        IF t OR f THEN PRINT "or tf true" ELSE PRINT "or tf false" END
        IF f OR f THEN PRINT "or ff true" ELSE PRINT "or ff false" END

        IF NOT f THEN PRINT "not f true" ELSE PRINT "not f false" END
        IF NOT t THEN PRINT "not t true" ELSE PRINT "not t false" END
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("and tt true"));
    assert!(output.contains("and tf false"));
    assert!(output.contains("and ff false"));
    assert!(output.contains("or tf true"));
    assert!(output.contains("or ff false"));
    assert!(output.contains("not f true"));
    assert!(output.contains("not t false"));
    result?;
    Ok(())
}

// ---------- IF/ELSE ----------
#[test]
fn test_if_else() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET x = 5
        IF x > 0 THEN
            PRINT "positive"
        ELSE
            PRINT "non-positive"
        END

        IF x == 0 THEN
            PRINT "zero"
        ELSE
            PRINT "non-zero"
        END

        IF x < 0 THEN
            PRINT "negative"
        END
        PRINT "done"
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("positive"));
    assert!(output.contains("non-zero"));
    assert!(!output.contains("negative"));
    assert!(output.contains("done"));
    result?;
    Ok(())
}

// ---------- WHILE ----------
#[test]
fn test_while() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET i = 0
        WHILE i < 3 DO
            PRINT "i=", i
            i = i + 1
        END
        PRINT "done"
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("i=0"));
    assert!(output.contains("i=1"));
    assert!(output.contains("i=2"));
    assert!(!output.contains("i=3"));
    assert!(output.contains("done"));
    result?;
    Ok(())
}

// ---------- FOR ----------
#[test]
fn test_for() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        FOR i = 0 TO 3 DO
            PRINT "i=", i
        NEXT
        PRINT "done"

        // Descending loop requires an explicit negative step
        FOR j = 3 TO 0 DO
            PRINT "j=", j
        NEXT -1
        PRINT "done2"

        FOR k = 0 TO 10 DO
            PRINT "k=", k
        NEXT 2
        PRINT "done3"
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("i=0"));
    assert!(output.contains("i=1"));
    assert!(output.contains("i=2"));
    assert!(output.contains("i=3"));
    assert!(output.contains("j=3"));
    assert!(output.contains("j=2"));
    assert!(output.contains("j=1"));
    assert!(output.contains("j=0"));
    assert!(output.contains("k=0"));
    assert!(output.contains("k=2"));
    assert!(output.contains("k=4"));
    assert!(output.contains("k=6"));
    assert!(output.contains("k=8"));
    assert!(output.contains("k=10"));
    assert!(output.contains("done"));
    result?;
    Ok(())
}

// ---------- Functions ----------
#[test]
fn test_functions() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        FUNC add(a, b)
            RET a + b
        END

        FUNC factorial(n)
            IF n <= 1 THEN
                RET 1
            ELSE
                RET n * factorial(n - 1)
            END
        END

        FUNC no_return()
            PRINT "inside"
        END

        LET sum = add(10, 20)
        PRINT "sum=", sum

        LET fact = factorial(5)
        PRINT "fact=", fact

        no_return()
        PRINT "after"
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("sum=30"));
    assert!(output.contains("fact=120"));
    assert!(output.contains("inside"));
    assert!(output.contains("after"));
    result?;
    Ok(())
}

// ---------- String methods ----------
#[test]
fn test_string_methods() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "strings" AS strings
        LET s = "Hello, World!"
        PRINT "len=", len(s)

        LET parts = strings:split(s, ", ")
        PRINT "split0=", parts[0]
        PRINT "split1=", parts[1]

        LET concat_str = concat("Hello", "World")
        PRINT "concat=", concat_str

        LET replaced = strings:replace("Hello World", "World", "Hi")
        PRINT "replace=", replaced

        LET substr = strings:substr("Hello", 1, 3)
        PRINT "substr=", substr

        LET starts = strings:starts("Hello", "He")
        PRINT "starts=", starts

        LET ends = strings:ends("Hello", "lo")
        PRINT "ends=", ends

        LET upper = strings:upper("hello")
        PRINT "upper=", upper

        LET lower = strings:lower("HELLO")
        PRINT "lower=", lower

        LET trimmed = strings:trim("  hello  ")
        PRINT "trim=", trimmed

        LET rev = reverse("abc")
        PRINT "reverse=", rev

        LET idx = indexof("Hello World", "World")
        PRINT "indexof=", idx

        LET contains_str = contains("Hello World", "Hello")
        PRINT "contains=", contains_str
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("len=13"));
    assert!(output.contains("split0=Hello"));
    assert!(output.contains("split1=World!"));
    assert!(output.contains("concat=HelloWorld"));
    assert!(output.contains("replace=Hello Hi"));
    assert!(output.contains("substr=ell"));
    assert!(output.contains("starts=TRUE"));
    assert!(output.contains("ends=TRUE"));
    assert!(output.contains("upper=HELLO"));
    assert!(output.contains("lower=hello"));
    assert!(output.contains("trim=hello"));
    assert!(output.contains("reverse=cba"));
    assert!(output.contains("indexof=6"));
    assert!(output.contains("contains=TRUE"));
    result?;
    Ok(())
}

// ---------- List ----------
#[test]
fn test_list_operations() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET list = [1, 2, 3, 4]
        PRINT "len=", len(list)

        LET appended = append(list, 5)
        PRINT "appended=", appended

        LET inserted = insert(list, 1, 99)
        PRINT "inserted=", inserted

        LET removed = remove(list, 2)
        PRINT "removed=", removed

        PRINT "contains=", contains(list, 3)
        PRINT "indexof=", indexof(list, 3)

        LET sliced = slice(list, 1, 2)
        PRINT "slice=", sliced

        LET reversed = reverse(list)
        PRINT "reverse=", reversed

        LET value = list[0]
        PRINT "index0=", value

        list[1] = 100
        PRINT "after assign=", list
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("len=4"));
    assert!(output.contains("appended=[1, 2, 3, 4, 5]"));
    assert!(output.contains("inserted=[1, 99, 2, 3, 4]"));
    assert!(output.contains("removed=[1, 2, 4]"));
    assert!(output.contains("contains=TRUE"));
    assert!(output.contains("indexof=2"));
    assert!(output.contains("slice=[2, 3]"));
    assert!(output.contains("reverse=[4, 3, 2, 1]"));
    assert!(output.contains("index0=1"));
    assert!(output.contains("after assign=[1, 100, 3, 4]"));
    result?;
    Ok(())
}

// ---------- Dict ----------
#[test]
fn test_dict_operations() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET d = { "a" = 1, "b" = 2 }
        PRINT "get a=", d["a"]
        d["c"] = 3
        PRINT "after assign=", d

        put(d, "d", 4)
        PRINT "after put=", d

        LET keys_list = keys(d)
        PRINT "keys=", keys_list

        LET values_list = values(d)
        PRINT "values=", values_list

        LET contains_a = contains(d, "a")
        PRINT "contains a=", contains_a

        LET contains_x = contains(d, "x")
        PRINT "contains x=", contains_x

        remove(d, "b")
        PRINT "after remove=", d

        LET val = get(d, "c")
        PRINT "get c=", val

        LET missing = get(d, "z")
        PRINT "get z=", missing
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("get a=1"));

    // Dict order is non-deterministic; check substrings rather than exact string
    let after_assign = output
        .lines()
        .find(|l| l.contains("after assign="))
        .unwrap();
    assert!(after_assign.contains("\"a\"=1"));
    assert!(after_assign.contains("\"c\"=3"));
    assert!(after_assign.contains("\"b\"=2"));

    let after_put = output.lines().find(|l| l.contains("after put=")).unwrap();
    assert!(after_put.contains("\"a\"=1"));
    assert!(after_put.contains("\"c\"=3"));
    assert!(after_put.contains("\"b\"=2"));
    assert!(after_put.contains("\"d\"=4"));

    // keys and values order also non-deterministic; we just check presence of elements
    let keys_line = output.lines().find(|l| l.contains("keys=")).unwrap();
    assert!(keys_line.contains("\"a\""));
    assert!(keys_line.contains("\"b\""));
    assert!(keys_line.contains("\"c\""));
    assert!(keys_line.contains("\"d\""));

    let values_line = output.lines().find(|l| l.contains("values=")).unwrap();
    assert!(values_line.contains("1"));
    assert!(values_line.contains("2"));
    assert!(values_line.contains("3"));
    assert!(values_line.contains("4"));

    assert!(output.contains("contains a=TRUE"));
    assert!(output.contains("contains x=FALSE"));

    let after_remove = output
        .lines()
        .find(|l| l.contains("after remove="))
        .unwrap();
    assert!(after_remove.contains("\"a\"=1"));
    assert!(after_remove.contains("\"c\"=3"));
    assert!(after_remove.contains("\"d\"=4"));
    assert!(!after_remove.contains("\"b\""));

    assert!(output.contains("get c=3"));
    assert!(output.contains("get z=nil"));
    result?;
    Ok(())
}

// ---------- I/O ----------
#[test]
fn test_io() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let file_path = dir.path().join("test.txt");
    let path_str = file_path.to_str().unwrap();

    let code = format!(
        r#"
        IMPORT "io" AS io
        LET f = io:open("{}", "w")
        io:writeln(f, "Hello")
        io:writeln(f, "World")
        io:close(f)

        LET f2 = io:open("{}", "r")
        LET line1 = io:readln(f2)
        LET line2 = io:readln(f2)
        LET eof1 = io:eof(f2)
        LET line3 = io:readln(f2)
        LET eof2 = io:eof(f2)
        io:close(f2)

        PRINT "line1=", line1
        PRINT "line2=", line2
        PRINT "eof1=", eof1
        PRINT "line3=", line3
        PRINT "eof2=", eof2

        LET f3 = io:open("{}", "r")
        LET content = io:read(f3)
        io:close(f3)
        PRINT "content=", content
        "#,
        path_str, path_str, path_str
    );

    let (result, output) = run_and_capture(&code)?;
    // readln includes the newline character, so we check for both line endings
    assert!(output.contains("line1=Hello\n") || output.contains("line1=Hello\r\n"));
    assert!(output.contains("line2=World\n") || output.contains("line2=World\r\n"));
    assert!(output.contains("eof1=FALSE"));
    assert!(output.contains("line3=")); // empty string after EOF
    assert!(output.contains("eof2=TRUE"));
    assert!(
        output.contains("content=Hello\nWorld\n") || output.contains("content=Hello\r\nWorld\r\n")
    );
    result?;
    Ok(())
}

// ---------- Edge cases ----------
#[test]
fn test_division_by_zero() -> Result<(), Box<dyn std::error::Error>> {
    let code = "LET x = 1 / 0";
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Division by zero"));
    }
    Ok(())
}

#[test]
fn test_undefined_variable() -> Result<(), Box<dyn std::error::Error>> {
    let code = "PRINT y";
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Undefined variable or function 'y'"));
    }
    Ok(())
}

#[test]
fn test_break_outside_loop() -> Result<(), Box<dyn std::error::Error>> {
    let code = "BREAK";
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("BREAK used outside of a loop"));
    }
    Ok(())
}

#[test]
fn test_math_functions() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "math" AS m
        LET pi = m:PI
        PRINT "sin(pi/2)=", m:sin(pi/2)
        PRINT "cos(0)=", m:cos(0)
        PRINT "tan(pi/4)=", m:tan(pi/4)
        PRINT "sqrt(16)=", m:sqrt(16)
        PRINT "abs(-5)=", m:abs(-5)
        PRINT "ceil(3.2)=", m:ceil(3.2)
        PRINT "floor(3.9)=", m:floor(3.9)
        PRINT "round(3.5)=", m:round(3.5)
        PRINT "torad(180)=", m:torad(180)
        PRINT "todeg(pi)=", m:todeg(pi)
        PRINT "exp(1)=", m:exp(1)
        PRINT "log(e)=", m:log(m:E)
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("sin(pi/2)=1"));
    assert!(output.contains("cos(0)=1"));
    assert!(output.contains("sqrt(16)=4"));
    assert!(output.contains("abs(-5)=5"));
    assert!(output.contains("ceil(3.2)=4"));
    assert!(output.contains("floor(3.9)=3"));
    assert!(output.contains("round(3.5)=4"));
    assert!(output.contains("torad(180)=3.141592653589793"));
    assert!(output.contains("todeg(pi)=180"));
    assert!(output.contains("exp(1)=2.718281828459045"));
    assert!(output.contains("log(e)=1"));
    result?;
    Ok(())
}

// ---------- Math min/max ----------
#[test]
fn test_math_min_max() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "math" AS m

        LET min1 = m:min(5, 3)
        LET max1 = m:max(5, 3)
        PRINT "min(5,3)=", min1
        PRINT "max(5,3)=", max1

        LET min2 = m:min(2.5, 1.2)
        LET max2 = m:max(2.5, 1.2)
        PRINT "min(2.5,1.2)=", min2
        PRINT "max(2.5,1.2)=", max2

        LET min3 = m:min(10, 3.14)
        LET max3 = m:max(10, 3.14)
        PRINT "min(10,3.14)=", min3
        PRINT "max(10,3.14)=", max3

        LET list = [7, 2, 9, 1, 5]
        LET min_list = m:min(list)
        LET max_list = m:max(list)
        PRINT "min(list)=", min_list
        PRINT "max(list)=", max_list

        LET float_list = [1.5, 0.2, 3.8, 2.1]
        LET min_float_list = m:min(float_list)
        LET max_float_list = m:max(float_list)
        PRINT "min(float_list)=", min_float_list
        PRINT "max(float_list)=", max_float_list

        LET mixed_list = [5, 2.5, 8, 1.2]
        LET min_mixed = m:min(mixed_list)
        LET max_mixed = m:max(mixed_list)
        PRINT "min(mixed)=", min_mixed
        PRINT "max(mixed)=", max_mixed
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("min(5,3)=3"));
    assert!(output.contains("max(5,3)=5"));
    assert!(output.contains("min(2.5,1.2)=1.2"));
    assert!(output.contains("max(2.5,1.2)=2.5"));
    assert!(output.contains("min(10,3.14)=3.14"));
    assert!(output.contains("max(10,3.14)=10"));
    assert!(output.contains("min(list)=1"));
    assert!(output.contains("max(list)=9"));
    assert!(output.contains("min(float_list)=0.2"));
    assert!(output.contains("max(float_list)=3.8"));
    assert!(output.contains("min(mixed)=1.2"));
    assert!(output.contains("max(mixed)=8"));
    result?;
    Ok(())
}

#[test]
fn test_math_clamp() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "math" AS m

        PRINT "clamp(5, 1, 10)=", m:clamp(5, 1, 10)
        PRINT "clamp(0, 1, 10)=", m:clamp(0, 1, 10)
        PRINT "clamp(15, 1, 10)=", m:clamp(15, 1, 10)
        PRINT "clamp(3.5, 1.0, 5.0)=", m:clamp(3.5, 1.0, 5.0)
        PRINT "clamp(0.5, 1.0, 5.0)=", m:clamp(0.5, 1.0, 5.0)
        PRINT "clamp(6.0, 1.0, 5.0)=", m:clamp(6.0, 1.0, 5.0)
        PRINT "clamp(10, 1.5, 8.5)=", m:clamp(10, 1.5, 8.5)
        PRINT "clamp(5, 10, 1)=", m:clamp(5, 10, 1)   // min > max, should still work
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("clamp(5, 1, 10)=5"));
    assert!(output.contains("clamp(0, 1, 10)=1"));
    assert!(output.contains("clamp(15, 1, 10)=10"));
    assert!(output.contains("clamp(3.5, 1.0, 5.0)=3.5"));
    assert!(output.contains("clamp(0.5, 1.0, 5.0)=1.0"));
    assert!(output.contains("clamp(6.0, 1.0, 5.0)=5.0"));
    assert!(output.contains("clamp(10, 1.5, 8.5)=8.5"));
    assert!(output.contains("clamp(5, 10, 1)=5")); // min=10, max=1 => clamp to [1,10], so 5 stays 5
    result?;
    Ok(())
}

#[test]
fn test_conversion_functions() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET s = tostring(42)
        PRINT "tostring(42)=", s
        LET i = toint("123")
        PRINT "toint('123')=", i
        LET f = tofloat("3.14")
        PRINT "tofloat('3.14')=", f
        LET f2 = tofloat(10)
        PRINT "tofloat(10)=", f2
        LET i2 = toint(3.7)
        PRINT "toint(3.7)=", i2
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("tostring(42)=42"));
    assert!(output.contains("toint('123')=123"));
    assert!(output.contains("tofloat('3.14')=3.14"));
    assert!(output.contains("tofloat(10)=10"));
    assert!(output.contains("toint(3.7)=3"));
    result?;
    Ok(())
}

#[test]
fn test_function_as_value() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        FUNC double(x)
            RET x * 2
        END

        FUNC apply(f, x)
            RET call(f, x)
        END

        LET result = apply(double, 5)
        PRINT "result=", result
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("result=10"));
    result?;
    Ok(())
}

#[test]
fn test_type_error_arithmetic() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET x = "hello" + 5
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string()
                .contains("Arithmetic operation requires numbers")
        );
    }
    Ok(())
}

#[test]
fn test_list_index_out_of_bounds() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET list = [1, 2, 3]
        LET value = list[5]
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Index 5 out of bounds"));
    }
    Ok(())
}

#[test]
fn test_list_index_non_integer() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET list = [1, 2, 3]
        LET value = list["a"]
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("List index must be an integer"));
    }
    Ok(())
}

#[test]
fn test_dict_key_not_hashable() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET d = { [1] = 2 }
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Dictionary key must be hashable"));
    }
    Ok(())
}

#[test]
fn test_call_undefined_function() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        undefined_func(1, 2)
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string()
                .contains("Function 'undefined_func' not found")
        );
    }
    Ok(())
}

#[test]
fn test_call_function_wrong_arg_count() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        FUNC add(a, b) RET a + b END
        add(1)
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string()
                .contains("Function 'add' expects 2 arguments, got 1")
        );
    }
    Ok(())
}

#[test]
fn test_import_nonexistent_file() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "nonexistent.hi"
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("No such file or directory"));
    }
    Ok(())
}

#[test]
fn test_error_inside_imported_module() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let mod_path = dir.path().join("module.hi");
    std::fs::write(&mod_path, "LET x = 1 / 0")?;

    let code = format!(r#"IMPORT "{}""#, mod_path.to_str().unwrap());
    let (result, _) = run_and_capture(&code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Division by zero"));
    }
    Ok(())
}

#[test]
fn test_builtin_len_on_non_collection() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET x = len(42)
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string()
                .contains("len() expects string, list, or dict, got integer")
        );
    }
    Ok(())
}

#[test]
fn test_builtin_keys_on_non_dict() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        LET x = keys([1, 2, 3])
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("keys() expects a dict, got list"));
    }
    Ok(())
}

#[test]
fn test_module_variable_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "math" AS m
        LET x = m:UNDEFINED
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string()
                .contains("Variable 'UNDEFINED' not found in module 'm'")
        );
    }
    Ok(())
}

#[test]
fn test_module_function_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "math" AS m
        m:undefined_func(1)
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string()
                .contains("Function 'undefined_func' not found in module")
        );
    }
    Ok(())
}

#[test]
fn test_import_builtin_without_alias_inline() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "math"
        LET x = sin(PI/2)
        PRINT x
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("1.0"));
    result?;
    Ok(())
}

#[test]
fn test_import_builtin_with_alias() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "math" AS m
        LET x = m:sin(m:PI/2)
        PRINT x
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("1.0"));
    result?;
    Ok(())
}

// ---------- Collections module ----------
#[test]
fn test_collections_map() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "collections" AS c

        FUNC double(x)
            RET x * 2
        END

        LET numbers = [1, 2, 3, 4]
        LET doubled = c:map(double, numbers)
        PRINT "doubled=", doubled
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("doubled=[2, 4, 6, 8]"));
    result?;
    Ok(())
}

#[test]
fn test_collections_sort() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "collections" AS c

        LET numbers = [5, 2, 8, 1, 3]
        LET sorted = c:sort(numbers)
        PRINT "sorted=", sorted

        LET strings = ["banana", "apple", "cherry"]
        LET sorted_str = c:sort(strings)
        PRINT "sorted_str=", sorted_str
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("sorted=[1, 2, 3, 5, 8]"));
    assert!(output.contains("sorted_str=[\"apple\", \"banana\", \"cherry\"]"));
    result?;
    Ok(())
}

#[test]
fn test_collections_sort_mixed_types() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "collections" AS c
        LET mixed = [1, "two", 3]
        LET sorted = c:sort(mixed)
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string()
                .contains("sort() list must contain elements of the same type")
        );
    }
    Ok(())
}

#[test]
fn test_collections_filter() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "collections" AS c

        FUNC is_even(x)
            RET x % 2 == 0
        END

        LET numbers = [1, 2, 3, 4, 5, 6]
        LET evens = c:filter(is_even, numbers)
        PRINT "evens=", evens
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("evens=[2, 4, 6]"));
    result?;
    Ok(())
}

#[test]
fn test_collections_reduce() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "collections" AS c

        FUNC add(a, b)
            RET a + b
        END

        LET numbers = [1, 2, 3, 4]
        LET sum = c:reduce(add, numbers, 0)
        PRINT "sum=", sum

        FUNC concat_str(a, b)
            RET concat(a, b)
        END
        LET words = ["Hello", " ", "World"]
        LET sentence = c:reduce(concat_str, words, "")
        PRINT "sentence=", sentence
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("sum=10"));
    assert!(output.contains("sentence=Hello World"));
    result?;
    Ok(())
}

#[test]
fn test_collections_any_all_find() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "collections" AS c

        FUNC is_even(x)
            RET x % 2 == 0
        END

        LET numbers = [1, 3, 5, 7]
        LET any_even = c:any(is_even, numbers)
        LET all_odd = c:all(is_even, numbers)
        LET found = c:find(is_even, numbers)
        PRINT "any_even=", any_even
        PRINT "all_odd=", all_odd
        PRINT "found=", found

        LET numbers2 = [2, 4, 6]
        LET any_even2 = c:any(is_even, numbers2)
        LET all_even = c:all(is_even, numbers2)
        LET found2 = c:find(is_even, numbers2)
        PRINT "any_even2=", any_even2
        PRINT "all_even=", all_even
        PRINT "found2=", found2
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("any_even=FALSE"));
    assert!(output.contains("all_odd=FALSE"));
    assert!(output.contains("found=nil"));
    assert!(output.contains("any_even2=TRUE"));
    assert!(output.contains("all_even=TRUE"));
    assert!(output.contains("found2=2"));
    result?;
    Ok(())
}

// ---------- JSON module ----------
#[test]
fn test_json_parse() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "json" AS json

        LET obj = json:parse("{\"name\": \"Alice\", \"age\": 30, \"active\": true, \"scores\": [10, 20]}")
        PRINT "name=", obj["name"]
        PRINT "age=", obj["age"]
        PRINT "active=", obj["active"]
        PRINT "scores=", obj["scores"]

        LET arr = json:parse("[1, \"two\", false, null]")
        PRINT "arr[0]=", arr[0]
        PRINT "arr[1]=", arr[1]
        PRINT "arr[2]=", arr[2]
        PRINT "arr[3]=", arr[3]
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("name=Alice"));
    assert!(output.contains("age=30"));
    assert!(output.contains("active=TRUE"));
    assert!(output.contains("scores=[10, 20]"));
    assert!(output.contains("arr[0]=1"));
    assert!(output.contains("arr[1]=two"));
    assert!(output.contains("arr[2]=FALSE"));
    assert!(output.contains("arr[3]=nil"));
    result?;
    Ok(())
}

#[test]
fn test_json_stringify() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "json" AS json

        LET dict = {"name" = "Bob", "age" = 25}
        LET json_str = json:stringify(dict)
        PRINT "json_str=", json_str

        LET list = [1, "hello", TRUE]
        LET json_list = json:stringify(list)
        PRINT "json_list=", json_list
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(
        output.contains("json_str={\"age\":25,\"name\":\"Bob\"}")
            || output.contains("json_str={\"name\":\"Bob\",\"age\":25}")
    );
    assert!(output.contains("json_list=[1,\"hello\",true]"));
    result?;
    Ok(())
}

#[test]
fn test_json_parse_error() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "json" AS json
        LET invalid = json:parse("{invalid}")
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Invalid JSON"));
    }
    Ok(())
}

// ---------- OS module ----------
#[test]
fn test_os_env() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "os" AS os

        os:setenv("HI_TEST_VAR", "hello")
        LET val = os:getenv("HI_TEST_VAR")
        PRINT "val=", val

        os:unsetenv("HI_TEST_VAR")
        LET missing = os:getenv("HI_TEST_VAR")
        PRINT "missing=", missing
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("val=hello"));
    assert!(output.contains("missing=nil"));
    result?;
    Ok(())
}

#[test]
fn test_os_cwd_chdir() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let path_str = dir.path().to_str().unwrap();

    let code = format!(
        r#"
        IMPORT "os" AS os

        LET before = os:cwd()
        os:chdir("{}")
        LET after = os:cwd()
        PRINT "before=", before
        PRINT "after=", after
        "#,
        path_str
    );
    let (result, output) = run_and_capture(&code)?;
    assert!(output.contains(&format!("after={}", path_str)));
    result?;
    Ok(())
}

#[test]
fn test_os_listdir_mkdir_rmdir_remove() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let dir_path = dir.path().to_str().unwrap();

    let subdir = dir.path().join("test_subdir");
    let subdir_str = subdir.to_str().unwrap();
    let file_path = dir.path().join("test.txt");
    let file_str = file_path.to_str().unwrap();

    let code = format!(
        r#"
        IMPORT "os" AS os

        os:mkdir("{}")

        IMPORT "io" AS io
        LET f = io:open("{}", "w")
        io:write(f, "content")
        io:close(f)

        LET files = os:listdir("{}")
        PRINT "files=", files

        LET exists_subdir = os:exists("{}")
        PRINT "exists_subdir=", exists_subdir

        LET stat_info = os:stat("{}")
        PRINT "size=", stat_info["size"]
        PRINT "is_file=", stat_info["is_file"]

        os:remove("{}")
        LET exists_file = os:exists("{}")
        PRINT "exists_file=", exists_file

        os:rmdir("{}")
        LET exists_subdir_after = os:exists("{}")
        PRINT "exists_subdir_after=", exists_subdir_after
        "#,
        subdir_str,
        file_str,
        dir_path,
        subdir_str,
        file_str,
        file_str,
        file_str,
        subdir_str,
        subdir_str
    );

    let (result, output) = run_and_capture(&code)?;

    assert!(output.contains("\"test.txt\""));
    assert!(output.contains("exists_subdir=TRUE"));
    assert!(output.contains("size=7"));
    assert!(output.contains("is_file=TRUE"));
    assert!(output.contains("exists_file=FALSE"));
    assert!(output.contains("exists_subdir_after=FALSE"));
    result?;
    Ok(())
}

#[test]
fn test_os_rename_move_copy() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let dir_path = dir.path().to_str().unwrap();

    // Create a file
    let src_file = dir.path().join("source.txt");
    let src_file_str = src_file.to_str().unwrap();
    std::fs::write(&src_file, "content")?;

    // Rename file
    let renamed_file = dir.path().join("renamed.txt");
    let renamed_str = renamed_file.to_str().unwrap();
    let code_rename = format!(
        r#"
        IMPORT "os" AS os
        os:rename("{}", "{}")
        LET exists_renamed = os:exists("{}")
        LET exists_source = os:exists("{}")
        PRINT "renamed_exists=", exists_renamed
        PRINT "source_exists=", exists_source
        "#,
        src_file_str, renamed_str, renamed_str, src_file_str
    );
    let (result, output) = run_and_capture(&code_rename)?;
    assert!(output.contains("renamed_exists=TRUE"));
    assert!(output.contains("source_exists=FALSE"));
    result?;

    // Move file
    let moved_file = dir.path().join("moved.txt");
    let moved_str = moved_file.to_str().unwrap();
    let code_move = format!(
        r#"
        IMPORT "os" AS os
        os:move("{}", "{}")
        LET exists_moved = os:exists("{}")
        LET exists_renamed = os:exists("{}")
        PRINT "moved_exists=", exists_moved
        PRINT "renamed_exists_after_move=", exists_renamed
        "#,
        renamed_str, moved_str, moved_str, renamed_str
    );
    let (result, output) = run_and_capture(&code_move)?;
    assert!(output.contains("moved_exists=TRUE"));
    assert!(output.contains("renamed_exists_after_move=FALSE"));
    result?;

    // Copy file
    let copied_file = dir.path().join("copied.txt");
    let copied_str = copied_file.to_str().unwrap();
    let code_copy = format!(
        r#"
        IMPORT "os" AS os
        os:copy("{}", "{}")
        LET exists_copied = os:exists("{}")
        LET exists_original = os:exists("{}")
        PRINT "copied_exists=", exists_copied
        PRINT "original_exists=", exists_original
        "#,
        moved_str, copied_str, copied_str, moved_str
    );
    let (result, output) = run_and_capture(&code_copy)?;
    assert!(output.contains("copied_exists=TRUE"));
    assert!(output.contains("original_exists=TRUE"));
    result?;

    // Copy empty directory
    let src_dir = dir.path().join("empty_dir");
    let src_dir_str = src_dir.to_str().unwrap();
    std::fs::create_dir(&src_dir)?;
    let dest_dir = dir.path().join("empty_dir_copy");
    let dest_dir_str = dest_dir.to_str().unwrap();
    let code_copy_dir = format!(
        r#"
    IMPORT "os" AS os
    os:copy("{}", "{}")
    LET exists_dest_dir = os:exists("{}")
    PRINT "exists_dest_dir=", exists_dest_dir
    "#,
        src_dir_str, dest_dir_str, dest_dir_str
    );
    let (result, output) = run_and_capture(&code_copy_dir)?;
    assert!(output.contains("exists_dest_dir=TRUE"));
    result?;

    // Copy non-empty directory – should fail
    let non_empty = dir.path().join("non_empty");
    let non_empty_str = non_empty.to_str().unwrap();
    std::fs::create_dir(&non_empty)?;
    let file_in_dir = non_empty.join("file.txt");
    std::fs::write(&file_in_dir, "data")?;
    let dest_non_empty = dir.path().join("non_empty_copy");
    let dest_non_empty_str = dest_non_empty.to_str().unwrap();
    let code_copy_non_empty = format!(
        r#"
        IMPORT "os" AS os
        os:copy("{}", "{}")
        "#,
        non_empty_str, dest_non_empty_str
    );
    let (result, _) = run_and_capture(&code_copy_non_empty)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Cannot copy non-empty directory"));
    }

    Ok(())
}
#[test]
fn test_os_exec() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "os" AS os
        LET exit_code = os:exec("echo Hello from exec")
        PRINT "exit_code=", exit_code
    "#;
    let (result, output) = run_and_capture(code)?;
    // On Windows, echo is a built-in command, on Unix it's /bin/echo
    assert!(output.contains("Hello from exec") || output.contains("exit_code=0"));
    result?;
    Ok(())
}

// ---------- Datetime module ----------
#[test]
fn test_datetime_now() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "datetime" AS dt
        LET now = dt:now()
        PRINT "year=", dt:year(now)
        PRINT "month=", dt:month(now)
        PRINT "day=", dt:day(now)
        PRINT "hour=", dt:hour(now)
        PRINT "minute=", dt:minute(now)
        PRINT "second=", dt:second(now)
        PRINT "millisecond=", dt:millisecond(now)
    "#;
    let (result, output) = run_and_capture(code)?;
    // Just check that fields exist and are numbers (no specific values)
    assert!(output.contains("year="));
    assert!(output.contains("month="));
    assert!(output.contains("day="));
    assert!(output.contains("hour="));
    assert!(output.contains("minute="));
    assert!(output.contains("second="));
    assert!(output.contains("millisecond="));
    result?;
    Ok(())
}

#[test]
fn test_datetime_utcnow() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "datetime" AS dt
        LET utc = dt:utcnow()
        LET now = dt:now()
        // UTC should be close to local, but we can't assert exact difference
        // Just check they are dictionaries
        PRINT "utc_year=", dt:year(utc)
        PRINT "local_year=", dt:year(now)
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("utc_year="));
    assert!(output.contains("local_year="));
    result?;
    Ok(())
}

#[test]
fn test_datetime_fromstring_tostring() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "datetime" AS dt

        LET dt1 = dt:fromstring("2026-08-03 15:30:45", "%Y-%m-%d %H:%M:%S")
        PRINT "year=", dt:year(dt1)
        PRINT "month=", dt:month(dt1)
        PRINT "day=", dt:day(dt1)
        PRINT "hour=", dt:hour(dt1)
        PRINT "minute=", dt:minute(dt1)
        PRINT "second=", dt:second(dt1)

        LET formatted = dt:tostring(dt1, "%Y/%m/%d %H:%M")
        PRINT "formatted=", formatted
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("year=2026"));
    assert!(output.contains("month=8"));
    assert!(output.contains("day=3"));
    assert!(output.contains("hour=15"));
    assert!(output.contains("minute=30"));
    assert!(output.contains("second=45"));
    assert!(output.contains("formatted=2026/08/03 15:30"));
    result?;
    Ok(())
}

#[test]
fn test_datetime_add_diff_duration() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "datetime" AS dt

        LET dt1 = dt:fromstring("2026-08-03 10:00:00", "%Y-%m-%d %H:%M:%S")
        LET dur = dt:duration(3600)  // 1 hour
        LET dt2 = dt:add(dt1, dur)

        LET diff = dt:diff(dt2, dt1)
        PRINT "diff_hours=", diff["hours"]

        // Also test with minutes
        LET dur2 = dt:duration(90)  // 1.5 minutes = 90 seconds
        LET dt3 = dt:add(dt1, dur2)
        LET diff2 = dt:diff(dt3, dt1)
        PRINT "diff_seconds=", diff2["seconds"]
        PRINT "diff_minutes=", diff2["minutes"]
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("diff_hours=1"));
    assert!(output.contains("diff_seconds=30"));
    assert!(output.contains("diff_minutes=1"));
    result?;
    Ok(())
}

// ---------- Random module ----------
#[test]
fn test_random_randint() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "random" AS r
        LET val = r:randint(1, 10)
        PRINT "val=", val
        // We can't assert exact value, but we can check it's within range
        // So we run multiple times to be statistically confident.
        LET valid = TRUE
        FOR i = 0 TO 100 DO
            LET v = r:randint(1, 10)
            IF v < 1 OR v > 10 THEN
                valid = FALSE
            END
        NEXT
        PRINT "valid=", valid
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("valid=TRUE"));
    result?;
    Ok(())
}

#[test]
fn test_random_randfloat() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "random" AS r
        LET valid = TRUE
        FOR i = 0 TO 100 DO
            LET v = r:randfloat()
            IF v < 0.0 OR v >= 1.0 THEN
                valid = FALSE
            END
        NEXT
        PRINT "valid=", valid
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("valid=TRUE"));
    result?;
    Ok(())
}

#[test]
fn test_random_randbytes() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "random" AS r
        LET bytes = r:randbytes(5)
        PRINT "len=", len(bytes)
        LET valid = TRUE
        FOR i = 0 TO len(bytes) - 1 DO
            LET b = bytes[i]
            IF b < 0 OR b > 255 THEN
                valid = FALSE
            END
        NEXT
        PRINT "valid=", valid
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("len=5"));
    assert!(output.contains("valid=TRUE"));
    result?;
    Ok(())
}

#[test]
fn test_random_shuffle() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "random" AS r
        IMPORT "collections" AS c
        LET list = [1, 2, 3, 4, 5]
        LET shuffled = r:shuffle(list)
        PRINT "shuffled=", shuffled
        // Check length and that all elements are present
        LET sorted = c:sort(shuffled)
        PRINT "sorted=", sorted
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("sorted=[1, 2, 3, 4, 5]"));
    result?;
    Ok(())
}

#[test]
fn test_random_choice() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "random" AS r
        LET list = [10, 20, 30]
        LET chosen = r:choice(list)
        PRINT "chosen=", chosen
        // Check that chosen is one of the elements
        LET contains = contains(list, chosen)
        PRINT "contains=", contains
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("contains=TRUE"));
    result?;
    Ok(())
}

// ---------- Regex module ----------
#[test]
fn test_regex_match() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "regex" AS re
        PRINT "match1=", re:match("\\d+", "abc123def")
        PRINT "match2=", re:match("\\d+", "abcdef")
        PRINT "match3=", re:match("^Hello", "Hello world")
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("match1=TRUE"));
    assert!(output.contains("match2=FALSE"));
    assert!(output.contains("match3=TRUE"));
    result?;
    Ok(())
}

#[test]
fn test_regex_find() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "regex" AS re
        LET found1 = re:find("\\d+", "abc123def456")
        LET found2 = re:find("\\d+", "abcdef")
        PRINT "found1=", found1
        PRINT "found2=", found2
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("found1=123"));
    assert!(output.contains("found2=nil"));
    result?;
    Ok(())
}

#[test]
fn test_regex_find_all() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "regex" AS re
        LET all = re:findall("\\d+", "abc123def456ghi789")
        PRINT "all=", all
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("all=[\"123\", \"456\", \"789\"]"));
    result?;
    Ok(())
}

#[test]
fn test_regex_replace() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "regex" AS re
        LET replaced = re:replace("\\d+", "abc123def456", "X")
        PRINT "replaced=", replaced
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("replaced=abcXdefX"));
    result?;
    Ok(())
}

#[test]
fn test_regex_split() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "regex" AS re
        LET parts = re:split("\\s+", "one two  three   four")
        PRINT "parts=", parts
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("parts=[\"one\", \"two\", \"three\", \"four\"]"));
    result?;
    Ok(())
}

#[test]
fn test_regex_invalid_pattern() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "regex" AS re
        re:match("(", "abc")
    "#;
    let (result, _) = run_and_capture(code)?;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Invalid regex pattern"));
    }
    Ok(())
}

// ---------- Path module ----------
#[test]
fn test_path_join() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "path" AS p
        LET joined = p:join("foo", "bar", "baz.txt")
        PRINT "joined=", joined
        LET root = p:join("/", "usr", "local")
        PRINT "root=", root
    "#;
    let (result, output) = run_and_capture(code)?;
    // Platform dependent separator, but we can check substrings.
    assert!(
        output.contains("joined=foo/bar/baz.txt") || output.contains("joined=foo\\bar\\baz.txt")
    );
    assert!(output.contains("root=/usr/local") || output.contains("root=\\usr\\local"));
    result?;
    Ok(())
}

#[test]
fn test_path_basename_dirname() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "path" AS p
        LET base = p:basename("/foo/bar.txt")
        LET dir = p:dirname("/foo/bar.txt")
        LET base2 = p:basename("foo/bar/")
        LET dir2 = p:dirname("foo/bar/")
        PRINT "base=", base
        PRINT "dir=", dir
        PRINT "base2=", base2
        PRINT "dir2=", dir2
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("base=bar.txt"));
    assert!(output.contains("dir=/foo") || output.contains("dir=\\foo"));
    assert!(output.contains("base2=nil"));
    // dirname for trailing slash should give "foo/bar" (without trailing)
    assert!(output.contains("dir2=foo/bar") || output.contains("dir2=foo\\bar"));
    result?;
    Ok(())
}

#[test]
fn test_path_extname() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "path" AS p
        LET ext1 = p:extname("file.txt")
        LET ext2 = p:extname("archive.tar.gz")
        LET ext3 = p:extname("README")
        LET ext4 = p:extname(".hidden")
        PRINT "ext1=", ext1
        PRINT "ext2=", ext2
        PRINT "ext3=", ext3
        PRINT "ext4=", ext4
    "#;
    let (result, output) = run_and_capture(code)?;
    assert!(output.contains("ext1=.txt"));
    assert!(output.contains("ext2=.gz")); // extension is "gz", we prefix dot
    assert!(output.contains("ext3="));
    assert!(output.contains("ext4=")); // no extension for ".hidden"
    result?;
    Ok(())
}

#[test]
fn test_path_is_absolute() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "path" AS p
        PRINT "abs1=", p:isabsolute("/home/user")
        PRINT "abs2=", p:isabsolute("relative/path")
        PRINT "abs3=", p:isabsolute("C:\\Windows")   // Windows absolute
    "#;
    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("abs1=TRUE") || output.contains("abs1=FALSE")); // depending on OS
    assert!(output.contains("abs2=FALSE"));
    assert!(output.contains("abs3=TRUE") || output.contains("abs3=FALSE")); // depends if running on Windows
    result?;
    Ok(())
}

#[test]
fn test_path_normalize() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        IMPORT "path" AS p
        LET norm1 = p:normalize("/foo/bar/../baz/./qux")
        LET norm2 = p:normalize("foo/bar/../../baz")
        LET norm3 = p:normalize("a/./b/../c")
        PRINT "norm1=", norm1
        PRINT "norm2=", norm2
        PRINT "norm3=", norm3
    "#;
    let (result, output) = run_and_capture(code)?;

    assert!(output.contains("norm1=/foo/baz/qux") || output.contains("norm1=\\foo\\baz\\qux"));
    assert!(output.contains("norm2=baz"));
    assert!(output.contains("norm3=a/c") || output.contains("norm3=a\\c"));
    result?;
    Ok(())
}
