# cli Specification

## Purpose
Defines the command-line interface capabilities for conclaude, including configuration validation.

## ADDED Requirements

### Requirement: Configuration Validation Subcommand
The system SHALL provide a `validate` subcommand that validates conclaude configuration files without executing any hooks or commands.

#### Scenario: Valid configuration with default location
- **WHEN** user runs `conclaude validate` with a valid .conclaude.yaml in the current directory
- **THEN** the system SHALL load the configuration file
- **AND** the system SHALL validate the configuration against the schema
- **AND** the system SHALL display the configuration file path
- **AND** the system SHALL display a success message
- **AND** the system SHALL exit with code 0

#### Scenario: Valid configuration with custom path
- **WHEN** user runs `conclaude validate --config-path /path/to/.conclaude.yaml` with a valid configuration
- **THEN** the system SHALL load the configuration from the specified path
- **AND** the system SHALL validate the configuration against the schema
- **AND** the system SHALL display the specified configuration file path
- **AND** the system SHALL display a success message
- **AND** the system SHALL exit with code 0

#### Scenario: Configuration file not found
- **WHEN** user runs `conclaude validate` and no configuration file exists in the search path
- **THEN** the system SHALL display an error message indicating the file was not found
- **AND** the system SHALL display the list of searched locations
- **AND** the system SHALL suggest running `conclaude init` to create a configuration
- **AND** the system SHALL exit with a non-zero code

#### Scenario: Invalid YAML syntax
- **WHEN** user runs `conclaude validate` with a configuration file containing invalid YAML syntax
- **THEN** the system SHALL display an error message indicating the YAML parsing failure
- **AND** the system SHALL include details about the syntax error (line number, issue description)
- **AND** the system SHALL provide guidance on common YAML syntax issues
- **AND** the system SHALL exit with a non-zero code

#### Scenario: Unknown configuration fields
- **WHEN** user runs `conclaude validate` with a configuration containing unknown fields
- **THEN** the system SHALL display an error message indicating the unknown field
- **AND** the system SHALL list valid field names for the relevant section
- **AND** the system SHALL provide guidance on common causes (typos, incorrect casing)
- **AND** the system SHALL exit with a non-zero code

#### Scenario: Invalid field types
- **WHEN** user runs `conclaude validate` with a configuration containing fields with incorrect types
- **THEN** the system SHALL display an error message indicating the type mismatch
- **AND** the system SHALL specify the expected type and actual type
- **AND** the system SHALL provide examples of correct type formatting
- **AND** the system SHALL exit with a non-zero code

#### Scenario: Configuration values out of valid range
- **WHEN** user runs `conclaude validate` with a configuration containing values outside valid ranges (e.g., maxOutputLines > 10000)
- **THEN** the system SHALL display an error message indicating the range violation
- **AND** the system SHALL specify the valid range for the field
- **AND** the system SHALL exit with a non-zero code

### Requirement: Configuration Discovery
The system SHALL discover configuration files using the same search strategy as runtime hooks to ensure validation matches actual behavior.

#### Scenario: Search strategy consistency
- **WHEN** user runs `conclaude validate` without specifying a config path
- **THEN** the system SHALL search for .conclaude.yaml and .conclaude.yml files
- **AND** the system SHALL search up the directory tree from the current directory
- **AND** the system SHALL stop at package.json boundaries or after 12 levels
- **AND** the search behavior SHALL match the behavior used during hook execution

### Requirement: Validation Output Format
The system SHALL provide clear, structured output that indicates validation status and configuration location.

#### Scenario: Validation output structure for success
- **WHEN** validation succeeds
- **THEN** the system SHALL display the message "Search: Validating conclaude configuration..."
- **AND** the system SHALL display "[OK] Configuration is valid: {path}"
- **AND** the output SHALL be human-readable and use status indicators

#### Scenario: Validation output structure for failure
- **WHEN** validation fails
- **THEN** the system SHALL display the message "Search: Validating conclaude configuration..."
- **AND** the system SHALL display "[NO] Configuration validation failed"
- **AND** the system SHALL display detailed error information
- **AND** the system SHALL provide actionable remediation steps
- **AND** the output SHALL be human-readable and use status indicators

### Requirement: Exit Code Semantics
The system SHALL return appropriate exit codes to support scripting and CI/CD integration.

#### Scenario: Success exit code
- **WHEN** configuration validation succeeds
- **THEN** the system SHALL exit with code 0

#### Scenario: Failure exit code
- **WHEN** configuration validation fails for any reason
- **THEN** the system SHALL exit with a non-zero code

#### Scenario: CI/CD pipeline integration
- **WHEN** conclaude validate is run in a CI/CD pipeline
- **THEN** the exit code SHALL accurately reflect validation status
- **AND** the pipeline SHALL be able to fail builds on invalid configurations
