## ADDED Requirements

### Requirement: Secondary Binary Location
The project SHALL organize secondary Cargo binaries (non-main entry points) under `src/bin/` to maintain consistency with Rust conventions and distinguish them from shell scripts in `scripts/`.

#### Scenario: Cargo binary location
- **WHEN** the project contains secondary binaries managed by Cargo (e.g., `generate-schema`, `generate-docs`)
- **THEN** they SHALL be located under `src/bin/<binary-name>.rs`
- **AND** the `[[bin]]` entries in `Cargo.toml` SHALL reference them as `path = "src/bin/<binary-name>.rs"`

#### Scenario: All Cargo binaries in src/bin/
- **WHEN** the project contains auxiliary Cargo binaries for development tooling (e.g., `generate-schema`, `generate-docs`, `download-ts-sdk`)
- **THEN** they SHALL be located under `src/bin/<binary-name>.rs`
- **AND** no `scripts/` directory SHALL exist for Cargo binaries

#### Scenario: Binary build verification
- **WHEN** a secondary binary exists in `src/bin/`
- **THEN** `cargo build --bin <binary-name>` SHALL succeed
- **AND** the compiled binary SHALL be available at `target/*/[binary-name]`
