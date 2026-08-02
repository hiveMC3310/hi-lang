# Getting Started

This chapter will help you install the Hi interpreter, write your first program, and run it.

---

## Installation

### Option 1: Pre‑built Binary (Recommended)

The easiest way to get started is to download a pre‑built binary for your platform from
the [Releases](https://github.com/hiveMC3310/hi-lang/releases) page.

1. Choose the appropriate executable for your operating system:
    - **Windows**: `hi.exe`
    - **Linux**: `hi` (x86_64)

2. Place the binary somewhere in your `PATH`:
    - On **Windows**, you can put it in `C:\Windows\System32` or add its folder to the `PATH` environment variable.
    - On **Linux**, you can move it to `/usr/local/bin` or `~/.local/bin` and **make it executable**:

```bash
chmod +x /path/to/hi
```

3. Verify the installation by running the interpreter without arguments – it should start the REPL:

```bash
hi
```

You should see a prompt like:

```
Hi REPL v2.0.0 — type :exit or :quit to quit
hi>
```

Type `:exit` to quit.

---

### Option 2: Building from Source

If you have Rust installed, you can build Hi from source:

```bash
git clone https://github.com/hiveMC3310/hi-lang.git
cd hi-lang
cargo build --release
```

The binary will be placed at `target/release/hi` (or `hi.exe` on Windows). You can copy it to a directory in your
`PATH` (and make it executable on Linux with `chmod +x`).

---

## Your First Program

Create a file named `hello.hi` with the following content:

```hi
hello()
```

Now run it:

```bash
hi hello.hi
```

You should see the output:

```
Hello, World!
```

Congratulations – you’ve just run your first Hi program!

---

## Using the REPL

The REPL (Read‑Eval‑Print Loop) is perfect for experimenting. Just type `hi` to start it.

You can enter multiple‑line blocks, such as `IF` statements or function definitions. The REPL will keep reading until
the block is complete.

Example REPL session:

```hi
hi> LET x = 10
hi> LET y = 20
hi> PRINT x + y
30
hi> IF x > y THEN
...>     PRINT "x is greater"
...> ELSE
...>     PRINT "y is greater"
...> END
y is greater
hi> :exit
```

Special commands in the REPL start with a colon (`:`):

| Command            | Description                          |
|--------------------|--------------------------------------|
| `:exit` or `:quit` | Quit the REPL.                       |
| `:clear`           | Clear all variables and reset state. |
| `:vars`            | Show all variables and their values. |
| `:load "file.hi"`  | Load and execute a Hi file.          |

---

## Next Steps

Now that you have Hi running, explore the language:

- Read the [Syntax](syntax.md) chapter to learn the language basics.
- See the [Built‑in Functions](builtins.md) reference.
- Check out the [Examples](examples.md) for ready‑to‑run scripts.

Happy coding in Hi! 🚀