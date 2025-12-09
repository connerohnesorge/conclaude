# Workspace Structure Specification

## Purpose

Define the organizational structure for internal workspace crates to maintain a clean project root and follow idiomatic Rust multi-crate workspace conventions.

## Requirements

### Requirement: Internal Crates Directory Structure
The project SHALL organize internal workspace crates under the `crates/` directory to maintain a clean project root and follow idiomatic Rust workspace conventions.

#### Scenario: Proc-macro crate location
- **WHEN** the project contains internal proc-macro crates
- **THEN** they SHALL be located under `crates/<crate-name>/`
- **AND** the workspace members in `Cargo.toml` SHALL reference them as `crates/<crate-name>`

#### Scenario: Workspace dependency resolution
- **WHEN** the main crate depends on an internal crate
- **THEN** the path dependency SHALL use `path = "crates/<crate-name>"`
- **AND** `cargo build` SHALL resolve all workspace members correctly

