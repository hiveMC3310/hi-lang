# Examples

This chapter contains a collection of ready‑to‑run Hi programs that demonstrate the language's features.  
You can find all these files in the `examples/` directory of the repository.

Each example is self‑contained and includes comments to explain what it does.

---

## Hello, World!

**File:** `examples/hello.hi`  
The classic introductory program.

```hi
// Simple "Hello, World!" program

PRINT "Hello, World!"

// or just

hello()

```

---

## Arithmetic

**File:** `examples/arithmetic.hi`  
Shows arithmetic expressions, variables, and operator precedence.

```hi
// Demonstrates arithmetic expressions

LET a = 10
LET b = 3

PRINT "a = ", a
PRINT "b = ", b
PRINT "a + b = ", a + b
PRINT "a - b = ", a - b
PRINT "a * b = ", a * b
PRINT "a / b = ", a / b          // yields float 3.333...
PRINT "a % b = ", a % b          // yields 1
PRINT "a ^ b = ", a ^ b          // 10^3 = 1000
PRINT "(a + b) * 2 = ", (a + b) * 2
```

---

## Conditionals

**File:** `examples/conditional.hi`  
Demonstrates `IF ... THEN ... ELSE ... END` with nested conditions.

```hi
// Demonstrates IF...THEN...ELSE...END

LET age = 18

IF age >= 18 THEN
    PRINT "You are an adult."
ELSE
    PRINT "You are a minor."
END

LET score = 85
IF score >= 90 THEN
    PRINT "Grade: A"
ELSE
    IF score >= 80 THEN
        PRINT "Grade: B"
    ELSE
        PRINT "Grade: C or lower"
    END
END

// Boolean conditions
LET is_sunny = TRUE
IF is_sunny THEN
    PRINT "Let's go outside!"
END
```

---

## Loops

**File:** `examples/loops.hi`  
Illustrates `WHILE` loops, `FOR` loops (with and without step), and `BREAK`.

```hi
// Demonstrates WHILE and FOR loops

// WHILE loop
LET i = 0
WHILE i < 5 DO
    PRINT "i = ", i
    i = i + 1
END
PRINT "Loop finished"

// FOR loop with default step (1)
PRINT "FOR loop 0 to 3:"
FOR j = 0 TO 3 DO
    PRINT "j = ", j
NEXT

// FOR loop with explicit step
PRINT "FOR loop 10 to 0 step -2:"
FOR k = 10 TO 0 DO
    PRINT "k = ", k
NEXT -2

// Nested loops with BREAK
PRINT "Nested loops with BREAK:"
FOR x = 0 TO 2 DO
    FOR y = 0 TO 2 DO
        IF x == 1 AND y == 1 THEN
            BREAK
        END
        PRINT "x=", x, " y=", y
    NEXT
NEXT
```

---

## Lists

**File:** `examples/lists.hi`  
Covers creation, indexing, appending, inserting, removing, slicing, reversing, searching, and concatenation.

```hi
// Demonstrates list operations

LET a = [10, 20, 30, 40]
PRINT "Original list: ", a

PRINT "Length: ", len(a)
PRINT "First element: ", a[0]
PRINT "Last element: ", a[3]

// Append
LET b = append(a, 50)
PRINT "After append 50: ", b

// Insert
LET c = insert(b, 2, 25)
PRINT "After insert 25 at index 2: ", c

// Remove
LET d = remove(c, 3)
PRINT "After remove index 3: ", d

// Slice
LET e = slice(d, 1, 3)
PRINT "Slice from 1 length 3: ", e

// Reverse
LET f = reverse(d)
PRINT "Reversed: ", f

// Search
PRINT "Index of 30: ", indexof(d, 30)
PRINT "Contains 25? ", contains(d, 25)
PRINT "Contains 99? ", contains(d, 99)

// List concatenation
LET g = concat([1, 2], [3, 4])
PRINT "Concat [1,2] and [3,4]: ", g
```

---

## Dictionaries

**File:** `examples/dicts.hi`  
Demonstrates dictionary creation, access, insertion, update, key existence, keys/values retrieval, removal, and length.

```hi
// Demonstrates dictionary operations

LET user = {"name" = "Alice", "age" = 30}
PRINT "Original dict: ", user

// Access
PRINT "Name: ", user["name"]
PRINT "Age: ", user["age"]

// Add/update via bracket notation
user["city"] = "New York"
PRINT "After adding city: ", user

// Update with put()
put(user, "age", 31)
PRINT "After updating age: ", user

// Check existence
PRINT "Contains 'name'? ", contains(user, "name")
PRINT "Contains 'country'? ", contains(user, "country")

// Get with default (nil)
LET missing = get(user, "country")
PRINT "Get 'country' (nil if missing): ", missing

// Keys and values
LET keys_list = keys(user)
PRINT "Keys: ", keys_list
LET values_list = values(user)
PRINT "Values: ", values_list

// Remove
remove(user, "city")
PRINT "After removing 'city': ", user

// Length
PRINT "Number of entries: ", len(user)
```

---

## Functions

**File:** `examples/functions.hi`  
Shows function definition, parameters, return values, recursion, and side‑effect functions.

```hi
// Demonstrates function definitions and recursion

// Simple function
FUNC greet(name)
    RET concat("Hello, ", name)
END

LET message = greet("World")
PRINT message

// Function with multiple parameters
FUNC add(a, b)
    RET a + b
END

PRINT "3 + 5 = ", add(3, 5)

// Recursive factorial
FUNC factorial(n)
    IF n <= 1 THEN
        RET 1
    ELSE
        RET n * factorial(n - 1)
    END
END

LET fact5 = factorial(5)
PRINT "5! = ", fact5

// Function with side effects (no explicit return)
FUNC print_sum(a, b)
    PRINT "Sum = ", a + b
END

print_sum(10, 20)   // prints sum, returns nil

// Recursive Fibonacci
FUNC fib(n)
    IF n <= 1 THEN
        RET n
    ELSE
        RET fib(n - 1) + fib(n - 2)
    END
END

PRINT "fib(6) = ", fib(6)   // 8
```

---

## Function Values

**File:** `examples/function_values.hi`  
Demonstrates first‑class functions and the `call()` built‑in for higher‑order programming.

```hi
// Demonstrates first‑class functions and the call() built‑in

FUNC double(x)
    RET x * 2
END

FUNC triple(x)
    RET x * 3
END

// Store functions in variables
LET f = double
LET g = triple

PRINT "double(5) via f: ", call(f, 5)
PRINT "triple(5) via g: ", call(g, 5)

// Higher‑order function: apply a function to a list
FUNC map_func(func, list)
    LET result = []
    FOR i = 0 TO len(list) - 1 DO
        LET val = list[i]
        LET mapped = call(func, val)
        result = append(result, mapped)
    NEXT
    RET result
END

LET numbers = [1, 2, 3, 4]
LET doubled = map_func(double, numbers)
PRINT "Original: ", numbers
PRINT "Doubled: ", doubled

// Passing a function defined inline (using a variable)
FUNC square(x) RET x * x END
LET sq = square
LET squared = map_func(sq, numbers)
PRINT "Squared: ", squared
```

---

## I/O

**File:** `examples/io.hi`  
Covers file writing, reading, reading lines, checking EOF, and console input with `INPUT`.

```hi
// Demonstrates file I/O and console input

// Write to a file
LET f = open("output.txt", "w")
writeln(f, "Hello, file!")
writeln(f, "Line 2")
close(f)

// Read from the file
LET f2 = open("output.txt", "r")
LET content = read(f2)
close(f2)
PRINT "File content: ", content

// Read line by line
LET f3 = open("output.txt", "r")
LET line1 = readln(f3)
LET line2 = readln(f3)
LET eof_flag = eof(f3)
LET line3 = readln(f3)   // empty if EOF
close(f3)

PRINT "Line1: ", line1
PRINT "Line2: ", line2
PRINT "EOF after line2? ", eof_flag
PRINT "Line3 (EOF): ", line3

// Console input (interactive – try typing)
PRINT "Enter your name: "
INPUT name
PRINT "Hello, ", name

// Using a prompt
INPUT "Enter your age: " age
PRINT "You are ", age, " years old."
```

---

## Import Example

**Files:** `examples/import_example/lib.hi` and `examples/import_example/main.hi`  
Shows how to split code into modules with `IMPORT`.

**lib.hi** (library module):

```hi
// Library module with utility functions

FUNC greet(name)
    RET concat("Hello, ", name)
END

FUNC add(a, b)
    RET a + b
END

FUNC multiply(a, b)
    RET a * b
END
```

**main.hi** (main program):

```hi
// Main program that imports lib.hi

IMPORT "lib.hi"

LET message = greet("Alice")
PRINT message

LET sum = add(10, 20)
PRINT "10 + 20 = ", sum

LET product = multiply(5, 6)
PRINT "5 * 6 = ", product
```

---

These examples cover most of the language features. Feel free to experiment by modifying them or writing your own
programs.