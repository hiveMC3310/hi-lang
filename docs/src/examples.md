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
LET f = io:open("output.txt", "w")
io:writeln(f, "Hello, file!")
io:writeln(f, "Line 2")
io:close(f)

// Read from the file
LET f2 = io:open("output.txt", "r")
LET content = io:read(f2)
io:close(f2)
PRINT "File content: ", content

// Read line by line
LET f3 = io:open("output.txt", "r")
LET line1 = io:readln(f3)
LET line2 = io:readln(f3)
LET eof_flag = io:eof(f3)
LET line3 = io:readln(f3)   // empty if EOF
io:close(f3)

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

## Mathematical Functions

**File:** `examples/math.hi`  
Demonstrates the `math` module – constants, trigonometry, rounding, and the new `min`/`max`/`clamp`.

```hi
IMPORT "math" AS m

PRINT "PI = ", m:PI
PRINT "E = ", m:E

PRINT "sin(PI/2) = ", m:sin(m:PI / 2)
PRINT "sqrt(16) = ", m:sqrt(16)
PRINT "abs(-5) = ", m:abs(-5)

PRINT "ceil(3.2) = ", m:ceil(3.2)
PRINT "floor(3.9) = ", m:floor(3.9)
PRINT "round(3.5) = ", m:round(3.5)

PRINT "torad(180) = ", m:torad(180)
PRINT "todeg(PI) = ", m:todeg(m:PI)

PRINT "min(5, 3) = ", m:min(5, 3)
PRINT "max(5, 3) = ", m:max(5, 3)
PRINT "min([7, 2, 9, 1]) = ", m:min([7, 2, 9, 1])
PRINT "max([7, 2, 9, 1]) = ", m:max([7, 2, 9, 1])

PRINT "clamp(5, 1, 10) = ", m:clamp(5, 1, 10)
PRINT "clamp(0, 1, 10) = ", m:clamp(0, 1, 10)
PRINT "clamp(15, 1, 10) = ", m:clamp(15, 1, 10)

PRINT "log(E) = ", m:log(m:E)
PRINT "exp(1) = ", m:exp(1)
```

---

## Collections Module

**File:** `examples/collections.hi`  
Shows functional list operations: `map`, `filter`, `reduce`, `any`, `all`, `find`, `sort`.

```hi
IMPORT "collections" AS c

FUNC double(x) RET x * 2 END
FUNC is_even(x) RET x % 2 == 0 END
FUNC add(a, b) RET a + b END

LET numbers = [5, 2, 8, 1, 3]

PRINT "Original: ", numbers
PRINT "map(double): ", c:map(double, numbers)
PRINT "filter(is_even): ", c:filter(is_even, numbers)
PRINT "reduce(add, 0): ", c:reduce(add, numbers, 0)
PRINT "sort(): ", c:sort(numbers)
PRINT "any(is_even): ", c:any(is_even, numbers)
PRINT "all(is_even): ", c:all(is_even, numbers)
PRINT "find(is_even): ", c:find(is_even, numbers)
```

---

## JSON Module

**File:** `examples/json.hi`  
Parses and serialises JSON data.

```hi
IMPORT "json" AS json

// Parse a JSON object (note: backslashes for quotes)
LET obj = json:parse("{\"name\":\"Alice\",\"age\":30,\"hobbies\":[\"reading\",\"gaming\"]}")
PRINT "Name: ", obj["name"]
PRINT "Age: ", obj["age"]
PRINT "Hobbies: ", obj["hobbies"]

// Modify and stringify
obj["age"] = 31
LET json_str = json:stringify(obj)
PRINT "Updated JSON: ", json_str

// Parse an array
LET arr = json:parse("[1, \"two\", false, null]")
PRINT "Array: ", arr
```

---

## OS Module

**File:** `examples/os.hi`  
Environment variables, file system operations, processes.

```hi
IMPORT "os" AS os

// Environment
LET user = os:getenv("USER")
PRINT "User: ", user

os:setenv("MY_VAR", "hello")
PRINT "MY_VAR = ", os:getenv("MY_VAR")
os:unsetenv("MY_VAR")

// Current directory
LET cwd = os:cwd()
PRINT "Current dir: ", cwd

// List files
LET files = os:listdir(".")
PRINT "Files: ", files

// Create and remove directory
os:mkdir("temp")
os:rmdir("temp")

// File stats
LET info = os:stat("os.hi")
PRINT "Size: ", info["size"]
PRINT "Is file? ", info["is_file"]

// Existence
PRINT "Exists? ", os:exists("somefile.txt")

// Execute a command (shell dependent)
LET exit_code = os:exec("echo 'Hello from shell'")
PRINT "Exit code: ", exit_code
```

---

## Datetime Module

**File:** `examples/datetime.hi`  
Working with dates, times, durations.

```hi
IMPORT "datetime" AS dt

LET now = dt:now()
PRINT "Local now: ", dt:tostring(now, "%Y-%m-%d %H:%M:%S")

LET utc = dt:utcnow()
PRINT "UTC now: ", dt:tostring(utc, "%Y-%m-%d %H:%M:%S")

LET parsed = dt:fromstring("2026-08-03 15:30:45", "%Y-%m-%d %H:%M:%S")
PRINT "Parsed year: ", dt:year(parsed)

LET dur = dt:duration(3600)   // 1 hour
LET later = dt:add(now, dur)
PRINT "In 1 hour: ", dt:tostring(later, "%H:%M")

LET diff = dt:diff(later, now)
PRINT "Difference: ", diff["hours"], " hours, ", diff["minutes"], " minutes"

PRINT "Timestamp: ", dt:timestamp(now)
```

---

## Random Module

**File:** `examples/random.hi`  
Generates random numbers and shuffles lists.

```hi
IMPORT "random" AS r

LET dice = r:randint(1, 6)
PRINT "Dice roll: ", dice

LET prob = r:randfloat()
PRINT "Random float [0,1): ", prob

LET bytes = r:randbytes(4)
PRINT "Random bytes: ", bytes

LET list = [10, 20, 30, 40, 50]
LET shuffled = r:shuffle(list)
PRINT "Shuffled: ", shuffled

LET pick = r:choice(list)
PRINT "Random choice: ", pick
```

---

## Regex Module

**File:** `examples/regex.hi`  
Regular expression matching, finding, replacing, splitting.  
Note: backslashes in patterns must be escaped: `"\\d+"` for `\d+`.

```hi
IMPORT "regex" AS re

LET text = "Hello 123 world 456!"

// Match
PRINT "Contains digits? ", re:match("\\d+", text)   // TRUE

// Find first
LET first = re:find("\\d+", text)
PRINT "First number: ", first   // "123"

// Find all
LET all = re:find_all("\\d+", text)
PRINT "All numbers: ", all      // ["123", "456"]

// Replace
LET replaced = re:replace("\\d+", text, "X")
PRINT "Replaced: ", replaced    // "Hello X world X!"

// Split
LET parts = re:split("\\s+", "one two  three")
PRINT "Split: ", parts          // ["one", "two", "three"]
```

---

## Path Module

**File:** `examples/path.hi`  
Cross‑platform path manipulation.

```hi
IMPORT "path" AS p

PRINT "join: ", p:join("usr", "local", "bin")
PRINT "basename: ", p:basename("/foo/bar.txt")
PRINT "dirname: ", p:dirname("/foo/bar.txt")
PRINT "dirname (trailing slash): ", p:dirname("foo/bar/")
PRINT "extname: ", p:extname("archive.tar.gz")
PRINT "is_absolute: ", p:is_absolute("/home")
PRINT "normalize: ", p:normalize("a/./b/../c")
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

These examples cover most of the language features and its standard library. Feel free to experiment by modifying them
or writing your own programs.