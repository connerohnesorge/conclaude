## ADDED Requirements

### Requirement: Secondary Binary Location
The project SHALL organize secondary Cargo binaries (non-main entry points) under `src/bin/` to maintain consistency with Rust conventions and distinguish them from shell scripts in `scripts/`.

#### Scenario: Cargo binary location
- **WHEN** the project contains secondary binaries managed by Cargo (e.g., `generate-schema`, `generate-docs`)
- **THEN** they SHALL be located under `src/bin/<binary-name>.rs`
- **AND** the `[[bin]]` entries in `Cargo.toml` SHALL reference them as `path = "src/bin/<binary-name>.rs"`

#### Scenario: Shell scripts remain in scripts/
- **WHEN** the project contains shell scripts for development tooling (e.g., `download-ts-sdk.sh`)
- **THEN** they SHALL remain in the `scripts/` directory
- **AND** only non-Cargo shell scripts SHALL be placed in `scripts/`

#### Scenario: Binary build verification
- **WHEN** a secondary binary exists in `src/bin/`
- **THEN** `cargo build --bin <binary-name>` SHALL succeed
- **AND** the compiled binary SHALL be available at `target/*/[binary-name]`
