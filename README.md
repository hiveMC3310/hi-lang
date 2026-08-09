# Hi Language v2.0.0

**Hi** is a minimalist, dynamically typed interpreted language with a **clean, readable syntax** inspired by BASIC.  
It supports variables, arithmetic/logic, control flow, functions, lists, dictionaries, file I/O, **modules**, and more —
all written in **Rust** for speed and safety.

> **.hi** is the official file extension for Hi source files.

---

## ✨ What's New in v2.0.0?

Hi v2.0.0 introduces a completely redesigned interpreter based on an **Abstract Syntax Tree (AST)**.  
The language has moved away from explicit stack operations to a more natural, expression‑oriented syntax:

- **Variables** declared with `LET` and assigned with `=`.
- **Arithmetic expressions** like `3 + 5 * (2 - 1)`.
- **Conditions** with `IF ... THEN ... ELSE ... END`.
- **Loops**: `WHILE ... DO ... END` and `FOR ... TO ... NEXT`.
- **Functions** defined with `FUNC ... END` and called with `name(args)`.
- **Built‑in modules** (`math`, `strings`, `io`) providing namespaced functions and constants.
- **User modules** – import your own `.hi` files with `IMPORT` and optional aliases.
- **Lists** and **dictionaries** as first‑class citizens.
- **File I/O** with `open`, `read`, `write`, etc.
- **Command‑line arguments** available as `ARGS` (list) and `ARGS_DICT` (dict).

The new syntax is more intuitive, easier to read, and less error‑prone — while keeping the lightweight feel of the
original.

---

## 📦 Installation

### Pre‑built Binaries (Recommended)

The easiest way to get started is to download the latest release from the
*[Releases](https://github.com/hiveMC3310/hi-lang/releases)* page.  
Each release includes both the interpreter (`hi`) and the Language Server (`hi-lsp`) for the following platforms:

| Platform                  | Archive                                         |
|---------------------------|-------------------------------------------------|
| **Linux (x86_64, musl)**  | `hi-<version>-x86_64-unknown-linux-musl.tar.gz` |
| **Windows (x86_64)**      | `hi-<version>-x86_64-pc-windows-msvc.zip`       |
| **macOS (Apple Silicon)** | `hi-<version>-aarch64-apple-darwin.tar.gz`      |

1. Download the archive for your platform.
2. Extract it to a directory of your choice.
3. (Optional) Add that directory to your `PATH` for easy access.

After extraction, you will have two executables:

- `hi` – the interpreter (use it to run `.hi` scripts or start the REPL).
- `hi-lsp` – the Language Server (used by editors like VS Code).

### Building from Source

If you prefer to compile from source, you'll need Rust installed:

```bash
git clone https://github.com/hiveMC3310/hi-lang.git
cd hi-lang
cargo build --release --bin hi --bin hi-lsp
```

The binaries will be placed at `target/release/hi` and `target/release/hi-lsp` (or with `.exe` on Windows).

---

## 🚀 Usage

### Running a Hi Script

```bash
hi path/to/program.hi
```

### Starting the REPL

```bash
hi
```

### Using the Language Server

The `hi-lsp` binary implements the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) and
can be used with any compatible editor.  
For **VS Code**, we provide a dedicated extension – see the [VS Code Extension](#-vs-code-extension) section below.

---

## 💡 Examples (v2.0.0 syntax)

### Hello World

```hi
PRINT "Hello, World!"
```

or just

```hi
hello()
```

### Arithmetic and Variables

```hi
LET a = 10
LET b = 3
LET sum = a + b
LET product = a * b
LET division = a / b
LET remainder = a % b 

PRINT "Sum: ", sum
PRINT "Product: ", product
PRINT "Division: ", division
PRINT "Remainder: ", remainder
```

### Conditional (IF)

```hi
LET age = 18
IF age >= 18 THEN
    PRINT "You are an adult."
ELSE
    PRINT "You are a minor."
END
```

### Loop (WHILE)

```hi
LET i = 0
WHILE i < 5 DO
    PRINT "i = ", i
    i = i + 1
END
```

### Loop (FOR)

```hi
FOR i = 0 TO 5 DO
    PRINT "i = ", i
NEXT          // step defaults to 1

// With explicit step
FOR j = 10 TO 0 DO
    PRINT "j = ", j
NEXT -2       // counts down by 2
```

### Functions

```hi
FUNC factorial(n)
    IF n <= 1 THEN
        RET 1
    ELSE
        RET n * factorial(n - 1)
    END
END

LET result = factorial(5)
PRINT "5! = ", result
```

### Lists

```hi
LET mylist = [1, 2, 3, "hello", TRUE]
PRINT "Length: ", len(mylist)

LET appended = append(mylist, 42)
PRINT appended              // [1, 2, 3, "hello", TRUE, 42]

LET removed = remove(appended, 2)
PRINT removed               // [1, 2, "hello", TRUE, 42]

LET slice = slice(mylist, 1, 3)
PRINT slice                 // [2, 3, "hello"]

LET reversed = reverse(mylist)
PRINT reversed
```

### Dictionaries

```hi
LET user = { "name" = "Alice", "age" = 30 }
PRINT user["name"]          // Alice

user["city"] = "New York"
PRINT user                  // {"name"="Alice", "age"=30, "city"="New York"}

LET keys = keys(user)
PRINT keys                  // ["name", "age", "city"]
```

### File I/O (using `io` module)

```hi
IMPORT "io" AS io
LET f = io:open("output.txt", "w")
io:writeln(f, "Hello, file!")
io:close(f)

LET f2 = io:open("output.txt", "r")
LET content = io:read(f2)
PRINT "File content: ", content
io:close(f2)
```

### Modules (IMPORT)

Hi has built‑in modules (`math`, `strings`, `io`) and supports user‑defined modules from `.hi` files.

**Using built‑in modules with aliases**:

```hi
IMPORT "math" AS m
LET x = m:sin(m:PI / 2)   // 1.0
PRINT x

IMPORT "strings" AS s
LET parts = s:split("a,b,c", ",")
PRINT parts                // ["a", "b", "c"]
```

**Inlining a built‑in module** (adds its functions to the global namespace):

```hi
IMPORT "math"
LET y = sin(PI / 2)        // sin and PI are now global
```

**User‑defined module**:

`double.hi`:

```hi
FUNC double(x)
    RET x * 2
END
```

`main.hi`:

```hi
IMPORT "double.hi" AS d
LET result = d:double(21)
PRINT result                // 42
```

**Inlining a user module** (its variables and functions become global):

```hi
IMPORT "double.hi"
LET result2 = double(10)   // double is now global
PRINT result2               // 20
```

**Imports are cached** – each file is loaded only once. Cyclic imports are detected and reported.

### Command‑Line Arguments

```bash
hi script.hi arg1 arg2 --name Alice --verbose
```

Inside the script:

```hi
PRINT "Positional args: ", ARGS          // ["arg1", "arg2"]
PRINT "Named args: ", ARGS_DICT["name"]  // "Alice"
PRINT "Verbose flag: ", ARGS_DICT["verbose"]  // TRUE
```

### And more…

For a complete set of ready‑to‑run programs, check out the [`examples/`](examples/) folder.

---

## 📚 Documentation

The full language reference (including all built‑in functions, syntax details, and advanced topics) is available as an *
*mdBook**:

👉 **[Hi Language Reference](https://hiveMC3310.github.io/hi-lang/)** (or read it locally in the `docs/` folder)

---

## 🖥️ VS Code Extension

For the best editing experience, we provide an
official [VS Code extension](https://github.com/hiveMC3310/hi-lang-support) that integrates the `hi-lsp` server,
offering:

- Syntax highlighting.
- Auto‑completion for keywords, built‑ins, and module members.
- Hover information and go‑to‑definition.
- Rename and find references.
- Inline diagnostics (errors/warnings).
- One‑click script execution.

After installing the extension, make sure `hi-lsp` is either in your `PATH` or configured via the `hi.serverPath`
setting.

---

## 📝 License

MIT License — feel free to use, modify, and distribute.

---

Happy coding in Hi! 🚀