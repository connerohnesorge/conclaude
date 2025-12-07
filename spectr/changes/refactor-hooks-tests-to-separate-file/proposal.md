# Change: Separate hooks tests to dedicated test file

## Why
The `src/hooks.rs` file is 2732 lines with ~960 lines of tests (lines 1772-2732). Extracting tests to a separate file improves maintainability by:
- Reducing cognitive load when working on implementation vs tests
- Following Rust convention of separate test modules for large test suites
- Making the hooks implementation easier to navigate

## What Changes
- Create `src/hooks_tests.rs` containing all tests from `src/hooks.rs`
- Remove `#[cfg(test)] mod tests { ... }` block from `src/hooks.rs`
- Add `#[cfg(test)] mod hooks_tests;` to `src/lib.rs` to include the test module

## Impact
- Affected specs: hooks-system
- Affected code: `src/hooks.rs`, `src/lib.rs`, new `src/hooks_tests.rs`
- No behavioral changes - purely structural refactor
- All existing tests continue to pass unchanged
