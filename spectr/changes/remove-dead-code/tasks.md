## 1. Remove Unused Dependencies from Cargo.toml

- [ ] 1.1 Remove `config = "0.14"` from dependencies (external crate never imported)
- [ ] 1.2 Remove `thiserror = "1.0"` from dependencies (crate never imported)
- [ ] 1.3 Remove `chrono = { version = "0.4", features = ["serde"] }` from dependencies (crate never imported)
- [ ] 1.4 Run `cargo build --all-targets` to verify no compilation errors
- [ ] 1.5 Run `cargo test` to verify no test failures

## 2. Remove HookPayload Enum and Methods from types.rs

- [ ] 2.1 Remove `HookPayload` enum (src/types.rs lines 200-227) - never instantiated or matched
- [ ] 2.2 Remove `impl HookPayload` block with `session_id()`, `transcript_path()`, `hook_event_name()` methods (lines 229-283)
- [ ] 2.3 Remove any `#[allow(dead_code)]` annotations that become unnecessary
- [ ] 2.4 Run `cargo clippy --all-targets` to verify no new warnings
- [ ] 2.5 Run `cargo test` to verify tests pass

## 3. Move Test-Only Schema Validation to cfg(test)

- [ ] 3.1 Add `#[cfg(test)]` annotation to `validate_config_against_schema()` in src/schema.rs
- [ ] 3.2 Remove existing `#[allow(dead_code)]` from the function
- [ ] 3.3 Run `cargo test` to verify schema tests still pass
- [ ] 3.4 Run `cargo build --release` to verify function is excluded from release binary

## 4. Clean Up generate-docs.rs Unused Struct Fields

- [ ] 4.1 In `Schema` struct: remove `title` and `description` fields (or keep if JSON parsing requires them, but remove `#[allow(dead_code)]`)
- [ ] 4.2 In `PropertyDefinition` struct: remove `name` field (or keep if JSON parsing requires, but remove annotation)
- [ ] 4.3 If fields are needed for deserialization but not accessed, use `#[serde(skip_deserializing)]` or accept them in parsing
- [ ] 4.4 Run `cargo build --bin generate-docs` to verify binary compiles

## 5. Remove Remaining Unnecessary Dead Code Annotations

- [ ] 5.1 Audit remaining `#[allow(dead_code)]` in src/schema.rs for `generate_config_schema()` and `write_schema_to_file()`
- [ ] 5.2 These are used by the generate-schema binary - remove annotations since they ARE used
- [ ] 5.3 Audit remaining annotations in src/types.rs for `validate_*_payload()` functions
- [ ] 5.4 These ARE used by hooks.rs - remove the annotations

## 6. Final Verification

- [ ] 6.1 Run full test suite: `cargo test --all`
- [ ] 6.2 Run clippy without warnings: `cargo clippy --all-targets`
- [ ] 6.3 Verify grep for `#[allow(dead_code)]` shows only legitimate uses
- [ ] 6.4 Build all binaries: `cargo build --all-targets`
- [ ] 6.5 Verify no compilation warnings about unused code
