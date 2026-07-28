# Hi Language — Language Reference

This document describes the syntax and semantics of the **Hi** programming language.

> **Note:** Hi is a stack‑based interpreted language. You can write programs in `.hi` files and run them with the `hi`
> interpreter.

---

## 1. Basic Syntax

### Comments

Comments start with `//` and go to the end of the line.

```hi
// This is a comment
```

### Values

- **Integers**: `42`, `-3`, `0`
- **Floats**: `3.14`, `-2.5`, `0.0`
- **Strings**: `"Hello, world!"` (double quotes, supports `\"` escape)
- **Booleans**: `True`, `False`
- **Lists**: `[1, 2, "three"]` (created via commands, not literal syntax)

### Variables

Variable names consist of letters, digits, and underscores, starting with a letter.  
Variables are **dynamically typed** – they can hold any value.

- **Global** variables are accessible everywhere.
- **Local** variables exist inside functions and are destroyed when the function returns.

---

## 2. The Stack

Hi maintains a **value stack**. Many commands interact with it:

- `PUSH` places a value on top.
- `POP` removes the top value (and optionally stores it).
- Arithmetic, comparison, logical, and string operations can take values from the stack (if no arguments are given) or
  use explicit arguments.
- The special token **`SP`** resolves to the current top‑of‑stack value and can be used anywhere a value is expected,
  e.g., `PRINT SP`.

---

## 3. Commands (Complete List)

### General

| Command | Description                            |
|---------|----------------------------------------|
| `HELLO` | Prints `Hello, World!` to the console. |

### Stack Operations

| Command      | Description                                                                                                                                          |
|--------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| `PUSH value` | Pushes a value onto the stack.                                                                                                                       |
| `POP [var]`  | Pops the top value from the stack. If a variable name is given, the value is stored in that variable (local if inside a function, otherwise global). |

### Variables

| Command              | Description                                                                                                                                                                          |
|----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `LET var value`      | Assigns a value to a variable (local if inside a function, otherwise global).                                                                                                        |
| `INPUT [prompt] var` | Reads a line from standard input. If a prompt is given, it is displayed. The input is parsed as a number if possible, otherwise as a boolean (`True`/`False`) or stored as a string. |

### Arithmetic

All arithmetic commands work in two modes:

- **With arguments**: `ADD 3 5` computes 3+5 and pushes the result.
- **Without arguments**: pops two values from the stack, computes the result, and pushes it.

| Command | Operation                                                                                                 |
|---------|-----------------------------------------------------------------------------------------------------------|
| `ADD`   | Addition                                                                                                  |
| `SUB`   | Subtraction                                                                                               |
| `MUL`   | Multiplication                                                                                            |
| `DIV`   | Division (integer division if both operands are integers, otherwise float). Division by zero is an error. |
| `MOD`   | Modulo (remainder). Works with integers and floats.                                                       |
| `POW`   | Exponentiation. With two integers, result is integer if exponent >= 0, otherwise float.                   |

### Comparisons

All comparison commands also work in two modes (with arguments or from the stack).  
They push `True` or `False` as a boolean value.

| Command | Meaning               |
|---------|-----------------------|
| `EQ`    | Equal to              |
| `NE`    | Not equal to          |
| `GT`    | Greater than          |
| `GE`    | Greater than or equal |
| `LT`    | Less than             |
| `LE`    | Less than or equal    |

Comparisons work for integers, floats, strings, and booleans. Mixed types (e.g., `INT` vs `FLOAT`) are automatically
handled where possible.

### Logical Operators

| Command | Description                                                                                                |
|---------|------------------------------------------------------------------------------------------------------------|
| `AND`   | Logical AND (works with two arguments or from stack). Values are converted to booleans using Hi semantics. |
| `OR`    | Logical OR (same as above).                                                                                |
| `NOT`   | Logical NOT (takes one argument or pops one value from stack).                                             |

### String Operations

| Command  | Description                                                                                                     |
|----------|-----------------------------------------------------------------------------------------------------------------|
| `LEN`    | Returns the length of a string (or a list). Takes one argument or pops one value from stack.                    |
| `CONCAT` | Concatenates two strings. Takes two arguments or pops two values from stack.                                    |
| `SUBSTR` | Extracts a substring. Takes three arguments: string, start index, length. Also works from stack (pop 3 values). |
| `UPPER`  | Converts a string to uppercase. Takes one argument or pops one value from stack.                                |
| `LOWER`  | Converts a string to lowercase. Takes one argument or pops one value from stack.                                |
| `TRIM`   | Removes leading and trailing whitespace from a string. Takes one argument or pops one value from stack.         |

### List Operations

| Command               | Description                                                        |
|-----------------------|--------------------------------------------------------------------|
| `LIST val1 val2 ...`  | Creates a list from the given values and pushes it onto the stack. |
| `INDEX list index`    | Pushes the element at the specified 0‑based index onto the stack.  |
| `APPEND list element` | Creates a new list by adding the element to the end and pushes it. |
| `LEN`                 | Works for lists as well (returns the number of elements).          |

### Control Flow

| Command           | Description                                                                                                                                                                      |
|-------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `IF condition`    | Starts a conditional block. If the condition is **false**, jumps to `ELSE` (if present) or `ENDIF`. <br> You can use **inline** conditions: `IF EQ x 5` or `IF AND flag1 flag2`. |
| `ELSE`            | Marks the start of the alternative block (executed if the preceding `IF` condition was false).                                                                                   |
| `ENDIF`           | Marks the end of an `IF`/`ELSE` block.                                                                                                                                           |
| `WHILE condition` | Starts a loop. If the condition is **false**, jumps to the matching `DO`. <br> Inline conditions are also supported: `WHILE LT i 10`.                                            |
| `DO`              | Marks the end of a `WHILE` block; jumps back to the corresponding `WHILE`.                                                                                                       |
| `BREAK`           | Immediately exits the innermost `WHILE` loop.                                                                                                                                    |

### Functions

| Command     | Description                                               |
|-------------|-----------------------------------------------------------|
| `FUNC name` | Defines a function. The body is executed when `CALL`ed.   |
| `RET`       | Returns from the current function (explicit return).      |
| `ENDF`      | Marks the end of a function definition (implicit return). |
| `CALL name` | Calls a previously defined function.                      |

---

## 4. Inline Conditions for IF and WHILE

Since v1.1.0, you can write conditions directly in `IF` and `WHILE` statements without using the stack:

```hi
LET x 5
IF EQ x 5
    PRINT "x is 5"
ENDIF

LET i 0
WHILE LT i 3
    PRINT i
    ADD i 1
    POP i
DO
```

Supported operators: `EQ`, `NE`, `GT`, `GE`, `LT`, `LE`, `AND`, `OR`.

Classic stack‑based syntax (`POP cond; IF cond`) continues to work.

---

## 5. Working with Lists

### Creating a List

```hi
LIST 1 2 3 "hello" True
POP mylist
```

### Accessing Elements

```hi
INDEX mylist 2
POP third
PRINT third   // prints 3
```

### Appending

```hi
APPEND mylist 42
POP newlist
PRINT newlist   // [1, 2, 3, "hello", True, 42]
```

### Length

```hi
LEN mylist
POP len
PRINT len      // 5
```

Lists are **immutable** – operations like `APPEND` create a new list and leave the original unchanged.

---

## 6. Functions

Functions have **local variables** – they are isolated from the global scope.  
Arguments can be passed via the stack:

```hi
FUNC sum
    // pops two numbers from stack, pushes their sum
    POP a
    POP b
    ADD b a
    RET
ENDF

PUSH 10
PUSH 20
CALL sum
POP result
PRINT result   // 30
```

---

## 7. Full Example Program

```hi
// Compute factorial of 5
FUNC factorial
    // expects n on stack
    POP n
    IF EQ n 0
        PUSH 1
    ELSE
        PUSH n
        SUB n 1
        POP new_n
        PUSH new_n
        CALL factorial
        MUL
    ENDIF
    RET
ENDF

PUSH 5
CALL factorial
POP result
PRINT "5! = " result
```

---

## 8. Notes

- **SP** (Stack Pointer) always refers to the top of the stack.
- **Booleans** are represented as `True` and `False` (case‑sensitive).
- **Strings** are immutable; operations like `UPPER` create new strings.
- **Lists** are immutable; `APPEND` returns a new list.
- The interpreter is **case‑insensitive** for command names, but **case‑sensitive** for variable names and boolean
  literals.

---

Happy coding! 🚀