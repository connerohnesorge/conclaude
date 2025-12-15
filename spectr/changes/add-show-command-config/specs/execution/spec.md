## ADDED Requirements

### Requirement: Command Execution Display Configuration

The system SHALL provide an optional `showCommand` field for individual stop commands in the configuration to control whether the command being executed is displayed in stdout.

#### Scenario: Command with showCommand explicitly set to true

- **WHEN** a stop command includes `showCommand` field set to `true`
- **THEN** the system SHALL print "Executing command X/Y: <command>" to stdout before execution
- **AND** this matches the current default behavior

#### Scenario: Command with showCommand set to false

- **WHEN** a stop command includes `showCommand` field set to `false`
- **THEN** the system SHALL NOT print "Executing command X/Y: <command>" to stdout
- **AND** the command SHALL still execute normally
- **AND** command output (stdout/stderr) SHALL still be controlled by `showStdout`/`showStderr` flags independently

#### Scenario: Command without showCommand configured

- **WHEN** a stop command does not include a `showCommand` field
- **THEN** the system SHALL default to `true`
- **AND** the system SHALL print "Executing command X/Y: <command>" to stdout
- **AND** existing behavior SHALL be preserved for backward compatibility

#### Scenario: showCommand with subagent stop commands

- **WHEN** a subagent stop command includes `showCommand` field
- **THEN** the same display control behavior SHALL apply as with regular stop commands
- **AND** the subagent stop command execution line SHALL be suppressed when `showCommand` is `false`

### Requirement: showCommand Configuration Validation

The system SHALL validate `showCommand` values in the configuration to ensure they are properly formatted.

#### Scenario: Valid showCommand value

- **WHEN** `showCommand` field contains a boolean value (`true` or `false`)
- **THEN** the configuration SHALL be accepted
- **AND** the display setting SHALL be applied during command execution

#### Scenario: Invalid showCommand value

- **WHEN** `showCommand` field contains a non-boolean value
- **THEN** the configuration loading SHALL fail with a validation error
- **AND** the error message SHALL indicate the `showCommand` value format issue
