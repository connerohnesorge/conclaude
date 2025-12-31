# execution Specification Delta

## ADDED Requirements

### Requirement: Per-Command Notification Integration

The system SHALL integrate per-command notifications into the command execution pipeline for both stop and subagent stop commands.

#### Scenario: Stop command execution with per-command notifications

- **WHEN** a stop command has `notifyPerCommand: true`
- **THEN** the system SHALL call `send_notification()` before command execution
- **AND** the notification SHALL use the hook name "Stop" with context indicating command start
- **AND** the system SHALL call `send_notification()` after command execution completes
- **AND** the notification status SHALL be "success" or "failure" based on exit code

#### Scenario: Subagent stop command execution with per-command notifications

- **WHEN** a subagent stop command has `notifyPerCommand: true`
- **THEN** the system SHALL call `send_notification()` before command execution
- **AND** the notification SHALL use the hook name "SubagentStop" with context indicating command start
- **AND** the system SHALL call `send_notification()` after command execution completes
- **AND** the notification status SHALL be "success" or "failure" based on exit code

#### Scenario: Command execution without per-command notifications

- **WHEN** a command has `notifyPerCommand: false` or omits the field
- **THEN** the system SHALL NOT call `send_notification()` for command start
- **AND** the system SHALL NOT call `send_notification()` for command completion
- **AND** only hook-level notifications SHALL be sent

### Requirement: Per-Command Notification Message Formatting

The system SHALL format per-command notification messages based on command configuration.

#### Scenario: Notification message with showCommand enabled

- **WHEN** generating a per-command notification for a command with `showCommand: true`
- **THEN** the start notification context SHALL include the command string (e.g., "Running: npm test")
- **AND** the completion notification context SHALL include the command string and status (e.g., "Completed: npm test")

#### Scenario: Notification message with showCommand disabled

- **WHEN** generating a per-command notification for a command with `showCommand: false`
- **THEN** the start notification context SHALL use a generic format (e.g., "Running command 2/5")
- **AND** the completion notification context SHALL use a generic format (e.g., "Command 2/5 completed")
- **AND** the command string SHALL NOT appear in the notification

#### Scenario: Notification message includes command index

- **WHEN** generating a per-command notification for any command
- **THEN** the notification context MAY include the command index and total count
- **AND** the format SHALL be "X/Y" where X is the current command number and Y is total commands

### Requirement: Per-Command Notification Configuration Propagation

The system SHALL propagate the `notifyPerCommand` configuration flag through the command execution pipeline.

#### Scenario: notifyPerCommand flag in StopCommandConfig

- **WHEN** collecting stop commands from configuration
- **THEN** the `StopCommandConfig` struct SHALL include a `notify_per_command` boolean field
- **AND** the field SHALL be populated from the command configuration's `notifyPerCommand` value
- **AND** the field SHALL default to `false` when not specified

#### Scenario: notifyPerCommand flag in SubagentStopCommandConfig

- **WHEN** collecting subagent stop commands from configuration
- **THEN** the `SubagentStopCommandConfig` struct SHALL include a `notify_per_command` boolean field
- **AND** the field SHALL be populated from the command configuration's `notifyPerCommand` value
- **AND** the field SHALL default to `false` when not specified

#### Scenario: notifyPerCommand flag during execution

- **WHEN** executing commands in `execute_stop_commands()` or `execute_subagent_stop_commands()`
- **THEN** the execution loop SHALL check the `notify_per_command` flag for each command
- **AND** notifications SHALL be sent conditionally based on this flag
