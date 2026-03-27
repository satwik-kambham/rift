# RSL Integration Test Scripts

Reference for the test scripts in this directory.
Each `.rsl` script is discovered and executed by the test harness in
`crates/rsl/tests/integration.rs`.

Scripts run in alphabetical order. A failing `assert()` or
`assertEqual()` panics the test immediately.

## Test Files

| File | Category | What it covers |
|---|---|---|
| `00_comments.rsl` | Syntax | Single-line `#` comments: inline, standalone, consecutive, between statements |
| `01_data_types.rsl` | Types | All 8 primitive types (null, bool, number, string, function, array, table, error), `typeOf`, `isNull` |
| `02_arithmetic.rsl` | Operators | `+`, `-`, `*`, `/`, `%`, unary `-`, `floor` |
| `03_comparison_and_equality.rsl` | Operators | `==`, `!=`, `<`, `<=`, `>`, `>=`; cross-type equality semantics |
| `04_logical_operators.rsl` | Operators | `and`, `or`, `!` truth tables and combinations |
| `05_operator_precedence.rsl` | Operators | Full precedence chain (unary > multiplicative > additive > comparison > equality > and > or), grouping with parentheses |
| `06_variables_and_scoping.rsl` | Variables | `let` declaration, reassignment, block scoping in if blocks, variable shadowing across nested scopes |
| `07_strings.rsl` | Strings | Concatenation, escape sequences, `stringLen`, `stringContains`, `stringSplitLines`, `stringToLower`, `stringWidth`, `stringTruncateWidth`, `toString` |
| `08_control_flow.rsl` | Control flow | `if`/`else`/`else if` chains, `loop`/`break`, early `return`, nested control flow |
| `09_functions.rsl` | Functions | Parameters, return values, closures, recursion (factorial, fibonacci), first-class functions, higher-order functions, nested calls |
| `10_arrays.rsl` | Data structures | `createArray`, index access/assignment, `arrayGet`, `arraySet`, `arrayPushBack`, `arrayPopBack`, `arrayInsert`, `arrayRemove`, `arrayLen`, nested arrays, mixed types |
| `11_tables.rsl` | Data structures | `createTable`, bracket access/assignment, `tableGet`, `tableSet`, `tableKeys`, nested tables, function values, table as accumulator |
| `12_json_and_utilities.rsl` | Stdlib | `toJson`/`fromJson` round-trips on all types, JSON parsing of objects and arrays, invalid JSON errors, `toString`, `print` |

## Available Test Helpers

| Function | Description |
|---|---|
| `assert(bool)` | Panics if the value is false |
| `assertEqual(actual, expected)` | Panics if actual != expected, showing both values |
| `isNull(value)` | Returns true if value is null |
| `typeOf(value)` | Returns type name: "null", "bool", "number", "string", "function", "array", "table", "error" |
| `toString(value)` | Converts any value to its string representation |
| `toJson(value)` / `fromJson(json)` | JSON serialization and deserialization |
| `print(args...)` | Prints to stdout (variadic) |
| `floor(number)` | Floor function |

## Known Limitations

- **Array/table equality is reference-based**: `assertEqual` on two arrays always fails even if contents match. Compare individual elements or lengths instead.
- **`assert()` requires a boolean**: Not truthy/falsy. `assert(1)` is a type error.
- **f32 precision**: Numbers are 32-bit floats. Avoid exact equality on results prone to rounding (e.g., `0.1 + 0.2`).
- **Table key order is nondeterministic**: `tableKeys` returns keys from a HashMap. Don't depend on iteration order.
- **No error-path testing**: The harness panics on runtime errors, so you cannot test that invalid operations produce errors. You can test error *values* returned by functions (e.g., `fromJson("bad")` returns an error primitive).
- **No I/O or network tests**: File/network natives require real resources and are not covered here.

## Adding New Tests

1. Create a new `.rsl` file in this directory. Use the numeric prefix convention to control execution order.
2. Use `assert()` and `assertEqual()` for all checks.
3. Add an entry to the table above.
4. Run `cargo test -p rsl` to verify.
