# Built‑in Functions

Hi provides a rich set of built‑in functions that you can call like any user‑defined function.  
They cover string manipulation, list and dictionary operations, file I/O, mathematics, type conversions, and more.

All built‑in functions are **global** and can be called anywhere. They are **case‑sensitive** – use the names exactly as
shown.

---

## Modules and Namespaces

Hi provides several **built‑in modules** that group related functions and constants. These modules are always available
as global variables:

- `math` – mathematical functions and constants (`PI`, `E`, `sin`, `cos`, etc.)
- `strings` – string‑specific functions (`split`, `join`, `upper`, `lower`, etc.)
- `io` – file I/O functions (`open`, `read`, `write`, `close`, etc.)
- `collections` – functional list operations (`map`, `filter`, `reduce`, etc.)
- `json` – JSON parsing and serialisation.
- `os` – operating system interaction (environment, file system, processes).
- `datetime` – date and time operations.
- `random` – random number generation.
- `regex` – regular expression matching and replacement.
- `path` – cross‑platform path manipulation.

You can access their members using colon notation:

```hi
LET x = math:sin(math:PI / 2)
LET parts = strings:split("a,b,c", ",")
LET file = io:open("data.txt", "r")
LET sorted = collections:sort([3, 1, 2])
LET data = json:parse("{\"name\":\"Alice\"}")
LET cwd = os:cwd()
LET now = datetime:now()
```

Alternatively, you can **import** a module to inline its functions into the global namespace (
see [Import Directives](syntax.md#import-directives)). For example, `IMPORT "math"` allows you to call `sin()` directly
without the `math:` prefix.

---

## Copy‑on‑Write (COW) for Lists

Many list‑modifying operations (`append`, `insert`, `remove`, `reverse`, `shuffle`) use **copy‑on‑write** semantics.
This means:

- If the list has **exactly one reference** (i.e., it is not shared), the operation modifies the list **in place** and
  returns the same list.
- If the list is **shared** (referenced by multiple variables), a **copy** is created, the operation is applied to the
  copy, and the new list is returned. The original list remains unchanged.

This gives you both **performance** (no unnecessary copies) and **safety** (shared data is not mutated unexpectedly).

**Example:**

```hi
LET a = [1, 2, 3]
LET b = a          // a and b share the same list

LET c = append(a, 4)   // a is shared, so a copy is made
PRINT a           // [1, 2, 3]  (unchanged)
PRINT c           // [1, 2, 3, 4]

LET d = append(c, 5)   // c is not shared, so it is modified in place
PRINT c           // [1, 2, 3, 4, 5]  (modified)
PRINT d           // same as c
```

Dictionaries **do not** use COW – they are always mutated in place (they are mutable references).

---

## String Functions

Many string functions are available both as global functions (after importing `strings`) and via the `strings:` module.

### `len(s)`

Returns the length of a string (or list/dict – see generic functions).

- **Arguments:** `s` – string
- **Returns:** integer

```hi
PRINT len("hello")   // 5
```

### `split(s, delimiter)`

Splits a string into a list of substrings using `delimiter`.  
(Module: `strings:split`)

- **Arguments:** `s` – string, `delimiter` – string
- **Returns:** list of strings

```hi
LET parts = strings:split("a,b,c", ",")
PRINT parts   // ["a", "b", "c"]
```

### `join(delimiter, list)`

Joins a list of strings with `delimiter` between each element.  
(Module: `strings:join`)

- **Arguments:** `delimiter` – string, `list` – list of strings
- **Returns:** string
- Throws if `list` contains non‑string elements.

```hi
LET words = ["Hello", "World", "!"]
LET sentence = strings:join(" ", words)
PRINT sentence   // "Hello World !"
```

### `concat(...)`

Concatenates **any number** of strings (or lists) of the same type.  
If arguments are strings, returns a string. If arguments are lists, returns a new list (COW).  
All arguments must be of the same type (all strings or all lists).  
Throws if types are mixed.

- **Arguments:** two or more strings, or two or more lists
- **Returns:** string or list

```hi
PRINT concat("Hello", " ", "World")   // "Hello World"
LET a = [1, 2]
LET b = [3, 4]
LET c = concat(a, b, [5, 6])
PRINT c   // [1, 2, 3, 4, 5, 6]
```

### `replace(s, old, new)`

Replaces all occurrences of `old` with `new` in string `s`.  
(Module: `strings:replace`)

- **Arguments:** `s` – string, `old` – string, `new` – string
- **Returns:** string

```hi
PRINT strings:replace("Hello World", "World", "Hi")   // "Hello Hi"
```

### `substr(s, start, length)`

Extracts a substring from `s` starting at `start` (0‑based) with given `length`.  
(Module: `strings:substr`)

- **Arguments:** `s` – string, `start` – integer, `length` – integer (non‑negative)
- **Returns:** string

```hi
PRINT strings:substr("Hello", 1, 3)   // "ell"
```

### `starts(s, prefix)`

Returns `TRUE` if `s` starts with `prefix`.  
(Module: `strings:starts`)

### `ends(s, suffix)`

Returns `TRUE` if `s` ends with `suffix`.  
(Module: `strings:ends`)

### `upper(s)`

Converts `s` to uppercase.  
(Module: `strings:upper`)

### `lower(s)`

Converts `s` to lowercase.  
(Module: `strings:lower`)

### `trim(s)`

Removes leading and trailing whitespace from `s`.  
(Module: `strings:trim`)

### `reverse(s)`

Reverses the characters in `s` (also works for lists).  
(Module: `strings:reverse` – available globally as `reverse`)

### `indexof(s, substring)`

Returns the first index of `substring` in `s`, or `-1` if not found.  
(Module: `strings:indexof` – available globally as `indexof`)

### `contains(s, substring)`

Returns `TRUE` if `s` contains `substring` (works for lists/dicts too – see generic functions).  
(Module: `strings:contains` – available globally as `contains`)

---

## List Functions

### `len(list)`

Returns the number of elements in the list (or string/dict).

### `append(list, value)`

Adds `value` to the end of `list`. Uses COW.

- **Returns:** the (possibly new) list.

```hi
LET a = [1, 2]
LET b = append(a, 3)
PRINT b   // [1, 2, 3]
```

### `insert(list, index, value)`

Inserts `value` at position `index` (0‑based). Uses COW. Returns the modified list.

- Throws an error if `index` is out of range (0 ≤ index ≤ len).

```hi
LET a = [1, 3]
LET b = insert(a, 1, 2)
PRINT b   // [1, 2, 3]
```

### `remove(list, index)`

Removes the element at `index` and returns the modified list. Uses COW.

- Throws if index out of bounds (0 ≤ index < len).

```hi
LET a = [1, 2, 3]
LET b = remove(a, 1)
PRINT b   // [1, 3]
```

### `slice(list, start, length)`

Returns a new list containing elements from `start` with given `length`.

- Indices are 0‑based; non‑negative. If `start` ≥ len, returns empty list. If `start+length` exceeds len, returns up to
  the end.

```hi
LET a = [10, 20, 30, 40]
LET b = slice(a, 1, 2)
PRINT b   // [20, 30]
```

### `reverse(list)`

Reverses the list (COW) and returns it.

```hi
LET a = [1, 2, 3]
LET b = reverse(a)
PRINT b   // [3, 2, 1]
```

### `indexof(list, value)`

Returns the first index where `value` appears, or `-1`.

```hi
LET a = [10, 20, 10]
PRINT indexof(a, 10)   // 0
PRINT indexof(a, 99)   // -1
```

### `contains(list, value)`

Returns `TRUE` if `value` is in the list.

---

## Dictionary Functions

### `len(dict)`

Returns the number of key‑value pairs.

### `keys(dict)`

Returns a list of all keys in the dictionary.

```hi
LET d = {"a" = 1, "b" = 2}
LET k = keys(d)
PRINT k   // ["a", "b"] (order may vary)
```

### `values(dict)`

Returns a list of all values.

### `contains(dict, key)`

Returns `TRUE` if `key` exists (key must be hashable – int, float, string, bool).

### `remove(dict, key)`

Removes the entry with `key` from the dictionary. Mutates in place.

- Returns `nil`. Throws if key not found.

```hi
LET d = {"x" = 10, "y" = 20}
remove(d, "x")
PRINT d   // {"y" = 20}
```

### `put(dict, key, value)`

Inserts or updates the key‑value pair. Mutates in place.

- Returns `nil`.

### `get(dict, key)`

Returns the value for `key`, or `nil` if not found.

```hi
LET d = {"name" = "Alice"}
PRINT get(d, "name")   // "Alice"
PRINT get(d, "age")    // nil
```

---

## File I/O Functions

All file functions use **file handles** returned by `open()`. They are available in the `io` module.

### `io:open(path, mode)`

Opens a file and returns a file handle.

- `path` – string, `mode` – string: `"r"` (read), `"w"` (write, overwrites), `"a"` (append).
- Returns a file handle (type `File`).

```hi
LET f = io:open("data.txt", "r")
```

### `io:close(file)`

Closes the file handle (flushes writes if needed). Returns `nil`.

### `io:read(file)`

Reads the entire remaining content of the file as a string. Returns string.

```hi
LET content = io:read(f)
```

### `io:readln(file)`

Reads one line (including newline if present). Returns string. At EOF, returns an empty string and sets EOF flag.

### `io:write(file, value)`

Writes the string representation of `value` to the file (without newline). Returns `nil`.

### `io:writeln(file, value)`

Writes `value` followed by a newline. Returns `nil`.

### `io:eof(file)`

Returns `TRUE` if the end of the file has been reached (or file is not open for reading). Returns boolean.

---

## Mathematical Functions

All math functions are in the `math` module and expect a numeric argument (integer or float) and return a float, except
`rand` which has been moved to the `random` module.

### Constants

- `math:PI` – π (3.14159…)
- `math:E` – Euler’s number (2.71828…)

### Trigonometric

- `math:sin(x)`, `math:cos(x)`, `math:tan(x)`, `math:asin(x)`, `math:acos(x)`, `math:atan(x)`

Input in radians. `asin` and `acos` require `-1 ≤ x ≤ 1`.

### `math:sqrt(x)`

Square root; `x ≥ 0`.

### `math:torad(degrees)`, `math:todeg(radians)`

Convert between degrees and radians.

### `math:exp(x)` – eˣ, `math:log(x)` – natural logarithm (x > 0), `math:log2(x)`, `math:log10(x)`

### `math:ceil(x)`, `math:floor(x)`, `math:round(x)` – rounding to nearest integer (as float).

### `math:abs(x)` – absolute value.

### `math:min(a, b)` / `math:min(list)`

Returns the minimum value.

- If two numbers are given, returns the smaller.
- If a list of numbers is given, returns the smallest element.  
  The result type is integer if all numbers are integers, otherwise float.

```hi
PRINT math:min(5, 3)          // 3
PRINT math:min([7, 2, 9, 1])  // 1
```

### `math:max(a, b)` / `math:max(list)`

Returns the maximum value (symmetric to `min`).

```hi
PRINT math:max(5, 3)          // 5
PRINT math:max([7, 2, 9, 1])  // 9
```

### `math:clamp(val, min, max)`

Constrains `val` to the inclusive range `[min, max]`. If `min > max`, they are swapped internally.  
Returns `val` if within range, otherwise the nearer bound.

```hi
PRINT math:clamp(5, 1, 10)    // 5
PRINT math:clamp(0, 1, 10)    // 1
PRINT math:clamp(15, 1, 10)   // 10
```

---

## Collections Module

The `collections` module provides functional programming helpers for lists.

### `collections:map(func, list)`

Applies `func` to each element and returns a new list.

### `collections:filter(pred, list)`

Returns a new list containing elements for which `pred` returns `TRUE`.

### `collections:reduce(func, list, initial)`

Folds the list using `func(accumulator, element)`, returning the final accumulator.

### `collections:any(pred, list)`

Returns `TRUE` if any element satisfies `pred`.

### `collections:all(pred, list)`

Returns `TRUE` if all elements satisfy `pred`.

### `collections:find(pred, list)`

Returns the first element that satisfies `pred`, or `nil` if none.

### `collections:sort(list)`

Returns a sorted copy of the list (ascending). Elements must be comparable (numbers or strings of same type).

**Examples:**

```hi
IMPORT "collections" AS c
FUNC double(x) RET x * 2 END
FUNC is_even(x) RET x % 2 == 0 END
FUNC add(a, b) RET a + b END

LET numbers = [5, 2, 8, 1, 3]
PRINT c:map(double, numbers)        // [10, 4, 16, 2, 6]
PRINT c:filter(is_even, numbers)    // [2, 8]
PRINT c:reduce(add, numbers, 0)     // 19
PRINT c:sort(numbers)               // [1, 2, 3, 5, 8]
PRINT c:any(is_even, numbers)       // TRUE
PRINT c:all(is_even, numbers)       // FALSE
PRINT c:find(is_even, numbers)      // 2
```

---

## JSON Module

The `json` module provides JSON parsing and serialization.

### `json:parse(string)`

Parses a JSON string and returns a Hi value (numbers → int/float, strings → string, booleans → bool, null → nil,
arrays → list, objects → dict).

### `json:stringify(value)`

Converts a Hi value to a JSON string. Supports int, float, string, bool, nil, list, dict (keys must be strings).
Functions, files, and modules cannot be serialized.

**Examples:**

```hi
IMPORT "json" AS json
LET data = json:parse("{\"name\":\"Alice\",\"age\":30}")
PRINT data["name"]        // "Alice"
data["age"] = 31
LET json_str = json:stringify(data)
PRINT json_str            // {"age":31,"name":"Alice"}
```

---

## OS Module

The `os` module provides operating system interactions.

### Environment

- `os:getenv(key)` – returns the value of the environment variable or `nil`.
- `os:setenv(key, value)` – sets a variable (returns `nil`).
- `os:unsetenv(key)` – removes a variable (returns `nil`).

### Process

- `os:exec(command)` – executes a shell command, returns exit code (int) or `nil` on failure.
- `os:exit(code)` – terminates the interpreter with the given exit code.

### File System

- `os:cwd()` – returns the current working directory as a string.
- `os:chdir(path)` – changes the current directory (returns `nil`).
- `os:listdir(path)` – returns a list of filenames in the directory.
- `os:mkdir(path)` – creates a directory (returns `nil`).
- `os:rmdir(path)` – removes an empty directory (returns `nil`).
- `os:remove(path)` – deletes a file (returns `nil`).
- `os:rename(old, new)` – renames a file or empty directory (returns `nil`).
- `os:move(src, dst)` – moves a file or empty directory (cross‑device copy‑delete for files, returns `nil`).
- `os:copy(src, dst)` – copies a file or empty directory (returns `nil`).
- `os:exists(path)` – returns `TRUE` if the file or directory exists.
- `os:stat(path)` – returns a dictionary with metadata: `size` (int), `modified`, `accessed`, `created` (Unix
  timestamps), `is_dir`, `is_file`.

**Examples:**

```hi
IMPORT "os" AS os
LET user = os:getenv("USER")
PRINT "Hello, ", user

LET files = os:listdir(".")
PRINT files

os:mkdir("temp")
os:copy("data.txt", "backup.txt")
os:rename("backup.txt", "old_backup.txt")
os:remove("old_backup.txt")
os:rmdir("temp")

LET info = os:stat("os.rs")
PRINT "Size: ", info["size"]
PRINT "Is file? ", info["is_file"]

PRINT "Exists? ", os:exists("somefile.txt")
os:exit(0)
```

---

## Datetime Module

The `datetime` module provides date and time operations. All dates are represented as dictionaries with fields: `year`,
`month`, `day`, `hour`, `minute`, `second`, `millisecond`, `timestamp` (Unix ms). Durations are dictionaries: `days`,
`hours`, `minutes`, `seconds`, `milliseconds`.

### `datetime:now()` – returns the current local date/time.

### `datetime:utcnow()` – returns the current UTC date/time.

### `datetime:fromstring(str, format)` – parses a string according to the format and returns a datetime dictionary.

### `datetime:tostring(dt, format)` – formats a datetime dictionary into a string.

### `datetime:add(dt, duration)` – adds a duration to a datetime, returns new datetime.

### `datetime:diff(dt1, dt2)` – returns `dt1 - dt2` as a duration dictionary.

### `datetime:year(dt)`, `month(dt)`, `day(dt)`, `hour(dt)`, `minute(dt)`, `second(dt)`,

`millisecond(dt)` – get corresponding component.

### `datetime:timestamp(dt)` – returns Unix timestamp in milliseconds.

### `datetime:duration(seconds)` – creates a duration dictionary from a number of seconds (int or float).

**Examples:**

```hi
IMPORT "datetime" AS dt
LET now = dt:now()
PRINT dt:tostring(now, "%Y-%m-%d %H:%M:%S")
LET utc = dt:utcnow()

LET dt1 = dt:fromstring("2026-08-03 15:30:45", "%Y-%m-%d %H:%M:%S")
LET dur = dt:duration(3600)   // 1 hour
LET later = dt:add(dt1, dur)
LET diff = dt:diff(later, dt1)
PRINT diff["hours"]           // 1
PRINT dt:year(now)
```

---

## Random Module

The `random` module provides random number generation.

### `random:randint(start, end)` – returns a random integer in `[start, end]` (inclusive).

### `random:randfloat()` – returns a random float in `[0.0, 1.0)`.

### `random:randbytes(n)` – returns a list of `n` random bytes (0‑255).

### `random:shuffle(list)` – returns a new list with elements shuffled (COW).

### `random:choice(list)` – returns a random element from the list, or `nil` if empty.

**Examples:**

```hi
IMPORT "random" AS r
LET dice = r:randint(1, 6)
LET prob = r:randfloat()
LET bytes = r:randbytes(4)       // e.g., [23, 145, 8, 200]
LET shuffled = r:shuffle([1, 2, 3, 4])
LET pick = r:choice(["apple", "banana", "cherry"])
```

---

## Regex Module

The `regex` module provides regular expression functions. Patterns follow the Rust regex syntax.

**Important:** In Hi strings, backslashes must be escaped, so a pattern like `\d+` must be written as `"\\d+"`.

### `regex:match(pattern, string)` – returns `TRUE` if the pattern matches anywhere.

### `regex:find(pattern, string)` – returns the first match as a string, or `nil`.

### `regex:find_all(pattern, string)` – returns a list of all non‑overlapping matches.

### `regex:replace(pattern, string, replacement)` – replaces all matches with `replacement`. You can use `$1`,

`$2`, etc. for capture groups.

### `regex:split(pattern, string)` – splits the string by the pattern and returns a list of substrings.

**Examples:**

```hi
IMPORT "regex" AS re
PRINT re:match("\\d+", "abc123")                // TRUE
PRINT re:find("\\w+", "Hello, world")           // "Hello"
PRINT re:find_all("[aeiou]", "hello")           // ["e", "o"]
PRINT re:replace("\\s+", "a b c", "-")          // "a-b-c"
PRINT re:split(", +", "one, two, three")        // ["one", "two", "three"]
```

---

## Path Module

The `path` module provides cross‑platform path manipulation.

### `path:join(parts...)` – concatenates path components using the platform separator. Returns a string.

### `path:basename(path)` – returns the last component (file or directory name) as string, or

`nil` if the path ends with a separator.

### `path:dirname(path)` – returns the parent directory, or

`nil` if none. Removes trailing separators if present, then removes the last component.

### `path:extname(path)` – returns the extension including the dot (e.g., `".txt"`), or empty string if none.

### `path:is_absolute(path)` – returns `TRUE` if the path is absolute.

### `path:normalize(path)` – resolves `.` and

`..` components and removes redundant separators without touching the filesystem.

**Examples:**

```hi
IMPORT "path" AS p
PRINT p:join("usr", "local", "bin")        // "usr/local/bin" (or backslashes)
PRINT p:basename("/foo/bar.txt")           // "bar.txt"
PRINT p:dirname("/foo/bar.txt")            // "/foo"
PRINT p:dirname("foo/bar/")                // "foo/bar" (trailing slash removed)
PRINT p:extname("archive.tar.gz")          // ".gz"
PRINT p:is_absolute("/home")               // TRUE
PRINT p:normalize("a/./b/../c")            // "a/c"
```

---

## Type Conversion Functions

### `tostring(value)`

Converts any value to its string representation.

### `toint(value)`

Converts a number (int/float) or a string to an integer. For strings, parses as integer; for floats, truncates toward
zero.

### `tofloat(value)`

Converts a number or string to a float.

---

## Special Functions

### `typeof(value)`

Returns a string with the type name of `value`. Possible names: `"integer"`, `"float"`, `"string"`, `"boolean"`,
`"list"`, `"dict"`, `"file"`, `"function"`, `"nil"`, `"module"`.

```hi
LET t = typeof(42)
PRINT t   // "integer"
```

### `call(func, ...args)`

Calls a function value (stored in a variable) with the given arguments. Enables higher‑order programming.

```hi
FUNC double(x) RET x * 2 END
LET f = double
LET result = call(f, 5)
PRINT result   // 10
```

### `hello()`

Prints `"Hello, World!"` to the console. For backwards compatibility; you can just use `PRINT "Hello, World!"`.

---

## Generic Functions

Some functions work with multiple types:

- `len` – works on strings, lists, dicts.
- `contains` – works on strings (substring), lists (element), dicts (key).
- `concat` – works on any number of strings or any number of lists (not mixed).
- `reverse` – works on strings and lists.

For `contains`, the second argument must match the type: for strings, a string; for lists, any value; for dicts, a
hashable key.

---

## Error Handling

All built‑in functions perform argument count and type checking. If you pass the wrong number of arguments or incorrect
types, a runtime error is raised with a descriptive message.

---

## Complete Example

```hi
// All modules in action
IMPORT "math" AS m
IMPORT "strings" AS s
IMPORT "json" AS j
IMPORT "os"
IMPORT "datetime" AS dt
IMPORT "random" AS r
IMPORT "regex" AS re
IMPORT "path" AS p
IMPORT "collections" AS c

LET now = dt:now()
PRINT dt:tostring(now, "%Y-%m-%d %H:%M:%S")

LET data = j:parse("{\"value\":42}")
PRINT data["value"]

FUNC double(x) RET x * 2 END
LET doubled = c:map(double, [1, 2, 3])
PRINT doubled

LET pattern = "\\d+"
PRINT re:match(pattern, "123abc")       // TRUE

LET abs_path = p:join(os:cwd(), "file.txt")
PRINT abs_path

PRINT m:min(10, 20)
PRINT m:clamp(5, 1, 10)
PRINT r:randint(1, 100)
```

---

## Next Steps

Check the [Examples](examples.md) chapter for more ready‑to‑run scripts that use these built‑ins.