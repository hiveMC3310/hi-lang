use hi_interpreter::error::InterpResult;
use hi_interpreter::interpreter::Interpreter;
use hi_interpreter::parser::Parser;
use hi_interpreter::parser::lexer::Lexer;
use std::fs::File;
use std::io::{Read, Write};
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
        PRINT "rand(1,10)=", m:rand(1,10)
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
    // rand should be between 1 and 10
    let rand_line = output.lines().find(|l| l.contains("rand(1,10)=")).unwrap();
    let num = rand_line.split('=').last().unwrap().trim().parse::<i64>()?;
    assert!(num >= 1 && num <= 10);
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
