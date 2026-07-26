# Hi Language

**Hi** is a minimalist, stack‑based interpreted language that blends a **BASIC‑like syntax** with **Forth‑style stack
operations**.  
It supports variables, loops, conditionals, functions, interactive input, and logical operators — all written in **Rust
** for speed and safety.

> **.hi** is the official file extension for Hi source files.

---

## 📦 Installation

The easiest way to get started is to **download a pre‑built binary** from
the [Releases](https://github.com/hiveMC3310/hi-lang/releases) page.  
Choose the executable for your platform (Windows, Linux) and place it somewhere in your `PATH`.

Alternatively, you can build from source (recommended for developers or if you want the latest unreleased changes):

```bash
git clone https://github.com/hiveMC3310/hi-lang.git
cd hi-lang
cargo build --release
```

The binary will be placed at `target/release/hi` (or `hi.exe` on Windows).

---

## 🚀 Usage

Run a Hi program:

```bash
hi path/to/program.hi
```

The interpreter reads the file line by line and executes it.

---

## 📖 Language Reference

### Comments

Comments start with `//` and extend to the end of the line.

```hi
// This is a comment
```

---

### Commands

| Command                            | Description                                                                                                                                                                                                                                                          |
|------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `HELLO`                            | Prints `Hello, World!` to the console.                                                                                                                                                                                                                               |
| `PUSH value`                       | Pushes a value onto the stack.                                                                                                                                                                                                                                       |
| `POP [var]`                        | Pops the top value from the stack. If a variable name is given, the value is stored in that variable (local or global).                                                                                                                                              |
| `LET var value`                    | Assigns a value to a variable (local if inside a function, otherwise global).                                                                                                                                                                                        |
| `PRINT arg1 arg2 ...`              | Concatenates and prints all arguments. Arguments are resolved as values, variables, or `SP`.                                                                                                                                                                         |
| `INPUT [prompt] var`               | Reads a line from standard input. If a prompt is given, it is displayed before reading. The input is parsed as a number if possible, otherwise as a boolean (`True`/`False`) or stored as a string.                                                                  |
| `ADD`, `SUB`, `MUL`, `DIV`         | Arithmetic operations. They can be used in two ways: <br> • **Without arguments**: pops two values from the stack and pushes the result. <br> • **With two arguments**: e.g., `ADD 3 5` computes 3+5 and pushes the result.                                          |
| `EQ`, `NE`, `GT`, `GE`, `LT`, `LE` | Comparison operators. They return a boolean (`True` or `False`). Same usage as arithmetic.                                                                                                                                                                           |
| `AND`, `OR`, `NOT`                 | Logical operators. They can be used with arguments or from the stack (for binary operators). `NOT` works with one argument or one stack value. All values are converted to booleans using Hi semantics (0, empty string, false → `False`; everything else → `True`). |
| `IF condition`                     | Starts a conditional block. If the condition is **false**, jumps to `ELSE` (if present) or `ENDIF`. <br> • **Classic**: `POP cond; IF cond` <br> • **Inline**: `IF EQ x 5`, `IF AND flag1 flag2` – condition is evaluated directly without touching the stack.       |
| `ELSE`                             | Marks the start of the alternative block. Executed only if the preceding `IF` condition was **false**. After executing this block, execution jumps to the matching `ENDIF`.                                                                                          |
| `ENDIF`                            | Marks the end of an `IF`/`ELSE` block.                                                                                                                                                                                                                               |
| `WHILE condition`                  | If the condition is **false**, jumps to `DO`. <br> • **Classic**: `POP cond; WHILE cond` <br> • **Inline**: `WHILE LT i 10` – condition evaluated directly.                                                                                                          |
| `DO`                               | Marks the end of a `WHILE` block. Jumps back to the corresponding `WHILE`.                                                                                                                                                                                           |
| `BREAK`                            | Immediately exits the innermost `WHILE` loop.                                                                                                                                                                                                                        |
| `FUNC name`                        | Defines a function. The function body is executed when `CALL`ed.                                                                                                                                                                                                     |
| `RET`                              | Returns from the current function (explicit return).                                                                                                                                                                                                                 |
| `ENDF`                             | Marks the end of a function definition (implicit return).                                                                                                                                                                                                            |
| `CALL name`                        | Calls a previously defined function.                                                                                                                                                                                                                                 |

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
- Arithmetic, comparison, and logical commands can pop operands from the stack (if no arguments are given) or use
  explicit arguments.
- The special token **`SP`** (Stack Pointer) resolves to the current top‑of‑stack value. It can be used anywhere a value
  is expected, e.g., `PRINT SP`.

---

### Control Flow

**Conditional execution:**

```hi
IF condition
    // code if true
ELSE
    // code if false
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
ENDF
```

To call a function, use `CALL`:

```hi
CALL greet
```

Functions have **local variables** that are isolated from the global scope.

---

## 💡 Examples

You can find ready‑to‑run example programs in the [`examples/`](examples/) folder of the repository. Here are a few
highlights:

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

### Input and conditional with ELSE

```hi
INPUT "Enter your age: " age
GE age 18
POP cond
IF cond
    PRINT "You are adult"
ELSE
    PRINT "Minor"
ENDIF
```

### Inline conditions (v1.1.0+)

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

---

## 📝 License

MIT License — feel free to use, modify, and distribute.

---

Happy coding in Hi! 🚀