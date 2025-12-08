## 1. Preparation
- [x] 1.1 Verify clean git state (no uncommitted changes)
- [x] 1.2 Confirm `cargo build` succeeds before changes

## 2. Directory Restructuring
- [x] 2.1 Create `crates/` directory at project root
- [x] 2.2 Move `conclaude-field-derive/` to `crates/conclaude-field-derive/`
- [x] 2.3 Remove old `conclaude-field-derive/target/` if present (build artifacts don't need to move)

## 3. Configuration Updates
- [x] 3.1 Update workspace members in `Cargo.toml`: `[".", "crates/conclaude-field-derive"]`
- [x] 3.2 Update dependency path in `Cargo.toml`: `path = "crates/conclaude-field-derive"`

## 4. Validation
- [x] 4.1 Run `cargo build` to verify workspace resolves correctly
- [x] 4.2 Run `cargo test` to ensure all tests pass
- [x] 4.3 Run `cargo clippy` to check for any new warnings
