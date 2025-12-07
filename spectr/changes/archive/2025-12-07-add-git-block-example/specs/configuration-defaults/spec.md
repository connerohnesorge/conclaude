## ADDED Requirements

### Requirement: Git Command Blocking Documentation

The system SHALL include commented examples in the default configuration demonstrating how to block git shell commands using toolUsageValidation.

#### Scenario: Default config includes git blocking example

- **WHEN** viewing the default configuration file at `src/default-config.yaml`
- **THEN** the `toolUsageValidation` examples section SHALL include a commented example blocking `git push --force*`
- **AND** SHALL include a commented example blocking all git commands with pattern `git *`
- **AND** SHALL include explanatory comments describing the security purpose of each example
- **AND** SHALL follow the existing documentation style and formatting conventions

#### Scenario: Examples use correct commandPattern syntax

- **WHEN** a user views the git blocking examples in the default config
- **THEN** each example SHALL use the `commandPattern` field for matching Bash commands
- **AND** SHALL specify `tool: "Bash"` to target shell commands
- **AND** SHALL include `action: "block"` to demonstrate blocking behavior
- **AND** SHALL include descriptive `message` fields explaining why the command is blocked

#### Scenario: Examples are positioned logically in config

- **WHEN** a user navigates to the `toolUsageValidation` section
- **THEN** the git command examples SHALL appear after existing file-based validation examples
- **AND** SHALL maintain consistent YAML indentation with surrounding content
- **AND** SHALL be clearly commented to indicate they are examples (not active rules)
