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

#### Documentation comments

Documentation comments start with `///` (three slashes) and are used to attach descriptive text to the following
function or module definition. They are not part of the program logic, but are stored in the AST and can be used by
tools such as the Language Server Protocol (LSP) to provide hover information and documentation inside editors.

```hi
/// Returns the square of a number.
FUNC square(x)
    RET x * x
END
```

Doc comments can span multiple lines by starting each line with `///`.

### Case Sensitivity

- **Keywords** (`LET`, `IF`, `WHILE`, `FUNC`, `PRINT`, etc.) must be written in **UPPERCASE**.
- **Variable names** are **case‑sensitive** – `myVar` and `myvar` are different.
- **Boolean literals** `TRUE` and `FALSE` are case‑sensitive (must be uppercase).
- **Built‑in function names** are **lowercase** (e.g., `len`, `append`, `sin`).
- **Module names** (like `math`, `strings`, `io`) are **lowercase** and reserved for built‑in modules.

### Tokens and Separators

Tokens are separated by whitespace, parentheses, brackets, braces, commas, or operators. The following are valid
separators:

- Spaces, tabs, newlines
- `(`, `)`, `[`, `]`, `{`, `}`, `,`
- Operators: `+`, `-`, `*`, `/`, `%`, `^`, `=`, `==`, `!=`, `<`, `<=`, `>`, `>=`
- Colon `:` is used for module access.

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
| Module   | `math`, `strings`, `io`             | A namespace containing variables and functions (see Modules).    |
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

| Precedence | Operator       | Description                     |
|------------|----------------|---------------------------------|
| 8          | `-`            | Unary negation                  |
| 8          | `NOT` (or `!`) | Logical NOT (both are synonyms) |

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

### Module Access

Use a colon `:` to access variables and functions inside a module:

```hi
math:PI              // variable
strings:split("a,b", ",")   // function call
```

The module name must be a variable that holds a module value (see [Import Directives](#import-directives)).

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

#### Compound Assignment

Hi also supports compound assignment operators that combine an arithmetic operation with assignment:

| Operator | Meaning           |
|----------|-------------------|
| `+=`     | `lhs = lhs + rhs` |
| `-=`     | `lhs = lhs - rhs` |
| `*=`     | `lhs = lhs * rhs` |
| `/=`     | `lhs = lhs / rhs` |
| `%=`     | `lhs = lhs % rhs` |
| `^=`     | `lhs = lhs ^ rhs` |

These operators work only when the left-hand side is a simple variable or an indexed element (list/dict). They evaluate
the left-hand side once, then apply the operation and assign the result back.

Examples:

```hi
x += 5          // same as x = x + 5
list[0] *= 2    // double the first element
mydict["key"] ^= 2   // square the value
```

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

## Import Directives

The `IMPORT` directive loads a module (either a built‑in module or a user‑defined `.hi` file) and makes its contents
available in the current scope.

```hi
IMPORT "path/to/file.hi"        // inline all contents (global)
IMPORT "path/to/file.hi" AS name // create a module variable 'name'
IMPORT "math"                   // inline built‑in module 'math'
IMPORT "strings" AS s           // create alias 's' for built‑in strings
```

- **With `AS`**: creates a variable (module) with the given name. You can then access its variables and functions using
  `name:var` and `name:func(...)`.
- **Without `AS`**: all functions and variables of the module are **inlined** into the current environment. For built‑in
  modules, this adds their functions to the global function namespace and their constants to global variables. For user
  modules, all exported (`LET` declared) variables and functions become directly accessible.

- **Built‑in modules** (`math`, `strings`, `io`) are always available as global variables with their original names (
  even
  if you import them without `AS`). Importing them without `AS` additionally adds their functions to the global function
  namespace, allowing you to call `sin()` directly instead of `math:sin()`.

- **User modules** are resolved relative to the current file (or the current working directory in the REPL). Only files
  with the `.hi` extension are allowed.

- **Cyclic imports** are detected and prevented at runtime, with a clear error message.

- **Caching**: Each module is loaded and evaluated only once; subsequent imports reuse the cached module instance.

### Examples

```hi
// Inline built‑in math functions
IMPORT "math"
LET x = sin(PI / 2)   // sin and PI are now global

// Use built‑in strings as a module
IMPORT "strings" AS s
LET parts = s:split("a,b,c", ",")

// Import a user module and create a namespace
IMPORT "lib.hi" AS lib
lib:some_function()

// Inline a user module (its variables and functions become global)
IMPORT "helpers.hi"
helper_func()
```

---

## Boolean Coercion

In conditional contexts (`IF`, `WHILE`, `AND`, `OR`, `NOT`), values are converted to booleans as follows:

- `FALSE`, `0`, `0.0`, empty string `""`, empty list `[]`, empty dict `{}`, and `nil` are **false**.
- All other values are **true** (including non‑zero numbers, non‑empty strings/lists/dicts, functions, file handles,
  modules).

This allows you to use expressions directly in conditions:

```hi
IF name THEN PRINT "Name is not empty" END
IF count THEN PRINT "Count is non‑zero" END
IF list THEN PRINT "List is non‑empty" END
```

---

## Complete Example

```hi
// A program demonstrating syntax and modules

LET greeting = "Hello"
LET name = "World"

FUNC greet(person)
    RET concat(greeting, ", ", person)  // concat is global
END

LET message = greet(name)
PRINT message

// Using built‑in module
IMPORT "strings" AS str
LET words = str:split("one,two,three", ",")
PRINT words

// Inline math functions
IMPORT "math"
LET area = PI * 5 ^ 2
PRINT "Area = ", area
```

---

## Summary of Keywords

| Keyword  | Purpose                     |
|----------|-----------------------------|
| `LET`    | Variable declaration        |
| `IF`     | Conditional start           |
| `THEN`   | Start of IF body            |
| `ELSE`   | Alternative branch          |
| `END`    | End of IF, WHILE, FUNC      |
| `WHILE`  | Loop start                  |
| `DO`     | Start of WHILE body         |
| `FOR`    | Numeric loop start          |
| `TO`     | Range separator in FOR      |
| `NEXT`   | End of FOR body (with step) |
| `FUNC`   | Function definition start   |
| `RET`    | Return from function        |
| `BREAK`  | Exit loop                   |
| `PRINT`  | Output to console           |
| `INPUT`  | Read from console           |
| `TRUE`   | Boolean true                |
| `FALSE`  | Boolean false               |
| `AND`    | Logical AND                 |
| `OR`     | Logical OR                  |
| `NOT`    | Logical NOT                 |
| `IMPORT` | Module import directive     |

---

## Next Steps

- Learn about **built‑in functions** in the [Built‑ins](builtins.md) chapter.
- Explore **examples** to see syntax in action: [Examples](examples.md).