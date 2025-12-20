## ADDED Requirements

### Requirement: Minimal Dependency Set

The project SHALL only include dependencies in Cargo.toml that are actively used in production or test code. Unused dependencies MUST be removed to reduce compile time, binary size, and audit surface.

#### Scenario: All dependencies are used

- WHEN `cargo build` is executed
- THEN every dependency listed in Cargo.toml is imported and used
- AND no compiler warnings about unused crates appear
- AND removing any dependency would cause a compilation error

### Requirement: No Dead Code Annotations for Unused Code

Code marked with `#[allow(dead_code)]` SHALL only be used for intentional library exports or API stability. Internal implementation code that is never called MUST be removed rather than annotated.

#### Scenario: Dead code annotation audit

- WHEN the codebase is audited for `#[allow(dead_code)]` annotations
- THEN each annotated item has a documented reason (library export, API stability, binary entry point)
- AND no annotations are used to hide truly unused internal code

### Requirement: Test-Only Code Isolation

Functions used exclusively by tests MUST be annotated with `#[cfg(test)]` to exclude them from production builds, reducing binary size and compilation time.

#### Scenario: Test-only function in release build

- WHEN building with `cargo build --release`
- THEN functions only called from test modules are not included in the binary
- AND tests still have access to these functions when running `cargo test`
