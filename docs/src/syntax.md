# Language Syntax

This chapter describes the complete syntax and structure of the Hi language. Hi is a **free‑form** language – whitespace
and line breaks are generally not significant, except for separating tokens.

---

## General Rules

### Comments

Comments start with `//` and continue to the end of the line:

```hi
// This is a comment
LET x = 5   // This is also a comment
```

### Case Sensitivity

- **Keywords** (`LET`, `IF`, `WHILE`, `FUNC`, `PRINT`, etc.) must be written in **UPPERCASE**.
- **Variable names** are **case‑sensitive** – `myVar` and `myvar` are different.
- **Boolean literals** `TRUE` and `FALSE` are case‑sensitive (must be uppercase).
- **Built‑in function names** are **lowercase** (e.g., `len`, `append`, `sin`).

### Tokens and Separators

Tokens are separated by whitespace, parentheses, brackets, braces, commas, or operators. The following are valid
separators:

- Spaces, tabs, newlines
- `(`, `)`, `[`, `]`, `{`, `}`, `,`
- Operators: `+`, `-`, `*`, `/`, `%`, `^`, `=`, `==`, `!=`, `<`, `<=`, `>`, `>=`

---

## Data Types

Hi is **dynamically typed** – variables can hold values of any type, and types are checked at runtime.

| Type     | Examples                            | Notes                                                            |
|----------|-------------------------------------|------------------------------------------------------------------|
| Integer  | `42`, `-3`, `0`                     | 64‑bit signed integers.                                          |
| Float    | `3.14`, `-2.5`, `0.0`, `.5`, `1.`   | 64‑bit floating point.                                           |
| String   | `"Hello"`, `"Hi\n"`, `"\"Quoted\""` | Double‑quoted with escape sequences.                             |
| Boolean  | `TRUE`, `FALSE`                     | Case‑sensitive; must be uppercase.                               |
| List     | `[1, 2, "three"]`                   | Ordered, mutable sequence of values.                             |
| Dict     | `{"name" = "Alice", "age" = 30}`    | Key‑value map; keys must be hashable (int, float, string, bool). |
| Function | `double`, `factorial`               | Functions are first‑class values (stored in variables).          |
| File     | returned by `open()`                | File handle for I/O operations.                                  |
| Nil      | `nil`                               | Represents “no value” (e.g., return of `put()`).                 |

### String Escapes

Inside double‑quoted strings, the following escape sequences are recognised:

| Sequence   | Meaning                  |
|------------|--------------------------|
| `\n`       | Newline                  |
| `\r`       | Carriage return          |
| `\t`       | Tab                      |
| `\\`       | Backslash                |
| `\"`       | Double quote             |
| `\u{XXXX}` | Unicode code point (hex) |

Example: `"Hello\u{1F600}"` → `"Hello😀"`

---

## Variables

### Declaration

Variables are declared with the `LET` keyword followed by a name, an equals sign, and an initial value:

```hi
LET count = 10
LET name = "Alice"
LET is_active = TRUE
```

Variables are **mutable** – you can assign new values later (see [Assignments](#assignments)).

### Scope

- Variables declared **outside** any function are **global** – accessible from anywhere.
- Variables declared **inside** a function are **local** – they exist only during that function call and shadow globals
  with the same name.

---

## Expressions

Expressions combine literals, variables, function calls, and operators to produce values.

### Literals

| Type    | Example         |
|---------|-----------------|
| Integer | `42`            |
| Float   | `3.14`, `.5`    |
| String  | `"Hello"`       |
| Boolean | `TRUE`, `FALSE` |
| List    | `[1, 2, 3]`     |
| Dict    | `{"a" = 1}`     |

### Identifiers

Variable and function names consist of letters, digits, and underscores, starting with a letter:

```hi
myVar
_counter
value123
```

### Operators

Operators are listed below in **decreasing precedence** (higher binds tighter):

| Precedence | Operators            | Associativity | Description                   |
|------------|----------------------|---------------|-------------------------------|
| 7          | `^`                  | Right         | Exponentiation                |
| 6          | `*`, `/`, `%`        | Left          | Multiplication, division, mod |
| 5          | `+`, `-`             | Left          | Addition, subtraction         |
| 4          | `<`, `<=`, `>`, `>=` | Left          | Comparisons                   |
| 3          | `==`, `!=`           | Left          | Equality / inequality         |
| 2          | `AND`                | Left          | Logical AND                   |
| 1          | `OR`                 | Left          | Logical OR                    |

Unary operators have higher precedence than all binary operators:

| Precedence | Operator | Description    |
|------------|----------|----------------|
| 8          | `-`      | Unary negation |
| 8          | `NOT`    | Logical NOT    |

Parentheses `( )` can override precedence.

### Function Calls

Call a function with its name followed by arguments in parentheses:

```hi
len("hello")        // returns 5
sin(PI / 2)         // returns 1.0
append(list, 42)    // returns new list
```

If a function takes no arguments, you can call it with empty parentheses, e.g., `keys()` (though most built‑ins take
arguments).

### Indexing

Access elements of a list or dict using square brackets:

```hi
mylist[0]          // first element
mydict["name"]     // value for key "name"
```

Indexing can be nested: `matrix[i][j]`.

### List and Dict Literals

- **List**: `[expr, expr, ...]` – empty list is `[]`.
- **Dict**: `{key = value, key = value, ...}` – empty dict is `{}`.

Keys can be any hashable expression (integer, float, string, boolean).

---

## Statements

Statements are the building blocks of a Hi program. Most statements are executed for their side effects.

### Assignment

You can assign a value to an existing variable or to an indexed element:

```hi
x = 10
mylist[2] = 42
mydict["age"] = 30
```

Assigning to a variable that has **not** been declared with `LET` is an error. Use `LET` for the first assignment.

### Print

The `PRINT` statement outputs values to the console. It takes a comma‑separated list of expressions:

```hi
PRINT "Hello"
PRINT "The answer is ", 42
PRINT "x = ", x, ", y = ", y
```

If multiple arguments are given, they are concatenated without separators.

### Input

The `INPUT` statement reads a line from standard input and stores it in a variable. It optionally takes a prompt string:

```hi
INPUT name              // reads a line, stores in 'name'
INPUT "Enter age: " age // displays prompt, then reads
```

The input is parsed as a number if possible (integer or float), otherwise as a boolean (`TRUE`/`FALSE`), otherwise as a
string.

### If Statement

Conditional execution:

```hi
IF condition THEN
    // statements
ELSE
    // statements (optional)
END
```

The `ELSE` branch is optional. The `condition` is any expression; it is evaluated for its truthiness (
see [Boolean Coercion](#boolean-coercion)).

Multiple `IF` statements can be nested.

### While Loop

Repeats a block while a condition is true:

```hi
WHILE condition DO
    // statements
END
```

The condition is checked before each iteration. Use `BREAK` to exit early.

### For Loop

Iterates over a range with an optional step:

```hi
FOR var = start TO end DO
    // statements
NEXT [step]
```

- `start`, `end`, and `step` (if provided) must be integers.
- The loop variable `var` is updated by `step` (default 1) after each iteration.
- If `step` is positive, the loop runs while `var <= end`; if negative, while `var >= end`.
- The step can be given as a literal or expression after the `NEXT` keyword.

Examples:

```hi
FOR i = 0 TO 5 DO
    PRINT i
NEXT                // prints 0,1,2,3,4,5

FOR j = 10 TO 0 DO
    PRINT j
NEXT -2             // prints 10,8,6,4,2,0
```

### Break

Exits the innermost `WHILE` or `FOR` loop immediately:

```hi
WHILE TRUE DO
    IF some_condition THEN
        BREAK
    END
END
```

`BREAK` outside a loop is a runtime error.

### Function Definition

Define a function with `FUNC`, parameters in parentheses, and body ending with `END`:

```hi
FUNC name(param1, param2, ...)
    // statements
    RET expression   // optional
END
```

- Parameters are local variables.
- `RET` returns a value from the function; if omitted, the function returns `nil`.
- Functions can be recursive.

Example:

```hi
FUNC factorial(n)
    IF n <= 1 THEN
        RET 1
    ELSE
        RET n * factorial(n - 1)
    END
END
```

### Return

Inside a function, `RET` exits the function and returns an optional value:

```hi
RET           // returns nil
RET expression // returns value of expression
```

If a function reaches the end without `RET`, it returns `nil`.

### Expression Statement

Any expression followed by a newline (or semicolon – though semicolons are not used) is a statement that evaluates the
expression and discards the result:

```hi
x + 5          // evaluated, result discarded
foo(10)        // call function, result discarded
```

This is useful for function calls that have side effects.

---

## Boolean Coercion

In conditional contexts (`IF`, `WHILE`, `AND`, `OR`, `NOT`), values are converted to booleans as follows:

- `FALSE`, `0`, `0.0`, empty string `""`, empty list `[]`, empty dict `{}`, and `nil` are **false**.
- All other values are **true** (including non‑zero numbers, non‑empty strings/lists/dicts, functions, file handles).

This allows you to use expressions directly in conditions:

```hi
IF name THEN PRINT "Name is not empty" END
IF count THEN PRINT "Count is non‑zero" END
IF list THEN PRINT "List is non‑empty" END
```

---

## Import Directives

The `IMPORT` directive is processed **before** the interpreter runs. It is not a statement, but a preprocessor
instruction.

```hi
IMPORT "path/to/file.hi"
```

- Paths are resolved relative to the current file.
- Only `.hi` extensions are allowed.
- Cyclic imports are detected and prevented.
- Each file is imported only once.

Imported code is inlined at the position of the `IMPORT` directive.

---

## Complete Example

```hi
// A simple program demonstrating syntax

LET greeting = "Hello"
LET name = "World"

FUNC greet(person)
    RET concat(greeting, ", ", person)
END

LET message = greet(name)
PRINT message

LET i = 0
WHILE i < 3 DO
    PRINT i
    i = i + 1
END
```

---

## Summary of Keywords

| Keyword  | Purpose                       |
|----------|-------------------------------|
| `LET`    | Variable declaration          |
| `IF`     | Conditional start             |
| `THEN`   | Start of IF body              |
| `ELSE`   | Alternative branch            |
| `END`    | End of IF, WHILE, FUNC        |
| `WHILE`  | Loop start                    |
| `DO`     | Start of WHILE body           |
| `FOR`    | Numeric loop start            |
| `TO`     | Range separator in FOR        |
| `NEXT`   | End of FOR body (with step)   |
| `FUNC`   | Function definition start     |
| `RET`    | Return from function          |
| `BREAK`  | Exit loop                     |
| `PRINT`  | Output to console             |
| `INPUT`  | Read from console             |
| `TRUE`   | Boolean true                  |
| `FALSE`  | Boolean false                 |
| `AND`    | Logical AND                   |
| `OR`     | Logical OR                    |
| `NOT`    | Logical NOT                   |
| `IMPORT` | Preprocessor import directive |

---

## Next Steps

- Learn about **built‑in functions** in the [Built‑ins](builtins.md) chapter.
- Explore **examples** to see syntax in action: [Examples](examples.md).