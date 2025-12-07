## ADDED Requirements

### Requirement: Configuration Documentation Generation

The system SHALL provide a documentation generator that extracts configuration metadata from the JSON Schema and outputs Markdown reference documentation compatible with Starlight.

#### Scenario: Generator produces valid Starlight Markdown

- **WHEN** the generator is invoked via `cargo run --bin generate-docs`
- **THEN** it SHALL output Markdown files to `docs/src/content/docs/reference/`
- **AND** each file SHALL include valid Starlight frontmatter with `title` and `description` fields

#### Scenario: Hybrid page structure is generated

- **WHEN** the generator runs
- **THEN** it SHALL produce an overview page at `reference/configuration.md`
- **AND** it SHALL produce detail pages for each configuration section:
  - `reference/config/stop.md`
  - `reference/config/subagent-stop.md`
  - `reference/config/pre-tool-use.md`
  - `reference/config/notifications.md`
  - `reference/config/permission-request.md`

#### Scenario: Overview page contains quick reference

- **WHEN** the overview page is generated
- **THEN** it SHALL include a summary table of all configuration sections
- **AND** it SHALL link to the detailed per-section pages

#### Scenario: Field metadata is extracted from schema

- **WHEN** a configuration field has metadata in the JSON Schema
- **THEN** the generated documentation SHALL include:
  - Field name (using the YAML/JSON key, not Rust field name)
  - Type information (string, boolean, integer, array, object)
  - Default value if specified
  - Description from schema if available
  - Validation constraints (min/max ranges) if applicable

#### Scenario: YAML examples are included

- **WHEN** detail pages are generated
- **THEN** each section SHALL include inline YAML code blocks showing common configurations
- **AND** each page SHALL link to `default-config.yaml` for the complete reference

#### Scenario: Nested types are documented

- **WHEN** a configuration field references a nested type (e.g., `StopCommand`, `ToolUsageRule`)
- **THEN** the generator SHALL include a subsection documenting that nested type's fields

### Requirement: Schema Description Consolidation

The system SHALL use Rust doc comments as the single source of truth for configuration field descriptions.

#### Scenario: Doc comments populate schema descriptions

- **WHEN** a struct field in `src/config.rs` has a `///` doc comment
- **THEN** the `schemars` derive SHALL include that comment as the field's `description` in `conclaude-schema.json`

#### Scenario: All configuration fields have descriptions

- **WHEN** the schema is generated
- **THEN** every user-facing configuration field SHALL have a non-empty description
- **AND** descriptions SHALL explain the field's purpose and valid values

#### Scenario: YAML comments are simplified

- **WHEN** descriptions are consolidated to Rust doc comments
- **THEN** `default-config.yaml` SHALL contain only minimal example-focused comments
- **AND** detailed explanations SHALL be in the generated documentation

### Requirement: Documentation Synchronization

The system SHALL support verification that generated documentation matches the current schema.

#### Scenario: CI validates documentation is current

- **WHEN** the CI pipeline runs
- **THEN** it SHALL regenerate documentation and compare with committed version
- **AND** the build SHALL fail if generated output differs from committed documentation

#### Scenario: Generator is idempotent

- **WHEN** the generator is run multiple times with identical schema input
- **THEN** the output SHALL be identical each time (no non-deterministic ordering or timestamps)
