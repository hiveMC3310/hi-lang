# Hi Language — Language Reference

> **⚠️ This document describes the **development version** of Hi (v2.0.0-dev).**  
> The syntax and semantics are not final and may change before the stable release.  
> For the current stable version, refer to the [v1.x documentation](https://github.com/hiveMC3310/hi-lang/tree/v1.x).

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
- **Strings**: `"Hello, world!"` (double quotes).  
  Supports escape sequences:
    - `\n` – newline
    - `\r` – carriage return
    - `\t` – tab
    - `\\` – backslash
    - `\"` – double quote
    - `\u{XXXX}` – Unicode code point (hex digits, e.g., `\u{1F600}` for 😀)

  Invalid escapes or unclosed strings cause a syntax error.
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
| `DUP`        | Duplicates the top value on the stack (pushes a copy).                                                                                               |
| `SWAP`       | Swaps the top two values on the stack.                                                                                                               |

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

### String Operations (v1.4.0+)

| Command    | Description                                                                                                         |
|------------|---------------------------------------------------------------------------------------------------------------------|
| `LEN`      | Returns the length of a string (or a list). Takes one argument or pops one value from stack.                        |
| `CONCAT`   | Concatenates two strings. Takes two arguments or pops two values from stack.                                        |
| `SUBSTR`   | Extracts a substring. Takes three arguments: string, start index, length. Also works from stack (pop 3 values).     |
| `UPPER`    | Converts a string to uppercase. Takes one argument or pops one value from stack.                                    |
| `LOWER`    | Converts a string to lowercase. Takes one argument or pops one value from stack.                                    |
| `TRIM`     | Removes leading and trailing whitespace from a string. Takes one argument or pops one value from stack.             |
| `STARTS`   | Checks if a string starts with a given prefix. Takes two arguments (base, prefix). Returns boolean.                 |
| `ENDS`     | Checks if a string ends with a given suffix. Takes two arguments (base, suffix). Returns boolean.                   |
| `REPLACE`  | Replaces all occurrences of a substring with another. Takes three arguments (base, old, new). Returns a new string. |
| `SPLIT`    | Splits a string by a delimiter. Takes two arguments (base, delimiter). Returns a list of substrings.                |
| `CONTAINS` | Checks if a string contains a substring. Takes two arguments (base, element). Works for lists too. Returns boolean. |

### List Operations (v1.4.0+)

Lists are now **mutable** with **copy‑on‑write** semantics. Operations like `APPEND`, `INSERT`, `REMOVE` modify the list
in place when possible (if there is only one reference), otherwise they create a copy. All list operations push the
resulting list back onto the stack.

| Command                     | Description                                                                                    |
|-----------------------------|------------------------------------------------------------------------------------------------|
| `LIST val1 val2 ...`        | Creates a list from the given values and pushes it onto the stack.                             |
| `INDEX list index`          | Pushes the element at the specified 0‑based index onto the stack.                              |
| `APPEND list element`       | Appends an element to the end. Returns the (possibly modified) list.                           |
| `LEN`                       | Returns the number of elements in a list (works for strings too).                              |
| `CONTAINS list element`     | Checks if a list contains a value. Returns boolean.                                            |
| `SLICE list start length`   | Extracts a sublist (non‑negative indices). Returns a new list.                                 |
| `REVERSE list`              | Reverses the list (in place if possible, otherwise creates a copy). Returns the reversed list. |
| `INSERT list index element` | Inserts an element at the given position. Returns the modified list.                           |
| `REMOVE list index`         | Removes the element at the given position. Returns the modified list.                          |
| `INDEXOF list element`      | Returns the first index of `element` in the list, or `-1` if not found.                        |

### Dictionary Operations (v1.7.0+)

Hi provides built‑in dictionaries (maps) with keys of type integer, float, string, or boolean. Dictionaries are *
*mutable** and stored in variables.

| Command              | Description                                                                                           |
|----------------------|-------------------------------------------------------------------------------------------------------|
| `DICT [var]`         | Creates an empty dictionary. If a variable name is given, stores it; otherwise pushes onto the stack. |
| `PUT dict key value` | Inserts or updates the `key` with `value` in the dictionary.                                          |
| `GET dict key`       | Retrieves the value for `key` and pushes it onto the stack.                                           |
| `HAS dict key`       | Returns `True` if `key` exists in the dictionary, `False` otherwise.                                  |
| `KEYS dict`          | Returns a list of all keys in the dictionary.                                                         |
| `VALUES dict`        | Returns a list of all values in the dictionary.                                                       |
| `LEN dict`           | Returns the number of key‑value pairs (works with `LEN` command).                                     |
| `REMOVE dict key`    | Removes the key from the dictionary. Also works for removing from lists by index.                     |

**Example:**

```hi
DICT user
PUT user "name" "Alice"
PUT user "age" 30
PUT user "active" True

GET user "name"
POP name
PRINT name      // Alice

HAS user "age"
POP has_age     // true

KEYS user
POP keys        // ["name", "age", "active"]
```

### File Operations (v1.5.0+)

Hi provides basic file I/O. All file operations work with file handles stored on the stack.

| Command              | Description                                                                                                                              |
|----------------------|------------------------------------------------------------------------------------------------------------------------------------------|
| `OPEN path mode`     | Opens a file. `mode` can be `"r"` (read), `"w"` (write, overwrites), or `"a"` (append). Pushes a file handle onto the stack.             |
| `CLOSE [file]`       | Closes a file handle. If no argument is given, pops the top file handle from the stack and closes it.                                    |
| `READ file [var]`    | Reads the entire remaining content of the file as a string. If a variable is given, stores it there; otherwise pushes onto the stack.    |
| `WRITE file value`   | Writes a string (or any value) to the file without adding a newline.                                                                     |
| `READLN file [var]`  | Reads one line from the file (including newline if present). If a variable is given, stores it there; otherwise pushes onto the stack.   |
| `WRITELN file value` | Writes a string and appends a newline.                                                                                                   |
| `EOF [file]`         | Returns `True` if the end of the file has been reached, otherwise `False`. If no argument is given, pops the file handle from the stack. |

**Example:**

```hi
OPEN "output.txt" "w"
POP f
WRITELN f "Hello, world!"
CLOSE f

OPEN "output.txt" "r"
POP f
READLN f
POP line
PRINT "Read: " line
CLOSE f
```

### Importing Files (v1.6.0+)

The `IMPORT` directive allows you to include code from other `.hi` files.  
This makes it possible to split your program into multiple modules, reuse functions, and organise larger projects.

**Syntax:**

```hi
IMPORT "path/to/file.hi"
```

Paths are resolved **relative to the current file**. Circular imports are detected and blocked (each file is imported
only once).

**Example:**

`utils.hi`:

```hi
FUNC greet
    PRINT "Hello from utils!"
RET
ENDF
```

`main.hi`:

```hi
IMPORT "utils.hi"
CALL greet
```

The preprocessor expands the `IMPORT` directive before the interpreter runs, so the interpreter sees a single flat list
of lines.

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

### Appending an Element

```hi
APPEND mylist 42
POP newlist
PRINT newlist   // [1, 2, 3, "hello", True, 42]
```

### Inserting and Removing

```hi
INSERT mylist 2 99
POP updated
PRINT updated   // [1, 2, 99, 3, "hello", True, 42]

REMOVE updated 3
POP shorter
PRINT shorter   // [1, 2, 99, "hello", True, 42]
```

### Slicing

```hi
SLICE mylist 1 3
POP slice
PRINT slice     // [2, 3, "hello"]
```

### Reversing

```hi
REVERSE mylist
POP rev
PRINT rev       // [42, True, "hello", 3, 2, 1]
```

### Searching

```hi
INDEXOF mylist "hello"
POP idx         // 2
CONTAINS mylist 99
POP has         // true
```

### Length

```hi
LEN mylist
POP len         // 6
```

> **Note:** Lists are mutable with copy‑on‑write. Operations like `APPEND`, `INSERT`, `REMOVE`, `REVERSE` modify the
> list in place if possible, but if there are multiple references, a copy is made. The resulting list is always pushed
> onto the stack.

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

## 7. Command‑Line Arguments (v1.7.0+)

When you run a Hi script, you can pass arguments after the filename:

```bash
hi script.hi arg1 arg2 --name Alice --verbose
```

The interpreter automatically provides two global variables:

| Variable    | Type            | Description                                                                                                                 |
|-------------|-----------------|-----------------------------------------------------------------------------------------------------------------------------|
| `ARGS`      | List of strings | All positional arguments (no flags) in the order they appear.                                                               |
| `ARGS_DICT` | Dictionary      | Parsed named arguments. Supports `--key value`, `--key=value`, `-k value`, `-k=value`. Flags without a value become `True`. |

Positional arguments (those not starting with `-`) are **not** included in `ARGS_DICT`, but they are present in `ARGS`.

**Example:**

```hi
ARGS
POP positional
PRINT "Positional args: " positional

GET ARGS_DICT "name"
POP name
PRINT "Name: " name

HAS ARGS_DICT "verbose"
POP is_verbose
PRINT "Verbose: " is_verbose
```

---

## 8. REPL Mode (Interactive Shell)

If you run the `hi` command without a filename, it starts an interactive REPL (Read‑Eval‑Print Loop):

```bash
hi
```

You can type commands directly, and they are executed immediately.  
Multi‑line blocks like `IF`/`ENDIF`, `WHILE`/`DO`, and `FUNC`/`ENDF` are supported: the REPL will keep reading lines
until the block is properly closed.

**Special commands** (start with a colon):

| Command            | Description                                                                               |
|--------------------|-------------------------------------------------------------------------------------------|
| `:exit` or `:quit` | Exits the REPL.                                                                           |
| `:clear`           | Clears the stack and all global variables.                                                |
| `:vars`            | Prints all global variables and their values.                                             |
| `:stack`           | Prints the current contents of the stack.                                                 |
| `:load "file.hi"`  | Loads and executes a `.hi` file in the current REPL session (file is imported only once). |

Example session:

```
Hi REPL v1.7.0 — type :exit or :quit to quit
Enter commands (multi-line blocks like IF/WHILE/FUNC are supported)

hi> LET x "Hello World!"
hi> FUNC greet
...> PRINT x
...> RET
...> ENDF
hi> CALL greet
Hello World!
hi> :vars
x = Hello World!
hi> :stack
Stack: [Int(1)]
hi> :clear
hi> :stack
Stack: []
hi> :exit
```

---

## 9. Full Example Program

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

## 10. Notes

- **SP** (Stack Pointer) always refers to the top of the stack.
- **Booleans** are represented as `True` and `False` (case‑sensitive).
- **Strings** are immutable; operations like `UPPER` create new strings.
- **Lists** are mutable (with copy‑on‑write) – operations that change the list return the modified list.
- **Dictionaries** are mutable – operations like `PUT`, `REMOVE` modify the dictionary in place.
- The interpreter is **case‑insensitive** for command names, but **case‑sensitive** for variable names and boolean
  literals.

---

Happy coding! 🚀