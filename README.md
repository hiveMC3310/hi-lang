# Hi Language

**Hi** is a minimalist, stack‑based interpreted language that blends a **BASIC‑like syntax** with **Forth‑style stack
operations**. It supports variables, loops, conditionals, functions, and interactive input — all written in **Rust** for
speed and safety.

> **.hi** is the official file extension for Hi source files.

---

## 📦 Installation

Clone the repository and build the interpreter:

```bash
git clone https://github.com/yourusername/hi-lang.git
cd hi-lang
cargo build --release
```

The binary will be placed at `target/release/hi` (or `hi.exe` on Windows). You can copy it to a directory in your `PATH`
if desired.

---

## 🚀 Usage

Run a Hi program:

```bash
hi path/to/program.hi
```

If the file has the `.hi` extension, the interpreter will execute it line by line.

---

## 📖 Language Reference

### Comments

Comments start with `//` and extend to the end of the line.

```hi
// This is a comment
```

---

### Commands

| Command                            | Description                                                                                                                                                                                                                 |
|------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `HELLO`                            | Prints `Hello, World!` to the console.                                                                                                                                                                                      |
| `PUSH value`                       | Pushes a value onto the stack.                                                                                                                                                                                              |
| `POP [var]`                        | Pops the top value from the stack. If a variable name is given, the value is stored in that variable (local or global).                                                                                                     |
| `LET var value`                    | Assigns a value to a variable (local if inside a function, otherwise global).                                                                                                                                               |
| `PRINT arg1 arg2 ...`              | Concatenates and prints all arguments. Arguments are resolved as values, variables, or `SP`.                                                                                                                                |
| `INPUT [prompt] var`               | Reads a line from standard input. If a prompt is given, it is displayed before reading. The input is parsed as a number if possible, otherwise stored as a string.                                                          |
| `ADD`, `SUB`, `MUL`, `DIV`         | Arithmetic operations. They can be used in two ways: <br> • **Without arguments**: pops two values from the stack and pushes the result. <br> • **With two arguments**: e.g., `ADD 3 5` computes 3+5 and pushes the result. |
| `EQ`, `NE`, `GT`, `GE`, `LT`, `LE` | Comparison operators. They return a boolean (`1` for true, `0` for false). Same usage as arithmetic.                                                                                                                        |
| `IF condition`                     | If the condition evaluates to **false**, jumps to the matching `ENDIF`.                                                                                                                                                     |
| `ENDIF`                            | Marks the end of an `IF` block.                                                                                                                                                                                             |
| `WHILE condition`                  | If the condition is **false**, jumps to the matching `DO`. Otherwise, enters the loop.                                                                                                                                      |
| `DO`                               | Marks the end of a `WHILE` block. Jumps back to the corresponding `WHILE`.                                                                                                                                                  |
| `BREAK`                            | Immediately exits the innermost `WHILE` loop.                                                                                                                                                                               |
| `FUNC name`                        | Defines a function. The function body is executed when `CALL`ed.                                                                                                                                                            |
| `RET`                              | Returns from the current function (explicit return).                                                                                                                                                                        |
| `ENDF`                             | Marks the end of a function definition (implicit return).                                                                                                                                                                   |
| `CALL name`                        | Calls a previously defined function.                                                                                                                                                                                        |

---

### Variables

- **Global variables** are available everywhere.
- **Local variables** are created inside functions (via `LET` or `POP`) and are destroyed when the function returns.

Variable names consist of letters, digits, and underscores, starting with a letter.

---

### The Stack

The language maintains a **value stack**. Many commands interact with it:

- `PUSH` places a value on top.
- `POP` removes the top value (and optionally stores it).
- Arithmetic and comparison commands can pop operands from the stack (if no arguments are given) or use explicit
  arguments.
- The special token **`SP`** (Stack Pointer) resolves to the current top‑of‑stack value. It can be used anywhere a value
  is expected, e.g., `PRINT SP`.

---

### Control Flow

**Conditional execution:**

```hi
IF condition
    // code if true
ENDIF
```

**Loops:**

```hi
WHILE condition
    // loop body
DO
```

You can use `BREAK` to exit a loop prematurely.

---

### Functions

Functions are defined with `FUNC`, followed by a name, and closed with `ENDF`:

```hi
FUNC greet
    PRINT "Hello from function!"
RET
```

To call a function, use `CALL`:

```hi
CALL greet
```

Functions have **local variables** that are isolated from the global scope.

---

## 💡 Examples

### Hello World

```hi
HELLO
```

### Stack arithmetic

```hi
PUSH 5
PUSH 3
ADD          // adds 5 and 3, result is on stack
POP result   // store result in variable "result"
PRINT "Result: " result
```

### Loop with condition

```hi
LET i 0
LET running 1
LT i 5
WHILE running
    PRINT i
    ADD i 1
    POP i
    LT i 5
    POP running
DO
```

### Function with local variable

```hi
FUNC square
    // expects a number on the stack
    PUSH SP   // duplicate top
    MUL       // multiply top two
    RET
ENDF

PUSH 7
CALL square
POP result
PRINT "7² = " result
```

### Input and conditional

```hi
INPUT "Enter your age: " age
GE age 18
POP cond
IF cond
    PRINT "You are adult"
ENDIF
LT age 18
POP cond
IF cond
    PRINT "Minor"
ENDIF
```

> **Note:** `ELSE` is not yet supported; you can use two separate `IF` statements.

---

## 📝 License

MIT License — feel free to use, modify, and distribute.

---

Happy coding in Hi! 🚀