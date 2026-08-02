# Introduction

Welcome to the **Hi Language** – a lightweight, dynamically typed interpreted language designed for simplicity,
readability, and fun.

Hi (pronounced like the English greeting) is a **high‑level scripting language** that blends a clean, BASIC‑inspired
syntax with the power of modern data structures and functions. It is written entirely in **Rust**, making it fast, safe,
and easy to embed.

---

## What is Hi?

Hi was created with a few core goals in mind:

- **Simplicity** – The syntax is minimal and consistent, making it an excellent choice for beginners, prototyping, and
  teaching programming concepts.
- **Readability** – Code in Hi reads like plain English, with clear keywords and intuitive constructs.
- **Embeddability** – Thanks to its Rust foundation, Hi can be easily integrated into other applications as a scripting
  language.
- **Batteries included** – The standard library provides strings, lists, dictionaries, file I/O, mathematics, and more –
  without external dependencies.

Hi is **interpreted**, meaning you can run scripts directly without compilation, making it ideal for rapid iteration.

---

## A Brief History

The original Hi (v1.x) was inspired by **Forth** and **BASIC**, combining a stack‑based execution model with a simple,
imperative syntax. While powerful, the stack paradigm had a steep learning curve and made complex expressions difficult
to read.

With **v2.0**, we decided to **completely rethink** the language. We kept the lightweight spirit but moved to an *
*AST‑based interpreter** with a more natural, expression‑oriented syntax. The new Hi is:

- **Easier to learn** – no more juggling stack values; just write expressions like `a + b * c`.
- **More expressive** – functions have named parameters, lists and dictionaries are first‑class, and control flow
  follows familiar patterns.
- **Safer** – better error messages with line/column information, and rigorous scoping rules.

We understand that this breaks backward compatibility, and we apologise for the inconvenience. However, we believe this
change makes Hi a much more enjoyable and productive language for everyone.

---

## Who is Hi for?

- **Beginners** – the syntax is gentle and the interpreter gives clear error messages.
- **Educators** – an ideal language to teach programming fundamentals without syntactic clutter.
- **Hobbyists** – a fun language to experiment with, write small scripts, or automate tasks.

---

## Getting Started

This book will guide you through every aspect of the language. We recommend starting with
the [Getting Started](getting-started.md) chapter, which covers installation and your first program.

If you prefer to dive right in, check out the [examples](examples.md) section for ready‑to‑run code snippets.

Happy coding, and welcome to the world of Hi! 🚀