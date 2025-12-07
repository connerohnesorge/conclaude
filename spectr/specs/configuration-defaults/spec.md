# Configuration Defaults Specification

## Purpose

Define default configuration values and security protections applied when initializing conclaude, including automatic protection of config files from AI modification to preserve guardrail settings.

## Requirements

### Requirement: Default Configuration File Protection

The system SHALL include conclaude configuration files in the default `uneditableFiles` list to prevent AI from modifying guardrail settings.

#### Scenario: Default configuration includes config file protection

- **WHEN** generating a default configuration with `conclaude init`
- **THEN** the `rules.uneditableFiles` array SHALL include `.conclaude.yml`
- **AND** the `rules.uneditableFiles` array SHALL include `.conclaude.yaml`
- **AND** the configuration SHALL include a comment explaining this security protection

#### Scenario: Config file protection prevents edits

- **WHEN** Claude Code attempts to edit `.conclaude.yml` and it is listed in `uneditableFiles`
- **THEN** the PreToolUse hook SHALL block the edit operation
- **AND** SHALL return an error message indicating the file cannot be modified

#### Scenario: Users can override default protection

- **WHEN** a user removes `.conclaude.yml` from their `uneditableFiles` list
- **THEN** the system SHALL allow edits to the configuration file
- **AND** SHALL respect the user's explicit configuration choice

### Requirement: Default Configuration Security Documentation

The system SHALL document the security rationale for protecting configuration files in the default configuration template.

#### Scenario: Default config includes security explanation

- **WHEN** viewing the generated default configuration file
- **THEN** the `uneditableFiles` section SHALL include a comment explaining that config files should be protected
- **AND** SHALL provide rationale about preventing AI from disabling guardrails
- **AND** SHALL inform users they can remove this protection if desired

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
