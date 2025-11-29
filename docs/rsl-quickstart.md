# RSL Quick Start

Rift Script Language (RSL) is a minimal, dynamically typed language used inside Rift. This guide shows how to run it and highlights the core syntax and standard library.

## Running RSL
- Run a script: `cargo run -p rsl -- crates/rsl/examples/test.rsl`.
- Open the REPL: `cargo run -p rsl`. Enter source, then press Enter twice to execute the buffered input.
- Scripts resolve imports relative to the current working directory.

## Values, variables, and scope
- Primitive values: `null`, booleans, numbers, strings. Unset identifiers read as `null`.
- Assign with `=`: `x = 5`; mutable by default. Prefer lowerCamelCase names for variables and functions.
- Scope modifiers:
  - `local` keeps the binding in the current block frame.
  - `export` makes the binding available to importers of the file.
- Functions, arrays, and tables are passed by reference; other values are cloned on pass.

```rsl
# comments start with #
local greeting = "hi"
export answer = 42
```

## Functions and modules
- Define with `fn name(params) { ... }`; `return` exits with a value.
- Functions are first-class and can be reassigned.
- Mark exported functions with `export fn` to expose them to importers.
- Import another script file with `import("path/to/file.rsl")`; it returns a table of exported bindings.

```rsl
export fn square(n) { return n * n }

fn run() {
    let lib = import("crates/rsl/examples/lib.rsl")
    print(lib.x)        # exported variable
    runFunctionById = lib.localTest  # functions are values
}
```

## Control flow
- `if condition { ... }` expects a boolean; the body executes in a nested scope.
- `loop { ... }` repeats until a `break`; use `return` inside functions to exit early.

```rsl
i = 0
loop {
    if i >= 3 { break }
    print(i)
    i = i + 1
}
```

## Collections
- Arrays: create and manipulate via the standard library.

```rsl
arr = createArray(1, 2, 3)
arrayPushBack(arr, 4)
print(arrayGet(arr, 0), arrayLen(arr))  # 1 4
```

- Tables: string-keyed maps.

```rsl
user = createTable()
tableSet(user, "name", "Rift")
tableSet(user, "active", true)
print(tableGet(user, "name"))           # Rift
print(tableKeys(user))                  # ["name", "active"]
```

## Standard library highlights
- I/O and OS: `print(...)`, `readFile(path)`, `getEnvVar(key)`, `runShellCommand(cmd, workspace_dir)`.
- Data helpers: `toJson(value)`, `fromJson(json)`, `stringSplitLines(str)`.
- Arrays: `createArray(...)`, `arrayLen`, `arrayGet`, `arraySet`, `arrayPushBack`, `arrayRemove`, `arrayPopBack`.
- Tables: `createTable()`, `tableSet`, `tableGet`, `tableKeys`.
- HTTP: `getRequest(url)`, `postRequest(url, body)`, `postRequestWithBearerToken(url, body, token)`.
- Editor/agent helpers (used when embedded in Rift): `agentReadFile`, `agentWriteFile`, `agentReplace`.

## End-to-end sample

```rsl
# crates/rsl/examples/test.rsl
arr = createArray(9, 8, 7)
applyArray = fn (xs, f) {
    i = 0
    loop {
        if i >= arrayLen(xs) { break }
        arraySet(xs, i, f(arrayGet(xs, i)))
        i = i + 1
    }
}

applyArray(arr, fn (n) { return n * n })
print(toJson(arr))  # => [81,64,49]
```

With these primitives you can build modules, script the editor, or prototype utilities quickly inside Rift. Buffers containing RSL scripts can also be executed directly from the Rift editor UI, so you can iterate without leaving your active buffer.
