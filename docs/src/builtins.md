# Built‑in Functions

Hi provides a rich set of built‑in functions that you can call like any user‑defined function.  
They cover string manipulation, list and dictionary operations, file I/O, mathematics, type conversions, and more.

All built‑in functions are **global** and can be called anywhere. They are **case‑sensitive** – use the names exactly as
shown.

---

## Copy‑on‑Write (COW) for Lists

Many list‑modifying operations (`append`, `insert`, `remove`, `reverse`) use **copy‑on‑write** semantics. This means:

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

### `len(s)`

Returns the length of a string (or list/dict – see generic functions).

- **Arguments:** `s` – string
- **Returns:** integer

```hi
PRINT len("hello")   // 5
```

### `split(s, delimiter)`

Splits a string into a list of substrings using `delimiter`.

- **Arguments:** `s` – string, `delimiter` – string
- **Returns:** list of strings

```hi
LET parts = split("a,b,c", ",")
PRINT parts   // ["a", "b", "c"]
```

### `concat(a, b)`

Concatenates two strings (or two lists – see generic functions).

- **Arguments:** `a` – string, `b` – string
- **Returns:** string

```hi
PRINT concat("Hello", " World")   // "Hello World"
```

### `replace(s, old, new)`

Replaces all occurrences of `old` with `new` in string `s`.

- **Arguments:** `s` – string, `old` – string, `new` – string
- **Returns:** string

```hi
PRINT replace("Hello World", "World", "Hi")   // "Hello Hi"
```

### `substr(s, start, length)`

Extracts a substring from `s` starting at `start` (0‑based) with given `length`.

- **Arguments:** `s` – string, `start` – integer, `length` – integer (non‑negative)
- **Returns:** string

```hi
PRINT substr("Hello", 1, 3)   // "ell"
```

### `starts(s, prefix)`

Returns `TRUE` if `s` starts with `prefix`.

- **Arguments:** `s` – string, `prefix` – string
- **Returns:** boolean

### `ends(s, suffix)`

Returns `TRUE` if `s` ends with `suffix`.

### `upper(s)`

Converts `s` to uppercase.

### `lower(s)`

Converts `s` to lowercase.

### `trim(s)`

Removes leading and trailing whitespace from `s`.

### `reverse(s)`

Reverses the characters in `s` (also works for lists).

### `indexof(s, substring)`

Returns the first index of `substring` in `s`, or `-1` if not found.

### `contains(s, substring)`

Returns `TRUE` if `s` contains `substring` (works for lists/dicts too – see generic functions).

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

All file functions use **file handles** returned by `open()`.

### `open(path, mode)`

Opens a file and returns a file handle.

- `path` – string, `mode` – string: `"r"` (read), `"w"` (write, overwrites), `"a"` (append).
- Returns a file handle (type `File`).

```hi
LET f = open("data.txt", "r")
```

### `close(file)`

Closes the file handle (flushes writes if needed). Returns `nil`.

### `read(file)`

Reads the entire remaining content of the file as a string. Returns string.

```hi
LET content = read(f)
```

### `readln(file)`

Reads one line (including newline if present). Returns string. At EOF, returns an empty string and sets EOF flag.

### `write(file, value)`

Writes the string representation of `value` to the file (without newline). Returns `nil`.

### `writeln(file, value)`

Writes `value` followed by a newline. Returns `nil`.

### `eof(file)`

Returns `TRUE` if the end of the file has been reached (or file is not open for reading). Returns boolean.

---

## Mathematical Functions

All math functions expect a numeric argument (integer or float) and return a float, except `rand()` which returns an
integer.

### `sin(x)`, `cos(x)`, `tan(x)`, `asin(x)`, `acos(x)`, `atan(x)`

Trigonometric functions. Input in radians. `asin` and `acos` require `-1 ≤ x ≤ 1`.

### `sqrt(x)`

Square root; `x ≥ 0`.

### `torad(degrees)`, `todeg(radians)`

Convert between degrees and radians.

### `exp(x)` – eˣ, `log(x)` – natural logarithm (x > 0), `log2(x)`, `log10(x)`

### `ceil(x)`, `floor(x)`, `round(x)` – rounding to nearest integer (as float).

### `abs(x)` – absolute value.

### `rand(start, end)`

Returns a random integer between `start` and `end` (inclusive). Both arguments must be integers, `start ≤ end`.

```hi
LET r = rand(1, 10)
PRINT r   // random number 1..10
```

### Constants

The interpreter predefines two global floating‑point constants: `PI` and `E`.

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
`"list"`, `"dict"`, `"file"`, `"function"`, `"nil"`.

```hi
LET t = typeof(42)
PRINT t   // "integer"
```

### `call(func, ...args)`

Calls a function value (stored in a variable) with the given arguments. This enables higher‑order programming.

```hi
FUNC double(x) RET x * 2 END
LET f = double
LET result = call(f, 5)
PRINT result   // 10
```

### `hello()`

Prints `"Hello, World!"` to the console. For backwards compatibility, but you can just use `PRINT "Hello, World!"`.

---

## Generic Functions

Some functions work with multiple types:

- `len` – works on strings, lists, dicts.
- `contains` – works on strings (substring), lists (element), dicts (key).
- `concat` – works on two strings or two lists.
- `reverse` – works on strings and lists.

For `contains`, the second argument must match the type: for strings, a string; for lists, any value; for dicts, a
hashable key.

---

## Error Handling

All built‑in functions perform argument count and type checking. If you pass the wrong number of arguments or incorrect
types, a runtime error is raised with a descriptive message.

---

## Examples

```hi
// Strings
LET s = "  Hello World!  "
PRINT trim(s)                 // "Hello World!"
PRINT upper(s)                // "  HELLO WORLD!  "
PRINT replace(s, "World", "Hi") // "  Hello Hi!  "

// Lists
LET a = [10, 20, 30]
LET b = append(a, 40)
PRINT b                       // [10, 20, 30, 40]
LET c = insert(b, 1, 15)
PRINT c                       // [10, 15, 20, 30, 40]
LET d = remove(c, 2)
PRINT d                       // [10, 15, 30, 40]

// Dictionaries
LET user = {"name" = "Bob", "age" = 25}
put(user, "city", "Paris")
PRINT get(user, "city")       // "Paris"
PRINT contains(user, "age")   // TRUE
remove(user, "age")
PRINT user                    // {"name"="Bob", "city"="Paris"}

// Math
PRINT sin(PI / 2)             // 1.0
PRINT rand(1, 6)              // random integer 1..6

// Type info
PRINT typeof(TRUE)            // "boolean"
```