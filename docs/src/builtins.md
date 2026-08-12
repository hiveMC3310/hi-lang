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

## Generic Functions

Some functions work with multiple types:

- `len` – works on strings, lists, dicts.
- `contains` – works on strings (substring), lists (element), dicts (key).
- `concat` – works on any number of strings or any number of lists (not mixed).
- `reverse` – works on strings and lists.

For `contains`, the second argument must match the type: for strings, a string; for lists, any value; for dicts, a
hashable key.

---

## String Functions

Many string functions are available both as global functions (after importing `strings`) and via the `strings:` module.

### `len(s)` (global)

Returns the length of a string (or list/dict – see generic functions).

**Arguments:**

- `s` – string

**Returns:** integer

**Example:**

```hi
PRINT len("hello")   // 5
```

---

### `split(s, delimiter)` (module: `strings`)

Splits a string into a list of substrings using `delimiter`.

**Arguments:**

- `s` – string
- `delimiter` – string

**Returns:** list of strings

**Example:**

```hi
LET parts = strings:split("a,b,c", ",")
PRINT parts   // ["a", "b", "c"]
```

---

### `join(delimiter, list)` (module: `strings`)

Joins a list of strings with `delimiter` between each element.

**Arguments:**

- `delimiter` – string
- `list` – list of strings

**Returns:** string  
Throws if `list` contains non‑string elements.

**Example:**

```hi
LET words = ["Hello", "World", "!"]
LET sentence = strings:join(" ", words)
PRINT sentence   // "Hello World !"
```

---

### `concat(...)` (global)

Concatenates **any number** of strings (or lists) of the same type. If arguments are strings, returns a string. If
arguments are lists, returns a new list (COW). All arguments must be of the same type (all strings or all lists). Throws
if types are mixed.

**Arguments:** two or more strings, or two or more lists  
**Returns:** string or list

**Example:**

```hi
PRINT concat("Hello", " ", "World")   // "Hello World"
LET a = [1, 2]
LET b = [3, 4]
LET c = concat(a, b, [5, 6])
PRINT c   // [1, 2, 3, 4, 5, 6]
```

---

### `replace(s, old, new)` (module: `strings`)

Replaces all occurrences of `old` with `new` in string `s`.

**Arguments:**

- `s` – string
- `old` – string
- `new` – string

**Returns:** string

**Example:**

```hi
PRINT strings:replace("Hello World", "World", "Hi")   // "Hello Hi"
```

---

### `substr(s, start, length)` (module: `strings`)

Extracts a substring from `s` starting at `start` (0‑based) with given `length`.

**Arguments:**

- `s` – string
- `start` – integer (non‑negative)
- `length` – integer (non‑negative)

**Returns:** string

**Example:**

```hi
PRINT strings:substr("Hello", 1, 3)   // "ell"
```

---

### `starts(s, prefix)` (module: `strings`)

Returns `TRUE` if `s` starts with `prefix`.

**Arguments:**

- `s` – string
- `prefix` – string

**Returns:** boolean

**Example:**

```hi
PRINT strings:starts("Hello", "He")   // TRUE
```

---

### `ends(s, suffix)` (module: `strings`)

Returns `TRUE` if `s` ends with `suffix`.

**Arguments:**

- `s` – string
- `suffix` – string

**Returns:** boolean

**Example:**

```hi
PRINT strings:ends("Hello", "lo")   // TRUE
```

---

### `upper(s)` (module: `strings`)

Converts `s` to uppercase.

**Arguments:**

- `s` – string

**Returns:** string

**Example:**

```hi
PRINT strings:upper("hello")   // "HELLO"
```

---

### `lower(s)` (module: `strings`)

Converts `s` to lowercase.

**Arguments:**

- `s` – string

**Returns:** string

**Example:**

```hi
PRINT strings:lower("HELLO")   // "hello"
```

---

### `trim(s)` (module: `strings`)

Removes leading and trailing whitespace from `s`.

**Arguments:**

- `s` – string

**Returns:** string

**Example:**

```hi
PRINT strings:trim("  hello  ")   // "hello"
```

---

### `reverse(s)` (global)

Reverses the characters in `s` (also works for lists).

**Arguments:**

- `s` – string or list

**Returns:** string or list

**Example:**

```hi
PRINT reverse("hello")   // "olleh"
```

---

### `indexof(container, item)` (global)

Returns the first index of `item` in a string (substring) or list (element), or `-1` if not found.

**Arguments:**

- `container` – string or list
- `item` – string (if container is string) or any value (if container is list)

**Returns:** integer

**Example:**

```hi
PRINT indexof("hello", "l")     // 2
PRINT indexof([1, 2, 3], 2)     // 1
PRINT indexof("hello", "x")     // -1
```

---

### `contains(container, item)` (global)

Returns `TRUE` if the container contains the item. Works for strings (substring), lists (element), dicts (key).

**Arguments:**

- `container` – string, list, or dict
- `item` – depends on container type (string, any value, or hashable key)

**Returns:** boolean

**Example:**

```hi
PRINT contains("hello", "ll")        // TRUE
PRINT contains([1, 2, 3], 2)         // TRUE
PRINT contains({"a" = 1}, "a")       // TRUE
```

---

## List Functions

### `len(list)` (global)

Returns the number of elements in the list (works with strings and dicts too).

**Arguments:**

- `list` – list

**Returns:** integer

**Example:**

```hi
PRINT len([1, 2, 3])   // 3
```

---

### `append(list, value)` (global)

Adds `value` to the end of `list`. Uses COW.

**Arguments:**

- `list` – list
- `value` – any value

**Returns:** list (possibly new)

**Example:**

```hi
LET a = [1, 2]
LET b = append(a, 3)
PRINT b   // [1, 2, 3]
```

---

### `insert(list, index, value)` (global)

Inserts `value` at position `index` (0‑based). Uses COW. Throws if `index` is out of range (0 ≤ index ≤ len).

**Arguments:**

- `list` – list
- `index` – integer
- `value` – any value

**Returns:** list (possibly new)

**Example:**

```hi
LET a = [1, 3]
LET b = insert(a, 1, 2)
PRINT b   // [1, 2, 3]
```

---

### `remove(list_or_dict, index_or_key)` (global)

Removes an element from a list by index or from a dictionary by key. For lists, uses COW; for dicts, mutates in place.

**Arguments:**

- `list_or_dict` – list or dict
- `index_or_key` – integer (for list) or hashable key (for dict)

**Returns:** list (new) for lists, `nil` for dicts  
Throws if index/key out of range or not found.

**Example:**

```hi
LET a = [1, 2, 3]
LET b = remove(a, 1)
PRINT b   // [1, 3]

LET d = {"x" = 10}
remove(d, "x")
PRINT d   // {}
```

---

### `slice(list, start, length)` (global)

Returns a new list containing elements from `start` with given `length`. Indices are 0‑based and non‑negative. If
`start` ≥ len, returns empty list. If `start+length` exceeds len, returns up to the end.

**Arguments:**

- `list` – list
- `start` – integer
- `length` – integer

**Returns:** list

**Example:**

```hi
LET a = [10, 20, 30, 40]
LET b = slice(a, 1, 2)
PRINT b   // [20, 30]
```

---

### `reverse(list)` (global)

Reverses the list (COW) and returns it. Also works on strings (see String Functions).

**Arguments:**

- `list` – list

**Returns:** list

**Example:**

```hi
LET a = [1, 2, 3]
LET b = reverse(a)
PRINT b   // [3, 2, 1]
```

---

### `indexof(list, value)` (global)

Returns the first index where `value` appears, or `-1`. Also works on strings (see String Functions).

**Arguments:**

- `list` – list
- `value` – any value

**Returns:** integer

**Example:**

```hi
LET a = [10, 20, 10]
PRINT indexof(a, 10)   // 0
PRINT indexof(a, 99)   // -1
```

---

### `contains(list, value)` (global)

Returns `TRUE` if `value` is in the list. Also works on strings and dicts (see String Functions).

**Arguments:**

- `list` – list
- `value` – any value

**Returns:** boolean

**Example:**

```hi
LET a = [1, 2, 3]
PRINT contains(a, 2)   // TRUE
```

---

## Dictionary Functions

### `len(dict)` (global)

Returns the number of key‑value pairs.

**Arguments:**

- `dict` – dict

**Returns:** integer

**Example:**

```hi
LET d = {"a" = 1, "b" = 2}
PRINT len(d)   // 2
```

---

### `keys(dict)` (global)

Returns a list of all keys in the dictionary (order not guaranteed).

**Arguments:**

- `dict` – dict

**Returns:** list

**Example:**

```hi
LET d = {"a" = 1, "b" = 2}
LET k = keys(d)
PRINT k   // ["a", "b"] (order may vary)
```

---

### `values(dict)` (global)

Returns a list of all values.

**Arguments:**

- `dict` – dict

**Returns:** list

**Example:**

```hi
LET d = {"a" = 1, "b" = 2}
LET v = values(d)
PRINT v   // [1, 2]
```

---

### `contains(dict, key)` (global)

Returns `TRUE` if `key` exists (key must be hashable – int, float, string, bool).

**Arguments:**

- `dict` – dict
- `key` – hashable value

**Returns:** boolean

**Example:**

```hi
LET d = {"a" = 1}
PRINT contains(d, "a")   // TRUE
```

---

### `remove(dict, key)` (global)

Removes the entry with `key` from the dictionary. Mutates in place. Throws if key not found.

**Arguments:**

- `dict` – dict
- `key` – hashable value

**Returns:** `nil`

**Example:**

```hi
LET d = {"x" = 10, "y" = 20}
remove(d, "x")
PRINT d   // {"y" = 20}
```

---

### `put(dict, key, value)` (global)

Inserts or updates the key‑value pair. Mutates in place.

**Arguments:**

- `dict` – dict
- `key` – hashable value
- `value` – any value

**Returns:** `nil`

**Example:**

```hi
LET d = {"a" = 1}
put(d, "b", 2)
PRINT d   // {"a"=1, "b"=2}
```

---

### `get(dict, key)` (global)

Returns the value for `key`, or `nil` if not found.

**Arguments:**

- `dict` – dict
- `key` – hashable value

**Returns:** value or `nil`

**Example:**

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

**Arguments:**

- `path` – string
- `mode` – string: `"r"` (read), `"w"` (write, overwrites), `"a"` (append)

**Returns:** file handle (type `File`)

**Example:**

```hi
LET f = io:open("data.txt", "r")
```

---

### `io:close(file)`

Closes the file handle (flushes writes if needed).

**Arguments:**

- `file` – file handle

**Returns:** `nil`

**Example:**

```hi
io:close(f)
```

---

### `io:read(file)`

Reads the entire remaining content of the file as a string.

**Arguments:**

- `file` – file handle (must be open for reading)

**Returns:** string

**Example:**

```hi
LET content = io:read(f)
```

---

### `io:readln(file)`

Reads one line (including newline if present). At EOF, returns an empty string and sets EOF flag.

**Arguments:**

- `file` – file handle (must be open for reading)

**Returns:** string

**Example:**

```hi
LET line = io:readln(f)
```

---

### `io:write(file, value)`

Writes the string representation of `value` to the file (without newline).

**Arguments:**

- `file` – file handle (must be open for writing)
- `value` – any value

**Returns:** `nil`

**Example:**

```hi
io:write(f, "Hello")
```

---

### `io:writeln(file, value)`

Writes `value` followed by a newline.

**Arguments:**

- `file` – file handle (must be open for writing)
- `value` – any value

**Returns:** `nil`

**Example:**

```hi
io:writeln(f, "Hello")
```

---

### `io:eof(file)`

Returns `TRUE` if the end of the file has been reached (or file is not open for reading).

**Arguments:**

- `file` – file handle

**Returns:** boolean

**Example:**

```hi
IF io:eof(f) THEN PRINT "End of file" END
```

---

## Mathematical Functions

All math functions are in the `math` module and expect a numeric argument (integer or float) and return a float, except
where noted.

### Constants

- `math:PI` – π (3.14159…)
- `math:E` – Euler’s number (2.71828…)

---

### `math:sin(x)`

Returns the sine of `x` (in radians).

**Arguments:**

- `x` – number (int or float)

**Returns:** float

**Example:**

```hi
PRINT math:sin(math:PI / 2)   // 1.0
```

---

### `math:cos(x)`

Returns the cosine of `x` (in radians).

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:cos(math:PI)   // -1.0
```

---

### `math:tan(x)`

Returns the tangent of `x` (in radians).

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:tan(math:PI / 4)   // 1.0
```

---

### `math:asin(x)`

Returns the arc sine of `x` (in radians). Requires `-1 ≤ x ≤ 1`.

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:asin(1.0)   // π/2
```

---

### `math:acos(x)`

Returns the arc cosine of `x` (in radians). Requires `-1 ≤ x ≤ 1`.

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:acos(0.0)   // π/2
```

---

### `math:atan(x)`

Returns the arc tangent of `x` (in radians).

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:atan(1.0)   // π/4
```

---

### `math:sqrt(x)`

Returns the square root of `x`. Requires `x ≥ 0`.

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:sqrt(9.0)   // 3.0
```

---

### `math:torad(degrees)`

Converts degrees to radians.

**Arguments:**

- `degrees` – number

**Returns:** float

**Example:**

```hi
PRINT math:torad(180)   // π
```

---

### `math:todeg(radians)`

Converts radians to degrees.

**Arguments:**

- `radians` – number

**Returns:** float

**Example:**

```hi
PRINT math:todeg(math:PI)   // 180.0
```

---

### `math:exp(x)`

Returns eˣ.

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:exp(1)   // 2.71828
```

---

### `math:log(x)`

Returns the natural logarithm of `x`. Requires `x > 0`.

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:log(2.71828)   // ~1.0
```

---

### `math:log2(x)`

Returns the base‑2 logarithm of `x`. Requires `x > 0`.

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:log2(8)   // 3.0
```

---

### `math:log10(x)`

Returns the base‑10 logarithm of `x`. Requires `x > 0`.

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:log10(100)   // 2.0
```

---

### `math:ceil(x)`

Returns the smallest integer ≥ `x` (as float).

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:ceil(3.2)   // 4.0
```

---

### `math:floor(x)`

Returns the largest integer ≤ `x` (as float).

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:floor(3.8)   // 3.0
```

---

### `math:round(x)`

Returns the nearest integer to `x` (rounding half away from zero) as float.

**Arguments:**

- `x` – number

**Returns:** float

**Example:**

```hi
PRINT math:round(3.5)   // 4.0
```

---

### `math:abs(x)`

Returns the absolute value of `x`.

**Arguments:**

- `x` – number

**Returns:** number (same type as input: int if input int, float if input float)

**Example:**

```hi
PRINT math:abs(-5)   // 5
```

---

### `math:min(a, b)` or `math:min(list)`

Returns the minimum of two numbers or of a list of numbers. Returns Int if all inputs are integers, otherwise Float.

**Arguments:**

- Two numbers, or a list of numbers

**Returns:** number (int or float)

**Example:**

```hi
PRINT math:min(5, 3)          // 3
PRINT math:min([7, 2, 9, 1])  // 1
```

---

### `math:max(a, b)` or `math:max(list)`

Returns the maximum of two numbers or of a list of numbers. Returns Int if all inputs are integers, otherwise Float.

**Arguments:**

- Two numbers, or a list of numbers

**Returns:** number (int or float)

**Example:**

```hi
PRINT math:max(5, 3)          // 5
PRINT math:max([7, 2, 9, 1])  // 9
```

---

### `math:clamp(val, min, max)`

Constrains `val` to the inclusive range `[min, max]`. If `min > max`, they are swapped internally. Returns `val` if
within range, otherwise the nearer bound.

**Arguments:**

- `val` – number
- `min` – number
- `max` – number

**Returns:** number (int or float depending on inputs)

**Example:**

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

**Arguments:**

- `func` – function that takes one argument and returns a value
- `list` – list

**Returns:** list

**Example:**

```hi
IMPORT "collections" AS c
FUNC double(x) RET x * 2 END
LET numbers = [5, 2, 8, 1, 3]
PRINT c:map(double, numbers)   // [10, 4, 16, 2, 6]
```

---

### `collections:filter(pred, list)`

Returns a new list containing elements for which `pred` returns `TRUE`.

**Arguments:**

- `pred` – function that takes one argument and returns boolean
- `list` – list

**Returns:** list

**Example:**

```hi
FUNC is_even(x) RET x % 2 == 0 END
PRINT c:filter(is_even, numbers)   // [2, 8]
```

---

### `collections:reduce(func, list, initial)`

Folds the list using `func(accumulator, element)`, returning the final accumulator.

**Arguments:**

- `func` – binary function: (accumulator, element) -> new accumulator
- `list` – list
- `initial` – initial accumulator value

**Returns:** value (type depends on `func`)

**Example:**

```hi
FUNC add(a, b) RET a + b END
PRINT c:reduce(add, numbers, 0)   // 19
```

---

### `collections:any(pred, list)`

Returns `TRUE` if any element satisfies `pred`.

**Arguments:**

- `pred` – function taking one argument returning boolean
- `list` – list

**Returns:** boolean

**Example:**

```hi
PRINT c:any(is_even, numbers)   // TRUE
```

---

### `collections:all(pred, list)`

Returns `TRUE` if all elements satisfy `pred`.

**Arguments:**

- `pred` – function taking one argument returning boolean
- `list` – list

**Returns:** boolean

**Example:**

```hi
PRINT c:all(is_even, numbers)   // FALSE
```

---

### `collections:find(pred, list)`

Returns the first element that satisfies `pred`, or `nil` if none.

**Arguments:**

- `pred` – function taking one argument returning boolean
- `list` – list

**Returns:** value or `nil`

**Example:**

```hi
PRINT c:find(is_even, numbers)   // 2
```

---

### `collections:sort(list)`

Returns a sorted copy of the list (ascending). Elements must be comparable (numbers or strings of same type).

**Arguments:**

- `list` – list

**Returns:** list

**Example:**

```hi
PRINT c:sort(numbers)   // [1, 2, 3, 5, 8]
```

---

## JSON Module

### `json:parse(string)`

Parses a JSON string and returns a Hi value (numbers → int/float, strings → string, booleans → bool, null → nil,
arrays → list, objects → dict).

**Arguments:**

- `string` – valid JSON string

**Returns:** Hi value

**Example:**

```hi
IMPORT "json" AS json
LET data = json:parse("{\"name\":\"Alice\",\"age\":30}")
PRINT data["name"]   // "Alice"
```

---

### `json:stringify(value)`

Converts a Hi value to a JSON string. Supports int, float, string, bool, nil, list, dict (keys must be strings).
Functions, files, and modules cannot be serialized.

**Arguments:**

- `value` – Hi value

**Returns:** string

**Example:**

```hi
LET data = {"name" = "Alice", "age" = 30}
LET json_str = json:stringify(data)
PRINT json_str   // {"age":30,"name":"Alice"}
```

---

## OS Module

### Environment

#### `os:getenv(key)`

Returns the value of the environment variable or `nil`.

**Arguments:**

- `key` – string

**Returns:** string or `nil`

**Example:**

```hi
IMPORT "os" AS os
LET user = os:getenv("USER")
PRINT user
```

---

#### `os:setenv(key, value)`

Sets an environment variable.

**Arguments:**

- `key` – string
- `value` – string

**Returns:** `nil`

**Example:**

```hi
os:setenv("MY_VAR", "hello")
```

---

#### `os:unsetenv(key)`

Removes an environment variable.

**Arguments:**

- `key` – string

**Returns:** `nil`

**Example:**

```hi
os:unsetenv("MY_VAR")
```

---

### Process

#### `os:exec(command)`

Executes a shell command and returns the exit code.

**Arguments:**

- `command` – string

**Returns:** integer (exit code), 0 for success

**Example:**

```hi
LET code = os:exec("ls -la")
PRINT code
```

---

#### `os:exit(code)`

Terminates the interpreter with the given exit code.

**Arguments:**

- `code` – integer

**Returns:** never (process exits)

**Example:**

```hi
os:exit(0)
```

---

### File System

#### `os:cwd()`

Returns the current working directory as a string.

**Arguments:** none  
**Returns:** string

**Example:**

```hi
PRINT os:cwd()
```

---

#### `os:chdir(path)`

Changes the current working directory to `path`.

**Arguments:**

- `path` – string

**Returns:** `nil`

**Example:**

```hi
os:chdir("/tmp")
```

---

#### `os:listdir(path)`

Returns a list of filenames in the directory.

**Arguments:**

- `path` – string

**Returns:** list of strings

**Example:**

```hi
LET files = os:listdir(".")
PRINT files
```

---

#### `os:mkdir(path)`

Creates a new directory.

**Arguments:**

- `path` – string

**Returns:** `nil`

**Example:**

```hi
os:mkdir("newdir")
```

---

#### `os:rmdir(path)`

Removes an empty directory.

**Arguments:**

- `path` – string

**Returns:** `nil`

**Example:**

```hi
os:rmdir("newdir")
```

---

#### `os:remove(path)`

Deletes a file.

**Arguments:**

- `path` – string

**Returns:** `nil`

**Example:**

```hi
os:remove("file.txt")
```

---

#### `os:rename(old, new)`

Renames a file or empty directory.

**Arguments:**

- `old` – string (old path)
- `new` – string (new path)

**Returns:** `nil`

**Example:**

```hi
os:rename("old.txt", "new.txt")
```

---

#### `os:move(src, dst)`

Moves a file or empty directory (cross‑device copy‑delete for files, directories require rename).

**Arguments:**

- `src` – string (source)
- `dst` – string (destination)

**Returns:** `nil`

**Example:**

```hi
os:move("file.txt", "/tmp/file.txt")
```

---

#### `os:copy(src, dst)`

Copies a file or an empty directory (non‑empty directories are not supported).

**Arguments:**

- `src` – string (source)
- `dst` – string (destination)

**Returns:** `nil`

**Example:**

```hi
os:copy("file.txt", "backup.txt")
```

---

#### `os:exists(path)`

Returns `TRUE` if the file or directory exists.

**Arguments:**

- `path` – string

**Returns:** boolean

**Example:**

```hi
PRINT os:exists("file.txt")
```

---

#### `os:stat(path)`

Returns a dictionary with file metadata.

**Arguments:**

- `path` – string

**Returns:** dict with fields: `size` (int), `modified`, `accessed`, `created` (Unix timestamps), `is_dir`, `is_file`

**Example:**

```hi
LET info = os:stat("file.txt")
PRINT "Size:", info["size"]
PRINT "Is file?", info["is_file"]
```

---

## Datetime Module

All dates are represented as dictionaries with fields: `year`, `month`, `day`, `hour`, `minute`, `second`,
`millisecond`, `timestamp` (Unix ms). Durations are dictionaries: `days`, `hours`, `minutes`, `seconds`, `milliseconds`.

### `datetime:now()`

Returns the current local date/time.

**Arguments:** none  
**Returns:** datetime dict

**Example:**

```hi
IMPORT "datetime" AS dt
LET now = dt:now()
```

---

### `datetime:utcnow()`

Returns the current UTC date/time.

**Arguments:** none  
**Returns:** datetime dict

**Example:**

```hi
LET utc = dt:utcnow()
```

---

### `datetime:fromstring(str, format)`

Parses a string according to the format and returns a datetime dictionary.

**Arguments:**

- `str` – string
- `format` – format string (see chrono format specifiers)

**Returns:** datetime dict

**Example:**

```hi
LET dt1 = dt:fromstring("2026-08-03 15:30:45", "%Y-%m-%d %H:%M:%S")
```

---

### `datetime:tostring(dt, format)`

Formats a datetime dictionary into a string.

**Arguments:**

- `dt` – datetime dict
- `format` – format string

**Returns:** string

**Example:**

```hi
PRINT dt:tostring(now, "%Y-%m-%d %H:%M:%S")
```

---

### `datetime:add(dt, duration)`

Adds a duration to a datetime and returns a new datetime dictionary.

**Arguments:**

- `dt` – datetime dict
- `duration` – duration dict

**Returns:** datetime dict

**Example:**

```hi
LET dur = dt:duration(3600)   // 1 hour
LET later = dt:add(dt1, dur)
```

---

### `datetime:diff(dt1, dt2)`

Returns `dt1 - dt2` as a duration dictionary.

**Arguments:**

- `dt1` – datetime dict
- `dt2` – datetime dict

**Returns:** duration dict

**Example:**

```hi
LET diff = dt:diff(later, dt1)
PRINT diff["hours"]
```

---

### `datetime:year(dt)`, `datetime:month(dt)`, `datetime:day(dt)`, `datetime:hour(dt)`, `datetime:minute(dt)`,
`datetime:second(dt)`, `datetime:millisecond(dt)`

Get the corresponding component from a datetime dictionary.

**Arguments:**

- `dt` – datetime dict

**Returns:** integer

**Example:**

```hi
PRINT dt:year(now)
```

---

### `datetime:timestamp(dt)`

Returns the Unix timestamp in milliseconds.

**Arguments:**

- `dt` – datetime dict

**Returns:** integer (milliseconds)

**Example:**

```hi
PRINT dt:timestamp(now)
```

---

### `datetime:duration(seconds)`

Creates a duration dictionary from a number of seconds (int or float).

**Arguments:**

- `seconds` – number

**Returns:** duration dict

**Example:**

```hi
LET dur = dt:duration(3600)   // 1 hour
```

---

## Random Module

### `random:randint(start, end)`

Returns a random integer in `[start, end]` (inclusive).

**Arguments:**

- `start` – integer
- `end` – integer

**Returns:** integer

**Example:**

```hi
IMPORT "random" AS r
LET dice = r:randint(1, 6)
```

---

### `random:randfloat()`

Returns a random float in `[0.0, 1.0)`.

**Arguments:** none  
**Returns:** float

**Example:**

```hi
LET prob = r:randfloat()
```

---

### `random:randbytes(n)`

Returns a list of `n` random bytes (0‑255).

**Arguments:**

- `n` – integer (non‑negative)

**Returns:** list of integers

**Example:**

```hi
LET bytes = r:randbytes(4)   // e.g., [23, 145, 8, 200]
```

---

### `random:shuffle(list)`

Returns a new list with elements shuffled (COW).

**Arguments:**

- `list` – list

**Returns:** list

**Example:**

```hi
LET shuffled = r:shuffle([1, 2, 3, 4])
```

---

### `random:choice(list)`

Returns a random element from the list, or `nil` if empty.

**Arguments:**

- `list` – list

**Returns:** value or `nil`

**Example:**

```hi
LET pick = r:choice(["apple", "banana", "cherry"])
```

---

## Regex Module

Patterns follow the Rust regex syntax. In Hi strings, backslashes must be escaped, so a pattern like `\d+` must be
written as `"\\d+"`.

### `regex:match(pattern, string)`

Returns `TRUE` if the pattern matches anywhere.

**Arguments:**

- `pattern` – string (regex)
- `string` – string

**Returns:** boolean

**Example:**

```hi
IMPORT "regex" AS re
PRINT re:match("\\d+", "abc123")   // TRUE
```

---

### `regex:find(pattern, string)`

Returns the first match as a string, or `nil`.

**Arguments:**

- `pattern` – string (regex)
- `string` – string

**Returns:** string or `nil`

**Example:**

```hi
PRINT re:find("\\w+", "Hello, world")   // "Hello"
```

---

### `regex:findall(pattern, string)`

Returns a list of all non‑overlapping matches.

**Arguments:**

- `pattern` – string (regex)
- `string` – string

**Returns:** list of strings

**Example:**

```hi
PRINT re:findall("[aeiou]", "hello")   // ["e", "o"]
```

---

### `regex:replace(pattern, string, replacement)`

Replaces all matches with `replacement`. You can use `$1`, `$2`, etc. for capture groups.

**Arguments:**

- `pattern` – string (regex)
- `string` – string
- `replacement` – string

**Returns:** string

**Example:**

```hi
PRINT re:replace("\\s+", "a b c", "-")   // "a-b-c"
```

---

### `regex:split(pattern, string)`

Splits the string by the pattern and returns a list of substrings.

**Arguments:**

- `pattern` – string (regex)
- `string` – string

**Returns:** list of strings

**Example:**

```hi
PRINT re:split(", +", "one, two, three")   // ["one", "two", "three"]
```

---

## Path Module

### `path:join(parts...)`

Concatenates any number of path parts using the platform separator.

**Arguments:** any number of strings (at least one)  
**Returns:** string

**Example:**

```hi
IMPORT "path" AS p
PRINT p:join("usr", "local", "bin")   // "usr/local/bin" (or backslashes)
```

---

### `path:basename(path)`

Returns the last component (file or directory name) as string, or `nil` if the path ends with a separator.

**Arguments:**

- `path` – string

**Returns:** string or `nil`

**Example:**

```hi
PRINT p:basename("/foo/bar.txt")   // "bar.txt"
```

---

### `path:dirname(path)`

Returns the parent directory, or `nil` if none. Removes trailing separators if present, then removes the last component.

**Arguments:**

- `path` – string

**Returns:** string or `nil`

**Example:**

```hi
PRINT p:dirname("/foo/bar.txt")   // "/foo"
PRINT p:dirname("foo/bar/")       // "foo/bar"
```

---

### `path:extname(path)`

Returns the extension including the dot (e.g., `".txt"`), or empty string if none.

**Arguments:**

- `path` – string

**Returns:** string

**Example:**

```hi
PRINT p:extname("archive.tar.gz")   // ".gz"
```

---

### `path:isabsolute(path)`

Returns `TRUE` if the path is absolute.

**Arguments:**

- `path` – string

**Returns:** boolean

**Example:**

```hi
PRINT p:isabsolute("/home")   // TRUE
```

---

### `path:normalize(path)`

Resolves `.` and `..` components and removes redundant separators without touching the filesystem.

**Arguments:**

- `path` – string

**Returns:** string

**Example:**

```hi
PRINT p:normalize("a/./b/../c")   // "a/c"
```

---

## Type Conversion Functions

### `tostring(value)`

Converts any value to its string representation.

**Arguments:**

- `value` – any Hi value

**Returns:** string

**Example:**

```hi
PRINT tostring(42)   // "42"
```

---

### `toint(value)`

Converts a number (int/float) or a string to an integer. For strings, parses as integer; for floats, truncates toward
zero.

**Arguments:**

- `value` – number or string

**Returns:** integer

**Example:**

```hi
PRINT toint("123")   // 123
PRINT toint(3.9)     // 3
```

---

### `tofloat(value)`

Converts a number or string to a float.

**Arguments:**

- `value` – number or string

**Returns:** float

**Example:**

```hi
PRINT tofloat("3.14")   // 3.14
```

---

## Special Functions

### `typeof(value)`

Returns a string with the type name of `value`. Possible names: `"integer"`, `"float"`, `"string"`, `"boolean"`,
`"list"`, `"dict"`, `"file"`, `"function"`, `"nil"`, `"module"`.

**Arguments:**

- `value` – any Hi value

**Returns:** string

**Example:**

```hi
PRINT typeof(42)   // "integer"
```

---

### `call(func, ...args)`

Calls a function value (stored in a variable) with the given arguments. Enables higher‑order programming.

**Arguments:**

- `func` – a function value (variable)
- `args...` – arguments to pass

**Returns:** value returned by `func`

**Example:**

```hi
FUNC double(x) RET x * 2 END
LET f = double
LET result = call(f, 5)
PRINT result   // 10
```

---

### `hello()`

Prints `"Hello, World!"` to the console. For backwards compatibility; you can just use `PRINT "Hello, World!"`.

**Arguments:** none  
**Returns:** `nil`

**Example:**

```hi
hello()
```

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