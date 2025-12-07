## 1. Implementation

- [ ] 1.1 Create `src/hooks_tests.rs` with the test module contents extracted from `src/hooks.rs`
- [ ] 1.2 Update imports in the new test file to reference `crate::hooks::*` instead of `super::*`
- [ ] 1.3 Remove the `#[cfg(test)] mod tests { ... }` block from `src/hooks.rs`
- [ ] 1.4 Add `#[cfg(test)] mod hooks_tests;` to `src/lib.rs`
- [ ] 1.5 Run `cargo test` to verify all hooks tests pass
- [ ] 1.6 Run `cargo clippy` to verify no linting issues
- [ ] 1.7 Run `cargo fmt` to ensure consistent formatting
