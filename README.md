# Hi Language

**Hi** is a minimalist, stack‑based interpreted language that blends a **BASIC‑like syntax** with **Forth‑style stack
operations**.  
It supports variables, loops, conditionals, functions, interactive input, lists, and string operations, also I/O work
with files — all written in
**Rust** for speed and safety.

> **.hi** is the official file extension for Hi source files.

---

## 📦 Installation

The easiest way to get started is to **download a pre‑built binary** from
the [Releases](https://github.com/hiveMC3310/hi-lang/releases) page.  
Choose the executable for your platform (Windows, Linux, macOS) and place it somewhere in your `PATH`.

Alternatively, you can build from source:

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
WHILE LT i 5
    PRINT i
    ADD i 1
    POP i
DO
```

### Inline conditions (v1.1.0+)

```hi
LET x 5
IF EQ x 5
    PRINT "x is 5"
ENDIF
```

### Functions

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

### Lists (v1.2.0+)

```hi
LIST 1 2 3 4
POP mylist
PRINT "List: " mylist

LEN mylist
POP len
PRINT "Length: " len

INDEX mylist 2
POP third
PRINT "Third element: " third

APPEND mylist 42
POP newlist
PRINT "New list: " newlist
```

---

## 📚 Documentation

For a complete language reference including all commands, see the full documentation:

- **[Language Reference](docs/language.md)** – complete command list, syntax, and examples.

---

## 📝 License

MIT License — feel free to use, modify, and distribute.

---

Happy coding in Hi! 🚀