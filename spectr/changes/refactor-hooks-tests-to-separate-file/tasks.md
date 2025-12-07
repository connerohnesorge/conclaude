## 1. Implementation

- [x] 1.1 Create `src/hooks_tests.rs` with the test module contents extracted from `src/hooks.rs`
- [x] 1.2 Update imports in the new test file to reference `crate::hooks::*` instead of `super::*`
- [x] 1.3 Remove the `#[cfg(test)] mod tests { ... }` block from `src/hooks.rs`
- [x] 1.4 Add `#[cfg(test)] mod hooks_tests;` to `src/lib.rs`
- [x] 1.5 Run `cargo test` to verify all hooks tests pass
- [x] 1.6 Run `cargo clippy` to verify no linting issues
- [x] 1.7 Run `cargo fmt` to ensure consistent formatting
