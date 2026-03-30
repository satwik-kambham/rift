---
name: review-rift
description: >
  In-depth code review pipeline for the Rift codebase. Triggers when the user asks to review changes,
  check code quality, or run the review pipeline. Use this skill whenever the user says things like
  "review my changes", "review this PR", "check my code", "run review", "review the diff",
  or any request to evaluate code quality of recent work. Also trigger when the user asks to
  "check before committing" or "is this ready to merge".
---

# Rift Code Review Pipeline

You are reviewing changes in the Rift editor codebase — a Rust workspace with an embedded scripting
language (RSL). Your job is to produce a thorough, honest review report that catches real issues
before they reach main.

## Determining the diff scope

Figure out what to review based on what the user asked:

- **Default (no specific scope given)**: Review staged and unstaged changes against HEAD.
  Run `git diff HEAD` to get the full picture.
- **Specific commit**: `git diff <commit>^ <commit>` or `git show <commit>`
- **Branch**: `git diff main...<branch>`
- **PR**: Use `gh pr diff <number>` to get the diff

Once you have the diff, identify every file that was modified, added, or deleted — you'll need
this list throughout the review.

## Review stages

Work through each stage below. For every stage, read the *actual source files* that were changed
(not just the diff hunks) so you can understand the surrounding context. The diff tells you *what*
changed; the full file tells you *whether the change fits*.

### 1. Formatting

**Rust files**: Run `cargo fmt --check` and report any formatting violations. Don't just say
"run cargo fmt" — list the specific files that need formatting.

**RSL files** (`.rsl`): There's no auto-formatter. Manually check:
- `lowerCamelCase` for variable and function names
- Consistent indentation (match surrounding code)
- No trailing whitespace or inconsistent line endings

### 2. Compilation and linting

Run `cargo check` and `cargo clippy -- -D warnings`. Report any errors or warnings with their
full context. If clippy suggests a fix, include the suggestion in your report.

### 3. Tests

Run `cargo test --workspace` (using the Makefile's split approach: `cargo test -p rsl` separately
from `cargo test --workspace --exclude rsl`).

Report:
- Any test failures with full output
- Whether the changes *should* have new tests but don't. Use judgment here — a one-line typo fix
  doesn't need a test, but a new function or changed behavior does.

**Test index check**: If any RSL test scripts in `crates/rsl/tests/scripts/` were added or modified,
verify that `crates/rsl/tests/scripts/TEST_INDEX.md` was updated to reflect the changes. If a new
test file was added without updating the index, flag it.

### 4. Documentation

Check whether documentation needs updating based on what changed:

- **RSL language changes** (new syntax, new builtins, changed semantics): Check if
  `docs/rsl-quickstart.md` was updated. Language changes without doc updates are a warning.
- **AGENTS.md**: If the change affects build commands, project structure, coding conventions,
  or development workflows, check whether `AGENTS.md` needs an update.
- **Doc comments**: If public APIs were added or changed in Rust code, check for doc comments.
  Don't require them on every function — but exported types, trait methods, and non-obvious
  public functions should have them.

### 5. Index comments

Some modules use structured comments to provide a table of contents or index of what's in the
file (similar to how `TEST_INDEX.md` indexes test scripts, but inline in source files). These
might look like section headers, function listings, or structured comment blocks at the top of
a module.

**Only check this if the module already has index comments.** If a module doesn't use them,
don't suggest adding them. But if a module *does* have them and the change adds/removes/renames
functions or sections, flag that the index comments may need updating.

To check: read the full file for any changed module and look for patterns like:
- Comment blocks listing functions or sections
- `// --- Section Name ---` style dividers
- Module-level doc comments with function inventories

### 6. Code quality

This is the most important stage. Read the changed code *and its surrounding context* carefully.

**What to check:**

- **Naming**: Do new identifiers follow Rust conventions (`snake_case` functions/vars,
  `CamelCase` types, `SCREAMING_SNAKE_CASE` consts)? Do RSL identifiers use `lowerCamelCase`?
  Are names descriptive and consistent with the rest of the codebase?

- **Error handling**: Are errors propagated properly with `?` or handled gracefully?
  Flag any new `unwrap()` or `expect()` on fallible operations (file I/O, parsing, RPC, network).
  `unwrap`/`expect` is fine in tests and truly unrecoverable spots, but not in library/application code.

- **Diagnostics**: Does the code use `tracing` for logging (not `println!`)? Are error messages
  helpful — do they include context like file paths, IDs, or what operation failed?

- **Duplication**: Is there copy-pasted code that should be extracted? Look at both the diff and
  the surrounding module. But don't flag every repeated pattern — three similar lines is often
  better than a premature abstraction.

- **Architecture fit**: Does the change follow existing patterns in the crate? For example:
  - `rift_core` actions should follow the command/state pattern
  - RSL interpreter changes should maintain the Statement/Expression trait pattern
  - Frontend code should stay stateless (state lives in `rift_core`)

- **Correctness concerns**: Off-by-one errors, missing edge cases, race conditions, resource
  leaks. Think about what happens with empty inputs, very large inputs, or unexpected types.

- **Security**: Check for command injection, path traversal, or unsafe handling of user input.
  This matters most in `rift_server` (web-facing) and any file I/O code.

- **Performance**: Only flag obvious issues — unnecessary clones of large data, O(n^2) loops
  where linear is possible, allocations in hot paths. Don't micro-optimize.

## Report format

Produce a structured report using this format:

```
# Code Review Report

## Summary
[1-2 sentence overview: what the changes do, overall assessment]

## Formatting
[Results from cargo fmt and manual RSL checks, or "All clear"]

## Compilation & Linting
[Results from cargo check and clippy, or "All clear"]

## Tests
[Test results, missing test coverage, test index status]

## Documentation
[Which docs need updating, or "No documentation changes needed"]

## Index Comments
[Any index comments that need updating, or "No index comment updates needed"]

## Code Quality
### Warnings
[Issues that should be fixed before merging — bugs, security issues, convention violations]

### Suggestions
[Optional improvements — better naming, refactoring opportunities, style nits]

## Verdict
[PASS / PASS WITH WARNINGS / NEEDS CHANGES — and a brief justification]
```

**Severity guide:**
- **Warning**: Should be addressed before merging. Bugs, security issues, missing error handling
  on I/O, convention violations that would confuse other contributors, missing docs for language changes.
- **Suggestion**: Nice-to-have improvements. Better variable names, minor refactoring opportunities,
  style preferences. The author can take or leave these.

Be specific in every finding — include the file path, line number, and a brief code snippet showing
the issue. Don't just say "consider better error handling" — say exactly where and what.

**Verdict criteria:**
- **PASS**: No warnings, tests pass, formatting clean.
- **PASS WITH WARNINGS**: Minor warnings that are low-risk but should ideally be addressed.
- **NEEDS CHANGES**: Any bugs, security issues, test failures, or significant convention violations.
