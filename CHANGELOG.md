# Changelog

All notable changes to the Hi language interpreter will be documented in this file.

---

## [2.0.0] – 2026-08-02

### 💔 Breaking Changes & Apology

**We are sincerely sorry** – v2.0.0 introduces a **complete rewrite** of the interpreter and a **new syntax** that is *
*not backward‑compatible** with v1.x.

This decision was not made lightly. The original stack‑based model (inspired by Forth) was powerful but proved difficult
to learn, debug, and extend. After listening to community feedback and analysing real‑world usage, we decided to pivot
to a more **expressive, readable, and beginner‑friendly** syntax, while keeping the lightweight and embeddable nature of
Hi.

We understand that this breaks existing scripts, and we deeply regret any inconvenience. We have prepared a short
migration guide (see below) to help you update your programs. The v1.x codebase remains available in the `v1.x` branch
and will continue to receive critical bug fixes for the foreseeable future.

Thank you for your understanding and continued support. 🙏

---

### ✨ Added (v2.0.0)

- **Complete AST‑based interpreter** – programs are now parsed into an Abstract Syntax Tree and evaluated recursively,
  enabling better error messages, scoping, and future optimisations.
- **New, BASIC‑like syntax**:
    - Expressions with operators (`+`, `-`, `*`, `/`, `%`, `^`), parentheses, and proper precedence.
    - `LET var = expr` for variable declaration.
    - `IF cond THEN ... ELSE ... END` for conditionals.
    - `WHILE cond DO ... END` and new `FOR var = start TO end DO ... NEXT [step]` for loops.
    - `FUNC name(params) ... END` for functions with named parameters.
    - `RET [expr]` to return values.
    - `PRINT expr, expr, ...` for output.
    - `INPUT ["prompt"] var` for interactive input.
- **Built‑in functions** (callable as `name(args)`):
    - String: `len`, `split`, `replace`, `starts`, `ends`, `upper`, `lower`, `trim`, `concat`, `substr`, `reverse`,
      `indexof`, `contains`.
    - List: `len`, `append`, `insert`, `remove`, `contains`, `indexof`, `slice`, `reverse`.
    - Dictionary: `len`, `keys`, `values`, `contains`, `remove`, `put`, `get`.
    - Math: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `torad`, `todeg`, `exp`, `log`, `log2`, `log10`,
      `ceil`, `floor`, `round`, `abs`, `rand`.
    - Conversion: `tostring`, `toint`, `tofloat`.
    - File I/O: `open`, `close`, `read`, `readln`, `write`, `writeln`, `eof`.
    - Higher‑order: `call` – calls a function stored in a variable.
- **Lists** as first‑class values with `[elem, ...]` literal syntax and full mutability (copy‑on‑write).
- **Dictionaries** with `{key = value, ...}` literal syntax and hashable keys (int, float, string, bool).
- **File I/O** via file handles returned by `open()`.
- **Module system** with `IMPORT "file.hi"` – recursive, with cycle detection and duplicate prevention.
- **Command‑line arguments** – automatically populate global variables `ARGS` (list) and `ARGS_DICT` (dictionary) from
  script arguments.
- **REPL enhancements** – supports multi‑line blocks, special commands (`:load`, `:vars`, `:clear`, `:exit`), and better
  error reporting.
- **Richer error messages** – span‑aware errors with line/column information.
- **Full test suite** – unit tests for lexer/parser and integration tests covering all language features.

---

### 🔄 Changed

- **Syntax** is now expression‑oriented and closer to BASIC/Lua, instead of stack‑based.
- **Variable scoping** – functions have local scopes; globals are accessible but shadowed by locals.
- **String escape sequences** now support `\u{XXXX}` Unicode escapes.
- **Parser** and **lexer** are completely rewritten to support the new grammar.
- **Preprocessor** handles `IMPORT` directives before parsing.
- **REPL** now uses `rustyline` for line editing and history.
- **CLI** now accepts a single file argument and optional script arguments after `--`.

---

### ❌ Removed (v1.x stack‑based commands)

All stack‑oriented commands have been removed:

- `PUSH`, `POP`, `DUP`, `SWAP`, `SP`
- `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `POW` (now replaced by infix operators)
- `EQ`, `NE`, `GT`, `GE`, `LT`, `LE` (now operators: `==`, `!=`, `>`, `>=`, `<`, `<=`)
- `AND`, `OR`, `NOT` (now operators: `AND`, `OR`, `NOT` as keywords in expressions)
- `LIST`, `INDEX`, `APPEND`, `INSERT`, `REMOVE`, `SLICE`, `REVERSE`, `INDEXOF`, `CONTAINS` (now replaced by built‑in
  functions)
- `DICT`, `PUT`, `GET`, `HAS`, `KEYS`, `VALUES` (replaced by built‑in functions)
- `OPEN`, `CLOSE`, `READ`, `READLN`, `WRITE`, `WRITELN`, `EOF` (now functions)
- `LEN`, `UPPER`, `LOWER`, `TRIM`, `STARTS`, `ENDS`, `REPLACE`, `SPLIT`, `CONCAT`, `SUBSTR` (now functions)
- `HELLO` command (replaced by `PRINT "Hello, World!"`)
- `CALL` command (replaced by function call syntax `name(args)`)
- `FUNC`/`ENDF` (replaced by `FUNC ... END`)
- `IF`/`ENDIF` (replaced by `IF ... THEN ... END`)
- `WHILE`/`DO` (replaced by `WHILE ... DO ... END`)

---

### 🛠️ Migration Guide (from v1.x to v2.0.0)

Since the syntax is completely different, migration requires rewriting your scripts. Here are the main steps:

1. **Replace stack operations** with expressions and assignments.
    - `PUSH 5` → `LET val = 5` or just use `5` in expressions.
    - `POP x` → `LET x = ...` or assignment `x = ...`.
    - `ADD`, `SUB`, etc. → use `+`, `-`, `*`, `/`, `%`, `^`.
2. **Rewrite conditions** using `IF ... THEN ... ELSE ... END`.
    - `IF EQ x 5` → `IF x == 5 THEN ... END`.
3. **Rewrite loops**:
    - `WHILE LT i 10 DO ... DO` → `WHILE i < 10 DO ... END`.
4. **Functions** – use named parameters instead of stack arguments.
    - `FUNC square` → `FUNC square(x) RET x * x END`.
    - `CALL square` → `square(5)`.
5. **Collections** – use built‑in functions.
    - `LIST 1 2 3` → `[1, 2, 3]`.
    - `APPEND list 42` → `append(list, 42)`.
    - `DICT` → `{}`.
    - `PUT dict key value` → `dict[key] = value` or `put(dict, key, value)`.
6. **File I/O** – use functions.
    - `OPEN "file" "r"` → `open("file", "r")`.
    - `READ f` → `read(f)`.
7. **Remove `HELLO`** – use `PRINT "Hello, World!"`.

We have provided many examples in the `examples/` folder to illustrate the new syntax.

---

### 🙏 Acknowledgements

We thank all contributors and users for their support and feedback that guided this major revision. Special thanks to
the early adopters who tested the development builds and helped shape the new design.