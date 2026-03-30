# RSL Quickstart

RSL (Rift Scripting Language) is a dynamically-typed, interpreted scripting language for automating and extending the Rift editor.

## Comments

```rsl
# Single-line comment (only style available)
let x = 10  # Inline comment
```

## Data Types

| Type | Examples | Notes |
|------|----------|-------|
| `null` | `null` | Default value for undeclared variables |
| `bool` | `true`, `false` | |
| `number` | `42`, `3.14`, `-0.5` | 32-bit float internally |
| `string` | `"hello"`, `""` | Supports `\n`, `\t`, `\\`, `\"` escapes |
| `array` | `[1, "two", true]` | Heterogeneous, 0-indexed |
| `table` | `{"key": value}` | String keys, any value type |
| `function` | `fn(x) { return x }` | First-class, supports closures |
| `error` | returned by `fromJson()` | Check with `typeOf(x) == "error"` |

```rsl
typeOf(42)        # "number"
typeOf("hello")   # "string"
isNull(null)      # true
isNull(0)         # false
```

## Variables

```rsl
let x = 10        # Declare a new block-scoped variable
x = 20            # Reassign an existing variable (searches parent scopes)
export y = 5      # Declare and export for module consumers
```

Values are copied on assignment except for functions, arrays, and tables which are shared by reference.

### Scoping

`let` creates a new variable in the current block. Without `let`, assignment modifies the nearest existing variable in the scope chain.

```rsl
let a = 1
if true {
    let a = 99     # New local — does not affect outer 'a'
    a              # 99
}
a                  # 1

let counter = 0
if true {
    counter = 5    # No 'let' — modifies outer 'counter'
}
counter            # 5
```

## Operators

**Arithmetic** (numbers): `+`, `-`, `*`, `/`, `%`, unary `-`

**String**: `+` (concatenation)

**Comparison** (numbers): `<`, `<=`, `>`, `>=`, `==`, `!=`

**Logical** (booleans): `and`, `or`, `!`

**Precedence** (high to low): `!` `-` → `*` `/` `%` → `+` `-` → `<` `<=` `>` `>=` → `==` `!=` → `and` → `or`

## Control Flow

### if / else if / else

```rsl
if x > 10 {
    print("big")
} else if x > 5 {
    print("medium")
} else {
    print("small")
}
```

### loop

Infinite loop — exit with `break`.

```rsl
let i = 0
loop {
    if i == 5 { break }
    i = i + 1
}
```

### while

```rsl
while x < 10 {
    x = x + 1
}
```

### for ... in

Iterates over a **copy** of the array, so mutations during iteration are safe.

```rsl
for item in [1, 2, 3] {
    print(item)
}

for i in range(0, 5, 1) {
    print(i)  # 0, 1, 2, 3, 4
}
```

`break` works inside all loop types. `return` exits the enclosing function.

## Functions

```rsl
fn add(a, b) {
    return a + b
}
add(2, 3)  # 5
```

Functions without an explicit `return` implicitly return `null`.

### Anonymous Functions (Lambdas)

```rsl
let square = fn(x) { return x * x }
square(4)  # 16

# Immediately invoked
fn(a, b) { return a + b }(3, 7)  # 10
```

### First-Class Functions

```rsl
fn apply(f, x) { return f(x) }
apply(fn(x) { return x * 3 }, 5)  # 15

let myFunc = add
myFunc(3, 4)  # 7
```

### Closures

Closures capture variables by reference and see updates.

```rsl
fn makeCounter(start) {
    let count = start
    fn increment(step) {
        count = count + step
        return count
    }
    return increment
}
let counter = makeCounter(0)
counter(1)  # 1
counter(1)  # 2
counter(5)  # 7
```

## Strings

```rsl
"hello" + " " + "world"          # Concatenation
stringLen("hello")                # 5
stringContains("hello", "ell")   # true
stringToLower("HELLO")           # "hello"
stringSplitLines("a\nb\nc")      # ["a", "b", "c"]
stringWidth("hello")             # 5 (unicode display width)
stringTruncateWidth("hello", 3)  # "hel"
```

No string interpolation — use concatenation: `"value: " + toString(x)`

## Arrays

```rsl
let arr = [10, 20, 30]
arr[0]                    # 10 (read)
arr[1] = 99               # write
arrayLen(arr)              # 3
arrayPushBack(arr, 40)     # append
arrayPopBack(arr)          # remove and return last
arrayInsert(arr, 0, 5)     # insert at index
arrayRemove(arr, 0)        # remove and return at index
arrayGet(arr, 1)           # safe get
arraySet(arr, 1, 50)       # safe set
```

### range

```rsl
range(0, 5, 1)    # [0, 1, 2, 3, 4]  (stop excluded)
range(0, 10, 2)   # [0, 2, 4, 6, 8]
range(5, 0, -1)   # [5, 4, 3, 2, 1]
```

## Tables

```rsl
let t = {"name": "rift", "version": 1}
t["name"]                 # "rift" (read)
t["new_key"] = true       # write
tableGet(t, "missing")    # null (no error for missing keys)
tableSet(t, "key", val)   # set key
tableKeys(t)              # array of all keys
```

### Method Calls

Dot notation passes the table as the first argument (`self`).

```rsl
fn greet(self) {
    return "hello " + self["name"]
}
let obj = {"name": "rift", "greet": greet}
obj.greet()  # "hello rift" — equivalent to greet(obj)
```

## Modules

```rsl
# lib.rsl
export x = 5
export fn helper() { return x }
let private = 10  # Not exported
```

```rsl
# main.rsl
let lib = import("lib.rsl")   # Cached — runs once, returns table of exports
lib["x"]                       # 5
lib["helper"]()                # 5

runScript("other.rsl")         # Always re-executes, returns null
```

## JSON

```rsl
toJson({"a": 1})             # '{"a":1.0}'
fromJson("{\"a\":1}")        # table: {"a": 1}
fromJson("bad")              # error type — check with typeOf()

let result = fromJson(data)
if typeOf(result) == "error" {
    print("parse failed")
}
```

## Utility Functions

| Function | Description |
|----------|-------------|
| `print(args...)` | Print values (space-separated) |
| `typeOf(value)` | Type name as string |
| `isNull(value)` | Check if null |
| `toString(value)` | Convert to string |
| `floor(number)` | Floor of a number |
| `assert(condition)` | Panic if false |
| `assertEqual(a, b)` | Panic if not equal |

## File I/O

| Function | Description |
|----------|-------------|
| `readFile(path)` | Read file contents |
| `createBlankFile(path)` | Create empty file |
| `createDirectory(path)` | Create directory |
| `listDir(path)` | List directory (returns JSON) |
| `joinPath(base, rel)` | Join path segments |
| `parentPath(path)` | Get parent directory |

## Shell & Environment

| Function | Description |
|----------|-------------|
| `runShellCommand(cmd)` | Execute shell command |
| `commandExists(cmd)` | Check if command exists |
| `getEnvVar(name)` | Get environment variable |

## HTTP

| Function | Description |
|----------|-------------|
| `getRequest(url)` | HTTP GET, returns string or error |
| `postRequest(url, body)` | HTTP POST |
| `postRequestWithBearerToken(url, body, token)` | POST with auth header |

## Editor RPC Functions

These functions are available when running inside the Rift editor (via the RPC bridge).

### Logging & Navigation

| Function | Description |
|----------|-------------|
| `log(message)` | Log message to editor |
| `openFile(path)` | Open a file in the editor |
| `setSearchQuery(query)` | Set the editor search query |

### Buffer Management

| Function | Description |
|----------|-------------|
| `getActiveBuffer()` | Get active buffer ID (or null) |
| `setActiveBuffer(id)` | Switch to buffer by ID |
| `listBuffers()` | List all buffers (JSON string) |
| `createSpecialBuffer(name)` | Create a special buffer, returns ID |
| `setBufferContent(id, content)` | Set buffer text content |
| `getBufferInput(id)` | Get buffer input field value |
| `setBufferInput(id, input)` | Set buffer input field value |
| `registerBufferInputHook(id, fn)` | Register callback for buffer input changes |

### Keybindings

| Function | Description |
|----------|-------------|
| `registerGlobalKeybind(def, fn)` | Bind a key globally to a function |
| `registerBufferKeybind(id, def, fn)` | Bind a key for a specific buffer |

### Workspace & Editor State

| Function | Description |
|----------|-------------|
| `getWorkspaceDir()` | Get workspace root path |
| `getViewportSize()` | Get viewport dimensions (JSON) |
| `selectRange(selection)` | Set editor selection |
| `getActions()` | List available actions (JSON) |
| `runAction(action)` | Execute an editor action |
| `getDefinitions()` | Get symbol definitions (JSON) |
| `getReferences()` | Get symbol references (JSON) |
| `getWorkspaceDiagnostics()` | Get diagnostics (JSON) |
| `tts(text)` | Text-to-speech |

## Style Conventions

- Use **lowerCamelCase** for function and variable names in `.rsl` files.
- Match the surrounding code style.
